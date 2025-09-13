use utils::rwoption::RwOption;
use wgpu::PipelineCompilationOptions;

/// Simple renderer that overwrites a scissored rectangle with a transparent color.
/// API mirrors other widgets_renderer modules: create a small struct with an inner impl and a `render` method.
#[derive(Default)]
pub struct ViewportClear {
    inner: RwOption<RectClearImpl>,
}

struct RectClearImpl {
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,
}

impl RectClearImpl {
    fn setup(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("viewport_clear_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("viewport_clear.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("viewport_clear_pipeline_layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("viewport_clear_pipeline"),
            layout: Some(&pipeline_layout),
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
                    format,
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
        });

        RectClearImpl {
            pipeline_layout,
            pipeline,
        }
    }
}

/// rect: [x, y, width, height] in pixels (u32)
pub fn render(
    rect_clear: &ViewportClear,
    render_pass: &mut wgpu::RenderPass<'_>,
    target_format: wgpu::TextureFormat,
    rect: [u32; 4],
    device: &wgpu::Device,
) {
    let impl_ref = rect_clear
        .inner
        .get_or_insert_with(|| RectClearImpl::setup(device, target_format));

    render_pass.set_pipeline(&impl_ref.pipeline);
    // scissor rect: x, y, width, height
    render_pass.set_scissor_rect(rect[0], rect[1], rect[2], rect[3]);
    // draw a triangle strip quad generated in the vertex shader
    render_pass.draw(0..4, 0..1);
}
