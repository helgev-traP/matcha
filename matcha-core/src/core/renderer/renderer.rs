use std::sync::Arc;
use texture_color_renderer::TextureObjectRenderer;
use wgpu::util::DeviceExt;

use crate::{
    context::SharedContext,
    ui::{Object, TextureColor},
    vertex::uv_vertex,
};

mod texture_color_renderer;

pub struct Renderer {
    // context
    context: SharedContext,

    // vello renderer
    // todo: remove
    vello_renderer: std::sync::Mutex<vello::Renderer>,

    // renderers
    texture_object_renderer: Option<TextureObjectRenderer>,
}

impl Renderer {
    pub fn new(context: SharedContext) -> Self {
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

        Self {
            context,
            vello_renderer: std::sync::Mutex::new(vello_renderer),
            texture_object_renderer: Some(texture_object_renderer),
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
        objects: Object,
    ) {
        self.render_impl(destination_view, destination_size, objects, false);
    }

    pub(crate) fn render_to_surface(
        &self,
        destination_view: &wgpu::TextureView,
        destination_size: [f32; 2],
        // objects
        objects: Object,
    ) {
        self.render_impl(destination_view, destination_size, objects, true);
    }

    fn render_impl(
        &self,
        destination_view: &wgpu::TextureView,
        destination_size: [f32; 2],
        // objects
        objects: Object,
        // render to surface or not
        render_to_surface: bool,
    ) {
        let device = self.context.get_wgpu_device();
        let queue = self.context.get_wgpu_queue();

        let normalize_matrix = nalgebra::Matrix4::new(
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
        );

        // todo !: try mesh integration

        if let Some(texture_object_renderer) = &self.texture_object_renderer {
            for TextureColor {
                texture,
                uv_vertices,
                indices,
                transform,
            } in objects.texture_color
            {
                // transform
                let uv_vertices = uv_vertices
                    .into_iter()
                    .map(|uv_vertex| uv_vertex.transform(&transform))
                    .collect::<Vec<_>>();

                // render
                texture_object_renderer.render(
                    device,
                    queue,
                    destination_view,
                    normalize_matrix,
                    texture,
                    uv_vertices,
                    indices,
                    render_to_surface,
                );
            }
        }
    }
}
