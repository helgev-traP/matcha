use crate::widgets::div_size::DivSize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

/// control the alignment of children on the **main axis**
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JustifyContent {
    FlexStart { gap: DivSize },
    FlexEnd { gap: DivSize },
    Center { gap: DivSize },
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// control the alignment of children on the **cross axis**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignContent {
    Start,
    End,
    Center,
}
