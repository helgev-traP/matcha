use std::{any::Any, sync::Arc};

use matcha_core::{
    context::WidgetContext,
    events::Event,
    observer::Observer,
    renderer::{RendererSetup, RendererMap},
    types::{
        cache::Cache,
        range::{CoverRange, Range2D},
    },
    ui::{Background, Dom, DomComPareResult, Object, UpdateWidgetError, Widget},
    vertex::UvVertex,
};

// todo: more documentation

type SizeFn = dyn for<'a> Fn([Option<f32>; 2], [f32; 2], &'a WidgetContext) -> [f32; 2]
    + Send
    + Sync
    + 'static;

// MARK: DOM

pub struct Image {
    label: Option<String>,

    image: Option<ImageSource>,
    size: Arc<SizeFn>,
}

#[derive(Clone)]
pub enum ImageSource {
    Path(String),
    Data(&'static [u8]),
}

impl Image {
    pub fn new(label: Option<&str>) -> Box<Self> {
        Box::new(Self {
            label: label.map(|s| s.to_string()),
            image: None,
            size: Arc::new(|_, size, _| size),
        })
    }

    pub fn image_path(mut self, path: &str) -> Self {
        self.image = Some(ImageSource::Path(path.to_string()));
        self
    }

    pub fn image_data(mut self, data: &'static [u8]) -> Self {
        self.image = Some(ImageSource::Data(data));
        self
    }

    pub fn size<F>(mut self, size: F) -> Self
    where
        F: Fn([Option<f32>; 2], [f32; 2], &WidgetContext) -> [f32; 2] + Send + Sync + 'static,
    {
        self.size = Arc::new(size);
        self
    }
}

#[async_trait::async_trait]
impl<T: Send + 'static> Dom<T> for Image {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(ImageNode {
            label: self.label.clone(),
            image_source: self.image.clone(),
            size: self.size.clone(),
            image: None,
            size_cache: Cache::new(),
            redraw: true,
            texture: None,
            polygon: None,
        })
    }

    async fn collect_observer(&self) -> Observer {
        // If your widget has any child widgets,
        // you should collect their observers for matcha ui system to catch child component updates.
        Observer::default()
    }
}

// MARK: Widget
pub struct ImageNode {
    label: Option<String>,
    image_source: Option<ImageSource>,
    size: Arc<SizeFn>,

    // cache
    image: Option<Option<image::DynamicImage>>,
    size_cache: Cache<[Option<f32>; 2], [f32; 2]>,

    // redraw flag
    redraw: bool,

    // render cache
    texture: Option<Arc<wgpu::Texture>>,
    polygon: Option<(Vec<UvVertex>, Vec<u16>)>,
}

impl ImageNode {
    fn load_image(source: &Option<ImageSource>) -> Option<image::DynamicImage> {
        if let Some(image) = source {
            match image {
                ImageSource::Path(path) => Some(image::open(path).ok()?),
                ImageSource::Data(data) => Some(image::load_from_memory(data).ok()?),
            }
        } else {
            None
        }
    }
}

// MARK: Widget trait

#[async_trait::async_trait]
impl<T: Send + 'static> Widget<T> for ImageNode {
    // label
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    // for dom handling
    // keep in mind to change redraw flag to true if some change is made.
    async fn update_widget_tree(
        &mut self,
        component_updated: bool,
        dom: &dyn Dom<T>,
    ) -> Result<(), UpdateWidgetError> {
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Image>() {
            todo!()
        } else {
            return Err(UpdateWidgetError::TypeMismatch);
        }
    }

    // comparing dom
    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Image>() {
            todo!()
        } else {
            DomComPareResult::Different
        }
    }

    // widget event
    fn widget_event(
        &mut self,
        event: &Event,
        parent_size: [Option<f32>; 2],
        context: &WidgetContext,
    ) -> Option<T> {
        let _ = (event, parent_size, context);
        None
    }

    // Actual size including its sub widgets with pixel value.
    fn px_size(&mut self, parent_size: [Option<f32>; 2], context: &WidgetContext) -> [f32; 2] {
        *self.size_cache.get_data_or_insert_with(&parent_size, || {
            let image = self
                .image
                .get_or_insert_with(|| Self::load_image(&self.image_source));

            let image_size = image.as_ref().map_or([0.0, 0.0], |image| {
                [image.width() as f32, image.height() as f32]
            });

            (self.size)(parent_size, image_size, context)
        })
    }

    // The drawing range and the area that the widget always covers.
    fn cover_range(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &WidgetContext,
    ) -> CoverRange<f32> {
        let px_size = Widget::<T>::px_size(self, parent_size, context);

        let range = Range2D::new([0.0, 0.0], px_size);
        CoverRange::new(range, range)
    }

    // if redraw is needed
    fn updated(&self) -> bool {
        self.redraw
    }

    // render
    fn render(
        &mut self,
        parent_size: [Option<f32>; 2],
        background: Background,
        ctx: &WidgetContext,
    ) -> Vec<Object> {
        let image = self
            .image
            .get_or_insert_with(|| Self::load_image(&self.image_source));
        if image.is_none() {
            return vec![];
        }
        let image = image.as_ref().unwrap();

        // calculate size
        let px_size = self.size_cache.get_data_or_insert_with(&parent_size, || {
            let image_size = [image.width() as f32, image.height() as f32];
            (self.size)(parent_size, image_size, ctx)
        });

        // prepare texture
        let texture = self.texture.get_or_insert_with(|| {
            let device = ctx.device();
            let queue = ctx.queue();

            let image_rgba = image.to_rgba8();

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Image Texture"),
                size: wgpu::Extent3d {
                    width: image.width(),
                    height: image.height(),
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
                image_rgba.as_raw(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * image.width()),
                    rows_per_image: Some(image.height()),
                },
                wgpu::Extent3d {
                    width: image.width(),
                    height: image.height(),
                    depth_or_array_layers: 1,
                },
            );

            // todo: remove
            queue.submit([]);

            Arc::new(texture)
        });

        let (vertices, indices) = self.polygon.get_or_insert_with(|| {
            let vertices = vec![
                UvVertex {
                    position: [0.0, 0.0, 0.0].into(),
                    uv: [0.0, 0.0].into(),
                },
                UvVertex {
                    position: [0.0, -px_size[1], 0.0].into(),
                    uv: [0.0, 1.0].into(),
                },
                UvVertex {
                    position: [px_size[0], -px_size[1], 0.0].into(),
                    uv: [1.0, 1.0].into(),
                },
                UvVertex {
                    position: [px_size[0], 0.0, 0.0].into(),
                    uv: [1.0, 0.0].into(),
                },
            ];
            let indices = vec![0, 1, 2, 2, 3, 0];

            (vertices, indices)
        });

        vec![Object::TextureColor {
            texture: texture.clone(),
            uv_vertices: vertices,
            indices,
            transform: nalgebra::Matrix4::new_translation(&[0.0, 0.0, 0.0].into()),
        }]
    }
}
