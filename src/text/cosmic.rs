use std::sync::Arc;

use cosmic_text as ct;
use wgpu;

pub struct TextureAttribute<'a> {
    pub width: u32,
    pub height: u32,
    pub texture: &'a wgpu::Texture,
}

pub struct TextureAttributeGpu<'a> {
    pub queue: &'a wgpu::Queue,
    pub width: u32,
    pub height: u32,
    pub texture: &'a wgpu::Texture,
}

pub use ct::Family;

#[derive(Clone)]
pub struct RenderAttribute<'a> {
    pub text_attr: ct::Attrs<'a>,
    pub font_size: f32,
    pub line_height: f32,
    pub offset_px: [f32; 2],
    pub font_color: [u8; 4],
}

impl<'a> RenderAttribute<'a> {
    pub fn new() -> Self {
        Self {
            text_attr: ct::Attrs::new(),
            font_size: 12.0,
            line_height: 12.0,
            offset_px: [0.0, 0.0],
            font_color: [255, 255, 255, 255],
        }
    }

    pub fn set_font_size(&mut self, font_size: f32) {
        self.font_size = font_size;
    }

    pub fn set_line_height(&mut self, line_height: f32) {
        self.line_height = line_height;
    }

    pub fn set_offset_px(&mut self, offset_px: [f32; 2]) {
        self.offset_px = offset_px;
    }

    pub fn set_font_color(&mut self, font_color: [u8; 4]) {
        self.font_color = font_color;
    }

    pub fn set_family(&mut self, family: Family<'a>) {
        self.text_attr.family = family;
    }

    pub fn set_weight(&mut self, weight: u16) {
        self.text_attr.weight(ct::Weight(weight));
    }
}

pub struct FontContext {
    pub font_system: Arc<std::sync::Mutex<ct::FontSystem>>,
    pub swash_cache: Arc<std::sync::Mutex<ct::SwashCache>>,
}

impl Clone for FontContext {
    fn clone(&self) -> Self {
        Self {
            font_system: self.font_system.clone(),
            swash_cache: self.swash_cache.clone(),
        }
    }
}

impl FontContext {
    pub fn new() -> Self {
        let font_system = ct::FontSystem::new();
        let swash_cache = ct::SwashCache::new();
        Self {
            font_system: Arc::new(std::sync::Mutex::new(font_system)),
            swash_cache: Arc::new(std::sync::Mutex::new(swash_cache)),
        }
    }

    pub fn render(
        &self,
        text: &str,
        atr: RenderAttribute,
        texture: &TextureAttributeGpu,
    ) -> [i32; 2] {
        // measure text size
        let mut text_size = [0, 0];

        // create image buffer
        let mut image_buffer: Vec<u8> =
            vec![0u8; texture.width as usize * texture.height as usize * 4];

        {
            // Mutex locked block
            // set up cosmic text
            let mut font_system = self.font_system.lock().unwrap();
            let mut swash_cache = self.swash_cache.lock().unwrap();

            let metrics = ct::Metrics::new(atr.font_size, atr.line_height);

            let mut buffer = ct::Buffer::new(&mut font_system, metrics);

            let mut buffer = buffer.borrow_with(&mut font_system);

            buffer.set_size(Some(texture.width as f32), Some(texture.height as f32));

            buffer.set_text(text, atr.text_attr, ct::Shaping::Advanced);

            buffer.shape_until_scroll(true);

            let text_color = ct::Color::rgba(
                atr.font_color[0],
                atr.font_color[1],
                atr.font_color[2],
                atr.font_color[3],
            );

            let texture_width_i32 = texture.width as i32;
            let texture_height_i32 = texture.height as i32;

            buffer.draw(
                &mut swash_cache,
                text_color,
                |mut x, mut y, _w, _h, color| {
                    text_size[0] = x.max(text_size[0]);
                    text_size[1] = y.max(text_size[1]);

                    x += atr.offset_px[0] as i32;
                    y += atr.offset_px[1] as i32;
                    if x < 0 || y < 0 || x >= texture_width_i32 || y >= texture_height_i32 {
                        return;
                    }
                    image_buffer[((x as u32 + y as u32 * texture.width) * 4) as usize] =
                        image_buffer[((x as u32 + y as u32 * texture.width) * 4) as usize]
                            .max(color.r());
                    image_buffer[((x as u32 + y as u32 * texture.width) * 4 + 1) as usize] =
                        image_buffer[((x as u32 + y as u32 * texture.width) * 4 + 1) as usize]
                            .max(color.g());
                    image_buffer[((x as u32 + y as u32 * texture.width) * 4 + 2) as usize] =
                        image_buffer[((x as u32 + y as u32 * texture.width) * 4 + 2) as usize]
                            .max(color.b());
                    image_buffer[((x as u32 + y as u32 * texture.width) * 4 + 3) as usize] =
                        image_buffer[((x as u32 + y as u32 * texture.width) * 4 + 3) as usize]
                            .max(color.a());
                },
            );
        }

        // copy buffer to texture

        texture.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image_buffer,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(texture.width * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: texture.width,
                height: texture.height,
                depth_or_array_layers: 1,
            },
        );

        text_size
    }
}
