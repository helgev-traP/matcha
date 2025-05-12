use std::sync::Arc;
use wgpu::util::DeviceExt;

use crate::vertex::uv_vertex::UvVertex;

// bind groups:
// - affine transform
// - texture

pub struct TextureObjectRenderer {
    texture_bind_group_layout: wgpu::BindGroupLayout,
    affine_bind_group_layout: wgpu::BindGroupLayout,
    _pipeline_layout: wgpu::PipelineLayout,
    render_pipeline: wgpu::RenderPipeline,
    render_pipeline_surface: wgpu::RenderPipeline,
    texture_sampler: wgpu::Sampler,
}

impl TextureObjectRenderer {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let affine_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("TextureObjectRenderer: Affine Bind Group Layout"),
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

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("TextureObjectRenderer: Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("TextureObjectRenderer: Pipeline Layout"),
            bind_group_layouts: &[&affine_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("TextureObjectRenderer: Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("texture_color_renderer.wgsl").into()),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("TextureObjectRenderer: Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[UvVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
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

        let render_pipeline_surface =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("TextureObjectRenderer: Render Pipeline(Surface)"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[UvVertex::desc()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
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

        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("TextureObjectRenderer: Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture_bind_group_layout,
            affine_bind_group_layout,
            _pipeline_layout: pipeline_layout,
            render_pipeline,
            render_pipeline_surface,
            texture_sampler,
        }
    }

    /// Use this after mesh integration.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        // gpu
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        // render target
        destination_view: &wgpu::TextureView,
        normalize_matrix: nalgebra::Matrix4<f32>,
        // object
        texture: Arc<wgpu::Texture>,
        vertex: Vec<UvVertex>,
        index: Vec<u16>,
        // render to surface or not
        render_to_surface: bool,
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("TextureObjectRenderer: Render Command Encoder"),
        });

        let normalize_affine_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("TextureObjectRenderer: Affine Bind Group"),
            layout: &self.affine_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("TextureObjectRenderer: Affine Buffer"),
                        contents: bytemuck::cast_slice(normalize_matrix.as_slice()),
                        usage: wgpu::BufferUsages::UNIFORM,
                    }),
                    offset: 0,
                    size: None,
                }),
            }],
        });

        // texture bind group

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("TextureObjectRenderer: Texture Bind Group"),
            layout: &self.texture_bind_group_layout,
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

        // vertex

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("TextureObjectRenderer: Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // index

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("TextureObjectRenderer: Index Buffer"),
            contents: bytemuck::cast_slice(&index),
            usage: wgpu::BufferUsages::INDEX,
        });

        // render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("TextureObjectRenderer: Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: destination_view,
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

            render_pass.set_pipeline(if render_to_surface {
                &self.render_pipeline_surface
            } else {
                &self.render_pipeline
            });

            render_pass.set_bind_group(0, &normalize_affine_bind_group, &[]);
            render_pass.set_bind_group(1, &texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..index.len() as u32, 0, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
