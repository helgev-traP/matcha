use std::{hash::Hasher, sync::Arc};

use dashmap::DashMap;
use matcha_core::{
    context::WidgetContext,
    device,
    types::{cache, range::Range2D},
    ui::Style,
    vertex::UvVertex,
};

// MARK: Cache

#[derive(Default)]
pub struct ImageCache {
    map: DashMap<ImageCacheKey, ImageCacheData, fxhash::FxBuildHasher>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum ImageCacheKey {
    /// Full path to the image file
    Path(String),
    /// Hash of the image data
    Data(u64),
}

struct ImageCacheData {
    /// The image data
    data: Option<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>>,
    texture: Option<wgpu::Texture>,
}

// MARK: Image

type SizeFn = dyn for<'a> Fn([f32; 2], &'a WidgetContext) -> [f32; 2] + Send + Sync + 'static;

pub struct Image {
    image: Option<ImageSource>,
    size: Arc<SizeFn>,
    offset: Arc<SizeFn>,
}

#[derive(Clone)]
enum ImageSource {
    Path(String),
    Data { data: &'static [u8], hash: u64 },
}

impl Image {
    pub fn new_from_path(path: &str) -> Self {
        Self {
            image: Some(ImageSource::Path(path.to_string())),
            size: Arc::new(|size, _| size),
            offset: Arc::new(|_, _| [0.0, 0.0]),
        }
    }

    pub fn new_from_data(data: &'static [u8]) -> Self {
        Self {
            image: Some(ImageSource::Data {
                data,
                hash: hash_data(data),
            }),
            size: Arc::new(|size, _| size),
            offset: Arc::new(|_, _| [0.0, 0.0]),
        }
    }

    pub fn size<F>(mut self, size: F) -> Self
    where
        F: Fn([f32; 2], &WidgetContext) -> [f32; 2] + Send + Sync + 'static,
    {
        self.size = Arc::new(size);
        self
    }
}

fn hash_data(data: &[u8]) -> u64 {
    let mut hasher = fxhash::FxHasher::default();
    hasher.write(data);
    hasher.finish()
}

impl Style for Image {
    fn draw_range(&mut self, boundary: [f32; 2], ctx: &WidgetContext) -> Range2D<f32> {
        let size = (self.size)(boundary, ctx);
        let offset = (self.offset)(boundary, ctx);

        Range2D::new_unchecked(
            [offset[0], offset[1]],
            [size[0] + offset[0], size[1] + offset[1]],
        )
    }

    fn draw(
        &mut self,
        boundary: [f32; 2],
        render_pass: &wgpu::RenderPass<'_>,
        texture_size: [u32; 2],
        offset: [f32; 2],
        ctx: &WidgetContext,
    ) {
        // get cache map
        let cache_map = ctx.common_resource().get_or_insert_default::<ImageCache>();

        let key = match &self.image {
            Some(ImageSource::Path(path)) => ImageCacheKey::Path(path.clone()),
            Some(ImageSource::Data { data, hash }) => ImageCacheKey::Data(*hash),
            None => return, // No image source provided
        };

        // get or insert the image data
        let image = cache_map.map.entry(key).or_insert_with(|| {
            let image_data = match &self.image {
                Some(ImageSource::Path(path)) => image::open(path).ok(),
                Some(ImageSource::Data { data, .. }) => image::load_from_memory(data).ok(),
                None => unreachable!(),
            };

            let Some(image_data) = image_data else {
                // If the image could not be loaded, return an empty cache entry
                return ImageCacheData {
                    data: None,
                    texture: None,
                };
            };

            let image_size = image::GenericImageView::dimensions(&image_data);
            let image_rgba = image_data.into_rgba8();

            // create texture and upload image data

            let device = ctx.device();
            let queue = ctx.queue();

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Image Texture"),
                size: wgpu::Extent3d {
                    width: image_size.0,
                    height: image_size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: ctx.texture_format(),
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
                image_rgba.as_raw(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * image_size.0),
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width: image_size.0,
                    height: image_size.1,
                    depth_or_array_layers: 1,
                },
            );

            ImageCacheData {
                data: Option::Some(image_rgba),
                texture: Some(texture),
            }
        });

        // render

        let texture_offset = offset;
        let draw_range = self.draw_range(boundary, ctx);
        let relative_position = draw_range.slide([-texture_offset[0], -texture_offset[1]]);
        let relative_position_x = relative_position.x_range();
        let relative_position_y = relative_position.y_range();

        let vertices = [UvVertex {
            position: [relative_position_x[0], relative_position_y[0], 0.0].into(),
            uv: [0.0, 0.0].into(),
        }];

        todo!("render to texture");
    }
}
