use nalgebra as na;
use wgpu::{
    core::device::{self, queue},
    util::DeviceExt,
};

use super::{
    affine,
    application_context::ApplicationContext,
    types::PxSize,
    ui::{self, RenderObject, TeaUi},
    vertex::TexturedVertex,
};

struct GpuState {
    // Textured Vertex
    texture_bind_group_layout: wgpu::BindGroupLayout,
    textured_render_pipeline: wgpu::RenderPipeline,

    // Colored Vertex
    colored_render_pipeline: wgpu::RenderPipeline,

    // common
    affine_bind_group_layout: wgpu::BindGroupLayout,
}

pub struct Renderer {
    base_color: super::types::Color,
    uis: Vec<Box<dyn TeaUi>>,
    app_context: Option<ApplicationContext>,
    gpu_state: Option<GpuState>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            base_color: super::types::Color::default(),
            uis: Vec::new(),
            app_context: None,
            gpu_state: None,
        }
    }

    pub fn base_color(self, color: super::types::Color) -> Self {
        Self {
            base_color: color,
            ..self
        }
    }

    pub fn ui(self, ui: Vec<Box<dyn TeaUi>>) -> Self {
        Self { uis: ui, ..self }
    }

    pub fn push_ui(&mut self, ui: Box<dyn TeaUi>) {
        self.uis.push(ui);
    }
}

impl Renderer {
    pub fn set_application_context(&mut self, context: ApplicationContext) {
        let device = context.get_wgpu_device();

        let surface_format = context.get_surface_format();

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
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let textured_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Panel Render Pipeline Layout"),
                bind_group_layouts: &[&affine_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let textured_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Panel Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("render_textured.wgsl").into()),
        });

        let textured_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Panel Render Pipeline"),
                layout: Some(&textured_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &textured_shader,
                    entry_point: "vs_main",
                    buffers: &[crate::vertex::TexturedVertex::desc()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &textured_shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
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

        let colored_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Panel Render Pipeline Layout"),
                bind_group_layouts: &[&affine_bind_group_layout],
                push_constant_ranges: &[],
            });

        let colored_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Panel Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("render_colored.wgsl").into()),
        });

        let colored_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Panel Render Pipeline"),
                layout: Some(&colored_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &colored_shader,
                    entry_point: "vs_main",
                    buffers: &[crate::vertex::ColoredVertex::desc()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &colored_shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
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

        self.gpu_state = Some(GpuState {
            texture_bind_group_layout,
            textured_render_pipeline,
            colored_render_pipeline,
            affine_bind_group_layout,
        });

        self.app_context = Some(context);
    }

    pub fn render(&self, surface_view: wgpu::TextureView) -> Result<(), ()> {
        let device = match self.app_context.as_ref() {
            Some(context) => context.get_wgpu_device(),
            None => return Err(()),
        };
        let queue = match self.app_context.as_ref() {
            Some(context) => context.get_wgpu_queue(),
            None => return Err(()),
        };

        // refresh the screen
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let base_color_f64 = self.base_color.to_rgba_f64();

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Panel Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: base_color_f64[0],
                        g: base_color_f64[1],
                        b: base_color_f64[2],
                        a: base_color_f64[3],
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // render the ui

        // let stencil_stacks = Vec::new();

        let mut accumulated_height = 0.0;

        for ui in &self.uis {
            let render_object = ui.render_object()?;
            self.render_objects(
                &mut encoder,
                &surface_view,
                &render_object,
                affine::translate_2d(0.0, accumulated_height) * na::Matrix3::identity(),
            )?;
            accumulated_height += render_object.px_size.height;
        }

        queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }

    fn render_objects(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
        object: &RenderObject,
        affine: na::Matrix3<f32>,
    ) -> Result<(), ()> {
        let device = match self.app_context.as_ref() {
            Some(context) => context.get_wgpu_device(),
            None => return Err(()),
        };

        // check gpu state
        if self.gpu_state.is_none() {
            return Err(());
        }

        // render the object
        match &object.object {
            ui::Object::Textured {
                vertex_buffer,
                index_buffer,
                index_len,
                texture,
            } => {
                let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("Renderer TexturedVertex Texture Sampler"),
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Nearest,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    ..Default::default()
                });
                let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Renderer TexturedVertex Texture Bind Group"),
                    layout: &self.gpu_state.as_ref().unwrap().texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&texture_sampler),
                        },
                    ],
                });

                let affine_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Renderer TexturedVertex Affine Bind Group"),
                    layout: &self.gpu_state.as_ref().unwrap().affine_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Render TexturedVertex Affine Buffer"),
                                contents: bytemuck::cast_slice(affine.as_slice()),
                                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                            }),
                            offset: 0,
                            size: None,
                        }),
                    }],
                });

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Renderer TexturedVertex Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: surface_view,
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

                render_pass.set_pipeline(&self.gpu_state.as_ref().unwrap().textured_render_pipeline);
                render_pass.set_bind_group(0, &affine_bind_group, &[]);
                render_pass.set_bind_group(1, &texture_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..*index_len, 0, 0..1);
            }
            ui::Object::Colored {
                vertex_buffer,
                index_buffer,
                index_len,
            } => {
                let affine_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Renderer TexturedVertex Affine Bind Group"),
                    layout: &self.gpu_state.as_ref().unwrap().affine_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Render TexturedVertex Affine Buffer"),
                                contents: bytemuck::cast_slice(affine.as_slice()),
                                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                            }),
                            offset: 0,
                            size: None,
                        }),
                    }],
                });

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Renderer TexturedVertex Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: surface_view,
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

                render_pass.set_pipeline(&self.gpu_state.as_ref().unwrap().textured_render_pipeline);
                render_pass.set_bind_group(0, &affine_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..*index_len, 0, 0..1);
            },
        }

        // recursively render the sub objects

        for sub_object in &object.sub_objects {
            self.render_objects(
                encoder,
                surface_view,
                &sub_object.object,
                affine * sub_object.affine,
            )?;
        }
        Ok(())
    }
}
