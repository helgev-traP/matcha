use wgpu::util::DeviceExt;

use crate::{
    application_context::ApplicationContext,
    types::Size,
    ui::{Layout, RenderArea, Ui, WidgetRenderObject, Widgets},
    vertex::ColoredVertex,
    widgets,
};

struct PannelRenderObject<'a> {
    position: [f32; 2],
    size: [f32; 2],
    base_color: [u8; 4],
    widgets: Vec<WidgetRenderObject<'a>>,
    panels: Vec<PannelRenderObject<'a>>,
}

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

impl InnerPanel {
    pub fn base_color(mut self, color: [u8; 4]) -> Self {
        self.panel.base_color = color;
        self
    }

    pub fn set_base_color(&mut self, color: [u8; 4]) {
        self.panel.base_color = color;
    }

    pub fn widgets(mut self, widgets: Vec<Box<dyn Widgets>>) -> Self {
        if let Layout::Column(w) = &mut self.panel.main_area_widgets {
            *w = widgets
        };
        self
    }

    pub fn set_widgets(&mut self, widgets: Vec<Box<dyn Widgets>>) {
        if let Layout::Column(w) = &mut self.panel.main_area_widgets {
            *w = widgets
        };
    }

    pub fn panels(mut self, panels: Vec<InnerPanel>) -> Self {
        self.panel.inner_panels = panels;
        self
    }

    pub fn set_panels(&mut self, panels: Vec<InnerPanel>) {
        self.panel.inner_panels = panels;
    }

    pub fn add_widget(&mut self, widget: Box<dyn Widgets>) {
        if let Layout::Column(w) = &mut self.panel.main_area_widgets {
            w.push(widget);
        }
    }

    pub fn add_top_panel(&mut self, thickness: f32) -> &mut InnerPanel {
        self.panel.add_top_panel(thickness)
    }

    pub fn add_bottom_panel(&mut self, thickness: f32) -> &mut InnerPanel {
        self.panel.add_bottom_panel(thickness)
    }

    pub fn add_left_panel(&mut self, thickness: f32) -> &mut InnerPanel {
        self.panel.add_left_panel(thickness)
    }

    pub fn add_right_panel(&mut self, thickness: f32) -> &mut InnerPanel {
        self.panel.add_right_panel(thickness)
    }

    pub fn add_floating_panel(&mut self, x: f32, y: f32, z: f32, size: Size) -> &mut InnerPanel {
        self.panel.add_floating_panel(x, y, z, size)
    }
}

struct GpuFields {
    texture: wgpu::Texture,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    affine_bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
    scissor_fill_pipeline: wgpu::RenderPipeline,
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
    pub fn new(size: Size) -> Self {
        Self {
            size,
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

    pub fn add_top_panel(&mut self, thickness: f32) -> &mut InnerPanel {
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
            panel: Panel::new(Size {
                width: self.size.width - sum_of_thickness,
                height: thickness,
            }),
        });
        self.inner_panels.last_mut().unwrap()
    }

    pub fn add_bottom_panel(&mut self, thickness: f32) -> &mut InnerPanel {
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
            panel: Panel::new(Size {
                width: self.size.width - sum_of_thickness,
                height: thickness,
            }),
        });
        self.inner_panels.last_mut().unwrap()
    }

    pub fn add_left_panel(&mut self, thickness: f32) -> &mut InnerPanel {
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
            panel: Panel::new(Size {
                width: thickness,
                height: self.size.height - sum_of_thickness,
            }),
        });
        self.inner_panels.last_mut().unwrap()
    }

    pub fn add_right_panel(&mut self, thickness: f32) -> &mut InnerPanel {
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
            panel: Panel::new(Size {
                width: thickness,
                height: self.size.height - sum_of_thickness,
            }),
        });
        self.inner_panels.last_mut().unwrap()
    }

    pub fn add_floating_panel(&mut self, x: f32, y: f32, z: f32, size: Size) -> &mut InnerPanel {
        self.inner_panels.push(InnerPanel {
            position: InnerPanelPosition::Floating { x, y, z },
            thickness: 0.0,
            panel: Panel::new(size),
        });
        self.inner_panels.last_mut().unwrap()
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
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

        // scissor fill pipeline

        let scissor_fill_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Scissor Fill Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let scissor_fill_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Scissor Fill Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("panel_scissor_fill.wgsl").into()),
        });

        let scissor_fill_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Scissor Fill Pipeline"),
                layout: Some(&scissor_fill_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &scissor_fill_shader,
                    entry_point: "vs_main",
                    buffers: &[crate::vertex::ColoredVertex::desc()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &scissor_fill_shader,
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

        self.main_area_widgets
            .set_application_context(context.clone());

        // set
        self.app_context = Some(context);
        self.gpu_fields = Some(GpuFields {
            texture,
            texture_bind_group_layout,
            affine_bind_group_layout,
            render_pipeline,
            scissor_fill_pipeline,
        });
    }

    fn size(&self) -> crate::types::Size {
        self.size
    }

    fn event(&mut self, event: &crate::event::Event) {
        match event {
            crate::event::Event::Resize(size) => {
                // set info
                self.size = *size;

                // resize texture
                if let Some(gpu_fields) = &self.gpu_fields {
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
                            | wgpu::TextureUsages::TEXTURE_BINDING
                            | wgpu::TextureUsages::COPY_SRC
                            | wgpu::TextureUsages::COPY_DST,
                        view_formats: &[],
                    });
                    self.gpu_fields.as_mut().unwrap().texture = texture;
                }

                // resize inner panels
                let mut sum_of_thickness_vertical = 0.0;
                let mut sum_of_thickness_horizontal = 0.0;

                for inner_panel in &mut self.inner_panels {
                    match inner_panel.position {
                        InnerPanelPosition::Top | InnerPanelPosition::Bottom => {
                            inner_panel.panel.event(&crate::event::Event::Resize(
                                crate::types::Size {
                                    width: self.size.width - sum_of_thickness_horizontal,
                                    height: inner_panel.thickness,
                                },
                            ));

                            sum_of_thickness_vertical += inner_panel.thickness;
                        }
                        InnerPanelPosition::Left | InnerPanelPosition::Right => {
                            inner_panel.panel.event(&crate::event::Event::Resize(
                                crate::types::Size {
                                    width: inner_panel.thickness,
                                    height: self.size.height - sum_of_thickness_vertical,
                                },
                            ));

                            sum_of_thickness_horizontal += inner_panel.thickness;
                        }
                        InnerPanelPosition::Floating { .. } => (),
                    }
                }
            }
            _ => (),
        }
    }
}

impl Panel {
    fn render_widget(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        texture_view: &wgpu::TextureView,
        base_color: [u8; 4],
        position: [f32; 2],
        scissor_size: crate::types::Size,
        base_panel_size: crate::types::Size,
        widgets: Vec<WidgetRenderObject>,
    ) {
        let device = self
            .app_context
            .as_ref()
            .expect("context not exist.")
            .get_wgpu_device();
        let queue = self
            .app_context
            .as_ref()
            .expect("context not exist.")
            .get_wgpu_queue();

        // scissor area

        let scissor_x = position[0].max(0.0) as u32;
        let scissor_y = (-position[1]).max(0.0) as u32;
        let scissor_width = (scissor_size.width).min(base_panel_size.width - position[0]) as u32;
        let scissor_height = (scissor_size.height).min(base_panel_size.height + position[1]) as u32;

        if scissor_width <= 0 || scissor_height <= 0 {
            return;
        }

        // fill texture with base color

        let (vertex_buffer, index_buffer, index_count) = ColoredVertex::rectangle_buffer_srgb(
            self.app_context.as_ref().unwrap(),
            -1.0,
            1.0,
            2.0,
            2.0,
            base_color,
            true,
        );

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

            render_pass.set_pipeline(&self.gpu_fields.as_ref().unwrap().scissor_fill_pipeline);
            render_pass.set_scissor_rect(scissor_x, scissor_y, scissor_width, scissor_height);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..index_count, 0, 0..1);
        }

        // render
        for obj in widgets {
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
                layout: &self.gpu_fields.as_ref().unwrap().texture_bind_group_layout,
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
                layout: &self.gpu_fields.as_ref().unwrap().affine_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Camera Buffer"),
                                contents: bytemuck::cast_slice(&[
                                    position[0] + obj.offset[0],
                                    position[1] + obj.offset[1],
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

                render_pass.set_scissor_rect(scissor_x, scissor_y, scissor_width, scissor_height);
                render_pass.set_pipeline(&self.gpu_fields.as_ref().unwrap().render_pipeline);
                render_pass.set_bind_group(0, &affine_bind_group, &[]);
                render_pass.set_bind_group(1, &texture_bind_group, &[]);
                render_pass.set_vertex_buffer(0, obj.vertex_buffer.slice(..));
                render_pass.set_index_buffer(obj.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..obj.index_count, 0, 0..1);
            }
        }
    }
}

impl RenderArea for Panel {
    fn render(&self) -> Option<&wgpu::Texture> {
        // begin wgpu

        let texture_view = self
            .gpu_fields
            .as_ref()?
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .app_context
            .as_ref()?
            .get_wgpu_device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Panel Command Encoder"),
            });

        // 1. render inner side panels

        let mut side_offset = [0.0, 0.0, 0.0, 0.0]; // top, bottom, left, right

        let mut side_panels = Vec::new();
        let mut floatings = Vec::new();

        for inner_panel in &self.inner_panels {
            let mut render_position = [0.0, 0.0];
            match inner_panel.position {
                InnerPanelPosition::Floating { x, y, z } => {
                    floatings.push(inner_panel);
                    continue;
                }
                InnerPanelPosition::Top => {
                    render_position = [side_offset[2], -side_offset[0]];
                    side_offset[0] += inner_panel.thickness;
                }
                InnerPanelPosition::Bottom => {
                    render_position = [
                        side_offset[2],
                        inner_panel.thickness + side_offset[1] - self.size.height,
                    ];
                    side_offset[1] += inner_panel.thickness;
                }
                InnerPanelPosition::Left => {
                    render_position = [side_offset[2], -side_offset[0]];
                    side_offset[2] += inner_panel.thickness;
                }
                InnerPanelPosition::Right => {
                    render_position = [
                        self.size.width - inner_panel.thickness - side_offset[3],
                        -side_offset[0],
                    ];
                    side_offset[3] += inner_panel.thickness;
                }
            }

            side_panels.push((inner_panel, render_position));
        }

        for (inner_panel, render_position) in side_panels.into_iter().rev() {
            self.render_widget(
                &mut encoder,
                &texture_view,
                inner_panel.panel.base_color,
                render_position,
                inner_panel.panel.size,
                self.size,
                inner_panel.panel.main_area_widgets.render_object()?,
            );
        }

        // 2. render main area widgets

        self.render_widget(
            &mut encoder,
            &texture_view,
            self.base_color,
            [side_offset[2], -side_offset[0]],
            Size {
                width: self.size.width - side_offset[2] - side_offset[3],
                height: self.size.height - side_offset[0] - side_offset[1],
            },
            self.size,
            self.main_area_widgets.render_object()?,
        );

        // 3. render floating panels

        floatings.sort_by(|a, b| match (&a.position, &b.position) {
            (
                InnerPanelPosition::Floating { z: z1, .. },
                InnerPanelPosition::Floating { z: z2, .. },
            ) => z1.partial_cmp(&z2).unwrap(),
            _ => panic!("Argorithm error."),
        });

        for inner_panel in floatings {
            if let InnerPanelPosition::Floating { x, y, .. } = inner_panel.position {
                self.render_widget(
                    &mut encoder,
                    &texture_view,
                    inner_panel.panel.base_color,
                    [x, y],
                    inner_panel.panel.size,
                    self.size,
                    inner_panel.panel.main_area_widgets.render_object()?,
                );
            }
        }

        // end wgpu

        self.app_context
            .as_ref()?
            .get_wgpu_queue()
            .submit(std::iter::once(encoder.finish()));

        Some(&self.gpu_fields.as_ref()?.texture)
    }
}

impl Widgets for Panel {
    fn render_object(&self) -> Option<Vec<WidgetRenderObject>> {
        todo!()
    }
}
