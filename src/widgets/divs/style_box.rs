use std::sync::Arc;

use rayon::vec;

use crate::{
    context::SharedContext,
    events::UiEvent,
    renderer::Renderer,
    types::{
        color::Color, range::Range2D, size::{Size, StdSize}
    },
    ui::{Dom, DomComPareResult, Object, TextureObject, Widget},
    vertex::uv_vertex::{self, UvVertex},
};

use super::style::{Border, BoxSizing, Padding, Visibility};

pub struct StyleBox<T: Send + 'static> {
    // label
    label: Option<String>,

    // style
    size: [Size; 2],
    border: Border,
    padding: Padding,
    box_sizing: BoxSizing,
    visibility: Visibility,
    background_color: Color,

    // content
    content: Option<Box<dyn Dom<T>>>,
}

/// build chain
impl<T: Send + 'static> StyleBox<T> {
    pub fn new(label: Option<&str>) -> Box<Self> {
        Box::new(Self {
            label: label.map(|s| s.to_string()),
            size: [Size::Content(1.0), Size::Content(1.0)],
            border: Border::default(),
            padding: Padding::default(),
            box_sizing: BoxSizing::default(),
            visibility: Visibility::Visible,
            background_color: Color::default(),
            content: None,
        })
    }

    pub fn size(mut self, width: Size, height: Size) -> Self {
        self.size = [width, height];
        self
    }

    pub fn border(mut self, border: Border) -> Self {
        self.border = border;
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

    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    pub fn content(mut self, children: Box<dyn Dom<T>>) -> Self {
        self.content = Some(children);
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
                border: self.border,
                padding: self.padding,
                box_sizing: self.box_sizing,
                visibility: self.visibility,
                background_color: self.background_color,
                content,
                has_dynamic,
                redraw: true,
                size_cache: std::cell::Cell::new(None),
                vello_scene: None,
                background_object: None,
                render_cache_object: None,
            }),
            has_dynamic,
        )
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct StyleBoxNode<T: Send + 'static> {
    // label
    label: Option<String>,
    // style
    size: [Size; 2],
    border: Border,
    padding: Padding,
    box_sizing: BoxSizing,
    visibility: Visibility,
    background_color: Color,
    // content
    content: Option<Box<dyn Widget<T>>>,
    // has dynamic
    has_dynamic: bool,
    // draw control
    redraw: bool,
    // cache
    size_cache: std::cell::Cell<Option<([f32; 2], [StdSize; 2])>>,
    vello_scene: Option<vello::Scene>,
    background_object: Option<Object>,
    render_cache_object: Option<Object>,
}

impl<T: Send + 'static> StyleBoxNode<T> {
    fn inside_outside_size(
        &self,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> ([f32; 2], [StdSize; 2]) {
        let current_std_size = [
            self.size[0].to_std_size(parent_size[0], context),
            self.size[1].to_std_size(parent_size[1], context),
        ];
        match self.box_sizing {
            BoxSizing::ContentBox => {
                match current_std_size {
                    [StdSize::Pixel(w), StdSize::Pixel(h)] => (
                        [
                            w + self.padding.left + self.padding.right + self.border.px * 2.0,
                            h + self.padding.top + self.padding.bottom + self.border.px * 2.0,
                        ],
                        current_std_size,
                    ),
                    _ => {
                        // need ask children
                        let content_size = self
                            .content
                            .as_ref()
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
                                width
                                    + self.padding.left
                                    + self.padding.right
                                    + self.border.px * 2.0,
                                height
                                    + self.padding.top
                                    + self.padding.bottom
                                    + self.border.px * 2.0,
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
                            StdSize::Pixel((w - self.padding.left - self.padding.right).max(0.0)),
                            StdSize::Pixel((h - self.padding.top - self.padding.bottom).max(0.0)),
                        ],
                    ),
                    _ => {
                        // need ask children
                        let content_size = self
                            .content
                            .as_ref()
                            .map(|content| content.px_size(current_std_size, context));

                        let width = match current_std_size[0] {
                            StdSize::Pixel(px) => px,
                            StdSize::Content(_) => {
                                content_size.map_or(0.0, |s| s[0])
                                    + self.padding.left
                                    + self.padding.right
                                    + self.border.px * 2.0
                            }
                        };

                        let height = match current_std_size[1] {
                            StdSize::Pixel(px) => px,
                            StdSize::Content(_) => {
                                content_size.map_or(0.0, |s| s[1])
                                    + self.padding.top
                                    + self.padding.bottom
                                    + self.border.px * 2.0
                            }
                        };

                        (
                            [width, height],
                            [
                                match current_std_size[0] {
                                    StdSize::Pixel(px) => StdSize::Pixel(
                                        (px - self.padding.left
                                            - self.padding.right
                                            - self.border.px * 2.0)
                                            .max(0.0),
                                    ),
                                    StdSize::Content(c) => StdSize::Content(c),
                                },
                                match current_std_size[1] {
                                    StdSize::Pixel(px) => StdSize::Pixel(
                                        (px - self.padding.top
                                            - self.padding.bottom
                                            - self.border.px * 2.0)
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
}

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
        &self,
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
    fn px_size(&self, parent_size: [StdSize; 2], context: &SharedContext) -> [f32; 2] {
        if let Some(cache) = self.size_cache.get() {
            return cache.0;
        } else {
            let (px_size, content_std_size) = self.inside_outside_size(parent_size, context);
            self.size_cache.set(Some((px_size, content_std_size)));
            px_size
        }
    }

    // The drawing range of the whole widget.
    fn drawing_range(&self, parent_size: [StdSize; 2], context: &SharedContext) -> [[f32; 2]; 2] {
        let px_size = self.px_size(parent_size, context);
        [[0.0, 0.0], [px_size[0], px_size[1]]]
    }

    // The area that the widget always covers.
    fn cover_area(
        &self,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> Option<[[f32; 2]; 2]> {
        // if background is transparent, return None
        if self.background_color.is_opaque() {
            // max border radius
            let max_border_radius = [
                self.border.top_left_radius,
                self.border.top_right_radius,
                self.border.bottom_left_radius,
                self.border.bottom_right_radius,
            ]
            .iter()
            .fold(0.0f32, |acc, &r| acc.max(r));

            let reduction = max_border_radius * (1.0 - (1.0 / (2.0f32).sqrt()));

            let px_size = self.px_size(parent_size, context);

            Some([
                [reduction, reduction],
                [px_size[0] - reduction, px_size[1] - reduction],
            ])
        } else {
            None
        }
    }

    // if there is any dynamic widget in children
    fn has_dynamic(&self) -> bool {
        self.has_dynamic
    }

    // if redraw is needed
    fn redraw(&self) -> bool {
        self.redraw
    }

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
        let px_size = self.px_size(parent_size, context);

        // todo: currently only support Object::TextureObject.

        // draw background

        if self.background_color.is_opaque()
            || (self.border.px > 0.0 && self.border.color.is_opaque())
        {
            // create texture etc.
            let background_object = self.background_object.get_or_insert_with(|| {
                self.redraw = true;
                Object::TextureObject(TextureObject {
                    texture: Arc::new(context.create_texture(
                        px_size[0] as u32 + 1,
                        px_size[1] as u32 + 1,
                        wgpu::TextureFormat::Rgba8Unorm,
                        wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::TEXTURE_BINDING,
                    )),
                    uv_vertices: Arc::new(vec![
                        UvVertex {
                            position: [0.0, 0.0, 0.0].into(),
                            tex_coords: [0.0, 0.0].into(),
                        },
                        UvVertex {
                            position: [0.0, -px_size[1], 0.0].into(),
                            tex_coords: [0.0, 1.0].into(),
                        },
                        UvVertex {
                            position: [px_size[0], -px_size[1], 0.0].into(),
                            tex_coords: [1.0, 1.0].into(),
                        },
                        UvVertex {
                            position: [px_size[0], 0.0, 0.0].into(),
                            tex_coords: [1.0, 0.0].into(),
                        },
                    ]),
                    indices: Arc::new(vec![0, 1, 2, 0, 2, 3]),
                    transform: nalgebra::Matrix4::identity(),
                })
            });

            let Object::TextureObject(to) = background_object else {
                unreachable!()
            };

            // prepare vello scene
            let vello_scene = self.vello_scene.get_or_insert_with(|| vello::Scene::new());
            vello_scene.reset();

            // background color
            let background_color = self.background_color.to_rgba_f64();
            if background_color[3] > 0.0 {
                vello_scene.fill(
                    vello::peniko::Fill::EvenOdd,
                    vello::kurbo::Affine::IDENTITY,
                    vello::peniko::Color::rgba(
                        background_color[0],
                        background_color[1],
                        background_color[2],
                        background_color[3],
                    ),
                    None,
                    &vello::kurbo::RoundedRect::new(
                        0.0,
                        0.0,
                        px_size[0] as f64,
                        px_size[1] as f64,
                        vello::kurbo::RoundedRectRadii::new(
                            self.border.top_left_radius as f64,
                            self.border.top_right_radius as f64,
                            self.border.bottom_right_radius as f64,
                            self.border.bottom_left_radius as f64,
                        ),
                    ),
                );
            }

            // border
            let border_color = self.border.color.to_rgba_f64();
            let border_px = self.border.px as f64;
            if border_px > 0.0 && border_color[3] > 0.0 {
                vello_scene.stroke(
                    &vello::kurbo::Stroke::new(border_px),
                    vello::kurbo::Affine::IDENTITY,
                    vello::peniko::Color::rgba(
                        border_color[0],
                        border_color[1],
                        border_color[2],
                        border_color[3],
                    ),
                    None,
                    &vello::kurbo::RoundedRect::new(
                        border_px / 2.0,
                        border_px / 2.0,
                        px_size[0] as f64 - border_px / 2.0,
                        px_size[1] as f64 - border_px / 2.0,
                        vello::kurbo::RoundedRectRadii::new(
                            self.border.top_left_radius as f64 - border_px / 2.0,
                            self.border.top_right_radius as f64 - border_px / 2.0,
                            self.border.bottom_right_radius as f64 - border_px / 2.0,
                            self.border.bottom_left_radius as f64 - border_px / 2.0,
                        ),
                    ),
                );
            }

            // render
            renderer
                .vello_renderer()
                .render_to_texture(
                    context.get_wgpu_device(),
                    context.get_wgpu_queue(),
                    &vello_scene,
                    &to.texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                    &vello::RenderParams {
                        base_color: vello::peniko::Color::TRANSPARENT,
                        width: px_size[0] as u32 + 1,
                        height: px_size[1] as u32 + 1,
                        antialiasing_method: vello::AaConfig::Area,
                    },
                )
                .unwrap();
        }

        todo!()
    }
}
