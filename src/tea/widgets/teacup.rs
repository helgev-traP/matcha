use cgmath::SquareMatrix;
use image::{GenericImageView, ImageError};
use wgpu::{core::device, util::DeviceExt};

use crate::application_context::ApplicationContext;
use crate::ui::{Ui, WidgetRenderObject, Widgets};

pub struct Teacup {
    // teacup
    teacup_rgba: image::RgbaImage,
    picture_size: crate::types::Size,

    // context
    app_context: Option<ApplicationContext>,

    // render object
    size: crate::types::Size,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    index_len: u32,
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
            picture_size: crate::types::Size {
                width: width as f32,
                height: height as f32,
            },
            size: crate::types::Size {
                width: width as f32,
                height: height as f32,
            },
            app_context: None,
            vertex_buffer: None,
            index_buffer: None,
            index_len: 0,
            texture: None,
        })
    }
}

impl Ui for Teacup {
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

        let (vertex_buffer, index_buffer, index_len) =
            crate::vertex::TexturedVertex::rectangle_buffer(
                &device_queue,
                0.0,
                0.0,
                self.size.width as f32,
                self.size.height as f32,
                false,
            );

        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
        self.index_len = index_len;
        self.texture = Some(texture);
        self.app_context = Some(device_queue);
    }

    fn size(&self) -> crate::types::Size {
        self.size
    }

    fn event(&mut self, event: &crate::event::Event) {
        match event {
            crate::event::Event::Resize(size) => {
                self.size = *size;

                let (vertex_buffer, index_buffer, index_len) =
                    crate::vertex::TexturedVertex::rectangle_buffer(
                        self.app_context.as_ref().unwrap(),
                        0.0,
                        0.0,
                        self.size.width as f32,
                        self.size.height as f32,
                        false,
                    );

                self.vertex_buffer = Some(vertex_buffer);
                self.index_buffer = Some(index_buffer);
                self.index_len = index_len;
            }
        }
    }
}

impl Widgets for Teacup {
    fn render_object(&self) -> Option<Vec<WidgetRenderObject>> {
        Some(vec![WidgetRenderObject {
            size: &self.size,
            offset: [0.0, 0.0],
            vertex_buffer: self.vertex_buffer.as_ref()?,
            index_buffer: self.index_buffer.as_ref()?,
            index_count: self.index_len,
            texture: self.texture.as_ref()?,
        }])
    }
}
