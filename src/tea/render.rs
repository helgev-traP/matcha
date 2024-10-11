use std::sync::{Arc, Mutex};

use nalgebra as na;
use wgpu::util::DeviceExt;

use super::{
    application_context::ApplicationContext,
    types::{color::Color, size::PxSize},
    ui::{Object, RenderItem},
};

pub struct Render {
    // context
    app_context: ApplicationContext,

    // wgpu state
    // texture
    texture_bind_group_layout: wgpu::BindGroupLayout,
    textured_render_pipeline: wgpu::RenderPipeline,

    // color
    colored_render_pipeline: wgpu::RenderPipeline,

    // common
    affine_bind_group_layout: wgpu::BindGroupLayout,
}

impl Render {
    pub fn new(context: ApplicationContext) -> Self {
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
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
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
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

        Self {
            app_context: context,
            texture_bind_group_layout,
            textured_render_pipeline,
            colored_render_pipeline,
            affine_bind_group_layout,
        }
    }

    pub fn encoder(
        &self,
        target_texture_view: wgpu::TextureView,
        depth_texture_view: wgpu::TextureView,
        size: PxSize,
    ) -> RenderCommandEncoder {
        let encoder = Arc::new(Mutex::new(self.app_context.get_wgpu_encoder()));

        let normalizer = size.make_normalizer();

        RenderCommandEncoder {
            app_context: &self.app_context,
            size,
            texture_bind_group_layout: &self.texture_bind_group_layout,
            textured_render_pipeline: &self.textured_render_pipeline,
            colored_render_pipeline: &self.colored_render_pipeline,
            affine_bind_group_layout: &self.affine_bind_group_layout,
            encoder,
            target_texture_view,
            depth_texture_view,
            normalizer,
        }
    }
}

pub struct RenderCommandEncoder<'a> {
    // context
    app_context: &'a ApplicationContext,
    size: PxSize,

    // wgpu state
    // texture
    texture_bind_group_layout: &'a wgpu::BindGroupLayout,
    textured_render_pipeline: &'a wgpu::RenderPipeline,

    // color
    colored_render_pipeline: &'a wgpu::RenderPipeline,

    // common
    affine_bind_group_layout: &'a wgpu::BindGroupLayout,

    // encoder
    encoder: Arc<Mutex<wgpu::CommandEncoder>>,
    target_texture_view: wgpu::TextureView,
    depth_texture_view: wgpu::TextureView,

    // linear transformation
    normalizer: na::Matrix4<f32>,
}

impl<'a> RenderCommandEncoder<'a> {
    pub fn get_context(&self) -> &ApplicationContext {
        self.app_context
    }

    pub fn get_size(&self) -> PxSize {
        self.size
    }

    pub fn clear(&mut self, color: Color) {
        let mut encoder = self.encoder.lock().unwrap();

        let base_color_f64 = color.to_rgba_f64();

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Panel Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.target_texture_view,
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
    }

    pub fn draw(&mut self, item: RenderItem, affine: na::Matrix4<f32>) {
        let device = self.app_context.get_wgpu_device();
        for object in item.object {
            match object {
                Object::Textured {
                    instance_affine,
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
                        layout: &self.texture_bind_group_layout,
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
                        layout: &self.affine_bind_group_layout,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &device.create_buffer_init(
                                    &wgpu::util::BufferInitDescriptor {
                                        label: Some("Render TexturedVertex Affine Buffer"),
                                        contents: bytemuck::cast_slice(
                                            (self.normalizer * affine * instance_affine).as_slice(),
                                        ),
                                        usage: wgpu::BufferUsages::UNIFORM
                                            | wgpu::BufferUsages::COPY_DST,
                                    },
                                ),
                                offset: 0,
                                size: None,
                            }),
                        }],
                    });

                    {
                        let mut encoder = self.encoder.lock().unwrap();

                        let mut render_pass =
                            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("Renderer TexturedVertex Render Pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &self.target_texture_view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Load,
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: Some(
                                    wgpu::RenderPassDepthStencilAttachment {
                                        view: &self.depth_texture_view,
                                        depth_ops: Some(wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(1.0),
                                            store: wgpu::StoreOp::Store,
                                        }),
                                        stencil_ops: None,
                                    },
                                ),
                                timestamp_writes: None,
                                occlusion_query_set: None,
                            });

                        render_pass.set_pipeline(&self.textured_render_pipeline);
                        render_pass.set_bind_group(0, &affine_bind_group, &[]);
                        render_pass.set_bind_group(1, &texture_bind_group, &[]);
                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        render_pass
                            .set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                        render_pass.draw_indexed(0..index_len, 0, 0..1);
                    }
                }
                Object::Colored {
                    instance_affine,
                    vertex_buffer,
                    index_buffer,
                    index_len,
                } => {
                    let affine_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Renderer ColoredVertex Affine Bind Group"),
                        layout: &self.affine_bind_group_layout,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &device.create_buffer_init(
                                    &wgpu::util::BufferInitDescriptor {
                                        label: Some("Render ColoredVertex Affine Buffer"),
                                        contents: bytemuck::cast_slice(
                                            (self.normalizer * affine * instance_affine).as_slice(),
                                        ),
                                        usage: wgpu::BufferUsages::UNIFORM
                                            | wgpu::BufferUsages::COPY_DST,
                                    },
                                ),
                                offset: 0,
                                size: None,
                            }),
                        }],
                    });

                    {
                        let mut encoder = self.encoder.lock().unwrap();

                        let mut render_pass =
                            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("Renderer ColoredVertex Render Pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &self.target_texture_view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Load,
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: Some(
                                    wgpu::RenderPassDepthStencilAttachment {
                                        view: &self.depth_texture_view,
                                        depth_ops: Some(wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(1.0),
                                            store: wgpu::StoreOp::Store,
                                        }),
                                        stencil_ops: None,
                                    },
                                ),
                                timestamp_writes: None,
                                occlusion_query_set: None,
                            });

                        render_pass.set_pipeline(&self.colored_render_pipeline);
                        render_pass.set_bind_group(0, &affine_bind_group, &[]);
                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        render_pass
                            .set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                        render_pass.draw_indexed(0..index_len, 0, 0..1);
                    }
                }
            }
        }
    }

    pub fn finish(self) -> Result<(), ()> {
        match Arc::try_unwrap(self.encoder) {
            Ok(encoder) => {
                let encoder = encoder.into_inner().unwrap();
                let queue = self.app_context.get_wgpu_queue();
                queue.submit(std::iter::once(encoder.finish()));
                Ok(())
            }
            Err(_) => Err(()),
        }
    }
}
