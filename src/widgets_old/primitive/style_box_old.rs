use std::{cell::LazyCell, sync::Arc};

use crate::{
    context::SharedContext,
    events::UiEvent,
    renderer::Renderer,
    types::{
        range::Range2D,
        size::{Size, StdSize},
    },
    ui::{Dom, DomComPareResult, Object, TextureObject, Widget},
    widgets::paint::Paint,
};

use crate::widgets::style::{Border, BoxSizing, Padding, Visibility};

mod blur_renderer;
mod texture_renderer;

// MARK: DOM

pub struct StyleBox<T: Send + 'static> {
    // label
    label: Option<String>,

    // layout
    size: [Size; 2],
    padding: Padding,
    box_sizing: BoxSizing,
    visibility: Visibility,

    // border painting
    border_shape: Border,
    border_paint: Vec<Paint>,

    // background painting
    background_paint: Vec<Paint>,

    // content
    content: Option<Box<dyn Dom<T>>>,
}

/// build chain
impl<T: Send + 'static> StyleBox<T> {
    pub fn new(label: Option<&str>) -> Box<Self> {
        Box::new(Self {
            label: label.map(|s| s.to_string()),
            size: [Size::Content(1.0), Size::Content(1.0)],
            padding: Padding::default(),
            box_sizing: BoxSizing::default(),
            visibility: Visibility::Visible,
            border_shape: Border::default(),
            border_paint: Vec::new(),
            background_paint: Vec::new(),
            content: None,
        })
    }

    pub fn size(mut self, width: Size, height: Size) -> Self {
        self.size = [width, height];
        self
    }

    pub fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    pub fn box_sizing(mut self, box_sizing: BoxSizing) -> Self {
        self.box_sizing = box_sizing;
        self
    }

    pub fn visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn border_shape(mut self, border_shape: Border) -> Self {
        self.border_shape = border_shape;
        self
    }

    pub fn border_paint(mut self, border_paint: Vec<Paint>) -> Self {
        self.border_paint = border_paint;
        self
    }

    pub fn push_border_paint(mut self, paint: Paint) -> Self {
        self.border_paint.push(paint);
        self
    }

    pub fn background_paint(mut self, paint: Vec<Paint>) -> Self {
        self.background_paint = paint;
        self
    }

    pub fn push_background_paint(mut self, paint: Paint) -> Self {
        self.background_paint.push(paint);
        self
    }

    pub fn content(mut self, content: Box<dyn Dom<T>>) -> Self {
        self.content = Some(content);
        self
    }
}

impl<T: Send + 'static> Dom<T> for StyleBox<T> {
    fn build_widget_tree(&self) -> (Box<dyn Widget<T>>, bool) {
        let mut has_dynamic = false;

        let content = self.content.as_ref().map(|content| {
            let (c, d) = content.build_widget_tree();
            has_dynamic |= d;
            c
        });

        (
            Box::new(StyleBoxNode {
                label: self.label.clone(),
                size: self.size,
                padding: self.padding,
                box_sizing: self.box_sizing,
                visibility: self.visibility,
                border_shape: self.border_shape,
                border_paint: self.border_paint.clone(),
                background_paint: self.background_paint.clone(),
                content,
                redraw: true,
                has_dynamic,
                blur_renderer: None,
                texture_renderer: None,
                cache: None,
            }),
            has_dynamic,
        )
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// MARK: Widget

pub struct StyleBoxNode<T: Send + 'static> {
    // label
    label: Option<String>,
    // layout
    size: [Size; 2],
    padding: Padding,
    box_sizing: BoxSizing,
    visibility: Visibility,
    // border
    border_shape: Border,
    border_paint: Vec<Paint>,
    // background
    background_paint: Vec<Paint>,

    // content
    content: Option<Box<dyn Widget<T>>>,

    // draw control
    redraw: bool,
    has_dynamic: bool,

    // renderer
    // this will be shared globally in the future
    blur_renderer: Option<blur_renderer::BlurRenderer>,
    texture_renderer: Option<texture_renderer::TextureRenderer>,
    // cache
    cache: Option<Cache>,
}

struct Cache {
    // id
    size: [StdSize; 2],
    tag: u64,
    // data
    px_size: [f32; 2],
    content_size: [StdSize; 2],
    // object cache
    object: Option<Object>,
    // render states cache
    render_states: Option<RenderStates>,
}

#[derive(Default)]
struct RenderStates {
    // viewport info
    viewport_info: Option<BufferData<texture_renderer::ViewportInfo>>,

    // background settings
    background_settings: Option<BufferData<texture_renderer::Settings>>,

    // border settings
    border_settings: Option<BufferData<texture_renderer::Settings>>,

    // vertex / index buffers
    vertex_buffer: Option<wgpu::Buffer>,
    bg_index_buffer: Option<wgpu::Buffer>,
    border_index_buffer: Option<wgpu::Buffer>,
}

struct BufferData<T> {
    raw: T,
    buffer: wgpu::Buffer,
    binding_group: wgpu::BindGroup,
}

// MARK: Widget trait

impl<T: Send + 'static> Widget<T> for StyleBoxNode<T> {
    // label
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    // for dom handling
    // keep in mind to change redraw flag to true if some change is made.
    fn update_widget_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<StyleBox<T>>() {
            Err(())
        } else {
            let dom = dom.as_any().downcast_ref::<StyleBox<T>>().unwrap();
            let _ = dom;
            todo!()
        }
    }

    // comparing dom
    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(_) = dom.as_any().downcast_ref::<StyleBox<T>>() {
            todo!()
        } else {
            DomComPareResult::Different
        }
    }

    // widget event
    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> crate::events::UiEventResult<T> {
        let _ = (event, parent_size, context);
        // todo
        Default::default()
    }

    // inside / outside check
    fn is_inside(
        &mut self,
        position: [f32; 2],
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> bool {
        let px_size = Widget::<T>::px_size(self, parent_size, context);

        !(position[0] < 0.0
            || position[0] > px_size[0]
            || position[1] < 0.0
            || position[1] > px_size[1])
    }

    // The size configuration of the widget.
    fn size(&self) -> [Size; 2] {
        self.size
    }

    // Actual size including its sub widgets with pixel value.
    fn px_size(&mut self, parent_size: [StdSize; 2], context: &SharedContext) -> [f32; 2] {
        // delete cache if size miss match
        if let Some(cache) = &self.cache {
            if cache.size != parent_size {
                self.cache = None;
            }
        }

        // get cache or calculate
        self.cache
            .get_or_insert_with(|| {
                let (px_size, content_std_size) = inside_outside_size(
                    self.size,
                    self.box_sizing,
                    self.padding,
                    self.border_shape,
                    self.content.as_mut(),
                    parent_size,
                    context,
                );
                Cache {
                    size: parent_size,
                    tag: 0,
                    px_size,
                    content_size: content_std_size,
                    object: None,
                    render_states: None,
                }
            })
            .px_size
    }

    // The drawing range of the whole widget.
    fn draw_range(
        &mut self,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> Option<Range2D<f32>> {
        let px_size = self.px_size(parent_size, context);
        Some(Range2D::new([0.0, px_size[0]], [0.0, px_size[1]]).unwrap())
    }

    // The area that the widget always covers.
    fn cover_area(
        &mut self,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> Option<Range2D<f32>> {
        // if background is transparent, return None
        for paint in &self.background_paint {
            if !paint.is_opaque() {
                return None;
            }
        }

        let max_border_radius = [
            self.border_shape.top_left_radius,
            self.border_shape.top_right_radius,
            self.border_shape.bottom_left_radius,
            self.border_shape.bottom_right_radius,
        ]
        .iter()
        .fold(0.0f32, |acc, &r| acc.max(r));

        let reduction = max_border_radius * (1.0 - (1.0 / (2.0f32).sqrt()));

        let px_size = self.px_size(parent_size, context);

        Some(
            Range2D::new(
                [reduction, px_size[0] - reduction],
                [reduction, px_size[1] - reduction],
            )
            .unwrap(),
        )
    }

    // if there is any dynamic widget in children
    fn has_dynamic(&self) -> bool {
        self.has_dynamic
    }

    // if redraw is needed
    fn redraw(&self) -> bool {
        self.redraw
    }

    // MARK: Widget::render()
    // render
    fn render(
        &mut self,
        // ui environment
        parent_size: [StdSize; 2],
        background_view: &wgpu::TextureView,
        background_position: Range2D<f32>,
        // context
        context: &SharedContext,
        renderer: &Renderer,
        frame: u64,
    ) -> Vec<Object> {
        // delete cache if size miss match
        if let Some(cache) = &self.cache {
            if cache.size != parent_size {
                self.cache = None;
            }
        }

        // get cache or calculate
        let cache = self.cache.get_or_insert_with(|| {
            let (px_size, content_std_size) = inside_outside_size(
                self.size,
                self.box_sizing,
                self.padding,
                self.border_shape,
                self.content.as_mut(),
                parent_size,
                context,
            );
            Cache {
                size: parent_size,
                tag: 0,
                px_size,
                content_size: content_std_size,
                object: None,
                render_states: None,
            }
        });

        // get object or render it
        let object = cache.object.get_or_insert_with(|| {
            let device = context.get_wgpu_device();

            // prepare texture to render
            // lazy init
            let render_attachment = LazyCell::new(|| {
                device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("style_box_texture"),
                    size: wgpu::Extent3d {
                        width: cache.px_size[0] as u32,
                        height: cache.px_size[1] as u32,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                })
            });

            // render background
            for paint in &self.background_paint {
                match paint {
                    Paint::Blur(px) => {
                        todo!()
                    }
                    Paint::Solid(color) => {
                        todo!()
                    }
                }
            }

            // render border
            for paint in &self.border_paint {
                match paint {
                    Paint::Blur(px) => {
                        todo!()
                    }
                    Paint::Solid(color) => {
                        todo!()
                    }
                }
            }

            // render content

            // now the texture is ready, create object
            let texture_object = TextureObject {
                texture: Arc::new(render_attachment),
                uv_vertices: todo!(),
                indices: todo!(),
                transform: todo!(),
            };

            Object::TextureObject(texture_object)
        });

        vec![object.clone()]
    }
}

fn render_blur(
    render_attachment: wgpu::TextureView,
    renderer: blur_renderer::BlurRenderer,
    bg_texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    viewport_info: BufferData<texture_renderer::ViewportInfo>,
    settings: BufferData<texture_renderer::Settings>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
) {
    // This method will render the blur effect from the background texture to the render attachment.

    // create blur renderer

    // create viewport info

    // create settings

    todo!()
}

fn render_raster(
    render_attachment: wgpu::TextureView,
    renderer: Renderer,
    texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    viewport_info: BufferData<texture_renderer::ViewportInfo>,
    settings: BufferData<texture_renderer::Settings>,
    vertex_buffer: wgpu::Buffer,
    bg_index_buffer: wgpu::Buffer,
    border_index_buffer: wgpu::Buffer,
) {
    // This method will render the raster effect from the background texture to the render attachment.

    // create raster renderer

    // create viewport info

    // create settings

    todo!()
}

// calculate size from properties
fn inside_outside_size<T>(
    // selfs
    size: [Size; 2],
    box_sizing: BoxSizing,
    padding: Padding,
    border_shape: Border,
    mut content: Option<&mut Box<dyn Widget<T>>>,
    // args
    parent_size: [StdSize; 2],
    context: &SharedContext,
) -> ([f32; 2], [StdSize; 2]) {
    let current_std_size = [
        size[0].to_std_size(parent_size[0], context),
        size[1].to_std_size(parent_size[1], context),
    ];

    match box_sizing {
        BoxSizing::ContentBox => {
            match current_std_size {
                [StdSize::Pixel(w), StdSize::Pixel(h)] => (
                    [
                        w + padding.left + padding.right + border_shape.px * 2.0,
                        h + padding.top + padding.bottom + border_shape.px * 2.0,
                    ],
                    current_std_size,
                ),
                _ => {
                    // need ask children
                    let content_size = content
                        .as_mut()
                        .map(|content| content.px_size(current_std_size, context));

                    let width = match current_std_size[0] {
                        StdSize::Pixel(px) => px,
                        StdSize::Content(_) => content_size.map_or(0.0, |s| s[0]),
                    };

                    let height = match current_std_size[1] {
                        StdSize::Pixel(px) => px,
                        StdSize::Content(_) => content_size.map_or(0.0, |s| s[1]),
                    };

                    (
                        [
                            width + padding.left + padding.right + border_shape.px * 2.0,
                            height + padding.top + padding.bottom + border_shape.px * 2.0,
                        ],
                        current_std_size,
                    )
                }
            }
        }
        BoxSizing::BorderBox => {
            match current_std_size {
                [StdSize::Pixel(w), StdSize::Pixel(h)] => (
                    [w, h],
                    [
                        StdSize::Pixel((w - padding.left - padding.right).max(0.0)),
                        StdSize::Pixel((h - padding.top - padding.bottom).max(0.0)),
                    ],
                ),
                _ => {
                    // need ask children
                    let content_size = content
                        .as_mut()
                        .map(|content| content.px_size(current_std_size, context));

                    let width = match current_std_size[0] {
                        StdSize::Pixel(px) => px,
                        StdSize::Content(_) => {
                            content_size.map_or(0.0, |s| s[0])
                                + padding.left
                                + padding.right
                                + border_shape.px * 2.0
                        }
                    };

                    let height = match current_std_size[1] {
                        StdSize::Pixel(px) => px,
                        StdSize::Content(_) => {
                            content_size.map_or(0.0, |s| s[1])
                                + padding.top
                                + padding.bottom
                                + border_shape.px * 2.0
                        }
                    };

                    (
                        [width, height],
                        [
                            match current_std_size[0] {
                                StdSize::Pixel(px) => StdSize::Pixel(
                                    (px - padding.left - padding.right - border_shape.px * 2.0)
                                        .max(0.0),
                                ),
                                StdSize::Content(c) => StdSize::Content(c),
                            },
                            match current_std_size[1] {
                                StdSize::Pixel(px) => StdSize::Pixel(
                                    (px - padding.top - padding.bottom - border_shape.px * 2.0)
                                        .max(0.0),
                                ),
                                StdSize::Content(c) => StdSize::Content(c),
                            },
                        ],
                    )
                }
            }
        }
    }
}
