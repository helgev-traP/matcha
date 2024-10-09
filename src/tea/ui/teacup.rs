use nalgebra as na;
use std::sync::{Mutex, RwLock};
use std::{
    any::{Any, TypeId},
    sync::Arc,
};
use wgpu::util::DeviceExt;

use crate::types::size::StdSizeUnit;
use crate::{
    application_context::ApplicationContext,
    events::WidgetEvent,
    types::size::{PxSize, StdSize},
    vertex,
};

use super::{DomComPareResult, DomNode, RenderItem, RenderNode, RenderTrait, SubNode};

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
    fn build_render_tree(&self) -> RenderNode<R> {
        let teacup_bytes = include_bytes!("./teacup.png");
        let teacup_image = image::load_from_memory(teacup_bytes).unwrap();
        let teacup_rgba = teacup_image.to_rgba8();
        let (width, height) = teacup_rgba.dimensions();

        Arc::new(RwLock::new(TeacupRenderNode {
            teacup_rgba,
            picture_size: crate::types::size::PxSize {
                width: width as f32,
                height: height as f32,
            },
            position: self.position,
            rotate: self.rotate_dig,
            size: self.size,
            texture: None.into(),
            vertex_buffer: None.into(),
            index_buffer: None.into(),
            index_len: 0.into(),
        }))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct TeacupRenderNode {
    teacup_rgba: image::RgbaImage,
    picture_size: crate::types::size::PxSize,
    position: [f32; 2],
    rotate: f32,

    size: crate::types::size::Size,

    // previous_size: Option<PxSize>,
    texture: Mutex<Option<Arc<wgpu::Texture>>>,
    vertex_buffer: Mutex<Option<Arc<wgpu::Buffer>>>,
    index_buffer: Mutex<Option<Arc<wgpu::Buffer>>>,
    index_len: Mutex<u32>,
}

impl<R: 'static> RenderTrait<R> for TeacupRenderNode {
    fn render(&self, app_context: &ApplicationContext, parent_size: PxSize) -> RenderItem {
        let device = app_context.get_wgpu_device();

        // calculate actual size

        let size = PxSize::from_size_parent_size(self.size, parent_size, app_context);

        // lock

        let mut self_texture = self.texture.lock().unwrap();
        let mut self_vertex_buffer = self.vertex_buffer.lock().unwrap();
        let mut self_index_buffer = self.index_buffer.lock().unwrap();
        let mut self_index_len = self.index_len.lock().unwrap();

        // create texture

        if self_texture.is_none() {
            let texture_size = wgpu::Extent3d {
                width: self.picture_size.width as u32,
                height: self.picture_size.height as u32,
                depth_or_array_layers: 1,
            };

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Teacup Texture"),
                size: texture_size,
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
                texture_size,
            );
            app_context.get_wgpu_queue().submit(Some(encoder.finish()));

            let (vertex, index, index_len) = vertex::TexturedVertex::rectangle_buffer(
                app_context,
                0.0,
                0.0,
                size.width,
                size.height,
                false,
            );
            *self_texture = Some(Arc::new(texture));
            *self_vertex_buffer = Some(Arc::new(vertex));
            *self_index_buffer = Some(Arc::new(index));
            *self_index_len = index_len;
        }

        RenderItem {
            object: vec![crate::ui::Object::Textured {
                vertex_buffer: self_vertex_buffer.as_ref().unwrap().clone(),
                index_buffer: self_index_buffer.as_ref().unwrap().clone(),
                index_len: *self_index_len,
                texture: self_texture.as_ref().unwrap().clone(),
                affine: na::Matrix4::identity(),
            }],
            px_size: size,
        }
    }

    // fn default_size(&self) -> PxSize {
    //     self.picture_size
    // }

    // fn size(&self) -> OptionPxSize {
    //     OptionPxSize {
    //         width: self.size.width,
    //         height: self.size.height,
    //     }
    // }

    fn widget_event(&self, event: &WidgetEvent) -> Option<R> {
        None
    }

    fn update_render_tree(&mut self, dom: &dyn DomNode<R>) -> Result<(), ()> {
        if (*dom).type_id() != (*self).type_id() {
            return Err(());
        }
        Ok(())
    }

    fn compare(&self, dom: &dyn DomNode<R>) -> DomComPareResult {
        if let Some(teacup) = dom.as_any().downcast_ref::<Teacup>() {
            if teacup.size == self.size
                && teacup.position == self.position
                && teacup.rotate_dig == self.rotate
            {
                DomComPareResult::Same
            } else {
                DomComPareResult::Changed
            }
        } else {
            DomComPareResult::Different
        }
    }

    fn sub_nodes(&self, parent_size: PxSize, context: &ApplicationContext) -> Vec<SubNode<R>> {
        vec![]
    }

    fn size(&self) -> crate::types::size::Size {
        self.size
    }

    fn px_size(&self, parent_size: PxSize, context: &ApplicationContext) -> PxSize {
        let mut size = StdSize::from_parent_size(self.size, parent_size, context);
        if size.width.is_none() {
            size.width = StdSizeUnit::Pixel(self.picture_size.width);
            size.height = StdSizeUnit::Pixel(self.picture_size.height);
        }
        size.unwrap()
    }

    fn default_size(&self) -> PxSize {
        self.picture_size
    }

    fn redraw(&self) -> bool {
        true
    }
}
