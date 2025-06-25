use std::sync::Arc;

use dashmap::DashMap;
use image::GenericImageView;
use matcha_core::{context::WidgetContext, types::range::Range2D, ui::Style};

// MARK: Cache

#[derive(Default)]
struct ImageCache {
    map: DashMap<ImageCacheKey, Option<ImageCacheData>, fxhash::FxBuildHasher>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum ImageCacheKey {
    /// Full path to the image file
    Path(String),
    /// Hash of the image data
    Data(u64),
}

struct ImageCacheData {
    data: image::DynamicImage,
    color_type: image::ColorType,
    texture: wgpu::Texture,
}

// MARK: Image Construct

type SizeFn =
    dyn for<'a> Fn([f32; 2], [f32; 2], &'a WidgetContext) -> [f32; 2] + Send + Sync + 'static;

pub struct Image {
    image: ImageSource,
    size: Arc<SizeFn>,
    offset: Arc<SizeFn>,
}

#[derive(Clone)]
pub enum ImageSource {
    Path(String),
    Slice { data: &'static [u8], hash: u64 },
    Vec { data: Vec<u8>, hash: u64 },
}

impl Image {
    pub fn new(source: impl IntoImageSource) -> Self {
        Self {
            image: source.into_source(),
            size: Arc::new(|_, size, _| size),
            offset: Arc::new(|_, _, _| [0.0, 0.0]),
        }
    }

    pub fn size<F>(mut self, size: F) -> Self
    where
        F: Fn([f32; 2], [f32; 2], &WidgetContext) -> [f32; 2] + Send + Sync + 'static,
    {
        self.size = Arc::new(size);
        self
    }

    fn key(&self) -> ImageCacheKey {
        match &self.image {
            ImageSource::Path(path) => ImageCacheKey::Path(path.clone()),
            ImageSource::Slice { hash, .. } | ImageSource::Vec { hash, .. } => {
                ImageCacheKey::Data(*hash)
            }
        }
    }
}

pub trait IntoImageSource {
    fn into_source(self) -> ImageSource;
}

impl IntoImageSource for &str {
    fn into_source(self) -> ImageSource {
        ImageSource::Path(self.to_string())
    }
}

impl IntoImageSource for (&'static [u8], u64) {
    fn into_source(self) -> ImageSource {
        ImageSource::Slice {
            data: self.0,
            hash: self.1,
        }
    }
}

impl IntoImageSource for (Vec<u8>, u64) {
    fn into_source(self) -> ImageSource {
        ImageSource::Vec {
            data: self.0,
            hash: self.1,
        }
    }
}

// MARK: Style implementation

impl Style for Image {
    fn is_inside(&self, position: [f32; 2], boundary_size: [f32; 2], ctx: &WidgetContext) -> bool {
        let draw_range = self.draw_range(boundary_size, ctx);
        draw_range.contains(position)
    }

    fn draw_range(&self, boundary_size: [f32; 2], ctx: &WidgetContext) -> Range2D<f32> {
        let cache_map = ctx.common_resource().get_or_insert_default::<ImageCache>();
        let key = self.key();
        let image_cache = cache_map
            .map
            .entry(key)
            .or_insert_with(|| image_data(&self.image, ctx));

        let Some(image) = image_cache.value() else {
            // If the image is not loaded, return an empty range
            return Range2D::new_unchecked([0.0, 0.0], [0.0, 0.0]);
        };

        let (width, height) = image.data.dimensions();

        let size = (self.size)([width as f32, height as f32], boundary_size, ctx);
        let offset = (self.offset)([width as f32, height as f32], boundary_size, ctx);

        Range2D::new_unchecked(
            [offset[0], -size[1] - offset[1]],
            [size[0] + offset[0], -offset[1]],
        )
    }

    fn draw(
        &self,
        render_pass: &mut wgpu::RenderPass<'_>,
        target_size: [u32; 2],
        target_format: wgpu::TextureFormat,
        boundary_size: [f32; 2],
        offset: [f32; 2],
        ctx: &WidgetContext,
    ) {
        let cache_map = ctx.common_resource().get_or_insert_default::<ImageCache>();
        let key = self.key();
        let image_cache = cache_map
            .map
            .entry(key)
            .or_insert_with(|| image_data(&self.image, ctx));

        if let Some(image_cache) = &image_cache.value() {
            // render

            let texture_offset = offset;
            let draw_range = self.draw_range(boundary_size, ctx);
            let relative_position = draw_range.slide(texture_offset);
            let min_point = relative_position.min_point();
            let max_point = relative_position.max_point();

            let texture_copy_renderer =
                ctx.common_resource()
                    .get_or_insert_default::<matcha_core::renderer::texture_copy::TextureCopy>();

            texture_copy_renderer.render(
                render_pass,
                matcha_core::renderer::texture_copy::TargetData {
                    target_size,
                    target_format,
                },
                matcha_core::renderer::texture_copy::RenderData {
                    source_texture_view: &image_cache.texture.create_view(&Default::default()),
                    source_texture_position: [min_point, max_point],
                    color_transformation: Some(color_transform(image_cache.color_type)),
                    color_offset: Some(color_offset(image_cache.color_type)),
                },
                ctx,
            );
        }
    }
}

fn image_data(image_source: &ImageSource, ctx: &WidgetContext) -> Option<ImageCacheData> {
    // load the image from the source

    let dynamic_image = match image_source {
        ImageSource::Path(path) => image::open(path).ok(),
        ImageSource::Slice { data, .. } => image::load_from_memory(data).ok(),
        ImageSource::Vec { data, .. } => image::load_from_memory(data).ok(),
    };

    let Some(dynamic_image) = dynamic_image else {
        // If the image could not be loaded, return an empty cache entry
        return None;
    };

    // Create a texture and upload image data

    match dynamic_image.color() {
        image::ColorType::L8 => make_cache(dynamic_image, wgpu::TextureFormat::R8Snorm, ctx),
        image::ColorType::L16 => make_cache(dynamic_image, wgpu::TextureFormat::R16Snorm, ctx),
        image::ColorType::La8 => make_cache(dynamic_image, wgpu::TextureFormat::Rg8Snorm, ctx),
        image::ColorType::La16 => make_cache(dynamic_image, wgpu::TextureFormat::Rg16Snorm, ctx),
        image::ColorType::Rgb8 => {
            // Convert to RGBA8 because wgpu do not support RGB8 format
            let image = image::DynamicImage::ImageRgba8(dynamic_image.to_rgba8());
            make_cache(image, wgpu::TextureFormat::Rgba8Unorm, ctx)
        }
        image::ColorType::Rgb16 => {
            // Convert to RGBA16 because wgpu do not support RGB16 format
            let image = image::DynamicImage::ImageRgba16(dynamic_image.to_rgba16());
            make_cache(image, wgpu::TextureFormat::Rgba16Unorm, ctx)
        }
        image::ColorType::Rgba8 => make_cache(dynamic_image, wgpu::TextureFormat::Rgba8Unorm, ctx),
        image::ColorType::Rgba16 => {
            make_cache(dynamic_image, wgpu::TextureFormat::Rgba16Unorm, ctx)
        }
        image::ColorType::Rgb32F => {
            // Convert to RGBA32F because wgpu do not support RGB32F format
            let image = image::DynamicImage::ImageRgba32F(dynamic_image.to_rgba32f());
            make_cache(image, wgpu::TextureFormat::Rgba32Float, ctx)
        }
        image::ColorType::Rgba32F => {
            make_cache(dynamic_image, wgpu::TextureFormat::Rgba32Float, ctx)
        }
        _ => unimplemented!("Unsupported image color type: {:?}", dynamic_image.color()),
    }
    .into()
}

fn make_cache(
    image: image::DynamicImage,
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
            bytes_per_row: Some(format.target_pixel_byte_cost().unwrap() * width),
            rows_per_image: None,
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    let color_type = image.color();

    ImageCacheData {
        data: image,
        color_type,
        texture,
    }
}

#[rustfmt::skip]
fn color_transform(color_type: image::ColorType) -> nalgebra::Matrix4<f32> {
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

fn color_offset(color_type: image::ColorType) -> [f32; 4] {
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
