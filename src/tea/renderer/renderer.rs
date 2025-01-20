use std::sync::Arc;
use wgpu::util::DeviceExt;

use crate::{context::SharedContext, ui::Object};

mod texture_object_renderer;

pub struct Renderer {
    // context
    context: SharedContext,

    // vello renderer
    vello_renderer: std::sync::Mutex<vello::Renderer>,

    // renderers
    texture_object_renderer: texture_object_renderer::TextureObjectRenderer,
}

impl Renderer {
    pub fn new(context: SharedContext) -> Self {
        let device = context.get_wgpu_device();

        // vello renderer
        let vello_renderer = vello::Renderer::new(
            &device,
            vello::RendererOptions {
                surface_format: Some(wgpu::TextureFormat::Rgba8Unorm),
                use_cpu: false,
                antialiasing_support: vello::AaSupport::all(),
                num_init_threads: None,
            },
        )
        .unwrap()
        .into();

        // pipelines

        let texture_object_renderer = texture_object_renderer::TextureObjectRenderer::new(
            &device,
            context.get_surface_format(),
        );

        Self {
            context,
            vello_renderer: std::sync::Mutex::new(vello_renderer),
            texture_object_renderer,
        }
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

        for object in objects {
            match object {
                Object::TextureObject(texture_object) => {
                    self.texture_object_renderer.render(
                        destination_view,
                        normalize_matrix,
                        texture_object.texture,
                        texture_object.uv_vertices,
                        texture_object.indices,
                        render_to_surface,
                        device,
                        queue,
                    );
                }
                Object::TextureBlur(texture_blur) => {
                    todo!()
                }
            }
        }
    }
}
