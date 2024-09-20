#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    // Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    // Stretch,
}

#[derive(Debug, Clone)]
pub enum Display {
    None,
    Flex {
        direction: FlexDirection,
        // wrap: FlexWrap,
        // justify_content: JustifyContent,
        // align_items: AlignItems,
        // align_content: AlignContent,
    },
    Grid {
        template_columns: Vec<crate::types::SizeUnit>,
        template_rows: Vec<crate::types::SizeUnit>,
        gap: crate::types::SizeUnit,
    },
}

impl Default for Display {
    fn default() -> Self {
        Self::Flex {
            direction: FlexDirection::Column,
            // wrap: FlexWrap::NoWrap,
            // justify_content: JustifyContent::FlexStart,
            // align_items: AlignItems::FlexStart,
            // align_content: AlignContent::FlexStart,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GritItem {
    pub row: [usize; 2],
    pub column: [usize; 2],
}