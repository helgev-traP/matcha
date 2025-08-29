use std::sync::Arc;

use matcha_core::{Constraints, ui::WidgetContext};
use nalgebra::constraint;

pub struct ChildSize<'a> {
    get_size: Box<dyn FnMut() -> [f32; 2] + 'a>,
    cached_child_size: std::cell::Cell<Option<[f32; 2]>>,
}

impl Default for ChildSize<'_> {
    fn default() -> Self {
        Self {
            get_size: Box::new(|| [0.0, 0.0]),
            cached_child_size: std::cell::Cell::new(Some([0.0, 0.0])),
        }
    }
}

impl<'a> ChildSize<'a> {
    pub fn new<F>(get_size: F) -> Self
    where
        F: FnMut() -> [f32; 2] + 'a,
    {
        Self {
            get_size: Box::new(get_size),
            cached_child_size: std::cell::Cell::new(None),
        }
    }

    pub fn with_size(size: [f32; 2]) -> Self {
        Self {
            get_size: Box::new(move || size),
            cached_child_size: std::cell::Cell::new(Some(size)),
        }
    }
}

impl ChildSize<'_> {
    pub fn get(&mut self) -> [f32; 2] {
        if let Some(size) = self.cached_child_size.get() {
            size
        } else {
            let size = (self.get_size)();
            self.cached_child_size.set(Some(size));
            size
        }
    }
}

type SizeFn =
    dyn Fn([Option<f32>; 2], &mut ChildSize, &WidgetContext) -> f32 + Send + Sync + 'static;

/// Calculate size from parent size child size and context.
/// `Size::Grow` will be treated as same size as parent size in widget that have only one child.
#[derive(Clone)]
pub enum Size {
    Size(Arc<SizeFn>),
    Grow(Arc<SizeFn>),
}

impl Size {
    /// Specify size in pixels.
    pub fn px(px: f32) -> Self {
        Size::Size(Arc::new(move |_, _, _| px))
    }

    /// Specify size in inches.
    pub fn inch(inch: f32) -> Self {
        Size::Size(Arc::new(move |_, _, ctx| inch * ctx.dpi() as f32))
    }

    /// Specify size in points.
    pub fn point(point: f32) -> Self {
        Size::Size(Arc::new(move |_, _, ctx| point * ctx.dpi() as f32 / 72.0))
    }

    /// Specify size in magnification of parent width.
    pub fn parent_w(mag: f32) -> Self {
        Size::Size(Arc::new(move |parent_size, child_size, _| {
            parent_size[0]
                .map(|size| size * mag)
                .unwrap_or_else(|| child_size.get()[0])
        }))
    }

    pub fn parent_h(mag: f32) -> Self {
        Size::Size(Arc::new(move |parent_size, child_size, _| {
            parent_size[1]
                .map(|size| size * mag)
                .unwrap_or_else(|| child_size.get()[1])
        }))
    }

    pub fn child_w(mag: f32) -> Self {
        Size::Size(Arc::new(move |_, child_size, _| child_size.get()[0] * mag))
    }

    pub fn child_h(mag: f32) -> Self {
        Size::Size(Arc::new(move |_, child_size, _| child_size.get()[1] * mag))
    }

    /// Specify size in magnification of font size.
    pub fn em(em: f32) -> Self {
        Size::Size(Arc::new(move |_, _, ctx| em * ctx.font_size()))
    }

    /// Specify size in magnification of root font size.
    pub fn rem(rem: f32) -> Self {
        Size::Size(Arc::new(move |_, _, ctx| rem * ctx.root_font_size()))
    }

    /// Specify size in magnification of viewport width.
    pub fn vw(vw: f32) -> Self {
        Size::Size(Arc::new(move |_, _, ctx| vw * ctx.viewport_size()[0]))
    }

    /// Specify size in magnification of viewport height.
    pub fn vh(vh: f32) -> Self {
        Size::Size(Arc::new(move |_, _, ctx| vh * ctx.viewport_size()[1]))
    }

    /// Specify size in magnification of vmax.
    pub fn vmax(vmax: f32) -> Self {
        Size::Size(Arc::new(move |_, _, ctx| {
            let viewport_size = ctx.viewport_size();
            vmax * viewport_size[0].max(viewport_size[1])
        }))
    }

    /// Specify size in magnification of vmin.
    pub fn vmin(vmin: f32) -> Self {
        Size::Size(Arc::new(move |_, _, ctx| {
            let viewport_size = ctx.viewport_size();
            vmin * viewport_size[0].min(viewport_size[1])
        }))
    }
}

impl Size {
    /// Specify size that grows to fill the available space.
    /// This is used in widgets that have multiple children.
    /// note that this will be treated as same size as parent size in widget that have only one child.
    pub fn grow(grow: f32) -> Self {
        Size::Grow(Arc::new(move |_, _, _| grow))
    }
}

impl Size {
    /// Specify size with a custom function.
    pub fn from_size<F>(f: F) -> Self
    where
        F: Fn([Option<f32>; 2], &mut ChildSize, &WidgetContext) -> f32 + Send + Sync + 'static,
    {
        Size::Size(Arc::new(f))
    }

    /// Specify size that grows with a custom function.
    pub fn from_grow<F>(f: F) -> Self
    where
        F: Fn([Option<f32>; 2], &mut ChildSize, &WidgetContext) -> f32 + Send + Sync + 'static,
    {
        Size::Grow(Arc::new(f))
    }
}

impl Size {
    pub fn size(
        &self,
        parent_size: [Option<f32>; 2],
        child_size: &mut ChildSize,
        ctx: &WidgetContext,
    ) -> f32 {
        match self {
            Size::Size(f) | Size::Grow(f) => f(parent_size, child_size, ctx),
        }
    }

    /// returns `[{min_size}, {max_size}]`
    pub fn constraints(&self, constraints: &Constraints, ctx: &WidgetContext) -> [f32; 2] {
        todo!()
    }
}

impl PartialEq for Size {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Size::Size(a), Size::Size(b)) => Arc::ptr_eq(a, b),
            (Size::Grow(a), Size::Grow(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}
