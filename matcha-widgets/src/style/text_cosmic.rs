use cosmic_text::{Attrs, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache};
use matcha_core::{ui::Style, ui::WidgetContext};
use parking_lot::Mutex;

struct FontContext {
    font_system: Mutex<FontSystem>,
    swash_cache: Mutex<SwashCache>,
}

impl Default for FontContext {
    fn default() -> Self {
        Self {
            font_system: Mutex::new(FontSystem::new()),
            swash_cache: Mutex::new(SwashCache::new()),
        }
    }
}

pub struct TextCosmic<'a> {
    pub texts: Vec<TextElement<'a>>,
    pub color: Color,
    pub metrics: Metrics,
    pub max_size: [Option<f32>; 2],
    pub buffer: Mutex<Option<Buffer>>,
    pub cache_in_memory: Mutex<Option<CacheInMemory>>,
    pub cache_in_texture: Mutex<Option<wgpu::Texture>>,
}

#[derive(Clone)]
pub struct TextElement<'a> {
    pub text: String,
    pub attrs: Attrs<'a>,
}

pub struct CacheInMemory {
    pub size: [u32; 2],
    /// ! y-axis heads up
    pub text_offset: [i32; 2],
    pub data: Vec<u8>,
}

impl<'a> Clone for TextCosmic<'a> {
    fn clone(&self) -> Self {
        Self {
            texts: self.texts.clone(),
            color: self.color,
            metrics: self.metrics,
            max_size: self.max_size,
            buffer: Mutex::new(None),
            cache_in_memory: Mutex::new(None),
            cache_in_texture: Mutex::new(None),
        }
    }
}

impl<'a> TextCosmic<'a> {
    pub fn new(
        texts: Vec<TextElement<'a>>,
        color: Color,
        metrics: Metrics,
        max_size: [Option<f32>; 2],
    ) -> Self {
        Self {
            texts,
            color,
            metrics,
            max_size,
            buffer: Mutex::new(None),
            cache_in_memory: Mutex::new(None),
            cache_in_texture: Mutex::new(None),
        }
    }
}

impl TextCosmic<'_> {
    fn set_buffer(font_system: &mut FontSystem, metrics: Metrics) -> Buffer {
        Buffer::new(font_system, metrics)
    }

    fn render_to_memory(
        texts: &[TextElement],
        color: Color,
        buffer: &mut Buffer,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
        max_size: [Option<f32>; 2],
    ) -> CacheInMemory {
        // ! y-axis heads down

        let mut buffer = buffer.borrow_with(font_system);

        buffer.set_size(
            max_size[0].unwrap_or(f32::MAX),
            max_size[1].unwrap_or(f32::MAX),
        );

        for text in texts {
            buffer.set_text(text.text.as_str(), text.attrs.clone(), Shaping::Advanced);
        }

        buffer.shape_until_scroll(true);

        let mut x_max = 0;
        let mut y_min = 0;
        let mut y_max = 0;

        for line in buffer.layout_runs() {
            if line.glyphs.is_empty() {
                continue;
            }

            x_max = x_max.max(line.line_w.ceil() as i32);
            y_min = y_min.min((line.line_y - line.line_top).floor() as i32);
            y_max = y_max.max(line.line_y.ceil() as i32);
        }

        let x_min = 0;
        let y_min = y_min;
        let width = (x_max - x_min).max(0) as usize;
        let height = (y_max - y_min).max(0) as usize;
        let size = [width, height];

        let mut data_rgba = vec![0u8; size[0].saturating_mul(size[1]).saturating_mul(4)];
        let data_offset = [x_min, y_min];

        if size[0] == 0 || size[1] == 0 {
            return CacheInMemory {
                size: [0, 0],
                text_offset: [0, 0],
                data: Vec::new(),
            };
        }

        buffer.draw(swash_cache, color, |x, y, _w, _h, color| {
            let x = (x - x_min) as usize;
            let y = (y - y_min) as usize;
            if let Some(index) = y.checked_mul(size[0]).and_then(|v| v.checked_add(x)).and_then(|v| v.checked_mul(4)) {
                if index + 3 < data_rgba.len() {
                    data_rgba[index] = color.r();
                    data_rgba[index + 1] = color.g();
                    data_rgba[index + 2] = color.b();
                    data_rgba[index + 3] = color.a();
                }
            }
        });

        // ! change y-axis heads up
        let data_offset = [data_offset[0], -data_offset[1]];

        CacheInMemory {
            size: [size[0] as u32, size[1] as u32],
            text_offset: [data_offset[0], data_offset[1]],
            data: data_rgba,
        }
    }

    fn render_to_texture(&self, size: [u32; 2], data: &[u8], ctx: &WidgetContext) -> wgpu::Texture {
        let device = ctx.device();
        let queue = ctx.queue();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("TextCosmic Texture"),
            size: wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(size[0] * 4),
                rows_per_image: Some(size[1]),
            },
            wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
        );

        texture
    }
}

impl Style for TextCosmic<'static> {
    fn clone_boxed(&self) -> Box<dyn Style> {
        Box::new(self.clone())
    }

    fn is_inside(&self, position: [f32; 2], boundary_size: [f32; 2], ctx: &WidgetContext) -> bool {
        let draw_range = self.draw_range(boundary_size, ctx);
        let x_range = draw_range.x_range();
        let y_range = draw_range.y_range();

        x_range[0] <= position[0]
            && position[0] <= x_range[1]
            && y_range[0] <= position[1]
            && position[1] <= y_range[1]
    }

    fn draw_range(
        &self,
        boundary_size: [f32; 2],
        ctx: &WidgetContext,
    ) -> matcha_core::types::range::Range2D<f32> {
        let font_context = ctx.any_resource().get_or_insert_default::<FontContext>();

        let font_system = &font_context.font_system;
        let swash_cache = &font_context.swash_cache;

        let buffer = &mut *self.buffer.lock();
        let cache_in_memory = &mut *self.cache_in_memory.lock();

        if buffer.is_none() {
            *buffer = Some(Self::set_buffer(&mut font_system.lock(), self.metrics));
        }
        let buffer = buffer.as_mut().unwrap();

        if cache_in_memory.is_none() {
            *cache_in_memory = Some(Self::render_to_memory(
                &self.texts,
                self.color,
                buffer,
                &mut font_system.lock(),
                &mut swash_cache.lock(),
                self.max_size,
            ));
        }
        let cache_in_memory = cache_in_memory.as_ref().unwrap();

        matcha_core::types::range::Range2D::new(
            [
                cache_in_memory.text_offset[0] as f32,
                cache_in_memory.text_offset[1] as f32,
            ],
            [
                cache_in_memory.size[0] as f32 + cache_in_memory.text_offset[0] as f32,
                cache_in_memory.size[1] as f32 + cache_in_memory.text_offset[1] as f32,
            ],
        )
    }

    fn draw(
        &self,
        _render_pass: &mut wgpu::RenderPass<'_>,
        _target_size: [u32; 2],
        _target_format: wgpu::TextureFormat,
        _boundary_size: [f32; 2],
        _offset: [f32; 2],
        _ctx: &WidgetContext,
    ) {
        // TODO: Reimplement with texture atlas and proper text rendering pipeline
    }
}
