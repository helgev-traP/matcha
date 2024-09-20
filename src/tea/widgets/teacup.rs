use cgmath::num_traits::real;
use image::{GenericImageView, ImageError};
use wgpu::util::DeviceExt;

use crate::{
    application_context::ApplicationContext,
    ui::{EventHandle, Object, RenderObject, Style, UiRendering},
    vertex::{self, TexturedVertex},
};

pub struct Teacup {
    // teacup
    teacup_rgba: image::RgbaImage,
    picture_size: crate::types::PxSize,
    position: [f32; 2],
    rotate: f32,

    // context
    app_context: Option<ApplicationContext>,

    // for render object
    size: crate::types::Size,
    texture: Option<wgpu::Texture>,
}

impl Teacup {
    pub fn new() -> Result<Self, ImageError> {
        let teacup_bytes = include_bytes!("./teacup.png");
        let teacup_image = match image::load_from_memory(teacup_bytes) {
            Ok(image) => image,
            Err(e) => return Err(e),
        };
        let teacup_rgba = teacup_image.to_rgba8();
        let (width, height) = teacup_rgba.dimensions();
        Ok(Self {
            teacup_rgba,
            picture_size: crate::types::PxSize {
                width: width as f32,
                height: height as f32,
            },
            position: [0.0, 0.0],
            rotate: 0.0,
            size: crate::types::Size {
                width: crate::types::SizeUnit::Pixel(width as f32),
                height: crate::types::SizeUnit::Pixel(height as f32),
            },
            app_context: None,
            texture: None,
        })
    }

    pub fn position(mut self, position: [f32; 2]) -> Self {
        self.position = position;
        self
    }

    pub fn rotate(mut self, rotate: f32) -> Self {
        self.rotate = rotate;
        self
    }
}

impl UiRendering for Teacup {
    fn set_application_context(&mut self, device_queue: ApplicationContext) {
        let size = wgpu::Extent3d {
            width: self.picture_size.width as u32,
            height: self.picture_size.height as u32,
            depth_or_array_layers: 1,
        };

        let texture = device_queue
            .get_wgpu_device()
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Teacup Texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

        let buffer =
            device_queue
                .get_wgpu_device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Texture Buffer"),
                    contents: &self.teacup_rgba,
                    usage: wgpu::BufferUsages::COPY_SRC,
                });

        let mut encoder = device_queue.get_wgpu_device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Texture Command Encoder"),
            },
        );
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
        device_queue.get_wgpu_queue().submit(Some(encoder.finish()));

        self.texture = Some(texture);
        self.app_context = Some(device_queue);
    }

    fn get_style(&self) -> &crate::ui::Property {
        todo!()
    }

    fn set_style(&mut self, property: crate::ui::Property) {
        todo!()
    }

    fn render_object(
        &self,
        parent_size: crate::types::ParentPxSize,
    ) -> Result<crate::ui::RenderObject, ()> {
        let real_size = crate::types::PxSize {
            width: self
                .size
                .width
                .to_px(parent_size.width, self.app_context.as_ref().unwrap())
                .unwrap(),
            height: self
                .size
                .height
                .to_px(parent_size.height, self.app_context.as_ref().unwrap())
                .unwrap(),
        };

        let (vertex_buffer, index_buffer, index_len) = TexturedVertex::rectangle_buffer(
            self.app_context.as_ref().unwrap(),
            0.0,
            0.0,
            real_size.width,
            real_size.height,
            false,
        );

        Ok(RenderObject {
            object: Object::Textured {
                vertex_buffer: Box::new(vertex_buffer),
                index_buffer: Box::new(index_buffer),
                index_len,
                texture: Box::new(self.texture.as_ref().unwrap()),
            },
            px_size: real_size,
            property: crate::ui::Property::default(),
            sub_objects: vec![],
        })
    }
}

impl EventHandle for Teacup {}

impl Style for Teacup {}
