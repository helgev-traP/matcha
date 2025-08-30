use crate::types::size::Size;

use matcha_core::ui::WidgetContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

/// control the alignment of children on the **main axis**
#[derive(Clone, PartialEq)]
pub enum JustifyContent {
    FlexStart { gap: Size },
    FlexEnd { gap: Size },
    Center { gap: Size },
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// control the alignment of children on the **cross axis**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
    Start,
    End,
    Center,
}
