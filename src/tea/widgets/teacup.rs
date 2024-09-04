use image::GenericImageView;
use wgpu::{core::device, util::DeviceExt};

use super::Widget;

pub struct Teacup {
    teacup_rgba: image::RgbaImage,
    size: crate::types::Size,

    device_queue: Option<crate::window::DeviceQueue>,
    texture: Option<wgpu::Texture>,
}

impl Teacup {
    pub fn new() -> Self {
        let teacup_bytes = include_bytes!("./teacup.png");
        let teacup_image = image::load_from_memory(teacup_bytes).unwrap();
        let teacup_rgba = teacup_image.to_rgba8();
        let (width, height) = teacup_rgba.dimensions();
        Self {
            teacup_rgba,
            size: crate::types::Size { width, height },
            device_queue: None,
            texture: None,
        }
    }
}

impl Widget for Teacup {
    fn set_device_queue(&mut self, device_queue: crate::window::DeviceQueue) {
        let size = wgpu::Extent3d {
            width: self.size.width,
            height: self.size.height,
            depth_or_array_layers: 1,
        };

        let texture = device_queue
            .get_device()
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Teacup Texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });

        let buffer =
            device_queue
                .get_device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Texture Buffer"),
                    contents: &self.teacup_rgba,
                    usage: wgpu::BufferUsages::COPY_SRC,
                });

        let mut encoder =
            device_queue
                .get_device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Texture Command Encoder"),
                });

        let mut encoder =
            device_queue
                .get_device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Texture Command Encoder"),
                });
        encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * self.size.width),
                    rows_per_image: Some(self.size.height),
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
        device_queue.get_queue().submit(Some(encoder.finish()));

        self.device_queue = Some(device_queue);
        self.texture = Some(texture);
    }

    fn size(&self) -> &crate::types::Size {
        &self.size
    }

    fn render(&self) -> Option<&wgpu::Texture> {
        self.texture.as_ref()
    }
}
