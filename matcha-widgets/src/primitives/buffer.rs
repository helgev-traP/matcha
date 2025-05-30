use std::{any::Any, sync::Arc};

use matcha_core::{
    context::WidgetContext,
    events::Event,
    observer::Observer,
    common_resource::principle_renderer::PrincipleRenderer,
    types::
        range::{CoverRange, Range2D}
    ,
    ui::{Background, Dom, DomComPareResult, Object, UpdateWidgetError, Widget},
    vertex::UvVertex,
};
use nalgebra::Vector3;

// MARK: DOM

pub struct TextureBuffer<T = ()> {
    label: Option<String>,

    content: Option<Box<dyn Dom<T>>>,
}

impl TextureBuffer {
    pub fn new(label: Option<&str>) -> Box<Self> {
        Box::new(Self {
            label: label.map(|s| s.to_string()),
            content: None,
        })
    }

    pub fn content(mut self, content: Box<dyn Dom<()>>) -> Self {
        self.content = Some(content);
        self
    }
}

#[async_trait::async_trait]
impl<T: Send + 'static> Dom<T> for TextureBuffer<T> {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(TextureBufferNode {
            label: self.label.clone(),
            content: self.content.as_ref().map(|c| c.build_widget_tree()),
            cache: None,
        })
    }

    async fn collect_observer(&self) -> Observer {
        // If your widget has any child widgets,
        // you should collect their observers for matcha ui system to catch child component updates.

        Observer::default()
    }
}

// MARK: Widget

pub struct TextureBufferNode<T> {
    label: Option<String>,

    content: Option<Box<dyn Widget<T>>>,

    cache: Option<Cache>,
}

struct Cache {
    texture: Arc<wgpu::Texture>,
    texture_position: Range2D,
}

// MARK: Widget trait

#[async_trait::async_trait]
impl<T: Send + 'static> Widget<T> for TextureBufferNode<T> {
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
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<TextureBuffer>() {
            todo!()
        } else {
            return Err(UpdateWidgetError::TypeMismatch);
        }
    }

    // comparing dom
    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<TextureBuffer<T>>() {
            match (&self.content, dom.content.as_ref()) {
                (None, None) => DomComPareResult::Same,
                (None, Some(_)) | (Some(_), None) => DomComPareResult::Different,
                (Some(content_w), Some(content_d)) => content_w.compare(content_d.as_ref()),
            }
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
        self.content
            .as_mut()
            .and_then(|content| content.widget_event(event, parent_size, context))
    }

    // inside / outside check
    // implement this if your widget has a non rectangular shape or has transparent area.

    fn is_inside(
        &mut self,
        position: [f32; 2],
        parent_size: [Option<f32>; 2],
        context: &WidgetContext,
    ) -> bool {
        self.content
            .as_mut()
            .is_some_and(|content| content.is_inside(position, parent_size, context))
    }

    // Actual size including its sub widgets with pixel value.
    fn px_size(&mut self, parent_size: [Option<f32>; 2], context: &WidgetContext) -> [f32; 2] {
        self.content
            .as_mut()
            .map_or([0.0, 0.0], |content| content.px_size(parent_size, context))
    }

    // The drawing range and the area that the widget always covers.
    fn cover_range(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &WidgetContext,
    ) -> CoverRange<f32> {
        self.content
            .as_mut()
            .map_or(CoverRange::new(None, None), |content| {
                content.cover_range(parent_size, context)
            })
    }

    // if redraw is needed
    fn updated(&self) -> bool {
        self.content
            .as_ref()
            .is_some_and(|content| content.updated())
    }

    // render
    fn render(
        &mut self,
        parent_size: [Option<f32>; 2],
        background: Background,
        ctx: &WidgetContext,
    ) -> Vec<Object> {
        if let Some(content) = &mut self.content {
            // 1. cache ready && content not updated -> use cache
            if let Some(cache) = &self.cache {
                if !content.updated() {
                    return vec![make_object(cache)];
                }
            }

            // 2. cache not ready || content updated -> render to texture and cache it
            // calculate required size for the texture
            let objects = content.render(parent_size, background, ctx);

            let mut x_min = f32::MAX;
            let mut x_max = f32::MIN;
            let mut y_min = f32::MAX;
            let mut y_max = f32::MIN;

            for object in &objects {
                match object {
                    Object::TextureColor { uv_vertices, .. } => {
                        for vertex in uv_vertices {
                            x_min = x_min.min(vertex.position[0]);
                            x_max = x_max.max(vertex.position[0]);
                            y_min = y_min.min(vertex.position[1]);
                            y_max = y_max.max(vertex.position[1]);
                        }
                    }
                    Object::VertexColor { vertices, .. } => {
                        for vertex in vertices {
                            x_min = x_min.min(vertex.position[0]);
                            x_max = x_max.max(vertex.position[0]);
                            y_min = y_min.min(vertex.position[1]);
                            y_max = y_max.max(vertex.position[1]);
                        }
                    }
                }
            }

            let x_range = [x_min, x_max];
            let y_range = [y_min, y_max];
            let texture_position = Range2D::new_unchecked(x_range, y_range);

            // try to recycle the texture in the cache
            let cache = self.cache.take();

            let mut texture = None;

            if let Some(cache) = cache {
                if texture_position.size() == cache.texture_position.size() {
                    texture = Some(cache.texture);
                }
            }

            let texture = texture.get_or_insert_with(|| {
                Arc::new(ctx.device().create_texture(&wgpu::TextureDescriptor {
                    label: self.label.as_deref(),
                    size: wgpu::Extent3d {
                        width: texture_position.width() as u32,
                        height: texture_position.height() as u32,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                }))
            });

            // render the content to the texture

            let offset = [texture_position.x_range()[0], texture_position.y_range()[0]];
            let offset =
                nalgebra::Matrix4::new_translation(&Vector3::new(offset[0], offset[1], 0.0));

            let principle_renderer = ctx
                .renderers()
                .get_or_setup::<PrincipleRenderer>(ctx);

            principle_renderer.render(
                ctx.device(),
                ctx.queue(),
                &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                [texture_position.width(), texture_position.height()],
                objects,
                Some(offset),
            );

            // store cache and return the object

            self.cache = Some(Cache {
                texture: Arc::clone(texture),
                texture_position,
            });

            self.cache
                .as_ref()
                .map(|cache| {
                    let object = make_object(cache);
                    vec![object]
                })
                .unwrap_or_default()
        } else {
            vec![]
        }
    }
}

fn make_object(cache: &Cache) -> Object {
    let x_range = cache.texture_position.x_range();
    let y_range = cache.texture_position.y_range();
    let vertices = [
        UvVertex {
            position: [x_range[0], y_range[1], 0.0].into(),
            uv: [0.0, 0.0].into(),
        },
        UvVertex {
            position: [x_range[0], y_range[0], 0.0].into(),
            uv: [0.0, 1.0].into(),
        },
        UvVertex {
            position: [x_range[1], y_range[0], 0.0].into(),
            uv: [1.0, 1.0].into(),
        },
        UvVertex {
            position: [x_range[1], y_range[1], 0.0].into(),
            uv: [1.0, 0.0].into(),
        },
    ];
    let indices = [0, 1, 2, 2, 3, 0];

    Object::TextureColor {
        texture: Arc::clone(&cache.texture),
        uv_vertices: vertices.to_vec(),
        indices: indices.to_vec(),
        transform: nalgebra::Matrix4::new_translation(&Vector3::new(x_range[0], y_range[0], 0.0)),
    }
}
