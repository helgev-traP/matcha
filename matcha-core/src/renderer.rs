use std::sync::Arc;

use crate::ui::Object;
use texture_atlas::TextureError;
use thiserror::Error;

const PIPELINE_CACHE_SIZE: u64 = 3;
const COMPUTE_WORKGROUP_SIZE: u32 = 64;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceData {
    /// transform vertex: {[0, 0], [0, -1], [1, 0], [1, -1]} to where the texture should be rendered
    viewport_position: nalgebra::Matrix4<f32>,
    atlas_page: u32,
    /// [x, y]
    in_atlas_offset: [f32; 2],
    /// [width, height]
    in_atlas_size: [f32; 2],
    /// the index of the stencil in the stencil data array.
    /// 0 if no stencil is used. Use `stencil_index - 1` in the shader.
    stencil_index: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct StencilData {
    /// transform vertex: {[0, 0], [0, -1], [1, 0], [1, -1]} to where the stencil should be rendered
    viewport_position: nalgebra::Matrix4<f32>,
    /// if the inverse of the viewport position exists.
    /// 0 if the inverse does not exist.
    viewport_position_inverse_exists: u32,
    /// inverse of the viewport position matrix.
    /// used to calculate stencil uv coordinates in the shader.
    viewport_position_inverse: nalgebra::Matrix4<f32>,
    atlas_page: u32,
    /// [x, y]
    in_atlas_offset: [f32; 2],
    /// [width, height]
    in_atlas_size: [f32; 2],
}

pub struct Renderer {
    // Bind Group Layouts
    texture_sampler: wgpu::Sampler,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    data_bind_group_layout: wgpu::BindGroupLayout,

    // Pipeline Layouts
    culling_pipeline_layout: wgpu::PipelineLayout,
    command_pipeline_layout: wgpu::PipelineLayout,
    render_pipeline_layout: wgpu::PipelineLayout,
    render_pipeline_shader_module: wgpu::ShaderModule,

    // Pipelines
    culling_pipeline: wgpu::ComputePipeline,
    command_pipeline: wgpu::ComputePipeline,
    render_pipeline: moka::sync::Cache<wgpu::TextureFormat, Arc<wgpu::RenderPipeline>>, // key: surface format
}

impl Renderer {
    pub fn new(device: &wgpu::Device) -> Self {
        // Sampler
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ObjectRenderer Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ObjectRenderer Texture Bind Group Layout"),
                entries: &[
                    // Texture Sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // Texture Atlas
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Stencil Atlas
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        // Culling Pipeline
        let data_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Culling Bind Group Layout"),
                entries: &[
                    // All Instances Buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // All Stencils Buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Visible Instances Buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Atomic Counter
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // command buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let (culling_pipeline_layout, culling_pipeline) =
            Self::create_culling_pipeline(device, &data_bind_group_layout);

        let (command_pipeline_layout, command_pipeline) =
            Self::create_command_pipeline(device, &data_bind_group_layout);

        let (render_pipeline_layout, render_pipeline_shader_module) =
            Self::create_render_pipeline_layout(
                device,
                &texture_bind_group_layout,
                &data_bind_group_layout,
            );

        let render_pipeline = moka::sync::Cache::builder()
            .max_capacity(PIPELINE_CACHE_SIZE)
            .build();

        Self {
            texture_sampler,
            texture_bind_group_layout,
            data_bind_group_layout,
            culling_pipeline_layout,
            command_pipeline_layout,
            render_pipeline_layout,
            render_pipeline_shader_module,
            culling_pipeline,
            command_pipeline,
            render_pipeline,
        }
    }

    fn create_culling_pipeline(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> (wgpu::PipelineLayout, wgpu::ComputePipeline) {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Culling Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("renderer/renderer_cull.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Culling Pipeline Layout"),
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::COMPUTE,
                range: 0..std::mem::size_of::<nalgebra::Matrix4<f32>>() as u32,
            }],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Culling Pipeline"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: Some("culling_main"),
            compilation_options: Default::default(),
            cache: None,
        });

        (pipeline_layout, pipeline)
    }

    fn create_command_pipeline(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> (wgpu::PipelineLayout, wgpu::ComputePipeline) {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Command Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("renderer/renderer_command.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Command Pipeline Layout"),
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Command Pipeline"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: Some("command_main"),
            compilation_options: Default::default(),
            cache: None,
        });

        (pipeline_layout, pipeline)
    }

    fn create_render_pipeline_layout(
        device: &wgpu::Device,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        data_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> (wgpu::PipelineLayout, wgpu::ShaderModule) {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Render Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("renderer/renderer_render.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[texture_bind_group_layout, data_bind_group_layout],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                range: 0..std::mem::size_of::<nalgebra::Matrix4<f32>>() as u32,
            }],
        });

        (pipeline_layout, module)
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        render_pipeline_layout: &wgpu::PipelineLayout,
        shader_module: &wgpu::ShaderModule,
        target_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fragment_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
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

        pipeline
    }

    pub fn render(
        &self,
        // gpu
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        // surface format
        surface_format: wgpu::TextureFormat,
        // destination
        destination_view: &wgpu::TextureView,
        destination_size: [f32; 2],
        // objects
        objects: Object,
        load_color: wgpu::Color,
        // texture atlas
        texture_atlas: wgpu::Texture,
        stencil_atlas: wgpu::Texture,
    ) -> Result<(), TextureValidationError> {
        // get or create render pipeline that matches given surface format
        let render_pipeline = self.render_pipeline.get_with(surface_format, || {
            Arc::new(Self::create_render_pipeline(
                device,
                &self.render_pipeline_layout,
                &self.render_pipeline_shader_module,
                surface_format,
            ))
        });

        // integrate objects into a instance array
        let (instances, stencils) = create_instance_and_stencil_data(
            objects,
            texture_atlas.format(),
            stencil_atlas.format(),
        )?;

        if instances.is_empty() {
            return Ok(());
        }

        // Create buffers
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ObjectRenderer Instance Buffer"),
            size: (std::mem::size_of::<InstanceData>() * instances.len()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let stencil_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ObjectRenderer Stencil Buffer"),
            size: (std::mem::size_of::<StencilData>() * stencils.len()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let visible_instances_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ObjectRenderer Visible Instances Buffer"),
            size: (std::mem::size_of::<u32>() * instances.len()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let atomic_counter_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ObjectRenderer Atomic Counter Buffer"),
            size: std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let command_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ObjectRenderer Command Buffer"),
            size: (std::mem::size_of::<wgpu::util::DrawIndirectArgs>()) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind groups
        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ObjectRenderer Texture Bind Group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&self.texture_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_atlas.create_view(
                        &wgpu::TextureViewDescriptor {
                            dimension: Some(wgpu::TextureViewDimension::D2Array),
                            aspect: wgpu::TextureAspect::All,
                            ..Default::default()
                        },
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&stencil_atlas.create_view(
                        &wgpu::TextureViewDescriptor {
                            dimension: Some(wgpu::TextureViewDimension::D2Array),
                            aspect: wgpu::TextureAspect::All,
                            ..Default::default()
                        },
                    )),
                },
            ],
        });

        let data_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ObjectRenderer Data Bind Group"),
            layout: &self.data_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: instance_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: stencil_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: visible_instances_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: atomic_counter_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: command_buffer.as_entire_binding(),
                },
            ],
        });

        // already checked that instances is not empty
        queue.write_buffer(&instance_buffer, 0, bytemuck::cast_slice(&instances));

        if !stencils.is_empty() {
            queue.write_buffer(&stencil_buffer, 0, bytemuck::cast_slice(&stencils));
        }

        queue.write_buffer(&atomic_counter_buffer, 0, bytemuck::cast_slice(&[0u32]));
        queue.write_buffer(
            &command_buffer,
            0,
            wgpu::util::DrawIndirectArgs {
                vertex_count: 4,
                instance_count: 0,
                first_vertex: 0,
                first_instance: 0,
            }
            .as_bytes(),
        );

        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ObjectRenderer: Command Encoder"),
        });

        let normalize_matrix = make_normalize_matrix(destination_size);

        // culling compute pass
        {
            let mut culling_pass =
                command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("ObjectRenderer: Culling Pass"),
                    timestamp_writes: None,
                });
            culling_pass.set_pipeline(&self.culling_pipeline);
            culling_pass.set_bind_group(0, &data_bind_group, &[]);
            culling_pass.set_push_constants(0, bytemuck::cast_slice(normalize_matrix.as_slice()));
            culling_pass.dispatch_workgroups(
                (instances.len() as u32).div_ceil(COMPUTE_WORKGROUP_SIZE),
                1,
                1,
            );
        }

        // command encoding pass
        {
            let mut command_pass =
                command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("ObjectRenderer: Command Pass"),
                    timestamp_writes: None,
                });
            command_pass.set_pipeline(&self.command_pipeline);
            command_pass.set_bind_group(0, &data_bind_group, &[]);
            command_pass.dispatch_workgroups(1, 1, 1);
        }

        // render pass
        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ObjectRenderer: Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: destination_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(load_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(render_pipeline.as_ref());
            render_pass.set_bind_group(0, &texture_bind_group, &[]);
            render_pass.set_bind_group(1, &data_bind_group, &[]);
            render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                0,
                bytemuck::cast_slice(normalize_matrix.as_slice()),
            );
            render_pass.draw_indirect(&command_buffer, 0);
        }

        queue.submit(std::iter::once(command_encoder.finish()));

        Ok(())
    }
}

fn create_instance_and_stencil_data(
    objects: Object,
    texture_format: wgpu::TextureFormat,
    stencil_format: wgpu::TextureFormat,
) -> Result<(Vec<InstanceData>, Vec<StencilData>), TextureValidationError> {
    let mut instances = Vec::new();
    let mut stencils = Vec::new();

    let mut texture_atlas_id = None;
    let mut stencil_atlas_id = None;

    create_instance_and_stencil_data_recursive(
        texture_format,
        stencil_format,
        &objects,
        nalgebra::Matrix4::identity(),
        &mut instances,
        &mut stencils,
        &mut texture_atlas_id,
        &mut stencil_atlas_id,
        0,
    )?;

    Ok((instances, stencils))
}

fn create_instance_and_stencil_data_recursive(
    texture_format: wgpu::TextureFormat,
    stencil_format: wgpu::TextureFormat,
    object: &Object,
    transform: nalgebra::Matrix4<f32>,
    instances: &mut Vec<InstanceData>,
    stencils: &mut Vec<StencilData>,
    texture_atlas_id: &mut Option<texture_atlas::TextureAtlasId>,
    stencil_atlas_id: &mut Option<texture_atlas::TextureAtlasId>,
    // the index + 1 of the current stencil in the stencils vector.
    // 0 if no stencil is used.
    mut current_stencil: u32,
) -> Result<(), TextureValidationError> {
    if let Some((stencil, stencil_position)) = &object.stencil_and_position {
        if stencil.formats() != &[stencil_format] {
            return Err(TextureValidationError::FormatMismatch);
        }

        let atlas_id = stencil_atlas_id.get_or_insert_with(|| stencil.atlas_id());

        if atlas_id != &stencil.atlas_id() {
            return Err(TextureValidationError::AtlasIdMismatch);
        }

        let (page, position_in_atlas) = stencil.position_in_atlas()?;

        let stencil_position = transform * stencil_position;
        let (inverse_exists, stencil_position_inverse) = stencil_position
            .try_inverse()
            .map(|m| (true, m))
            .unwrap_or_else(|| (false, nalgebra::Matrix4::identity()));

        stencils.push(StencilData {
            viewport_position: stencil_position,
            viewport_position_inverse_exists: if inverse_exists { 1 } else { 0 },
            viewport_position_inverse: stencil_position_inverse,
            atlas_page: page,
            in_atlas_offset: [position_in_atlas.min_x(), position_in_atlas.min_y()],
            in_atlas_size: [position_in_atlas.width(), position_in_atlas.height()],
        });

        current_stencil = stencils.len() as u32;
    }

    if let Some((texture, texture_position)) = &object.texture_and_position {
        if texture.formats() != [texture_format] {
            return Err(TextureValidationError::FormatMismatch);
        }

        let atlas_id = texture_atlas_id.get_or_insert_with(|| texture.atlas_id());

        if atlas_id != &texture.atlas_id() {
            return Err(TextureValidationError::AtlasIdMismatch);
        }

        let (page, position_in_atlas) = texture.position_in_atlas()?;

        instances.push(InstanceData {
            viewport_position: transform * texture_position,
            atlas_page: page,
            in_atlas_offset: [position_in_atlas.min_x(), position_in_atlas.min_y()],
            in_atlas_size: [position_in_atlas.width(), position_in_atlas.height()],
            stencil_index: current_stencil,
        });
    }

    for (child, child_transform) in object.child_elements() {
        create_instance_and_stencil_data_recursive(
            texture_format,
            stencil_format,
            child,
            transform * child_transform,
            instances,
            stencils,
            texture_atlas_id,
            stencil_atlas_id,
            current_stencil,
        )?;
    }

    Ok(())
}

#[derive(Error, Debug)]
pub enum TextureValidationError {
    #[error("texture format mismatch")]
    FormatMismatch,
    #[error("texture atlas id mismatch")]
    AtlasIdMismatch,
    #[error("texture atlas error: {0}")]
    AtlasError(#[from] TextureError),
}

#[rustfmt::skip]
fn make_normalize_matrix(destination_size: [f32; 2]) -> nalgebra::Matrix4<f32> {
    nalgebra::Matrix4::new(
        2.0 / destination_size[0], 0.0, 0.0, -1.0,
        0.0, 2.0 / destination_size[1], 0.0, 1.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    )
}
