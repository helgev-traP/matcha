use wgpu::util::DeviceExt;

use crate::{
    application_context::ApplicationContext,
    ui::{Layout, RenderArea, Ui, WidgetRenderObject, Widgets},
    widgets,
};

pub enum InnerPanelPosition {
    Top,
    Bottom,
    Left,
    Right,
    Floating {
        x: f32,
        y: f32,
        z: f32, // use for treating as a layer.
    },
}

pub struct InnerPanel {
    position: InnerPanelPosition,
    thickness: f32,
    panel: Panel,
}

impl Panel {
    pub fn as_inner_panel(&self) -> InnerPanel {
        todo!()
    }
}

struct GpuFields {
    texture: wgpu::Texture,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    affine_bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
}

pub struct Panel {
    size: crate::types::Size,
    base_color: [u8; 4],

    // main area
    main_area_widgets: Layout,

    // inner panels
    inner_panels: Vec<InnerPanel>,

    // app context
    app_context: Option<ApplicationContext>,

    // wgpu
    gpu_fields: Option<GpuFields>,
}

impl Panel {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            size: crate::types::Size { width, height },
            base_color: [0, 0, 0, 0],
            main_area_widgets: Layout::Column(Vec::new()),
            inner_panels: Vec::new(),
            app_context: None,
            gpu_fields: None,
        }
    }

    pub fn base_color(mut self, color: [u8; 4]) -> Self {
        self.base_color = color;
        self
    }

    pub fn set_base_color(&mut self, color: [u8; 4]) {
        self.base_color = color;
    }

    pub fn widgets(mut self, widgets: Vec<Box<dyn Widgets>>) -> Self {
        if let Layout::Column(w) = &mut self.main_area_widgets {
            *w = widgets
        };
        self
    }

    pub(crate) fn set_widgets(&mut self, widgets: Vec<Box<dyn Widgets>>) {
        if let Layout::Column(w) = &mut self.main_area_widgets {
            *w = widgets
        };
    }

    pub fn panels(mut self, panels: Vec<InnerPanel>) -> Self {
        self.inner_panels = panels;
        self
    }

    pub(crate) fn set_panels(&mut self, panels: Vec<InnerPanel>) {
        self.inner_panels = panels;
    }

    pub fn add_widget(&mut self, widget: Box<dyn Widgets>) {
        if let Layout::Column(w) = &mut self.main_area_widgets {
            w.push(widget);
        }
    }

    pub fn add_top_panel(&mut self, thickness: f32) -> &InnerPanel {
        let sum_of_thickness: f32 = self
            .inner_panels
            .iter()
            .map(|inner_panel| match inner_panel.position {
                InnerPanelPosition::Left | InnerPanelPosition::Right => inner_panel.thickness,
                _ => 0.0,
            })
            .sum();
        self.inner_panels.push(InnerPanel {
            position: InnerPanelPosition::Top,
            thickness,
            panel: Panel::new(self.size.width - sum_of_thickness, thickness),
        });
        self.inner_panels.last().unwrap()
    }

    pub fn add_bottom_panel(&mut self, thickness: f32) -> &InnerPanel {
        let sum_of_thickness: f32 = self
            .inner_panels
            .iter()
            .map(|inner_panel| match inner_panel.position {
                InnerPanelPosition::Left | InnerPanelPosition::Right => inner_panel.thickness,
                _ => 0.0,
            })
            .sum();
        self.inner_panels.push(InnerPanel {
            position: InnerPanelPosition::Bottom,
            thickness,
            panel: Panel::new(self.size.width - sum_of_thickness, thickness),
        });
        self.inner_panels.last().unwrap()
    }

    pub fn add_left_panel(&mut self, thickness: f32) -> &InnerPanel {
        let sum_of_thickness: f32 = self
            .inner_panels
            .iter()
            .map(|inner_panel| match inner_panel.position {
                InnerPanelPosition::Top | InnerPanelPosition::Bottom => inner_panel.thickness,
                _ => 0.0,
            })
            .sum();
        self.inner_panels.push(InnerPanel {
            position: InnerPanelPosition::Left,
            thickness,
            panel: Panel::new(thickness, self.size.height - sum_of_thickness),
        });
        self.inner_panels.last().unwrap()
    }

    pub fn add_right_panel(&mut self, thickness: f32) -> &InnerPanel {
        let sum_of_thickness: f32 = self
            .inner_panels
            .iter()
            .map(|inner_panel| match inner_panel.position {
                InnerPanelPosition::Top | InnerPanelPosition::Bottom => inner_panel.thickness,
                _ => 0.0,
            })
            .sum();
        self.inner_panels.push(InnerPanel {
            position: InnerPanelPosition::Right,
            thickness,
            panel: Panel::new(thickness, self.size.height - sum_of_thickness),
        });
        self.inner_panels.last().unwrap()
    }

    pub fn add_floating_panel(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        width: f32,
        height: f32,
    ) -> &InnerPanel {
        self.inner_panels.push(InnerPanel {
            position: InnerPanelPosition::Floating { x, y, z },
            thickness: 0.0,
            panel: Panel::new(width, height),
        });
        self.inner_panels.last().unwrap()
    }
}

impl Ui for Panel {
    fn set_application_context(&mut self, context: crate::application_context::ApplicationContext) {
        let device = context.get_wgpu_device();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Panel Texture"),
            size: wgpu::Extent3d {
                width: self.size.width as u32,
                height: self.size.height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Panel Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let affine_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Panel Affine Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Panel Render Pipeline Layout"),
                bind_group_layouts: &[&affine_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Panel Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("panel.wgsl").into()),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Panel Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[crate::vertex::TexturedVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // for inner panels and main area widgets
        for inner_panel in &mut self.inner_panels {
            inner_panel.panel.set_application_context(context.clone());
        }

        self.main_area_widgets.set_application_context(context.clone());

        // set
        self.app_context = Some(context);
        self.gpu_fields = Some(GpuFields {
            texture,
            texture_bind_group_layout,
            affine_bind_group_layout,
            render_pipeline,
        });
    }

    fn size(&self) -> crate::types::Size {
        self.size
    }

    fn resize(&mut self, size: crate::types::Size) {
        self.size = size;
        if let Some(gpu_fields) = &self.gpu_fields {
            // resize texture
            let device = self.app_context.as_ref().unwrap().get_wgpu_device();
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Panel Texture"),
                size: wgpu::Extent3d {
                    width: self.size.width as u32,
                    height: self.size.height as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            self.gpu_fields.as_mut().unwrap().texture = texture;
        }
        // resize inner panels
        let sum_of_thickness_vertical = 0.0;
        let sum_of_thickness_horizontal = 0.0;
        for inner_panel in &mut self.inner_panels {
            match inner_panel.position {
                InnerPanelPosition::Top | InnerPanelPosition::Bottom => {
                    inner_panel.panel.resize(crate::types::Size {
                        width: self.size.width - sum_of_thickness_horizontal,
                        height: inner_panel.thickness,
                    });
                }
                InnerPanelPosition::Left | InnerPanelPosition::Right => {
                    inner_panel.panel.resize(crate::types::Size {
                        width: inner_panel.thickness,
                        height: self.size.height - sum_of_thickness_vertical,
                    });
                }
                InnerPanelPosition::Floating { .. } => (),
            }
        }
    }
}

impl RenderArea for Panel {
    fn render(&self) -> Option<&wgpu::Texture> {
        // 1. render main area widgets
        // calculate offset

        let mut offset = [0.0, 0.0];

        for ip in &self.inner_panels {
            match ip.position {
                InnerPanelPosition::Top => {
                    offset[1] -= ip.thickness;
                }
                InnerPanelPosition::Left => {
                    offset[0] += ip.thickness;
                }
                _ => (),
            }
        }

        // begin wgpu

        let device = self.app_context.as_ref()?.get_wgpu_device();
        let queue = self.app_context.as_ref()?.get_wgpu_queue();

        let texture_view = self
            .gpu_fields
            .as_ref()?
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // fill texture with base color

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Panel Command Encoder"),
        });
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Panel Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: if self.base_color[0] as f64 / 255.0 <= 0.04045 {
                            self.base_color[0] as f64 / 255.0 / 12.92
                        } else {
                            ((self.base_color[0] as f64 / 255.0 + 0.055) / 1.055).powf(2.4)
                        },
                        g: if self.base_color[1] as f64 / 255.0 <= 0.04045 {
                            self.base_color[1] as f64 / 255.0 / 12.92
                        } else {
                            ((self.base_color[1] as f64 / 255.0 + 0.055) / 1.055).powf(2.4)
                        },
                        b: if self.base_color[2] as f64 / 255.0 <= 0.04045 {
                            self.base_color[2] as f64 / 255.0 / 12.92
                        } else {
                            ((self.base_color[2] as f64 / 255.0 + 0.055) / 1.055).powf(2.4)
                        },
                        a: self.base_color[3] as f64 / 255.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        queue.submit(std::iter::once(encoder.finish()));

        // collect widget's render objects

        let widgets_objs = self.main_area_widgets.render_object()?;

        // render
        for obj in widgets_objs {
            let diffuse_texture_view = obj
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let diffuse_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });

            let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Panel Texture Bind Group"),
                layout: &self.gpu_fields.as_ref()?.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture_sampler),
                    },
                ],
            });

            let affine_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Panel Affine Bind Group"),
                layout: &self.gpu_fields.as_ref()?.affine_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Camera Buffer"),
                                contents: bytemuck::cast_slice(&[
                                    offset[0] + obj.offset[0],
                                    offset[1] + obj.offset[1],
                                ]),
                                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                            })
                            .as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Camera Buffer"),
                                contents: bytemuck::cast_slice(&[
                                    self.size.width as f32,
                                    self.size.height as f32,
                                ]),
                                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                            })
                            .as_entire_binding(),
                    },
                ],
            });

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Panel Command Encoder"),
            });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Panel Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                render_pass.set_pipeline(&self.gpu_fields.as_ref()?.render_pipeline);
                render_pass.set_bind_group(0, &affine_bind_group, &[]);
                render_pass.set_bind_group(1, &texture_bind_group, &[]);
                render_pass.set_vertex_buffer(0, obj.vertex_buffer.slice(..));
                render_pass.set_index_buffer(obj.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..obj.index_count, 0, 0..1);
            }

            queue.submit(std::iter::once(encoder.finish()));
        }

        // todo!()

        Some(&self.gpu_fields.as_ref()?.texture)
    }
}

impl Widgets for Panel {
    fn render_object(&self) -> Option<Vec<WidgetRenderObject>> {
        todo!()
    }
}
