use std::sync::Arc;

use wgpu::core::device::{self, queue};
use wgpu::util::DeviceExt;

use winit::{self, event::Event, platform::run_on_demand::EventLoopExtRunOnDemand};

use crate::application_context::ApplicationContext;
use crate::ui::{RenderArea, Ui, WidgetRenderObject, Widgets};

pub struct Panel {
    // panel
    size: crate::types::Size,
    base_color: [f64; 4],

    // wgpu
    device_queue: Option<ApplicationContext>,
    texture: Option<wgpu::Texture>,
    texture_bind_group_layout: Option<wgpu::BindGroupLayout>,
    affine_bind_group_layout: Option<wgpu::BindGroupLayout>,
    render_pipeline: Option<wgpu::RenderPipeline>,

    // widgets
    pub inner_elements: Vec<Box<dyn Widgets>>,
}

impl Panel {
    pub fn new(width: f32, height: f32, base_color: [f64; 4]) -> Self {
        Self {
            size: crate::types::Size { width, height },
            base_color,
            device_queue: None,
            texture: None,
            texture_bind_group_layout: None,
            affine_bind_group_layout: None,
            render_pipeline: None,
            inner_elements: Vec::new(),
        }
    }

    pub fn set_base_color(&mut self, base_color: [f64; 4]) {
        self.base_color = base_color;
    }

    pub fn set_inner_elements(&mut self, inner_elements: Box<dyn Widgets>) {
        self.inner_elements.push(inner_elements);
    }

    fn set_inner_app_context(&mut self, device_queue: ApplicationContext) {
        for inner_element in self.inner_elements.iter_mut() {
            inner_element.set_application_context(device_queue.clone());
        }
    }
}

impl Ui for Panel {
    fn set_application_context(&mut self, device_queue: crate::application_context::ApplicationContext) {
        let device = device_queue.get_wgpu_device();

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

        self.set_inner_app_context(device_queue.clone());

        self.device_queue = Some(device_queue);
        self.texture = Some(texture);
        self.texture_bind_group_layout = Some(texture_bind_group_layout);
        self.affine_bind_group_layout = Some(affine_bind_group_layout);
        self.render_pipeline = Some(render_pipeline);
    }

    fn size(&self) -> crate::types::Size {
        self.size
    }

    fn resize(&mut self, size: crate::types::Size) {
        self.size = size;
        if self.device_queue.is_some() {
            let device = self.device_queue.as_ref().unwrap().get_wgpu_device();
            self.texture = Some(device.create_texture(&wgpu::TextureDescriptor {
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
            }));
        }
    }
}

impl RenderArea for Panel {
    fn render(&self) -> Option<&wgpu::Texture> {
        let device = self.device_queue.as_ref()?.get_wgpu_device();
        let queue = self.device_queue.as_ref()?.get_wgpu_queue();

        let mut widgets_obj = Vec::new();

        for inner_element in self.inner_elements.iter() {
            widgets_obj.extend(inner_element.render_object()?);
        }

        let mut cumulative_height: f32 = 0.0;

        let texture_view = self
            .texture
            .as_ref()?
            .create_view(&wgpu::TextureViewDescriptor::default());

        // set back ground color
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
                        r: self.base_color[0],
                        g: self.base_color[1],
                        b: self.base_color[2],
                        a: self.base_color[3],
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        queue.submit(std::iter::once(encoder.finish()));

        for obj in widgets_obj {
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
                layout: self.texture_bind_group_layout.as_ref()?,
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
                layout: self.affine_bind_group_layout.as_ref()?,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Camera Buffer"),
                                contents: bytemuck::cast_slice(&[cumulative_height]),
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

                render_pass.set_pipeline(self.render_pipeline.as_ref()?);
                render_pass.set_bind_group(0, &affine_bind_group, &[]);
                render_pass.set_bind_group(1, &texture_bind_group, &[]);
                render_pass.set_vertex_buffer(0, obj.vertex_buffer.slice(..));
                render_pass.set_index_buffer(obj.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..obj.index_count, 0, 0..1);
            }

            queue.submit(std::iter::once(encoder.finish()));

            cumulative_height += obj.size.height as f32;
        }
        self.texture.as_ref()
    }
}

impl Widgets for Panel {
    fn render_object(&self) -> Option<Vec<WidgetRenderObject>> {
        todo!()
    }
}
