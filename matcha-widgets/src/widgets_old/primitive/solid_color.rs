use std::sync::Arc;

use nalgebra::Matrix4;

use crate::context::SharedContext;
use crate::events::{UiEvent, UiEventResult};
use crate::renderer::Renderer;
use crate::types::color::Color;
use crate::types::double_cache_set::DoubleSetCache;
use crate::types::range::Range2D;
use crate::types::size::Size;
use crate::types::size::StdSize;
use crate::ui::{Dom, DomComPareResult, Object, TextureObject, Widget};
use crate::vertex::uv_vertex::UvVertex;

use crate::widgets::style::{Border, BoxSizing, Padding, Visibility};

// MARK: DOM

pub struct SolidColor<T: Send + 'static> {
    // label
    label: Option<String>,

    // layout
    size: [Size; 2],
    padding: Padding,
    box_sizing: BoxSizing,
    visibility: Visibility,

    // border painting
    border_shape: Border,
    border_color: Color,

    // background painting
    background_color: Color,

    // content
    content: Option<Box<dyn Dom<T>>>,
}

/// build chain
impl<T: Send + 'static> SolidColor<T> {
    pub fn new(label: Option<&str>) -> Self {
        Self {
            label: label.map(|s| s.to_string()),
            size: [Size::Content(1.0), Size::Content(1.0)],
            padding: Padding::default(),
            box_sizing: BoxSizing::default(),
            visibility: Visibility::Visible,
            border_shape: Border::default(),
            border_color: [0, 0, 0, 0].into(),
            background_color: [255, 255, 255, 0].into(),
            content: None,
        }
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

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    pub fn content(mut self, content: Box<dyn Dom<T>>) -> Self {
        self.content = Some(content);
        self
    }

    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl<T: Send + 'static> Dom<T> for SolidColor<T> {
    fn build_widget_tree(&self) -> Box<dyn crate::ui::Widget<T>> {
        let content = self.content.as_ref().map(|c| c.build_widget_tree());

        let node = SolidColorNode {
            label: self.label.clone(),
            size: self.size,
            padding: self.padding,
            box_sizing: self.box_sizing,
            visibility: self.visibility,
            border_shape: self.border_shape,
            border_color: self.border_color,
            background_color: self.background_color,
            content,
            settings_cache: None,
            context_cache: DoubleSetCache::new(),
        };

        Box::new(node)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// MARK: Widget

pub struct SolidColorNode<T: Send + 'static> {
    // label
    label: Option<String>,

    // layout
    size: [Size; 2],
    padding: Padding,
    box_sizing: BoxSizing,
    visibility: Visibility,

    // border painting
    border_shape: Border,
    border_color: Color,

    // background painting
    background_color: Color,

    // content
    content: Option<Box<dyn Widget<T>>>,

    // cache and buffers

    // settings cache
    settings_cache: Option<SettingsCache>,

    // context cache
    // context_key: Option<CacheKey>,
    // context_cache: Option<ContextCache>,
    context_cache: DoubleSetCache<CacheKey, ContextCache<()>>,
}

// MARK: settings cache

struct SettingsCache {
    // any cache that
}

// MARK: context cache

/// stores tenfold width and height with integer.
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct CacheKey {
    size: [Option<u32>; 2],
    tag: u64,
}

impl CacheKey {
    fn new(size: [Option<f32>; 2], tag: u64) -> Self {
        Self {
            size: [
                size[0].map(|f| (f * 10.0) as u32),
                size[1].map(|f| (f * 10.0) as u32),
            ],
            tag,
        }
    }
}

struct ContextCache<T> {
    // actual size equals to texture size
    actual_size: [f32; 2],
    content_option_size: [Option<f32>; 2],
    // any cache
    any_cache: Option<T>,
    // rendering result
    render_result_texture: Option<Arc<wgpu::Texture>>,
    uv_vertices: Option<Arc<Vec<UvVertex>>>,
    indices: Option<Arc<Vec<u16>>>,
    transform: Option<Matrix4<f32>>,
}

impl<T> ContextCache<T> {
    fn new(actual_size: [f32; 2], content_option_size: [Option<f32>; 2]) -> Self {
        Self {
            actual_size,
            content_option_size,
            any_cache: None,
            render_result_texture: None,
            uv_vertices: None,
            indices: None,
            transform: None,
        }
    }

    fn get_object_or_create_with<F1, F2, S>(
        &mut self,
        settings_cache: &mut Option<S>,
        f_texture: F1,
        f_transform: F2,
    ) -> Object
    where
        F1: FnOnce([f32; 2], [Option<f32>; 2], &mut Option<S>, &mut Option<T>) -> wgpu::Texture,
        F2: FnOnce([f32; 2], [Option<f32>; 2], &mut Option<S>, &mut Option<T>) -> Matrix4<f32>,
    {
        let texture = self.render_result_texture.get_or_insert_with(|| {
            Arc::new(f_texture(
                self.actual_size,
                self.content_option_size,
                settings_cache,
                &mut self.any_cache,
            ))
        });

        // vertices:
        // 0 - 3
        // | \ |
        // 1 - 2

        let uv_vertices = self.uv_vertices.get_or_insert_with(|| {
            Arc::new(vec![
                UvVertex {
                    position: [0.0, 0.0, 0.0].into(),
                    uv: [0.0, 0.0].into(),
                },
                UvVertex {
                    position: [0.0, -self.actual_size[1], 0.0].into(),
                    uv: [0.0, 1.0].into(),
                },
                UvVertex {
                    position: [self.actual_size[0], -self.actual_size[1], 0.0].into(),
                    uv: [1.0, 1.0].into(),
                },
                UvVertex {
                    position: [self.actual_size[0], 0.0, 0.0].into(),
                    uv: [1.0, 0.0].into(),
                },
            ])
        });

        let indices = self
            .indices
            .get_or_insert_with(|| Arc::new(vec![0, 1, 2, 0, 2, 3]));

        let transform = self.transform.get_or_insert_with(|| {
            f_transform(
                self.actual_size,
                self.content_option_size,
                settings_cache,
                &mut self.any_cache,
            )
        });

        let texture_object = TextureObject {
            texture: Arc::clone(texture),
            uv_vertices: Arc::clone(uv_vertices),
            indices: Arc::clone(indices),
            transform: *transform,
        };

        Object::TextureObject(texture_object)
    }
}

// MARK: Widget trait

impl<T: Send + 'static> Widget<T> for SolidColorNode<T> {
    // label

    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    // dom handling

    fn update_widget_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<SolidColor<T>>() {
            Err(())
        } else {
            let dom = dom.as_any().downcast_ref::<SolidColor<T>>().unwrap();
            let _ = dom;
            todo!()
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> crate::ui::DomComPareResult {
        if let Some(_) = dom.as_any().downcast_ref::<SolidColor<T>>() {
            todo!()
        } else {
            DomComPareResult::Different
        }
    }

    // event handling

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> UiEventResult<T> {
        let _ = (event, parent_size, context);
        // todo
        Default::default()
    }

    // for rendering

    fn px_size(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> [f32; 2] {
        // todo: update this after Widget interface is changed

        let current_key = CacheKey::new(parent_size, tag);

        // get context cache
        let context_cache = self
            .context_cache
            .get_or_insert_with(current_key, frame, || {
                let (actual_size, content_option_size) = inside_outside_size(
                    self.size,
                    self.box_sizing,
                    self.padding,
                    self.border_shape,
                    self.content.as_mut(),
                    parent_size,
                    context,
                    tag,
                    frame,
                );

                ContextCache::new(actual_size, content_option_size)
            });

        context_cache.actual_size
    }

    fn draw_range(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> Option<Range2D<f32>> {
        // todo: optimize this. if this widget is completely transparent, this should return content's range.
        let px_size = self.px_size(parent_size, context, tag, frame);
        Some(Range2D::new([0.0, px_size[0]], [0.0, px_size[1]]).unwrap())
    }

    fn cover_area(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> Option<Range2D<f32>> {
        todo!()
    }

    fn redraw(&self) -> bool {
        let dummy_self_redraw = false;

        dummy_self_redraw || self.content.as_ref().map(|c| c.redraw()).unwrap_or(false)
    }

    // MARK: Widget::render()

    fn render(
        &mut self,
        // ui environment
        parent_size: [Option<f32>; 2],
        background_view: &wgpu::TextureView,
        background_range: Range2D<f32>,
        // context
        context: &SharedContext,
        renderer: &Renderer,
        tag: u64,
        frame: u64,
    ) -> Vec<crate::ui::Object> {
        // this is a temporary implementation
        let current_key = CacheKey::new(parent_size, tag);

        let context_cache = self
            .context_cache
            .get_or_insert_with(current_key, frame, || {
                let (actual_size, content_option_size) = inside_outside_size(
                    self.size,
                    self.box_sizing,
                    self.padding,
                    self.border_shape,
                    self.content.as_mut(),
                    parent_size,
                    context,
                    tag,
                    frame,
                );

                ContextCache::new(actual_size, content_option_size)
            });

        let settings_cache = &mut self.settings_cache;

        let object = context_cache.get_object_or_create_with(
            settings_cache,
            |actual_size, content_option_size, settings_cache, any_cache| {
                texture_render(
                    context,
                    actual_size,
                    content_option_size,
                    settings_cache,
                    any_cache,
                    &self.padding,
                    &self.visibility,
                    &self.border_shape,
                    &self.border_color,
                    &self.background_color,
                    self.content.as_mut(),
                )
            },
            |actual_size, content_option_size, settings_cache, any_cache| {
                texture_transform(
                    actual_size,
                    content_option_size,
                    settings_cache,
                    any_cache,
                    self.box_sizing,
                )
            },
        );

        vec![object]
    }
}

// MARK: render functions

fn texture_render<T>(
    context: &SharedContext,
    actual_size: [f32; 2],
    content_option_size: [Option<f32>; 2],
    settings_cache: &mut Option<SettingsCache>,
    any_cache: &mut Option<()>,
    padding: &Padding,
    visibility: &Visibility,
    border_shape: &Border,
    border_color: &Color,
    background_color: &Color,
    content: Option<&mut Box<dyn Widget<T>>>,
) -> wgpu::Texture {
    todo!()
}

fn texture_transform(
    actual_size: [f32; 2],
    content_option_size: [Option<f32>; 2],
    settings_cache: &mut Option<SettingsCache>,
    any_cache: &mut Option<()>,
    box_sizing: BoxSizing,
) -> Matrix4<f32> {
    todo!()
}

// MARK: i/o size

// calculate size from properties
fn inside_outside_size<T>(
    // selfs
    size: [Size; 2],
    box_sizing: BoxSizing,
    padding: Padding,
    border_shape: Border,
    mut content: Option<&mut Box<dyn Widget<T>>>,
    // args
    parent_size: [Option<f32>; 2],
    context: &SharedContext,
    tag: u64,
    frame: u64,
) -> ([f32; 2], [Option<f32>; 2]) {
    let current_std_size = to_std_size(size, parent_size, context);

    let (px_size, op_size) = match box_sizing {
        BoxSizing::ContentBox => calculate_content_box(
            current_std_size,
            padding,
            border_shape,
            &mut content,
            context,
            tag,
            frame,
        ),
        BoxSizing::BorderBox => calculate_border_box(
            current_std_size,
            padding,
            border_shape,
            &mut content,
            context,
            tag,
            frame,
        ),
    };

    (px_size, to_op_size(op_size))
}

fn to_std_size(
    size: [Size; 2],
    parent_size: [Option<f32>; 2],
    context: &SharedContext,
) -> [StdSize; 2] {
    [
        size[0].to_std_size(parent_size[0], context),
        size[1].to_std_size(parent_size[1], context),
    ]
}

fn to_op_size(size: [StdSize; 2]) -> [Option<f32>; 2] {
    [
        std_size_to_option_f32(size[0]),
        std_size_to_option_f32(size[1]),
    ]
}

fn calculate_content_box<T>(
    current_std_size: [StdSize; 2],
    padding: Padding,
    border_shape: Border,
    content: &mut Option<&mut Box<dyn Widget<T>>>,
    context: &SharedContext,
    tag: u64,
    frame: u64,
) -> ([f32; 2], [StdSize; 2]) {
    match current_std_size {
        [StdSize::Pixel(w), StdSize::Pixel(h)] => (
            apply_padding_and_border_to_outer([w, h], padding, border_shape.px),
            current_std_size,
        ),
        _ => {
            // 子要素からサイズを取得
            let content_size = get_content_size(content, current_std_size, context, tag, frame);

            // 各次元のサイズを計算
            let width = calculate_dimension_size(current_std_size[0], content_size, 0);
            let height = calculate_dimension_size(current_std_size[1], content_size, 1);

            (
                apply_padding_and_border_to_outer([width, height], padding, border_shape.px),
                current_std_size,
            )
        }
    }
}

fn calculate_border_box<T>(
    current_std_size: [StdSize; 2],
    padding: Padding,
    border_shape: Border,
    content: &mut Option<&mut Box<dyn Widget<T>>>,
    context: &SharedContext,
    tag: u64,
    frame: u64,
) -> ([f32; 2], [StdSize; 2]) {
    match current_std_size {
        [StdSize::Pixel(w), StdSize::Pixel(h)] => (
            [w, h],
            [
                StdSize::Pixel((w - padding.left - padding.right - border_shape.px * 2.0).max(0.0)),
                StdSize::Pixel((h - padding.top - padding.bottom - border_shape.px * 2.0).max(0.0)),
            ],
        ),
        _ => {
            // 子要素からサイズを取得
            let content_size = get_content_size(content, current_std_size, context, tag, frame);

            // 各次元のサイズを計算
            let width = calculate_dimension_size(current_std_size[0], content_size, 0);
            let height = calculate_dimension_size(current_std_size[1], content_size, 1);

            let outer_size = match current_std_size {
                [StdSize::Pixel(w), _] => [w, height],
                [_, StdSize::Pixel(h)] => [width, h],
                _ => apply_padding_and_border_to_outer([width, height], padding, border_shape.px),
            };

            (
                outer_size,
                adjust_inner_size(current_std_size, padding, border_shape.px),
            )
        }
    }
}

fn get_content_size<T>(
    content: &mut Option<&mut Box<dyn Widget<T>>>,
    current_std_size: [StdSize; 2],
    context: &SharedContext,
    tag: u64,
    frame: u64,
) -> Option<[f32; 2]> {
    let current_op_size = current_std_size.map(std_size_to_option_f32);

    content
        .as_mut()
        .map(|content| content.px_size(current_op_size, context, tag, frame))
}

fn calculate_dimension_size(
    std_size: StdSize,
    content_size: Option<[f32; 2]>,
    index: usize,
) -> f32 {
    match std_size {
        StdSize::Pixel(px) => px,
        StdSize::Content(_) => content_size.map_or(0.0, |s| s[index]),
    }
}

fn apply_padding_and_border_to_outer(size: [f32; 2], padding: Padding, border_px: f32) -> [f32; 2] {
    [
        size[0] + padding.left + padding.right + border_px * 2.0,
        size[1] + padding.top + padding.bottom + border_px * 2.0,
    ]
}

fn adjust_inner_size(std_size: [StdSize; 2], padding: Padding, border_px: f32) -> [StdSize; 2] {
    [
        match std_size[0] {
            StdSize::Pixel(px) => {
                StdSize::Pixel((px - padding.left - padding.right - border_px * 2.0).max(0.0))
            }
            StdSize::Content(c) => StdSize::Content(c),
        },
        match std_size[1] {
            StdSize::Pixel(px) => {
                StdSize::Pixel((px - padding.top - padding.bottom - border_px * 2.0).max(0.0))
            }
            StdSize::Content(c) => StdSize::Content(c),
        },
    ]
}

// MARK: temporary functions

fn std_size_to_option_f32(size: StdSize) -> Option<f32> {
    match size {
        StdSize::Pixel(px) => Some(px),
        StdSize::Content(_) => None,
    }
}

fn option_f32_to_std_size(size: Option<f32>) -> StdSize {
    match size {
        Some(px) => StdSize::Pixel(px),
        None => StdSize::Content(0.0),
    }
}
