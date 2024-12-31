use std::{default, sync::{Arc, Mutex}};

use crate::{
    context::SharedContext,
    events::UiEvent,
    renderer::Renderer,
    text,
    types::size::{Size, StdSize},
    ui::{Dom, DomComPareResult, Widget},
    vertex::uv_vertex::UvVertex,
    widgets::div_size::{DivSize, StdDivSize},
};

// todo:
// add padding, margin, border, box-sizing, etc.

pub struct GridDescriptor<T> {
    pub label: Option<String>,

    pub template_columns: Vec<DivSize>,
    pub template_rows: Vec<DivSize>,
    pub gap_columns: DivSize,
    pub gap_rows: DivSize,

    pub items: Vec<GridItem<T>>,
}

pub struct GridItem<T> {
    pub item: Box<dyn Dom<T>>,
    pub column: [usize; 2],
    pub row: [usize; 2],
}

pub struct Grid<T> {
    label: Option<String>,

    template_columns: Vec<DivSize>,
    template_rows: Vec<DivSize>,
    gap_columns: DivSize,
    gap_rows: DivSize,

    items: Vec<GridItem<T>>,
}

impl<T> Grid<T> {
    pub fn new(disc: GridDescriptor<T>) -> Box<Self> {
        Box::new(Self {
            label: disc.label,
            template_columns: disc.template_columns,
            template_rows: disc.template_rows,
            gap_columns: disc.gap_columns,
            gap_rows: disc.gap_rows,
            items: disc.items,
        })
    }
}

impl<T: Send + 'static> Dom<T> for Grid<T> {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(GridNode {
            label: self.label.clone(),
            template_columns: self.template_columns.clone(),
            template_rows: self.template_rows.clone(),
            gap_columns: self.gap_columns,
            gap_rows: self.gap_rows,
            items: self
                .items
                .iter()
                .map(|item| GridItemNode {
                    item: item.item.build_widget_tree(),
                    column: item.column,
                    row: item.row,
                })
                .collect(),
            grid_cache: Mutex::new(None),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct GridNode<T> {
    label: Option<String>,

    template_columns: Vec<DivSize>,
    template_rows: Vec<DivSize>,
    gap_columns: DivSize,
    gap_rows: DivSize,

    items: Vec<GridItemNode<T>>,

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

    // rendering cache
    rendering_cache: Option<(Arc<wgpu::Texture>, Arc<Vec<UvVertex>>)>,
}

pub struct GridItemNode<T> {
    column: [usize; 2],
    row: [usize; 2],
    item: Box<dyn Widget<T>>,
}

impl<T: Send + 'static> GridNode<T> {
    fn cache_grid(&self, parent_size: [StdSize; 2], context: &SharedContext) -> GridCache {
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

        GridCache {
            tag: parent_size,
            column_accumulate_template,
            column_gap,
            row_accumulate_template,
            row_gap,
            rendering_cache: None,
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
        // todo: >>>>>>>>>>>>>> ここから <<<<<<<<<<<<<<<<
        Default::default()
    }

    fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> bool {
        let px_size = self.px_size(parent_size, context);

        !(position[0] < 0.0
            || position[0] > px_size[0]
            || position[1] < 0.0
            || position[1] > px_size[1])
    }

    fn size(&self) -> [Size; 2] {
        todo!()
    }

    fn px_size(&self, parent_size: [StdSize; 2], context: &SharedContext) -> [f32; 2] {
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
        let grid = grid_cache.get_or_insert_with(|| self.cache_grid(parent_size, context));

        [
            grid.column_accumulate_template[grid.column_accumulate_template.len() - 1],
            grid.row_accumulate_template[grid.row_accumulate_template.len() - 1],
        ]
    }

    fn render(
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
        let grid = grid_cache.get_or_insert_with(|| self.cache_grid(parent_size, context));

        // render

        let mut overflows = vec![];

        let render_items = self
            .items
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
                    grid.column_accumulate_template[horizontal_template_start - 1] + grid.column_gap
                };

                let horizontal_end = grid.column_accumulate_template[horizontal_template_end];

                // vertical position range
                let (vertical_template_start, vertical_template_end) = if item.row[0] <= item.row[1] {
                    (item.row[0], item.row[1])
                } else {
                    (item.row[1], item.row[0])
                };

                let vertical_start = if vertical_template_start == 0 {
                    0.0
                } else {
                    grid.row_accumulate_template[vertical_template_start - 1] + grid.row_gap
                };

                let vertical_end = grid.row_accumulate_template[vertical_template_end];

                // get render items
                let render_items = item.item.render(
                    [
                        StdSize::Pixel(horizontal_end - horizontal_start),
                        StdSize::Pixel(vertical_end - vertical_start),
                    ],
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

                // todo: add property: overflow.
                if false {
                    // overflow: visible
                    for (texture, vertices, indices, matrix) in render_items.into_iter() {
                        overflows.push((texture, vertices, indices, translation * matrix));
                    }
                    vec![]
                } else {
                    // overflow: hidden
                    // render to texture
                    render_items
                        .into_iter()
                        .map(|(texture, vertices, indices, matrix)| {
                            (texture, vertices, indices, translation * matrix)
                        })
                        .collect::<Vec<_>>()
                }
            })
            .flatten()
            .collect::<Vec<_>>();

        if !render_items.is_empty() {
            // texture
            let texture = grid.rendering_cache.get_or_insert_with(|| {
                (
                    Arc::new(context.create_texture(
                        grid.column_accumulate_template[grid.column_accumulate_template.len() - 1]
                            as u32,
                        grid.row_accumulate_template[grid.row_accumulate_template.len() - 1] as u32,
                        wgpu::TextureFormat::Rgba8UnormSrgb,
                        wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::TEXTURE_BINDING,
                    )),
                    Arc::new(vec![
                        UvVertex {
                            position: [0.0, 0.0, 0.0].into(),
                            tex_coords: [0.0, 0.0].into(),
                        },
                        UvVertex {
                            position: [
                                0.0,
                                -grid.row_accumulate_template
                                    [grid.row_accumulate_template.len() - 1],
                                0.0,
                            ]
                            .into(),
                            tex_coords: [0.0, 1.0].into(),
                        },
                        UvVertex {
                            position: [
                                grid.column_accumulate_template
                                    [grid.column_accumulate_template.len() - 1],
                                -grid.row_accumulate_template
                                    [grid.row_accumulate_template.len() - 1],
                                0.0,
                            ]
                            .into(),
                            tex_coords: [1.0, 1.0].into(),
                        },
                        UvVertex {
                            position: [
                                grid.column_accumulate_template
                                    [grid.column_accumulate_template.len() - 1],
                                0.0,
                                0.0,
                            ]
                            .into(),
                            tex_coords: [1.0, 0.0].into(),
                        },
                    ]),
                )
            });

            // render
            renderer.render_to_texture(
                &texture
                    .0
                    .create_view(&wgpu::TextureViewDescriptor::default()),
                [texture.0.width() as f32, texture.0.height() as f32],
                render_items,
            );
        }

        // make return value
        if let Some((texture, vertices)) = grid.rendering_cache.as_ref() {
            let mut v = vec![(
                texture.clone(),
                vertices.clone(),
                Arc::new(vec![0, 1, 2, 2, 3, 0]),
                nalgebra::Matrix4::identity(),
            )];

            v.append(&mut overflows);

            v
        } else {
            overflows
        }
    }
}
