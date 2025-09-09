use std::sync::Arc;

use crate::style::Style;
use gpu_utils::texture_atlas::atlas_simple::atlas::AtlasRegion;
use matcha_core::types::range::Range2D;
use matcha_core::ui::WidgetContext;
use utils::cache::Cache;

pub struct Buffer {
    style: Vec<Arc<dyn Style>>,
    buffer_format: wgpu::TextureFormat,
    cache: Cache<[f32; 2], BufferData>,
}

pub struct BufferData {
    pub texture: AtlasRegion,
    pub texture_position: Range2D<f32>,
}

impl Buffer {
    pub fn new(style: Vec<Arc<dyn Style>>) -> Self {
        Self {
            style,
            buffer_format: wgpu::TextureFormat::Rgba8UnormSrgb,
            cache: Cache::new(),
        }
    }

    pub fn format(mut self, format: wgpu::TextureFormat) -> Self {
        self.buffer_format = format;
        self
    }

    pub fn is_inside(
        &self,
        position: [f32; 2],
        boundary_size: [f32; 2],
        ctx: &WidgetContext,
    ) -> bool {
        for style in &self.style {
            if style.is_inside(position, boundary_size, ctx) {
                return true;
            }
        }
        false
    }

    pub fn render(
        &mut self,
        boundary: [f32; 2],
        encoder: &mut wgpu::CommandEncoder,
        ctx: &WidgetContext,
    ) -> &BufferData {
        let (_, cache) = self.cache.get_or_insert_with(boundary, || {
            // calculate necessary size for the texture
            let mut x_min = f32::MAX;
            let mut x_max = f32::MIN;
            let mut y_min = f32::MAX;
            let mut y_max = f32::MIN;

            for style in &self.style {
                let range = style.draw_range(boundary, ctx);
                x_min = x_min.min(range.left());
                x_max = x_max.max(range.right());
                y_min = y_min.min(range.bottom());
                y_max = y_max.max(range.top());
            }

            let texture_position = Range2D::new([x_min, x_max], [y_min, y_max]);

            let x_min_int = x_min.floor() as i32;
            let x_max_int = x_max.ceil() as i32;
            let y_min_int = y_min.floor() as i32;
            let y_max_int = y_max.ceil() as i32;

            // create a texture with the calculated size

            let texture_size = [
                (x_max_int - x_min_int) as u32,
                (y_max_int - y_min_int) as u32,
            ];

            // Allocate a region in the texture atlas and render each style into it.
            // We unwrap here because allocation failure is unexpected in normal operation.
            let atlas_region = ctx
                .texture_atlas()
                .allocate_color(ctx.device(), ctx.queue(), texture_size)
                .expect("Texture atlas allocation failed for Buffer");

            for style in &self.style {
                style.draw(
                    encoder,
                    &atlas_region,
                    boundary,
                    [x_min_int as f32, y_min_int as f32],
                    ctx,
                );
            }

            BufferData {
                texture: atlas_region,
                texture_position,
            }
        });

        cache
    }
}
