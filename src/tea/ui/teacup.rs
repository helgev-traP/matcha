use std::{
    any::{Any, TypeId},
    sync::Arc,
};
use wgpu::util::DeviceExt;

use crate::{
    application_context::ApplicationContext,
    events::WidgetEvent,
    types::size::{ParentPxSize, PxSize},
    vertex,
};

use super::{DomComPareResult, DomNode, RenderNode, RenderObject};

pub struct Teacup {
    size: crate::types::size::Size,
    position: [f32; 2],
    rotate_dig: f32,
}

impl Teacup {
    pub fn new() -> Self {
        Self {
            size: crate::types::size::Size {
                width: crate::types::size::SizeUnit::Pixel(100.0),
                height: crate::types::size::SizeUnit::Pixel(100.0),
            },
            position: [0.0, 0.0],
            rotate_dig: 0.0,
        }
    }

    pub fn size(mut self, size: crate::types::size::Size) -> Self {
        self.size = size;
        self
    }

    pub fn position(mut self, position: [f32; 2]) -> Self {
        self.position = position;
        self
    }

    pub fn rotate(mut self, rotate: f32) -> Self {
        self.rotate_dig = rotate;
        self
    }
}

impl<R: 'static> DomNode<R> for Teacup {
    fn build_render_tree(&self) -> Box<dyn RenderNode<R>> {
        let teacup_bytes = include_bytes!("./teacup.png");
        let teacup_image = image::load_from_memory(teacup_bytes).unwrap();
        let teacup_rgba = teacup_image.to_rgba8();
        let (width, height) = teacup_rgba.dimensions();

        Box::new(TeacupRenderNode {
            teacup_rgba,
            picture_size: crate::types::size::PxSize {
                width: width as f32,
                height: height as f32,
            },
            position: self.position,
            rotate: self.rotate_dig,
            size: self.size,
            texture: None,
            vertex_buffer: None,
            index_buffer: None,
            index_len: 0,
        })
    }
}

pub struct TeacupRenderNode {
    teacup_rgba: image::RgbaImage,
    picture_size: crate::types::size::PxSize,
    position: [f32; 2],
    rotate: f32,

    size: crate::types::size::Size,

    // previous_size: Option<PxSize>,
    texture: Option<Arc<wgpu::Texture>>,
    vertex_buffer: Option<Arc<wgpu::Buffer>>,
    index_buffer: Option<Arc<wgpu::Buffer>>,
    index_len: u32,
}

impl<R: 'static> RenderNode<R> for TeacupRenderNode {
    fn render(
        &mut self,
        app_context: &ApplicationContext,
        parent_size: ParentPxSize,
    ) -> RenderObject {
        let device = app_context.get_wgpu_device();
        if self.texture.is_none() {
            let size = wgpu::Extent3d {
                width: self.picture_size.width as u32,
                height: self.picture_size.height as u32,
                depth_or_array_layers: 1,
            };

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Teacup Texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Texture Buffer"),
                contents: &self.teacup_rgba,
                usage: wgpu::BufferUsages::COPY_SRC,
            });

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Texture Command Encoder"),
            });
            encoder.copy_buffer_to_texture(
                wgpu::ImageCopyBuffer {
                    buffer: &buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * self.picture_size.width as u32),
                        rows_per_image: Some(self.picture_size.height as u32),
                    },
                },
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                size,
            );
            app_context.get_wgpu_queue().submit(Some(encoder.finish()));

            let (vertex, index, index_len) = vertex::TexturedVertex::rectangle_buffer(
                app_context,
                0.0,
                0.0,
                self.size
                    .width
                    .to_px(parent_size.width, app_context)
                    .unwrap(),
                self.size
                    .height
                    .to_px(parent_size.height, app_context)
                    .unwrap(),
                false,
            );
            self.texture = Some(Arc::new(texture));
            self.vertex_buffer = Some(Arc::new(vertex));
            self.index_buffer = Some(Arc::new(index));
            self.index_len = index_len;
        }

        RenderObject {
            object: crate::ui::Object::Textured {
                vertex_buffer: self.vertex_buffer.as_ref().unwrap().clone(),
                index_buffer: self.index_buffer.as_ref().unwrap().clone(),
                index_len: self.index_len,
                texture: self.texture.as_ref().unwrap().clone(),
            },
            px_size: crate::types::size::PxSize {
                width: self
                    .size
                    .width
                    .to_px(parent_size.width, app_context)
                    .unwrap(),
                height: self
                    .size
                    .height
                    .to_px(parent_size.height, app_context)
                    .unwrap(),
            },
            sub_objects: vec![],
        }
    }

    fn widget_event(&self, event: &WidgetEvent) -> Option<R> {
        None
    }

    fn update_render_tree(&self, dom: &dyn DomNode<R>) {}

    fn compare(&self, dom: &dyn DomNode<R>) -> DomComPareResult {
        if dom.type_id() == TypeId::of::<Teacup>() {
            DomComPareResult::Same
        } else {
            DomComPareResult::Different
        }
    }
}
