/*
push constants:
    [[f32; 4]; 4] // composed affine matrix
*/

use crate::vertex::ColorVertex;
use matcha_core::ui::WidgetContext;
use wgpu::{PipelineCompilationOptions, util::DeviceExt};

pub struct LineStripColor {
    inner: utils::RwOption<LineStripColorImpl>,
}

const PIPELINE_CACHE_SIZE: u64 = 4;

struct LineStripColorImpl {
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: moka::sync::Cache<wgpu::TextureFormat, wgpu::RenderPipeline, fxhash::FxBuildHasher>,
}

impl LineStripColorImpl {
    fn setup(ctx: &WidgetContext) -> Self {
        let device = ctx.device();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("line_strip_pipeline_layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX,
                range: 0..(std::mem::size_of::<nalgebra::Matrix4<f32>>() as u32),
            }],
        });

        let pipeline = moka::sync::CacheBuilder::new(PIPELINE_CACHE_SIZE)
            .build_with_hasher(fxhash::FxBuildHasher::default());

        Self {
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
    pub position: [f32; 2],
    pub vertices: &'a [ColorVertex],
}

impl LineStripColor {
    pub fn render(
        &self,
        render_pass: &mut wgpu::RenderPass<'_>,
        TargetData {
            target_size,
            target_format,
        }: TargetData,
        RenderData { position, vertices }: RenderData,
        ctx: &WidgetContext,
    ) {
        let LineStripColorImpl {
            pipeline_layout,
            pipeline,
        } = &*self
            .inner
            .get_or_insert_with(|| LineStripColorImpl::setup(ctx));

        let render_pipeline = pipeline.get_with(target_format, || {
            make_pipeline(ctx, target_format, pipeline_layout)
        });

        let view_port_affine_transform =
            affine_transform([target_size[0] as f32, target_size[1] as f32], position);

        let device = ctx.device();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("line_strip_vertex_buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        render_pass.set_pipeline(&render_pipeline);
        render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX,
            0,
            bytemuck::cast_slice(view_port_affine_transform.as_slice()),
        );
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..vertices.len() as u32, 0..1);
    }
}

#[rustfmt::skip]
fn affine_transform(
    viewport_size: [f32; 2],
    position: [f32; 2],
) -> nalgebra::Matrix4<f32> {
    let position = nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(
        position[0],
        position[1],
        0.0,
    ));

    let transform = nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(
        -1.0,
        1.0,
        0.0,
    ));

    let scale = nalgebra::Matrix4::new_nonuniform_scaling(
        &nalgebra::Vector3::new(
            2.0 / viewport_size[0],
            -2.0 / viewport_size[1],
            1.0,
        ),
    );

    transform * scale * position
}

fn make_pipeline(
    ctx: &WidgetContext,
    target_format: wgpu::TextureFormat,
    pipeline_layout: &wgpu::PipelineLayout,
) -> wgpu::RenderPipeline {
    let device = ctx.device();

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("line_strip_shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("line_strip.wgsl").into()),
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("line_strip_pipeline"),
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[ColorVertex::desc()],
            compilation_options: PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(target_format.into())],
            compilation_options: PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::LineStrip,
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
