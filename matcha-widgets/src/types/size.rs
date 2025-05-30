use std::sync::Arc;

use matcha_core::context::WidgetContext;

type SizeFn = dyn for<'a> Fn(Option<f32>, &'a WidgetContext) -> f32 + Send + Sync + 'static;

pub enum Size {
    /// returned value will interpret as pixel size
    Size(Arc<SizeFn>),
    /// returned value will interpret as grow value
    Grow(Arc<SizeFn>),
}

impl Size {
    pub fn px(px: f32) -> Self {
        Size::Size(Arc::new(move |_, _| px))
    }

    pub fn inch(inch: f32) -> Self {
        Size::Size(Arc::new(move |_, ctx| inch * ctx.dpi() as f32))
    }

    pub fn point(point: f32) -> Self {
        Size::Size(Arc::new(move |_, ctx| point * ctx.dpi() as f32 / 72.0))
    }

    pub fn parent(mag: f32) -> Self {
        Size::Size(Arc::new(move |parent_size, _| {
            parent_size.unwrap_or(0.0) * mag
        }))
    }

    pub fn em(em: f32) -> Self {
        Size::Size(Arc::new(move |_, ctx| em * ctx.font_size()))
    }

    pub fn rem(rem: f32) -> Self {
        Size::Size(Arc::new(move |_, ctx| rem * ctx.root_font_size()))
    }

    pub fn vw(vw: f32) -> Self {
        Size::Size(Arc::new(move |_, ctx| vw * ctx.viewport_size()[0] as f32))
    }

    pub fn vh(vh: f32) -> Self {
        Size::Size(Arc::new(move |_, ctx| vh * ctx.viewport_size()[1] as f32))
    }

    pub fn vmax(vmax: f32) -> Self {
        Size::Size(Arc::new(move |_, ctx| {
            let viewport_size = ctx.viewport_size();
            vmax * viewport_size[0].max(viewport_size[1]) as f32
        }))
    }

    pub fn vmin(vmin: f32) -> Self {
        Size::Size(Arc::new(move |_, ctx| {
            let viewport_size = ctx.viewport_size();
            vmin * viewport_size[0].min(viewport_size[1]) as f32
        }))
    }

    pub fn size_fn<F>(f: F) -> Self
    where
        F: Fn(Option<f32>, &WidgetContext) -> f32 + Send + Sync + 'static,
    {
        Size::Size(Arc::new(f))
    }
}

impl Size {
    pub fn grow(grow: f32) -> Self {
        Size::Grow(Arc::new(move |_, _| grow))
    }

    pub fn grow_fn<F>(f: F) -> Self
    where
        F: Fn(Option<f32>, &WidgetContext) -> f32 + Send + Sync + 'static,
    {
        Size::Grow(Arc::new(f))
    }
}
