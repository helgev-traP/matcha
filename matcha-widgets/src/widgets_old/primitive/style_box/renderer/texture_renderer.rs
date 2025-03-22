use crate::vertex::vertex::Vertex;

pub struct TextureRenderer {
    // group 1
    texture_bind_group_layout: wgpu::BindGroupLayout,

    // group 2
    // viewport_info_bind_group_layout: wgpu::BindGroupLayout,

    // group 3
    // blur_settings_bind_group_layout: wgpu::BindGroupLayout,

    // pipeline
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,
}

impl TextureRenderer {
    pub fn new(device: &wgpu::Device, target_format: &[wgpu::TextureFormat]) -> Self {
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
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

        let viewport_info_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("viewport_info_bind_group_layout"),
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

        let blur_settings_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("blur_settings_bind_group_layout"),
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("texture_pipeline_layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &viewport_info_bind_group_layout,
                &blur_settings_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("texture_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("texture.wgsl").into()),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("texture_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
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
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: target_format
                    .iter()
                    .map(|format| {
                        Some(wgpu::ColorTargetState {
                            format: *format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })
                    })
                    .collect::<Vec<_>>()
                    .as_slice(),
            }),
            multiview: None,
            cache: None,
        });

        Self {
            texture_bind_group_layout,
            pipeline_layout,
            pipeline,
        }
    }
}

pub struct TextureResources<'a> {
    pub vertex_buffer_slice: wgpu::BufferSlice<'a>,
    pub indices_buffer_slice: wgpu::BufferSlice<'a>,
    pub texture_bind_group: &'a wgpu::BindGroup,
    pub viewport_info_bind_group: &'a wgpu::BindGroup,
    pub blur_settings_bind_group: &'a wgpu::BindGroup,
}

impl TextureRenderer {
    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        // target
        target: &wgpu::TextureView,
        // resources
        TextureResources {
            vertex_buffer_slice,
            indices_buffer_slice,
            texture_bind_group,
            viewport_info_bind_group,
            blur_settings_bind_group,
        }: TextureResources,
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("texture_command_encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("texture_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, texture_bind_group, &[]);
            render_pass.set_bind_group(1, viewport_info_bind_group, &[]);
            render_pass.set_bind_group(2, blur_settings_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer_slice);
            render_pass.set_index_buffer(indices_buffer_slice, wgpu::IndexFormat::Uint16);
            render_pass.draw(0..6, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
