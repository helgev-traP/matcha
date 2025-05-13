use texture_color_renderer::TextureObjectRenderer;
use vertex_color_renderer::VertexColorRenderer;

use crate::{context::SharedContext, ui::Object};

mod texture_color_renderer;
mod vertex_color_renderer;

pub struct Renderer {
    // context
    context: SharedContext,

    // vello renderer
    // todo: remove
    vello_renderer: std::sync::Mutex<vello::Renderer>,

    // renderers
    texture_color_renderer: Option<TextureObjectRenderer>,
    vertex_color_renderer: Option<VertexColorRenderer>,
}

impl Renderer {
    pub fn new(context: &SharedContext) -> Self {
        let device = context.get_wgpu_device();

        // vello renderer
        let vello_renderer = vello::Renderer::new(
            device,
            vello::RendererOptions {
                surface_format: Some(wgpu::TextureFormat::Rgba8Unorm),
                use_cpu: false,
                antialiasing_support: vello::AaSupport::all(),
                num_init_threads: None,
            },
        )
        .unwrap();

        // pipelines

        let texture_object_renderer = texture_color_renderer::TextureObjectRenderer::new(
            device,
            context.get_surface_format(),
        );

        let vertex_color_renderer =
            vertex_color_renderer::VertexColorRenderer::new(device, context.get_surface_format());

        Self {
            context: context.clone(),
            vello_renderer: std::sync::Mutex::new(vello_renderer),
            texture_color_renderer: Some(texture_object_renderer),
            vertex_color_renderer: Some(vertex_color_renderer),
        }
    }

    pub fn vello_renderer(&self) -> std::sync::MutexGuard<vello::Renderer> {
        self.vello_renderer.lock().unwrap()
    }

    pub fn render(
        &self,
        destination_view: &wgpu::TextureView,
        destination_size: [f32; 2],
        // objects
        objects: Vec<Object>,
    ) {
        self.render_impl(destination_view, destination_size, objects, false);
    }

    pub(crate) fn render_to_surface(
        &self,
        destination_view: &wgpu::TextureView,
        destination_size: [f32; 2],
        // objects
        objects: Vec<Object>,
    ) {
        self.render_impl(destination_view, destination_size, objects, true);
    }

    fn render_impl(
        &self,
        destination_view: &wgpu::TextureView,
        destination_size: [f32; 2],
        // objects
        objects: Vec<Object>,
        // render to surface or not
        render_to_surface: bool,
    ) {
        let device = self.context.get_wgpu_device();
        let queue = self.context.get_wgpu_queue();

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
                            &indices,
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
                            &indices,
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
