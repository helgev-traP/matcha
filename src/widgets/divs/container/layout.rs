pub mod display;
use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

pub use display::*;
pub mod position;
pub use position::*;
pub mod overflow;
pub use overflow::*;

use crate::{
    context::SharedContext,
    events::{UiEvent, UiEventResult},
    renderer::Renderer,
    types::size::{Size, StdSize},
    ui::{Dom, Widget},
    vertex::uv_vertex::UvVertex,
};

// todo: note that Layout::Grid will unwrap the StdSize:Content to 0.0.
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
        template_columns: Vec<Size>,
        template_rows: Vec<Size>,
        /// `Size::Content` will be unwrapped to `0.0`
        gap_columns: Size,
        /// `Size::Content` will be unwrapped to `0.0`
        gap_rows: Size,
    },
}

pub struct FlexItem<T> {
    pub item: Box<dyn Dom<T>>,
    pub grow: f32,
}

pub struct GridItem<T> {
    pub item: Box<dyn Dom<T>>,
    pub column_start: u32,
    pub column_end: u32,
    pub row_start: u32,
    pub row_end: u32,
}

impl<T> Default for Layout<T> {
    fn default() -> Self {
        Self::Flex {
            item: Vec::new(),
            direction: FlexDirection::Row,
            wrap: FlexWrap::NoWrap,
            justify_content: JustifyContent::FlexStart,
            align_content: AlignContent::Start,
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
        template_columns: Vec<Size>,
        template_rows: Vec<Size>,
        gap_columns: Size,
        gap_rows: Size,
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

    pub(super) fn build(&self) -> LayoutNode<T> {
        match self {
            Self::None { item } => LayoutNode::None {
                item: item
                    .into_iter()
                    .map(|item| item.build_widget_tree())
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
                        item: item.item.build_widget_tree(),
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
                        item: item.item.build_widget_tree(),
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
        size: RefCell<Option<[f32; 2]>>,
        item_cache_valid: bool,
    },
    Grid {
        item: Vec<GridItemNode<T>>,
        template_columns: Vec<Size>,
        template_rows: Vec<Size>,
        gap_columns: Size,
        gap_rows: Size,

        // cache
        size: RefCell<Option<[f32; 2]>>,
        item_cache_valid: bool,
    },
}

pub(super) struct FlexItemNode<T> {
    item: Box<dyn Widget<T>>,
    grow: f32,

    // cache
    size: Mutex<[f32; 2]>,
    position: Mutex<[f32; 2]>,
}

pub(super) struct GridItemNode<T> {
    item: Box<dyn Widget<T>>,
    column_start: u32,
    column_end: u32,
    row_start: u32,
    row_end: u32,

    // cache
    size: Mutex<[f32; 2]>,
    position: Mutex<[f32; 2]>,
}

struct FlexLine<'a, T> {
    items: Vec<&'a FlexItemNode<T>>,
    size: [f32; 2],
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
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> UiEventResult<T> {
        // todo: event handling
        UiEventResult::default()
    }

    pub fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> bool {
        todo!()
    }

    pub fn content_size(&self) {
        todo!()
    }

    pub fn px_size(&self, parent_size: [StdSize; 2], context: &SharedContext) -> [f32; 2] {
        match self {
            LayoutNode::None { .. } => [0.0, 0.0],
            LayoutNode::Flex {
                item,
                direction,
                wrap,
                justify_content,
                align_content,
                size,
                item_cache_valid,
            } => match direction {
                FlexDirection::Row => match parent_size[0] {
                    StdSize::Content(x) => {
                        // todo: use x to calculate the width of the flex container
                        // no width limit
                        // item will be placed in a single line and no space between them

                        let mut max_height: f32 = 0.0;
                        let mut total_width: f32 = 0.0;

                        let mut tmp_height_size = Vec::with_capacity(item.len());

                        // calculate total size
                        for item in item.iter() {
                            let size = item.item.px_size(parent_size, context);

                            total_width += size[0];
                            max_height = max_height.max(size[1]);

                            // cache the horizontal position
                            let mut cache_position = item.position.lock().unwrap();
                            cache_position[0] = total_width;
                            // vertical position need to be calculated after max_height is determined
                            tmp_height_size.push(size[1]);

                            // cache the size
                            let mut cache_size = item.size.lock().unwrap();
                            cache_size[0] = size[0];
                            cache_size[1] = size[1];
                        }

                        max_height = match parent_size[1] {
                            StdSize::Pixel(y_px) => y_px,
                            StdSize::Content(y) => max_height * y, // todo consider this.
                        };

                        // calculate vertical position and cache it
                        for (i, item) in item.iter().enumerate() {
                            let mut cache_position = item.position.lock().unwrap();
                            cache_position[1] = match align_content {
                                AlignContent::Start => 0.0,
                                AlignContent::End => max_height - tmp_height_size[i],
                                AlignContent::Center => (max_height - tmp_height_size[i]) / 2.0,
                            };
                        }

                        [total_width, max_height]
                    }
                    StdSize::Pixel(width_px) => {
                        match wrap {
                            FlexWrap::NoWrap => {
                                // sum up the width of all items
                                let mut total_width: f32 = 0.0;
                                let mut max_height: f32 = 0.0;

                                // calculate total width
                                for item in item.iter() {
                                    let size = item.item.px_size(parent_size, context);

                                    total_width += size[0];
                                    max_height = max_height.max(size[1]);

                                    // cache the size
                                    let mut cache_size = item.size.lock().unwrap();
                                    cache_size[0] = size[0];
                                    cache_size[1] = size[1];
                                }

                                // calculate the spaces between items and horizontal position
                                match justify_content {
                                    JustifyContent::FlexStart => todo!(),
                                    JustifyContent::FlexEnd => todo!(),
                                    JustifyContent::Center => todo!(),
                                    JustifyContent::SpaceBetween => todo!(),
                                    JustifyContent::SpaceAround => todo!(),
                                    JustifyContent::SpaceEvenly => todo!(),
                                }

                                todo!()
                            }
                            FlexWrap::Wrap => todo!(),
                            FlexWrap::WrapReverse => todo!(),
                        }
                    }
                },
                FlexDirection::RowReverse => todo!(),
                FlexDirection::Column => todo!(),
                FlexDirection::ColumnReverse => todo!(),
            },
            LayoutNode::Grid {
                item,
                template_columns,
                template_rows,
                gap_columns,
                gap_rows,
                size,
                item_cache_valid,
            } => todo!(),
        }
    }

    pub fn render(
        &mut self,
        // ui environment
        parent_size: [StdSize; 2],
        // context
        context: &SharedContext,
        renderer: &Renderer,
        frame: u64,
    ) -> Vec<(
        Arc<wgpu::Texture>,
        Arc<Vec<UvVertex>>,
        Arc<Vec<u16>>,
        nalgebra::Matrix4<f32>,
    )> {
        match self {
            LayoutNode::None { item } => {
                // display none
                vec![]
            }
            LayoutNode::Flex {
                item,
                direction,
                wrap,
                justify_content,
                align_content,
                // cache
                size,
                item_cache_valid,
            } => {
                todo!()
            }
            LayoutNode::Grid {
                item,
                template_columns,
                template_rows,
                gap_columns,
                gap_rows,
                // cache
                size,
                item_cache_valid,
            } => {
                // calculate the size of the grid
                let horizontal_grid = template_columns
                    .iter()
                    .map(|size| size.to_std_size(parent_size[0], context))
                    .map(|size| size.unwrap_or(0.0))
                    .sum::<f32>();

                let vertical_grid = template_rows
                    .iter()
                    .map(|size| size.to_std_size(parent_size[1], context))
                    .map(|size| size.unwrap_or(0.0))
                    .sum::<f32>();

                let gap_columns = gap_columns
                    .to_std_size(parent_size[0], context)
                    .unwrap_or(0.0);
                let gap_rows = gap_rows.to_std_size(parent_size[1], context).unwrap_or(0.0);

                

                todo!()
            }
        }
    }
}
