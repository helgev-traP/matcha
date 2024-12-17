use std::sync::Arc;

use wgpu::util::DeviceExt;

use crate::{context::SharedContext, vertex::uv_vertex::UvVertex};

/// Render to Rgba8UnormSrgb texture
pub struct Renderer {
    // context
    context: SharedContext,

    // wgpu state
    // texture must be Rgba8UnormSrgb
    bind_group_layout: wgpu::BindGroupLayout,
    texture_sampler: wgpu::Sampler,
    render_pipeline: wgpu::RenderPipeline,
    surface_render_pipeline: wgpu::RenderPipeline,
    // stencil
    // todo
    // stencil_bind_group_layout: wgpu::BindGroupLayout,
    affine_bind_group_layout: wgpu::BindGroupLayout,

    // vello renderer
    vello_renderer: std::sync::Mutex<vello::Renderer>,
}

impl Renderer {
    pub fn new(context: SharedContext) -> Self {
        let device = context.get_wgpu_device();

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

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let textured_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Panel Render Pipeline Layout"),
                bind_group_layouts: &[&affine_bind_group_layout, &bind_group_layout],
                push_constant_ranges: &[],
            });

        let textured_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Panel Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("render_textured.wgsl").into()),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Panel Render Pipeline"),
            layout: Some(&textured_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &textured_shader,
                entry_point: "vs_main",
                buffers: &[UvVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &textured_shader,
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

        let surface_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Panel Render Pipeline"),
                layout: Some(&textured_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &textured_shader,
                    entry_point: "vs_main",
                    buffers: &[UvVertex::desc()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &textured_shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: context.get_surface_format(),
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

        let vello_renderer = vello::Renderer::new(
            &device,
            vello::RendererOptions {
                surface_format: Some(wgpu::TextureFormat::Rgba8Unorm),
                use_cpu: false,
                antialiasing_support: vello::AaSupport::all(),
                num_init_threads: None,
            },
        ).unwrap().into();

        Self {
            context,
            bind_group_layout,
            texture_sampler,
            render_pipeline,
            surface_render_pipeline,
            affine_bind_group_layout,
            vello_renderer,
        }
    }

    pub fn vello_renderer<'a>(&'a self) -> std::sync::MutexGuard<'a, vello::Renderer> {
        self.vello_renderer.lock().unwrap()
    }

    pub fn render_to_screen(
        &self,
        destination_view: &wgpu::TextureView,
        texture_size: [f32; 2],
        source: Vec<(
            Arc<wgpu::Texture>,
            Arc<Vec<UvVertex>>,
            Arc<Vec<u16>>,
            nalgebra::Matrix4<f32>,
        )>,
    ) {
        self.render_process(destination_view, texture_size, source, true);
    }

    pub fn render(
        &self,
        destination_view: &wgpu::TextureView,
        texture_size: [f32; 2],
        source: Vec<(
            Arc<wgpu::Texture>,
            Arc<Vec<UvVertex>>,
            Arc<Vec<u16>>,
            nalgebra::Matrix4<f32>,
        )>,
    ) {
        self.render_process(destination_view, texture_size, source, false);
    }

    fn render_process(
        &self,
        destination_view: &wgpu::TextureView,
        texture_size: [f32; 2],
        source: Vec<(
            Arc<wgpu::Texture>,
            Arc<Vec<UvVertex>>,
            Arc<Vec<u16>>,
            nalgebra::Matrix4<f32>,
        )>,
        render_to_surface: bool,
    ) {
        let device = self.context.get_wgpu_device();
        let mut encoder = self.context.get_wgpu_device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Renderer Project Command Encoder"),
            },
        );

        let normalize_matrix = nalgebra::Matrix4::new(
            // x
            2.0 / texture_size[0],
            0.0,
            0.0,
            -1.0,
            // y
            0.0,
            2.0 / texture_size[1],
            0.0,
            1.0,
            // z
            0.0,
            0.0,
            1.0,
            0.0,
            // w
            0.0,
            0.0,
            0.0,
            1.0,
        );

        let normalize_affine_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Renderer TexturedVertex Affine Bind Group"),
            layout: &self.affine_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Render TexturedVertex Affine Buffer"),
                        contents: bytemuck::cast_slice(normalize_matrix.as_slice()),
                        usage: wgpu::BufferUsages::UNIFORM,
                    }),
                    offset: 0,
                    size: None,
                }),
            }],
        });

        for (texture, vertex, index, matrix) in source {
            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Renderer TexturedVertex Texture Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.texture_sampler),
                    },
                ],
            });

            let vertex = vertex
                .iter()
                .map(|v| UvVertex {
                    position: matrix.transform_point(&v.position),
                    tex_coords: v.tex_coords,
                })
                .collect::<Vec<_>>();

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Renderer TexturedVertex Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertex),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Renderer TexturedVertex Index Buffer"),
                contents: bytemuck::cast_slice(&index),
                usage: wgpu::BufferUsages::INDEX,
            });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Renderer TexturedVertex Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &destination_view,
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

                if render_to_surface {
                    render_pass.set_pipeline(&self.surface_render_pipeline);
                } else {
                    render_pass.set_pipeline(&self.render_pipeline);
                }
                render_pass.set_bind_group(0, &normalize_affine_bind_group, &[]);
                render_pass.set_bind_group(1, &texture_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..index.len() as u32, 0, 0..1);
            }
        }

        self.context
            .get_wgpu_queue()
            .submit(std::iter::once(encoder.finish()));
    }
}
