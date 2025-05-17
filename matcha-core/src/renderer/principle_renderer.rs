use crate::ui::Object;

mod texture_color_renderer;
use texture_color_renderer::TextureObjectRenderer;
mod vertex_color_renderer;
use vertex_color_renderer::VertexColorRenderer;

use super::Renderer;

#[derive(Default)]
pub struct PrincipleRenderer {
    // renderers
    texture_color_renderer: Option<TextureObjectRenderer>,
    vertex_color_renderer: Option<VertexColorRenderer>,
}

impl Renderer for PrincipleRenderer {
    fn setup(&mut self, device: &wgpu::Device, _: &wgpu::Queue, format: wgpu::TextureFormat) {
        let texture_object_renderer =
            texture_color_renderer::TextureObjectRenderer::new(device, format);
        let vertex_color_renderer = vertex_color_renderer::VertexColorRenderer::new(device, format);

        self.texture_color_renderer = Some(texture_object_renderer);
        self.vertex_color_renderer = Some(vertex_color_renderer);
    }
}

impl PrincipleRenderer {
    pub fn new() -> Self {
        Self::default()
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
    ) {
        self.render_impl(
            device,
            queue,
            destination_view,
            destination_size,
            objects,
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
    ) {
        self.render_impl(
            device,
            queue,
            destination_view,
            destination_size,
            objects,
            true,
        );
    }

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
        // render to surface or not
        render_to_surface: bool,
    ) {
        let normalize_matrix = make_normalize_matrix(destination_size);

        // todo !: try mesh integration

        for obj in objects {
            match obj {
                Object::TextureColor {
                    texture,
                    uv_vertices,
                    indices,
                    transform,
                } => {
                    if let Some(renderer) = &self.texture_color_renderer {
                        let uv_vertices = uv_vertices
                            .iter()
                            .map(|vertex| vertex.transform(&transform))
                            .collect::<Vec<_>>();

                        renderer.render(
                            device,
                            queue,
                            destination_view,
                            &normalize_matrix,
                            texture,
                            &uv_vertices,
                            indices,
                            render_to_surface,
                        );
                    }
                }
                Object::VertexColor {
                    vertices,
                    indices,
                    transform,
                } => {
                    if let Some(renderer) = &self.vertex_color_renderer {
                        let vertices = vertices
                            .iter()
                            .map(|vertex| vertex.transform(&transform))
                            .collect::<Vec<_>>();

                        renderer.render(
                            device,
                            queue,
                            destination_view,
                            &normalize_matrix,
                            &vertices,
                            indices,
                            render_to_surface,
                        );
                    }
                }
            }
        }
    }
}

fn make_normalize_matrix(destination_size: [f32; 2]) -> nalgebra::Matrix4<f32> {
    nalgebra::Matrix4::new(
        // x
        2.0 / destination_size[0],
        0.0,
        0.0,
        -1.0,
        // y
        0.0,
        2.0 / destination_size[1],
        0.0,
        1.0,
        // z
        0.0,
        0.0,
        1.0,
        0.0,
        // w
        0.0,
        0.0,
        0.0,
        1.0,
    )
}
