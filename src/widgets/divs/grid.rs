use std::sync::{Arc, Mutex};

use wgpu::naga::back;

use crate::{
    context::SharedContext,
    events::UiEvent,
    renderer::Renderer,
    types::{
        range::Range2D,
        size::{Size, StdSize},
    },
    ui::{Dom, DomComPareResult, Widget},
    widgets::div_size::{DivSize, StdDivSize},
};

pub struct Grid<T: Send + 'static> {
    // label
    label: Option<String>,

    // layout
    template_columns: Vec<DivSize>,
    template_rows: Vec<DivSize>,
    gap_columns: DivSize,
    gap_rows: DivSize,

    // items
    items: Vec<GridItem<T>>,
}

pub struct GridItem<T: Send + 'static> {
    pub item: Box<dyn Dom<T>>,
    pub column: [usize; 2],
    pub row: [usize; 2],
}

impl<T: Send + 'static> Grid<T> {
    // todo: add build chain
}

impl<T: Send + 'static> Dom<T> for Grid<T> {
    fn build_widget_tree(&self) -> (Box<dyn Widget<T>>, bool) {
        let mut has_dynamic = false;

        let items = self
            .items
            .iter()
            .map(|item| GridItemNode {
                item: {
                    let (tree, d) = item.item.build_widget_tree();
                    has_dynamic |= d;
                    tree
                },
                column: item.column,
                row: item.row,
            })
            .collect();

        let widget_tree = Box::new(GridNode {
            label: self.label.clone(),
            template_columns: self.template_columns.clone(),
            template_rows: self.template_rows.clone(),
            gap_columns: self.gap_columns,
            gap_rows: self.gap_rows,
            items,
            has_dynamic,
            redraw: true,
            grid_cache: Mutex::new(None),
        });

        (widget_tree, has_dynamic)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct GridNode<T: Send + 'static> {
    label: Option<String>,

    template_columns: Vec<DivSize>,
    template_rows: Vec<DivSize>,
    gap_columns: DivSize,
    gap_rows: DivSize,

    items: Vec<GridItemNode<T>>,
    has_dynamic: bool,

    redraw: bool,

    // cache
    grid_cache: Mutex<Option<GridCache>>,
}

// todo: document the cache usage
struct GridCache {
    tag: [StdSize; 2],
    column_accumulate_template: Vec<f32>,
    column_gap: f32,
    row_accumulate_template: Vec<f32>,
    row_gap: f32,

    px_size: [f32; 2],
}

pub struct GridItemNode<T: Send + 'static> {
    column: [usize; 2],
    row: [usize; 2],
    item: Box<dyn Widget<T>>,
}

impl<T: Send + 'static> GridNode<T> {
    fn cache_grid(&self, parent_size: [StdSize; 2], context: &SharedContext) -> GridCache {
        // TODO: check efficiency and refactor
        // TODO: divide into smaller functions

        // calculate template size

        let (total_horizontal_size, total_horizontal_grow) =
            self.template_columns
                .iter()
                .fold((0.0, 0.0), |(acc_size, acc_grow), size| {
                    match size.to_std_size(parent_size[0], context) {
                        crate::widgets::div_size::StdDivSize::Pixel(px) => {
                            (acc_size + px, acc_grow)
                        }
                        crate::widgets::div_size::StdDivSize::Grow(grow) => {
                            (acc_size, acc_grow + grow)
                        }
                    }
                });

        let horizontal_gap = self.gap_columns.to_std_size(parent_size[0], context);

        let (column_accumulate_template, column_gap) = match parent_size[0] {
            StdSize::Pixel(parent_px) => {
                if parent_px < total_horizontal_size {
                    // overflowing
                    // gap = 0.0
                    // grows = 0.0
                    (
                        self.template_columns
                            .iter()
                            .scan(0.0, |acc, size| {
                                let increment = match size.to_std_size(parent_size[0], context) {
                                    crate::widgets::div_size::StdDivSize::Pixel(px) => px,
                                    crate::widgets::div_size::StdDivSize::Grow(grow) => grow,
                                };
                                let accumulate_template = *acc + increment;
                                *acc = accumulate_template;
                                Some(accumulate_template)
                            })
                            .collect::<Vec<_>>(),
                        0.0,
                    )
                } else {
                    // not overflowing
                    match horizontal_gap {
                        StdDivSize::Pixel(gap_px) => {
                            let true_gap = gap_px.min(
                                (parent_px - total_horizontal_size)
                                    / (self.template_columns.len() as f32 - 1.0),
                            );

                            let remain_px = parent_px
                                - total_horizontal_size
                                - true_gap * (self.template_columns.len() as f32 - 1.0);

                            let unit_grow = remain_px / total_horizontal_grow;

                            (
                                self.template_columns
                                    .iter()
                                    .scan(0.0, |acc, size| {
                                        let increment = match size
                                            .to_std_size(parent_size[0], context)
                                        {
                                            crate::widgets::div_size::StdDivSize::Pixel(px) => px,
                                            crate::widgets::div_size::StdDivSize::Grow(grow) => {
                                                grow * unit_grow
                                            }
                                        };
                                        let accumulate_template = *acc + increment;
                                        *acc = accumulate_template + true_gap;
                                        Some(accumulate_template)
                                    })
                                    .collect::<Vec<_>>(),
                                true_gap,
                            )
                        }
                        StdDivSize::Grow(gap_grow) => {
                            let total_gap_grow =
                                gap_grow * (self.template_columns.len() as f32 - 1.0);

                            let remain_px = parent_px - total_horizontal_size;

                            let unit_grow = remain_px / (total_horizontal_grow + total_gap_grow);

                            (
                                self.template_columns
                                    .iter()
                                    .scan(0.0, |acc, size| {
                                        let increment = match size
                                            .to_std_size(parent_size[0], context)
                                        {
                                            crate::widgets::div_size::StdDivSize::Pixel(px) => px,
                                            crate::widgets::div_size::StdDivSize::Grow(grow) => {
                                                grow * unit_grow
                                            }
                                        };
                                        let accumulate_template = *acc + increment;
                                        *acc = accumulate_template + gap_grow * unit_grow;
                                        Some(accumulate_template)
                                    })
                                    .collect::<Vec<_>>(),
                                gap_grow * unit_grow,
                            )
                        }
                    }
                }
            }
            StdSize::Content(_) => {
                // grow = 0.0
                (
                    self.template_columns
                        .iter()
                        .scan(0.0, |acc, size| {
                            let increment = match size.to_std_size(parent_size[0], context) {
                                crate::widgets::div_size::StdDivSize::Pixel(px) => px,
                                crate::widgets::div_size::StdDivSize::Grow(_) => 0.0,
                            };
                            let accumulate_template = *acc + increment;
                            *acc = accumulate_template
                                + match horizontal_gap {
                                    StdDivSize::Pixel(gap_px) => gap_px,
                                    StdDivSize::Grow(_) => 0.0,
                                };
                            Some(accumulate_template)
                        })
                        .collect::<Vec<_>>(),
                    match horizontal_gap {
                        StdDivSize::Pixel(gap_px) => gap_px,
                        StdDivSize::Grow(_) => 0.0,
                    },
                )
            }
        };

        let (total_vertical_size, total_vertical_grow) =
            self.template_rows
                .iter()
                .fold((0.0, 0.0), |(acc_size, acc_grow), size| {
                    match size.to_std_size(parent_size[1], context) {
                        crate::widgets::div_size::StdDivSize::Pixel(px) => {
                            (acc_size + px, acc_grow)
                        }
                        crate::widgets::div_size::StdDivSize::Grow(grow) => {
                            (acc_size, acc_grow + grow)
                        }
                    }
                });

        let vertical_gap = self.gap_rows.to_std_size(parent_size[1], context);

        let (row_accumulate_template, row_gap) = match parent_size[1] {
            StdSize::Pixel(parent_px) => {
                if parent_px < total_vertical_size {
                    // overflowing
                    // gap = 0.0
                    // grows = 0.0
                    (
                        self.template_rows
                            .iter()
                            .scan(0.0, |acc, size| {
                                let increment = match size.to_std_size(parent_size[1], context) {
                                    crate::widgets::div_size::StdDivSize::Pixel(px) => px,
                                    crate::widgets::div_size::StdDivSize::Grow(grow) => grow,
                                };
                                let accumulate_template = *acc + increment;
                                *acc = accumulate_template;
                                Some(accumulate_template)
                            })
                            .collect::<Vec<_>>(),
                        0.0,
                    )
                } else {
                    // not overflowing
                    match vertical_gap {
                        StdDivSize::Pixel(gap_px) => {
                            let true_gap = gap_px.min(
                                (parent_px - total_vertical_size)
                                    / (self.template_rows.len() as f32 - 1.0),
                            );

                            let remain_px = parent_px
                                - total_vertical_size
                                - true_gap * (self.template_rows.len() as f32 - 1.0);

                            let unit_grow = remain_px / total_vertical_grow;

                            (
                                self.template_rows
                                    .iter()
                                    .scan(0.0, |acc, size| {
                                        let increment = match size
                                            .to_std_size(parent_size[1], context)
                                        {
                                            crate::widgets::div_size::StdDivSize::Pixel(px) => px,
                                            crate::widgets::div_size::StdDivSize::Grow(grow) => {
                                                grow * unit_grow
                                            }
                                        };
                                        let accumulate_template = *acc + increment;
                                        *acc = accumulate_template + true_gap;
                                        Some(accumulate_template)
                                    })
                                    .collect::<Vec<_>>(),
                                true_gap,
                            )
                        }
                        StdDivSize::Grow(gap_grow) => {
                            let total_gap_grow = gap_grow * (self.template_rows.len() as f32 - 1.0);

                            let remain_px = parent_px - total_vertical_size;

                            let unit_grow = remain_px / (total_vertical_grow + total_gap_grow);

                            (
                                self.template_rows
                                    .iter()
                                    .scan(0.0, |acc, size| {
                                        let increment = match size
                                            .to_std_size(parent_size[1], context)
                                        {
                                            crate::widgets::div_size::StdDivSize::Pixel(px) => px,
                                            crate::widgets::div_size::StdDivSize::Grow(grow) => {
                                                grow * unit_grow
                                            }
                                        };
                                        let accumulate_template = *acc + increment;
                                        *acc = accumulate_template + gap_grow * unit_grow;
                                        Some(accumulate_template)
                                    })
                                    .collect::<Vec<_>>(),
                                gap_grow * unit_grow,
                            )
                        }
                    }
                }
            }
            StdSize::Content(_) => {
                // grow = 0.0
                (
                    self.template_rows
                        .iter()
                        .scan(0.0, |acc, size| {
                            let increment = match size.to_std_size(parent_size[1], context) {
                                crate::widgets::div_size::StdDivSize::Pixel(px) => px,
                                crate::widgets::div_size::StdDivSize::Grow(_) => 0.0,
                            };
                            let accumulate_template = *acc + increment;
                            *acc = accumulate_template
                                + match vertical_gap {
                                    StdDivSize::Pixel(gap_px) => gap_px,
                                    StdDivSize::Grow(_) => 0.0,
                                };
                            Some(accumulate_template)
                        })
                        .collect::<Vec<_>>(),
                    match vertical_gap {
                        StdDivSize::Pixel(gap_px) => gap_px,
                        StdDivSize::Grow(_) => 0.0,
                    },
                )
            }
        };

        let px_size = [
            *column_accumulate_template.last().unwrap_or(&0.0),
            *row_accumulate_template.last().unwrap_or(&0.0),
        ];

        GridCache {
            tag: parent_size,
            column_accumulate_template,
            column_gap,
            row_accumulate_template,
            row_gap,
            px_size,
        }
    }
}

impl<T: Send + 'static> Widget<T> for GridNode<T> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn update_widget_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<Grid<T>>() {
            Err(())
        } else {
            let dom = dom.as_any().downcast_ref::<Grid<T>>().unwrap();
            todo!()
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(_) = dom.as_any().downcast_ref::<Grid<T>>() {
            todo!()
        } else {
            DomComPareResult::Different
        }
    }

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> crate::events::UiEventResult<T> {
        // todo
        Default::default()
    }

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

    fn size(&self) -> [Size; 2] {
        [Size::Parent(1.0), Size::Parent(1.0)]
    }

    fn px_size(&self, parent_size: [StdSize; 2], context: &SharedContext) -> [f32; 2] {
        self.grid_cache
            .lock()
            .unwrap()
            .get_or_insert_with(|| self.cache_grid(parent_size, context))
            .px_size
    }

    fn drawing_range(&self, parent_size: [StdSize; 2], context: &SharedContext) -> [[f32; 2]; 2] {
        let px_size = self.px_size(parent_size, context);

        [[0.0, 0.0], [px_size[0], px_size[1]]]
    }

    fn cover_area(
        &self,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> Option<[[f32; 2]; 2]> {
        None
    }

    fn has_dynamic(&self) -> bool {
        self.has_dynamic
    }

    fn redraw(&self) -> bool {
        self.redraw
    }

    fn render(
        &mut self,
        // ui environment
        parent_size: [StdSize; 2],
        background_view: &wgpu::TextureView,
        background_range: Range2D<f32>,
        // context
        context: &SharedContext,
        renderer: &Renderer,
        frame: u64,
    ) -> Vec<crate::ui::Object> {
        // ! pay attention to that this widget are always Overflow::Visible and do not cache children's render result.

        // delete cache if cache miss.
        {
            let mut grid_cache = self.grid_cache.lock().unwrap();

            if let Some(cache) = grid_cache.as_ref() {
                if cache.tag != parent_size {
                    *grid_cache = None;
                }
            }
        }

        // calculate grid
        let mut grid_cache = self.grid_cache.lock().unwrap();
        let grid_cache = grid_cache.get_or_insert_with(|| self.cache_grid(parent_size, context));

        // render

        self.items
            .iter_mut()
            .map(|item| {
                // horizontal position range
                let (horizontal_template_start, horizontal_template_end) =
                    if item.column[0] <= item.column[1] {
                        (item.column[0], item.column[1])
                    } else {
                        (item.column[1], item.column[0])
                    };

                let horizontal_start = if horizontal_template_start == 0 {
                    0.0
                } else {
                    grid_cache.column_accumulate_template[horizontal_template_start - 1]
                        + grid_cache.column_gap
                };

                let horizontal_end = grid_cache.column_accumulate_template[horizontal_template_end];

                // vertical position range
                let (vertical_template_start, vertical_template_end) = if item.row[0] <= item.row[1]
                {
                    (item.row[0], item.row[1])
                } else {
                    (item.row[1], item.row[0])
                };

                let vertical_start = if vertical_template_start == 0 {
                    0.0
                } else {
                    grid_cache.row_accumulate_template[vertical_template_start - 1]
                        + grid_cache.row_gap
                };

                let vertical_end = grid_cache.row_accumulate_template[vertical_template_end];

                // get render items
                let render_items = item.item.render(
                    [
                        StdSize::Pixel(horizontal_end - horizontal_start),
                        StdSize::Pixel(vertical_end - vertical_start),
                    ],
                    background_view,
                    Range2D {
                        // rewrite it to use completion function
                        x: [
                            completion(background_range.x, vertical_start / grid_cache.px_size[0]),
                            completion(background_range.x, vertical_end / grid_cache.px_size[0]),
                        ],
                        y: [
                            completion(
                                background_range.y,
                                horizontal_start / grid_cache.px_size[1],
                            ),
                            completion(background_range.y, horizontal_end / grid_cache.px_size[1]),
                        ],
                    },
                    context,
                    renderer,
                    frame,
                );

                // affine matrix
                let translation = nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(
                    horizontal_start,
                    -vertical_start,
                    0.0,
                ));

                render_items.into_iter().map(move |mut object| {
                    object.translate(translation);
                    object
                })
            })
            .flatten()
            .collect()
    }
}

fn completion(completion: [f32; 2], ratio: f32) -> f32 {
    completion[0] * ratio + completion[1] * (1.0 - ratio)
}
