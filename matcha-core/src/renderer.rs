use crate::{context::WidgetContext, ui::Object};

pub mod texture_copy;

mod texture_color_renderer;
use texture_color_renderer::TextureObjectRenderer;
mod vertex_color_renderer;
use vertex_color_renderer::VertexColorRenderer;

pub struct PrincipleRenderer {
    // renderers
    texture_color_renderer: TextureObjectRenderer,
    vertex_color_renderer: VertexColorRenderer,
}

impl PrincipleRenderer {}

impl PrincipleRenderer {
    pub fn new(ctx: &WidgetContext) -> Self {
        let texture_color_renderer = TextureObjectRenderer::new(ctx.device(), ctx.surface_format());
        let vertex_color_renderer = VertexColorRenderer::new(ctx.device(), ctx.surface_format());

        Self {
            texture_color_renderer,
            vertex_color_renderer,
        }
    }

    pub fn render(
        &self,
        // gpu
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        // destination
        destination_view: &wgpu::TextureView,
        destination_size: [f32; 2],
        // objects
        objects: Vec<Object>,
        offset: Option<nalgebra::Matrix4<f32>>,
    ) {
        self.render_impl(
            device,
            queue,
            destination_view,
            destination_size,
            objects,
            offset,
            false,
        );
    }

    pub(crate) fn render_to_surface(
        &self,
        // gpu
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        // destination
        destination_view: &wgpu::TextureView,
        destination_size: [f32; 2],
        // objects
        objects: Vec<Object>,
        offset: Option<nalgebra::Matrix4<f32>>,
    ) {
        self.render_impl(
            device,
            queue,
            destination_view,
            destination_size,
            objects,
            offset,
            true,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn render_impl(
        &self,
        // gpu
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        // destination
        destination_view: &wgpu::TextureView,
        destination_size: [f32; 2],
        // objects
        objects: Vec<Object>,
        offset: Option<nalgebra::Matrix4<f32>>,
        // render to surface or not
        render_to_surface: bool,
    ) {
        let normalize_matrix = make_normalize_matrix(destination_size);

        let composed_matrix = offset.map_or(normalize_matrix, |offset| normalize_matrix * offset);

        // todo !: try mesh integration

        {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("VColorObjectRenderer: Command Encoder"),
            });

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("VColorObjectRenderer: Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: destination_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            for Object {
                texture,
                uv_vertices,
                indices,
                transform,
            } in objects
            {
                {
                    let uv_vertices = uv_vertices
                        .iter()
                        .map(|vertex| vertex.transform(&transform))
                        .collect::<Vec<_>>();

                    self.texture_color_renderer.render(
                        device,
                        &mut render_pass,
                        &composed_matrix,
                        texture,
                        &uv_vertices,
                        &indices,
                        render_to_surface,
                    );
                }
            }
        }
    }
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
