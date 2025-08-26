use std::sync::Arc;

use matcha_core::ui::WidgetContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

/// control the alignment of children on the **main axis**
#[derive(Clone)]
pub enum JustifyContent {
    FlexStart { gap: Arc<GapFn> },
    FlexEnd { gap: Arc<GapFn> },
    Center { gap: Arc<GapFn> },
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

type GapFn = dyn Fn(Option<f32>, &WidgetContext) -> f32 + Sync + Send;

/// control the alignment of children on the **cross axis**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
    Start,
    End,
    Center,
}
