use std::sync::{Arc, Mutex};

use wgpu::{util::DeviceExt, TextureUsages};

use crate::{
    context::SharedContext,
    types::{color::Color, size::PxSize},
    vertex::{colored_vertex::ColoredVertex, textured_vertex::TexturedVertex},
    widgets::text,
};

/// Render to Rgba8UnormSrgb texture
pub struct Renderer {
    // context
    context: SharedContext,

    // wgpu state
    // texture must be Rgba8UnormSrgb
    // texture projection

    // texture
    texture_bind_group_layout: wgpu::BindGroupLayout,
    textured_render_pipeline: wgpu::RenderPipeline,

    // color
    color_bind_group_layout: wgpu::BindGroupLayout,
    colored_render_pipeline: wgpu::RenderPipeline,

    // stencil
    stencil_bind_group_layout: wgpu::BindGroupLayout,

    // affine
    affine_bind_group_layout: wgpu::BindGroupLayout,
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

        let stencil_bind_group_layout =
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
                    buffers: &[TexturedVertex::desc()],
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
                    count: 4,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

        let color_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Panel Affine Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let colored_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Panel Render Pipeline Layout"),
                bind_group_layouts: &[&affine_bind_group_layout, &color_bind_group_layout],
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
                    buffers: &[ColoredVertex::desc()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &colored_shader,
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
                    count: 4,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

        Self {
            context,
            texture_bind_group_layout,
            textured_render_pipeline,
            color_bind_group_layout,
            colored_render_pipeline,
            stencil_bind_group_layout,
            affine_bind_group_layout,
        }
    }

    pub fn project(
        &self,
        destination: &wgpu::Texture,
        source: &wgpu::Texture,
        stencil: &wgpu::Texture,
    ) {
        // return if STORAGE_BINDING are not set
        if destination.usage().contains(TextureUsages::STORAGE_BINDING)
            || source.usage().contains(TextureUsages::STORAGE_BINDING)
        {
            return;
        }

        let device = self.context.get_wgpu_device();

        // texture bind group
        let texture_source_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Projection Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadOnly,
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });

        let texture_destination_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Projection Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });

        let stencil_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Projection Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadOnly,
                        format: wgpu::TextureFormat::R8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });

        let texture_source_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_source_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &source.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            }],
            label: Some("Texture Projection Bind Group (Source)"),
        });

        let texture_destination_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_destination_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &destination.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            }],
            label: Some("Texture Projection Bind Group (Destination)"),
        });

        let stencil_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &stencil_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &stencil.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            }],
            label: Some("Texture Projection Bind Group (Stencil)"),
        });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Texture Projection Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_source_bind_group_layout,
                    &texture_destination_bind_group_layout,
                    &stencil_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Texture Projection Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("render_projection.wgsl").into()),
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Texture Projection Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Projection Command Encoder"),
        });

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Projection Compute Pass"),
                timestamp_writes: None,
            });

            cpass.set_pipeline(&compute_pipeline);
            cpass.set_bind_group(0, &texture_source_bind_group, &[]);
            cpass.set_bind_group(1, &texture_destination_bind_group, &[]);
            cpass.set_bind_group(2, &stencil_bind_group, &[]);

            cpass.dispatch_workgroups(32, 32, 1);
        }
    }
}
