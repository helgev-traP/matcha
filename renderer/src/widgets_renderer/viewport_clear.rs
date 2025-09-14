use utils::rwoption::RwOption;
use wgpu::PipelineCompilationOptions;

/// Simple renderer that overwrites a scissored rectangle with a transparent color.
/// API mirrors other widgets_renderer modules: create a small struct with an inner impl and a `render` method.
#[derive(Default)]
pub struct ViewportClear {
    inner: RwOption<ViewportClearImpl>,
}

const PIPELINE_CACHE_SIZE: u64 = 4;

struct ViewportClearImpl {
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: moka::sync::Cache<wgpu::TextureFormat, wgpu::RenderPipeline, fxhash::FxBuildHasher>,
}

impl ViewportClearImpl {
    fn setup(device: &wgpu::Device) -> Self {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("viewport_clear_pipeline_layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = moka::sync::CacheBuilder::new(PIPELINE_CACHE_SIZE)
            .build_with_hasher(fxhash::FxBuildHasher::default());

        ViewportClearImpl {
            pipeline_layout,
            pipeline,
        }
    }
}

impl ViewportClear {
    pub fn render(
        &self,
        render_pass: &mut wgpu::RenderPass<'_>,
        target_format: wgpu::TextureFormat,
        device: &wgpu::Device,
    ) {
        let ViewportClearImpl {
            pipeline_layout,
            pipeline,
        } = &*self
            .inner
            .get_or_insert_with(|| ViewportClearImpl::setup(device));

        let pipeline = pipeline.get_with(target_format, || {
            make_pipeline(device, target_format, pipeline_layout)
        });

        render_pass.set_pipeline(&pipeline);
        render_pass.draw(0..4, 0..1);
    }
}

fn make_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    pipeline_layout: &wgpu::PipelineLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("viewport_clear_shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("viewport_clear.wgsl").into()),
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("viewport_clear_pipeline"),
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
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}
