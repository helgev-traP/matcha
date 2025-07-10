use matcha_core::{context::WidgetContext, types::range::Range2D, ui::Style};
use utils::cache::Cache;

pub struct Buffer {
    style: Vec<Box<dyn Style>>,
    buffer_format: wgpu::TextureFormat,
    cache: Cache<[f32; 2], BufferData>,
}

pub struct BufferData {
    pub texture: wgpu::Texture,
    pub texture_position: Range2D<f32>,
}

impl Buffer {
    pub fn new(style: Vec<Box<dyn Style>>) -> Self {
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

            let texture = ctx.device().create_texture(&wgpu::TextureDescriptor {
                label: Some("Buffer Texture"),
                size: wgpu::Extent3d {
                    width: texture_size[0],
                    height: texture_size[1],
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.buffer_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            // render each style into the texture

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Buffer Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            for style in &self.style {
                style.draw(
                    &mut render_pass,
                    texture_size,
                    self.buffer_format,
                    boundary,
                    [x_min_int as f32, y_min_int as f32],
                    ctx,
                );
            }

            BufferData {
                texture,
                texture_position,
            }
        });

        cache
    }
}
