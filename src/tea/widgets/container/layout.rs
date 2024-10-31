pub mod display;
use std::cell::{Ref, RefCell};

pub use display::*;
pub mod position;
pub use position::*;
pub mod overflow;
pub use overflow::*;

use crate::{
    application_context::ApplicationContext,
    events::{UiEvent, UiEventResult},
    renderer::RendererCommandEncoder,
    types::size::{PxSize, SizeUnit},
    ui::{Dom, Widget},
};
use nalgebra as na;

pub enum Layout<T: 'static> {
    None {
        item: Vec<Box<dyn Dom<T>>>,
    },
    Flex {
        item: Vec<FlexItem<T>>,
        direction: FlexDirection,
        wrap: FlexWrap,
        justify_content: JustifyContent,
        align_content: AlignContent,
    },
    Grid {
        item: Vec<GridItem<T>>,
        template_columns: Vec<SizeUnit>,
        template_rows: Vec<SizeUnit>,
        gap_columns: SizeUnit,
        gap_rows: SizeUnit,
    },
}

impl<T> Default for Layout<T> {
    fn default() -> Self {
        Self::Flex {
            item: Vec::new(),
            direction: FlexDirection::Row,
            wrap: FlexWrap::NoWrap,
            justify_content: JustifyContent::FlexStart,
            align_content: AlignContent::FlexStart,
        }
    }
}

impl<T> Layout<T> {
    pub fn none(item: Vec<Box<dyn Dom<T>>>) -> Self {
        Self::None { item }
    }

    pub fn flex(
        item: Vec<FlexItem<T>>,
        direction: FlexDirection,
        wrap: FlexWrap,
        justify_content: JustifyContent,
        align_content: AlignContent,
    ) -> Self {
        Self::Flex {
            item,
            direction,
            wrap,
            justify_content,
            align_content,
        }
    }

    pub fn grid(
        item: Vec<GridItem<T>>,
        template_columns: Vec<SizeUnit>,
        template_rows: Vec<SizeUnit>,
        gap_columns: SizeUnit,
        gap_rows: SizeUnit,
    ) -> Self {
        Self::Grid {
            item,
            template_columns,
            template_rows,
            gap_columns,
            gap_rows,
        }
    }

    pub fn to_none(self) -> Self {
        Self::None {
            item: match self {
                Self::None { item } => item,
                Self::Flex { item, .. } => item.into_iter().map(|item| item.item).collect(),
                Self::Grid { item, .. } => item.into_iter().map(|item| item.item).collect(),
            },
        }
    }

    pub fn build(&self) -> LayoutNode<T> {
        match self {
            Self::None { item } => LayoutNode::None {
                item: item
                    .into_iter()
                    .map(|item| item.build_render_tree())
                    .collect(),
            },
            Self::Flex {
                item,
                direction,
                wrap,
                justify_content,
                align_content,
            } => LayoutNode::Flex {
                item: item
                    .into_iter()
                    .map(|item| FlexItemNode {
                        item: item.item.build_render_tree(),
                        grow: item.grow.clone(),
                        size: Default::default(),
                        position: Default::default(),
                    })
                    .collect(),
                direction: direction.clone(),
                wrap: wrap.clone(),
                justify_content: justify_content.clone(),
                align_content: align_content.clone(),
                size: RefCell::new(None),
                item_cache_valid: false,
            },
            Self::Grid {
                item,
                template_columns,
                template_rows,
                gap_columns,
                gap_rows,
            } => LayoutNode::Grid {
                item: item
                    .into_iter()
                    .map(|item| GridItemNode {
                        item: item.item.build_render_tree(),
                        column_start: item.column_start,
                        column_end: item.column_end,
                        row_start: item.row_start,
                        row_end: item.row_end,
                        size: Default::default(),
                        position: Default::default(),
                    })
                    .collect(),
                template_columns: template_columns.clone(),
                template_rows: template_rows.clone(),
                gap_columns: gap_columns.clone(),
                gap_rows: gap_rows.clone(),
                size: RefCell::new(None),
                item_cache_valid: false,
            },
        }
    }
}

pub struct FlexItem<T> {
    pub item: Box<dyn Dom<T>>,
    pub grow: FlexGrow,
}

pub struct GridItem<T> {
    pub item: Box<dyn Dom<T>>,
    pub column_start: u32,
    pub column_end: u32,
    pub row_start: u32,
    pub row_end: u32,
}

// render node

pub(super) enum LayoutNode<T> {
    None {
        item: Vec<Box<dyn Widget<T>>>,
    },
    Flex {
        item: Vec<FlexItemNode<T>>,
        direction: FlexDirection,
        wrap: FlexWrap,
        justify_content: JustifyContent,
        align_content: AlignContent,

        // cache
        size: RefCell<Option<PxSize>>,
        item_cache_valid: bool,
    },
    Grid {
        item: Vec<GridItemNode<T>>,
        template_columns: Vec<SizeUnit>,
        template_rows: Vec<SizeUnit>,
        gap_columns: SizeUnit,
        gap_rows: SizeUnit,

        // cache
        size: RefCell<Option<PxSize>>,
        item_cache_valid: bool,
    },
}

impl<T> LayoutNode<T> {
    pub fn update_render_tree(&mut self, dom: Layout<T>) -> Result<(), ()> {
        // ! todo !
        *self = dom.build();
        Ok(())
    }

    pub fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> UiEventResult<T> {
        // todo: event handling
        UiEventResult::default()
    }

    pub fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> bool {
        todo!()
    }

    pub fn content_size(&self) {
        todo!()
    }

    pub fn px_size(&self, parent_size: PxSize, context: &ApplicationContext) -> PxSize {
        todo!()
    }

    pub fn default_size(&self) -> PxSize {
        todo!()
    }

    pub fn render(
        &mut self,
        s: &rayon::Scope,
        parent_size: PxSize,
        affine: na::Matrix4<f32>,
        encoder: &mut RendererCommandEncoder,
    ) {
        match &self {
            LayoutNode::Flex { size, .. } | LayoutNode::Grid { size, .. } => {
                if size.borrow().is_none() {
                    self.px_size(parent_size, encoder.get_context());
                }
            }
            _ => (),
        }

        match self {
            LayoutNode::None { .. } => (),
            LayoutNode::Flex {
                item,
                direction,
                wrap,
                justify_content,
                align_content,
                size,
                item_cache_valid
            } => {
                if *item_cache_valid {
                    // render as cache
                    for item in item {
                    }
                }
            }
            LayoutNode::Grid {
                item,
                template_columns,
                template_rows,
                gap_columns,
                gap_rows,
                size,
                item_cache_valid
            } => todo!(),
        }
    }
}

pub(super) struct FlexItemNode<T> {
    item: Box<dyn Widget<T>>,
    grow: FlexGrow,

    // cache
    size: PxSize,
    position: [f32; 2],
}

pub(super) struct GridItemNode<T> {
    item: Box<dyn Widget<T>>,
    column_start: u32,
    column_end: u32,
    row_start: u32,
    row_end: u32,

    // cache
    size: PxSize,
    position: [f32; 2],
}

struct FlexLine<'a, T> {
    items: Vec<&'a FlexItemNode<T>>,
    size: PxSize,
}
