use crate::ui::WidgetContext;
use nalgebra::Matrix4;
use wgpu::PipelineCompilationOptions;

/*
bind group 0:
    texture
    sampler
push constants:
    [f32; 2] // target texture size
    [[f32; 2]; 2] // source texture position relative to target texture. [[x_min, y_min], [x_max, y_max]]
    [[f32; 4]; 4] // color transformation matrix (optional, can be used for color adjustments)
    [f32; 4] // color offset
*/

// vertex position will be calculated as:
// position = (position * 2.0) / target_texture_size + [-1.0, 1.0]

// color will be calculated as:
// color = color_transformation * source_color + color_offset

const RANGE_TEXTURE_SIZE: u32 = std::mem::size_of::<[f32; 2]>() as u32;
const RANGE_SRC_POSITION: u32 = RANGE_TEXTURE_SIZE + std::mem::size_of::<[[f32; 2]; 2]>() as u32;
const RANGE_COL_TRANSFORM: u32 = RANGE_SRC_POSITION + std::mem::size_of::<[[f32; 4]; 4]>() as u32;
const RANGE_COL_OFFSET: u32 = RANGE_COL_TRANSFORM + std::mem::size_of::<[f32; 4]>() as u32;

/// Copy texture data from one texture to another in a wgpu pipeline,
/// with offset and size parameters.
#[derive(Default)]
pub struct TextureCopy {
    inner: utils::RwOption<TextureCopyImpl>,
}

const PIPELINE_CACHE_SIZE: u64 = 4;

struct TextureCopyImpl {
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_sampler: wgpu::Sampler,
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: moka::sync::Cache<wgpu::TextureFormat, wgpu::RenderPipeline, fxhash::FxBuildHasher>,
}

impl TextureCopyImpl {
    fn setup(ctx: &WidgetContext) -> Self {
        let device = ctx.device();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_copy_bind_group_layout"),
                entries: &[
                    // texture
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
                    // sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("texture_copy_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("texture_copy_pipeline_layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX,
                    range: 0..RANGE_TEXTURE_SIZE,
                },
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX,
                    range: RANGE_TEXTURE_SIZE..RANGE_SRC_POSITION,
                },
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::FRAGMENT,
                    range: RANGE_SRC_POSITION..RANGE_COL_TRANSFORM,
                },
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::FRAGMENT,
                    range: RANGE_COL_TRANSFORM..RANGE_COL_OFFSET,
                },
            ],
        });

        let pipeline = moka::sync::CacheBuilder::new(PIPELINE_CACHE_SIZE)
            .build_with_hasher(fxhash::FxBuildHasher::default());

        TextureCopyImpl {
            texture_bind_group_layout,
            texture_sampler,
            pipeline_layout,
            pipeline,
        }
    }
}

pub struct TargetData {
    pub target_size: [u32; 2],
    pub target_format: wgpu::TextureFormat,
}

pub struct RenderData<'a> {
    pub source_texture_view: &'a wgpu::TextureView,
    /// y-axis heads upward, x-axis heads rightward.
    ///
    /// `[[x_min, y_min], [x_max, y_max]]`
    pub source_texture_position: [[f32; 2]; 2],
    pub color_transformation: Option<Matrix4<f32>>,
    pub color_offset: Option<[f32; 4]>,
}

impl TextureCopy {
    pub fn render(
        &self,
        render_pass: &mut wgpu::RenderPass<'_>,
        TargetData {
            target_size,
            target_format,
        }: TargetData,
        RenderData {
            source_texture_view: source_texture,
            source_texture_position,
            color_transformation,
            color_offset,
        }: RenderData<'_>,
        ctx: &WidgetContext,
    ) {
        let TextureCopyImpl {
            texture_bind_group_layout,
            texture_sampler,
            pipeline_layout,
            pipeline,
        } = &*self
            .inner
            .get_or_insert_with(|| TextureCopyImpl::setup(ctx));

        let render_pipeline = pipeline.get_with(target_format, || {
            make_pipeline(ctx, target_format, pipeline_layout)
        });

        render_pass.set_pipeline(&render_pipeline);
        render_pass.set_bind_group(
            0,
            &ctx.device().create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("TextureCopyBindGroup"),
                layout: texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(source_texture),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(texture_sampler),
                    },
                ],
            }),
            &[],
        );
        render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX,
            0,
            bytemuck::cast_slice(&[target_size[0] as f32, target_size[1] as f32]),
        );
        render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX,
            RANGE_TEXTURE_SIZE,
            bytemuck::cast_slice(&source_texture_position),
        );
        render_pass.set_push_constants(
            wgpu::ShaderStages::FRAGMENT,
            RANGE_SRC_POSITION,
            bytemuck::cast_slice(
                color_transformation
                    .unwrap_or(Matrix4::identity())
                    .as_slice(),
            ),
        );
        render_pass.set_push_constants(
            wgpu::ShaderStages::FRAGMENT,
            RANGE_COL_TRANSFORM,
            bytemuck::cast_slice(&color_offset.unwrap_or([0.0, 0.0, 0.0, 0.0])),
        );
        render_pass.draw(0..4, 0..1);
    }
}

fn make_pipeline(
    ctx: &WidgetContext,
    target_format: wgpu::TextureFormat,
    pipeline_layout: &wgpu::PipelineLayout,
) -> wgpu::RenderPipeline {
    let device = ctx.device();

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("texture_copy_shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("texture_copy.wgsl").into()),
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("texture_copy_pipeline"),
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            cull_mode: None,
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
    })
}
