use std::sync::Arc;

use dashmap::DashMap;
use image::{EncodableLayout, GenericImageView};
use matcha_core::{types::range::Range2D, ui::Style, ui::WidgetContext};
use renderer::widgets_renderer::texture_copy::{RenderData, TargetData, TextureCopy};

use crate::types::size::{ChildSize, Size};

#[derive(Default)]
struct ImageCache {
    map: DashMap<ImageSourceKey, Option<ImageCacheData>, fxhash::FxBuildHasher>,
}

#[derive(Clone, PartialEq)]
pub enum ImageSource {
    Path(String),
    StaticSlice { data: &'static [u8] },
    Arc(Arc<Vec<u8>>),
}

impl ImageSource {
    fn to_key(&self) -> ImageSourceKey {
        match self {
            ImageSource::Path(path) => ImageSourceKey::Path(path.clone()),
            ImageSource::StaticSlice { data } => ImageSourceKey::StaticSlice {
                ptr: data.as_ptr() as usize,
                size: data.len(),
            },
            ImageSource::Arc(data) => ImageSourceKey::Arc {
                ptr: Arc::as_ptr(data) as usize,
                size: data.len(),
            },
        }
    }
}

impl From<&str> for ImageSource {
    fn from(path: &str) -> Self {
        ImageSource::Path(path.to_string())
    }
}

impl From<String> for ImageSource {
    fn from(path: String) -> Self {
        ImageSource::Path(path)
    }
}

impl<const N: usize> From<&'static [u8; N]> for ImageSource {
    fn from(data: &'static [u8; N]) -> Self {
        ImageSource::StaticSlice { data }
    }
}

impl From<Arc<Vec<u8>>> for ImageSource {
    fn from(data: Arc<Vec<u8>>) -> Self {
        ImageSource::Arc(data)
    }
}

impl From<Vec<u8>> for ImageSource {
    fn from(data: Vec<u8>) -> Self {
        ImageSource::Arc(Arc::new(data))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum ImageSourceKey {
    /// Full path to the image file
    Path(String),
    /// pointer address (as usize) and size of the image data
    /// This is safe because the data is guaranteed to be static
    StaticSlice {
        ptr: usize,
        size: usize,
    },
    Arc {
        ptr: usize,
        size: usize,
    },
}

struct ImageCacheData {
    width: u32,
    height: u32,
    texture: wgpu::Texture,
}

// MARK: Image Construct

pub enum HAlign {
    Left,
    Center,
    Right,
}

pub enum VAlign {
    Top,
    Center,
    Bottom,
}

#[derive(Clone, PartialEq)]
pub struct Image {
    image: ImageSource,
    size: Arc<[Size; 2]>,
    offset: Arc<[Size; 2]>,
}

impl Image {
    pub fn new(source: impl Into<ImageSource>) -> Self {
        Self {
            image: source.into(),
            size: Arc::new([Size::child_w(1.0), Size::child_h(1.0)]),
            offset: Arc::new([Size::px(0.0), Size::px(0.0)]),
        }
    }

    pub fn stretch_to_boundary(mut self) -> Self {
        self.size = Arc::new([Size::parent_w(1.0), Size::parent_h(1.0)]);
        self
    }

    /// Set absolute size in pixels.
    pub fn size_px(mut self, w: f32, h: f32) -> Self {
        self.size = Arc::new([Size::px(w), Size::px(h)]);
        self
    }

    /// Set width in pixels, keep height as is.
    pub fn size_px_w(mut self, w: f32) -> Self {
        let h = self.size[1].clone();
        self.size = Arc::new([Size::px(w), h]);
        self
    }

    /// Set height in pixels, keep width as is.
    pub fn size_px_h(mut self, h: f32) -> Self {
        let w = self.size[0].clone();
        self.size = Arc::new([w, Size::px(h)]);
        self
    }

    /// Set size as percentage of parent (percent values, e.g. 50.0 == 50%).
    pub fn size_percent(mut self, w_percent: f32, h_percent: f32) -> Self {
        self.size = Arc::new([
            Size::parent_w(w_percent / 100.0),
            Size::parent_h(h_percent / 100.0),
        ]);
        self
    }

    /// Set absolute offset in pixels.
    pub fn offset_px(mut self, x: f32, y: f32) -> Self {
        self.offset = Arc::new([Size::px(x), Size::px(y)]);
        self
    }

    /// Set offset as percentage of parent (percent values).
    pub fn offset_percent(mut self, x_percent: f32, y_percent: f32) -> Self {
        self.offset = Arc::new([
            Size::parent_w(x_percent / 100.0),
            Size::parent_h(y_percent / 100.0),
        ]);
        self
    }

    /// Align center both axes.
    pub fn align_center(mut self) -> Self {
        // offset = (parent - child) * 0.5
        let ox = Size::from_size(|parent, child, _ctx| {
            let parent_w = parent[0].unwrap_or(child.get()[0]);
            (parent_w - child.get()[0]) * 0.5
        });
        let oy = Size::from_size(|parent, child, _ctx| {
            let parent_h = parent[1].unwrap_or(child.get()[1]);
            (parent_h - child.get()[1]) * 0.5
        });
        self.offset = Arc::new([ox, oy]);
        self
    }

    /// Align horizontally (left/center/right) with optional margin (Size).
    pub fn align_h(mut self, align: HAlign, margin: Size) -> Self {
        let m1 = margin.clone();
        let m2 = margin.clone();
        let m3 = margin.clone();
        let ox = match align {
            HAlign::Left => Size::from_size(move |_parent, _child, ctx| {
                m1.size([None, None], &mut ChildSize::default(), ctx)
            }),
            HAlign::Center => Size::from_size(move |parent, child, ctx| {
                let parent_w = parent[0].unwrap_or(child.get()[0]);
                (parent_w - child.get()[0]) * 0.5 + m2.size(parent, child, ctx)
            }),
            HAlign::Right => Size::from_size(move |parent, child, ctx| {
                let parent_w = parent[0].unwrap_or(child.get()[0]);
                parent_w - child.get()[0] - m3.size(parent, child, ctx)
            }),
        };
        let oy = self.offset[1].clone();
        self.offset = Arc::new([ox, oy]);
        self
    }

    /// Align vertically (top/center/bottom) with optional margin (Size).
    pub fn align_v(mut self, align: VAlign, margin: Size) -> Self {
        let m1 = margin.clone();
        let m2 = margin.clone();
        let m3 = margin.clone();
        let oy = match align {
            VAlign::Top => Size::from_size(move |_parent, _child, ctx| {
                m1.size([None, None], &mut ChildSize::default(), ctx)
            }),
            VAlign::Center => Size::from_size(move |parent, child, ctx| {
                let parent_h = parent[1].unwrap_or(child.get()[1]);
                (parent_h - child.get()[1]) * 0.5 + m2.size(parent, child, ctx)
            }),
            VAlign::Bottom => Size::from_size(move |parent, child, ctx| {
                let parent_h = parent[1].unwrap_or(child.get()[1]);
                parent_h - child.get()[1] - m3.size(parent, child, ctx)
            }),
        };
        let ox = self.offset[0].clone();
        self.offset = Arc::new([ox, oy]);
        self
    }

    /// Generic anchor: sets horizontal and vertical alignment with margins.
    pub fn anchor(mut self, halign: HAlign, valign: VAlign, margin: [Size; 2]) -> Self {
        self = self.align_h(halign, margin[0].clone());
        self = self.align_v(valign, margin[1].clone());
        self
    }

    // Existing simple setters kept below (they will overwrite)
    pub fn size(mut self, size: [Size; 2]) -> Self {
        self.size = Arc::new(size);
        self
    }

    pub fn offset(mut self, offset: [Size; 2]) -> Self {
        self.offset = Arc::new(offset);
        self
    }
}

impl Image {
    fn key(&self) -> ImageSourceKey {
        self.image.to_key()
    }
}

// helper methods
impl Image {
    fn boundary_opt(boundary_size: [f32; 2]) -> [Option<f32>; 2] {
        [Some(boundary_size[0]), Some(boundary_size[1])]
    }

    fn calc_layout(
        &self,
        boundary: [Option<f32>; 2],
        base: [f32; 2],
        ctx: &WidgetContext,
    ) -> (f32, f32, f32, f32) {
        let mut child_size = ChildSize::new(|| base);

        let size_x = self.size[0].size(boundary, &mut child_size, ctx);
        let size_y = self.size[1].size(boundary, &mut child_size, ctx);
        let offset_x = self.offset[0].size(boundary, &mut child_size, ctx);
        let offset_y = self.offset[1].size(boundary, &mut child_size, ctx);
        (size_x, size_y, offset_x, offset_y)
    }

    fn with_image<R>(
        &self,
        ctx: &WidgetContext,
        f: impl FnOnce(&ImageCacheData) -> R,
    ) -> Option<R> {
        let cache_map = ctx.any_resource().get_or_insert_default::<ImageCache>();
        let image_cache = cache_map
            .map
            .entry(self.key())
            .or_insert_with(|| load_image_to_texture(&self.image, ctx));

        let Some(image) = image_cache.value() else {
            return None;
        };
        Some(f(image))
    }
}

// MARK: Style implementation

impl Style for Image {
    fn clone_boxed(&self) -> Box<dyn Style> {
        Box::new(self.clone())
    }

    fn required_size(&self, ctx: &WidgetContext) -> Option<[f32; 2]> {
        self.with_image(ctx, |image| [image.width as f32, image.height as f32])
    }

    fn is_inside(&self, position: [f32; 2], boundary_size: [f32; 2], ctx: &WidgetContext) -> bool {
        let draw_range = self.draw_range(boundary_size, ctx);
        draw_range.contains(position)
    }

    fn draw_range(&self, boundary_size: [f32; 2], ctx: &WidgetContext) -> Range2D<f32> {
        self.with_image(ctx, |image| {
            let base = [image.width as f32, image.height as f32];
            let boundary = Self::boundary_opt(boundary_size);
            let (size_x, size_y, offset_x, offset_y) = self.calc_layout(boundary, base, ctx);
            Range2D::new([offset_x, offset_x + size_x], [offset_y, offset_y + size_y])
        })
        .unwrap_or_default()
    }

    fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &gpu_utils::texture_atlas::atlas_simple::atlas::AtlasRegion,
        boundary_size: [f32; 2],
        offset: [f32; 2],
        ctx: &WidgetContext,
    ) {
        let target_size = target.size();
        let target_format = target.format();
        self.with_image(ctx, |image| {
            let image_size = [image.width as f32, image.height as f32];
            let boundary = Self::boundary_opt(boundary_size);
            let (size_x, size_y, offset_x, offset_y) = self.calc_layout(boundary, image_size, ctx);
            let draw_offset = [offset_x + offset[0], offset_y + offset[1]];

            // begin a render pass targeting the atlas region so the renderer can create its own passes if needed
            let mut render_pass = match target.begin_render_pass(encoder) {
                Ok(rp) => rp,
                Err(_) => return,
            };

            println!(
                "Drawing image at offset {:?} with size [{}, {}] boundary {:?} image_size {:?}",
                draw_offset, size_x, size_y, boundary, image_size
            );

            let texture_copy = TextureCopy::default();
            texture_copy.render(
                &mut render_pass,
                TargetData {
                    target_size,
                    target_format,
                },
                RenderData {
                    source_texture_view: &image
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                    source_texture_position_min: [draw_offset[0], draw_offset[1]],
                    source_texture_position_max: [draw_offset[0] + size_x, draw_offset[1] + size_y],
                    color_transformation: None,
                    color_offset: None,
                },
                ctx.device(),
            );
        });
    }
}

fn load_image_to_texture(
    image_source: &ImageSource,
    ctx: &WidgetContext,
) -> Option<ImageCacheData> {
    // load the image from the source

    let dynamic_image = match image_source {
        ImageSource::Path(path) => image::open(path).ok(),
        ImageSource::StaticSlice { data, .. } => image::load_from_memory(data).ok(),
        ImageSource::Arc(data) => image::load_from_memory(data).ok(),
    };

    let Some(dynamic_image) = dynamic_image else {
        // If the image could not be loaded, return an empty cache entry
        return None;
    };

    // Create a texture and upload image data
    let (image, format) = prepare_image_and_format(dynamic_image);
    make_cache(image, format, ctx).into()
}

fn prepare_image_and_format(
    dynamic_image: image::DynamicImage,
) -> (
    image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    wgpu::TextureFormat,
) {
    // Normalize all incoming images to RGBA8 to simplify bytes_per_row handling.
    // This avoids format-dependent byte-per-pixel calculations and prevents copy overruns.
    let image_rgba8: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = dynamic_image.to_rgba8();
    (image_rgba8, wgpu::TextureFormat::Rgba8UnormSrgb)
}

fn make_cache(
    image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    format: wgpu::TextureFormat,
    ctx: &WidgetContext,
) -> ImageCacheData {
    let (width, height) = image.dimensions();
    let data = image.as_bytes();

    let device = ctx.device();
    let queue = ctx.queue();

    // create texture
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Image Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    // We converted image bytes to RGBA8 in prepare_image_and_format, so use 4 bytes per pixel.]
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
            // use explicit 4 bytes per pixel for RGBA8
            bytes_per_row: Some(4 * width),
            rows_per_image: None,
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    ImageCacheData {
        width,
        height,
        texture,
    }
}

#[rustfmt::skip]
// note: this function is currently not being used but may be useful in the future
fn _color_transform(color_type: image::ColorType) -> nalgebra::Matrix4<f32> {
    match color_type {
        // stored as r
        image::ColorType::L8
        | image::ColorType::L16 => nalgebra::Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            1.0, 0.0, 0.0, 0.0,
            1.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0,
        ),
        // stored as rg
        image::ColorType::La8
        | image::ColorType::La16 => nalgebra::Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            1.0, 0.0, 0.0, 0.0,
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
        ),
        // stored as rgba
        image::ColorType::Rgb8
        | image::ColorType::Rgb16
        | image::ColorType::Rgb32F
        | image::ColorType::Rgba8
        | image::ColorType::Rgba16
        | image::ColorType::Rgba32F => nalgebra::Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ),
        _ => todo!(),
    }
}

// note: this function is currently not being used but may be useful in the future
fn _color_offset(color_type: image::ColorType) -> [f32; 4] {
    match color_type {
        // alpha is not stored, so we set it to 1.0
        image::ColorType::L8 | image::ColorType::L16 => [0.0, 0.0, 0.0, 1.0],
        // alpha is stored in the texture
        image::ColorType::La8
        | image::ColorType::La16
        | image::ColorType::Rgb8
        | image::ColorType::Rgb16
        | image::ColorType::Rgb32F
        | image::ColorType::Rgba8
        | image::ColorType::Rgba16
        | image::ColorType::Rgba32F => [0.0, 0.0, 0.0, 0.0],
        _ => todo!(),
    }
}
