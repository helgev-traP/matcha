use nalgebra as na;
use wgpu::naga::Type;
use std::{any::Any, sync::Arc};
use wgpu::util::DeviceExt;

use crate::render::RenderCommandEncoder;
use crate::types::size::StdSizeUnit;
use crate::{
    application_context::ApplicationContext,
    events::{WidgetEvent, WidgetEventResult},
    types::size::{PxSize, StdSize},
    vertex,
};

use super::{DomComPareResult, Dom, RenderItem, RenderingTrait, Widget, WidgetTrait};

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

impl<R: 'static> Dom<R> for Teacup {
    fn build_render_tree(&self) -> Box<dyn Widget<R>> {
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
    texture: Option<Arc<wgpu::Texture>>,
    vertex_buffer: Option<Arc<wgpu::Buffer>>,
    index_buffer: Option<Arc<wgpu::Buffer>>,
    index_len: u32,
}

impl<R: 'static> WidgetTrait<R> for TeacupRenderNode {
    fn widget_event(&self, _: &WidgetEvent, _: PxSize, _: &ApplicationContext) -> WidgetEventResult<R> {
        Default::default()
    }

    fn update_render_tree(&mut self, dom: &dyn Dom<R>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<Teacup>() {
            return Err(());
        }

        let dom = dom.as_any().downcast_ref::<Teacup>().unwrap();

        self.size = dom.size;
        self.position = dom.position;
        self.rotate = dom.rotate_dig;

        // clear cache

        self.vertex_buffer = None;
        self.index_buffer = None;
        self.index_len = 0;

        Ok(())
    }

    fn compare(&self, dom: &dyn Dom<R>) -> DomComPareResult {
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
}

impl RenderingTrait for TeacupRenderNode {
    fn render(
        &mut self,
        _: &rayon::Scope,
        parent_size: PxSize,
        affine: na::Matrix4<f32>,
        encoder: &mut RenderCommandEncoder,
    ) {
        let context = encoder.get_context();
        let device = context.get_wgpu_device();

        // calculate actual size

        let size = PxSize::from_size_parent_size(self.size, parent_size, context);

        // create texture

        if self.texture.is_none() {
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
            context.get_wgpu_queue().submit(Some(encoder.finish()));

            self.texture = Some(Arc::new(texture));
        }

        // create / update vertex buffer

        if self.vertex_buffer.is_none() || self.index_buffer.is_none() || self.index_len == 0 {
            let (vertex, index, index_len) = vertex::TexturedVertex::rectangle_buffer(
                context,
                0.0,
                0.0,
                size.width,
                size.height,
                false,
            );

            self.vertex_buffer = Some(Arc::new(vertex));
            self.index_buffer = Some(Arc::new(index));
            self.index_len = index_len;
        }

        // draw

        encoder.draw(
            RenderItem {
                object: vec![crate::ui::Object::Textured {
                    vertex_buffer: self.vertex_buffer.as_ref().unwrap().clone(),
                    index_buffer: self.index_buffer.as_ref().unwrap().clone(),
                    index_len: self.index_len,
                    texture: self.texture.as_ref().unwrap().clone(),
                    instance_affine: na::Matrix4::identity(),
                }],
            },
            affine,
        );
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
}
