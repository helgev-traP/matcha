use matcha_core::{
    context::SharedContext,
    events::{UiEvent, UiEventResult},
    types::{double_cache_set::DoubleSetCache, range::Range2D},
    ui::{Dom, DomComPareResult, Object, UiBackground, UiContext, UpdateWidgetError, Widget},
};

mod property;
use num::Float;
pub use property::DivSize;
use property::StdDivSize;

// todo: remove this memo
// ********************************
// columnsは横幅の設定、rowsは縦幅の設定
// ********************************

// MARK: Dom

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

impl<T> Default for Grid<T>
where
    T: Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Send + 'static> Grid<T> {
    pub fn new() -> Self {
        Self {
            label: None,
            template_columns: Vec::new(),
            template_rows: Vec::new(),
            gap_columns: DivSize::Pixel(0.0),
            gap_rows: DivSize::Pixel(0.0),
            items: Vec::new(),
        }
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn template_columns(mut self, columns: Vec<DivSize>) -> Self {
        self.template_columns = columns;
        self
    }

    pub fn template_rows(mut self, rows: Vec<DivSize>) -> Self {
        self.template_rows = rows;
        self
    }

    pub fn gap_columns(mut self, gap: DivSize) -> Self {
        self.gap_columns = gap;
        self
    }

    pub fn gap_rows(mut self, gap: DivSize) -> Self {
        self.gap_rows = gap;
        self
    }

    pub fn item(
        mut self,
        item: impl Dom<T> + 'static,
        column: [usize; 2],
        row: [usize; 2],
    ) -> Self {
        self.items.push(GridItem {
            item: Box::new(item),
            column,
            row,
        });
        self
    }
}

impl<T: Send + 'static> Dom<T> for Grid<T> {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        todo!()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// MARK: Widget

pub struct GridNode<T: Send + 'static> {
    // label
    label: Option<String>,

    // layout properties
    template_columns: Vec<DivSize>,
    template_rows: Vec<DivSize>,
    gap_columns: DivSize,
    gap_rows: DivSize,

    // items
    items: Vec<GridNodeItem<T>>,

    // redraw flag
    redraw: bool,

    // render cache
    context_cache: DoubleSetCache<CacheKey, GridCache>,
}

// MARK: GridNodeItem

struct GridNodeItem<T: Send + 'static> {
    column: [usize; 2],
    row: [usize; 2],
    item: Box<dyn Widget<T>>,
}

// MARK: Cache

/// stores tenfold width and height with integer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    size: [Option<u32>; 2],
    tag: u64,
}

impl CacheKey {
    fn new(size: [Option<f32>; 2], tag: u64) -> Self {
        Self {
            size: [
                size[0].map(|f| (f * 10.0) as u32),
                size[1].map(|f| (f * 10.0) as u32),
            ],
            tag,
        }
    }
}

struct GridCache {
    // [[column_start, column_end]; num_columns]
    column_range: Vec<[f32; 2]>,
    // [[row_start, row_end]; num_rows]
    row_range: Vec<[f32; 2]>,
}

impl GridCache {
    fn get_actual_size(&self) -> [f32; 2] {
        let column_end = self
            .column_range
            .last()
            .map(|range| range[1])
            .unwrap_or(0.0);
        let row_end = self.row_range.last().map(|range| range[1]).unwrap_or(0.0);

        [column_end, row_end]
    }
}

// MARK: Widget impl

impl<T: Send + 'static> Widget<T> for GridNode<T> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn update_widget_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), UpdateWidgetError> {
        // todo: be sure to update redraw flag

        if let Some(dom) = dom.as_any().downcast_ref::<Grid<T>>() {
            todo!()
        } else {
            Err(UpdateWidgetError::TypeMismatch)
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(dom) = dom.as_any().downcast_ref::<Grid<T>>() {
            todo!()
        } else {
            DomComPareResult::Different
        }
    }

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> UiEventResult<T> {
        // todo !
        UiEventResult::default()
    }

    fn px_size(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> [f32; 2] {
        let cache =
            self.context_cache
                .get_or_insert_with(CacheKey::new(parent_size, tag), frame, || {
                    let (column_range, row_range) = calc_px_siz(
                        parent_size,
                        &self.template_columns,
                        self.gap_columns,
                        &self.template_rows,
                        self.gap_rows,
                        context,
                    );

                    GridCache {
                        column_range,
                        row_range,
                    }
                });

        cache.get_actual_size()
    }

    fn draw_range(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> Option<Range2D<f32>> {
        // todo: optimize
        let [width, height] = self.px_size(parent_size, context, tag, frame);

        Some(Range2D::new([0.0, width], [0.0, height]).unwrap())
    }

    fn cover_area(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> Option<Range2D<f32>> {
        // todo: optimize
        let [width, height] = self.px_size(parent_size, context, tag, frame);

        Some(Range2D::new([0.0, width], [0.0, height]).unwrap())
    }

    fn redraw(&self) -> bool {
        self.redraw || self.items.iter().any(|item| item.item.redraw())
    }

    fn render(&mut self, ui_background: UiBackground, ui_context: UiContext) -> Vec<Object> {
        let grid_cache = self.context_cache.get_or_insert_with(
            CacheKey::new(ui_background.parent_size, ui_context.tag),
            ui_context.frame,
            || {
                let (column_range, row_range) = calc_px_siz(
                    ui_background.parent_size,
                    &self.template_columns,
                    self.gap_columns,
                    &self.template_rows,
                    self.gap_rows,
                    ui_context.context,
                );

                GridCache {
                    column_range,
                    row_range,
                }
            },
        );

        self.items
            .iter_mut()
            .flat_map(|item| render_item(item, grid_cache, ui_background, ui_context))
            .collect()
    }
}

// MARK: render fn

fn render_item<T: Send + 'static>(
    item: &mut GridNodeItem<T>,
    grid_cache: &GridCache,
    UiBackground {
        parent_size,
        background_view,
        background_range,
    }: UiBackground,
    UiContext {
        context,
        renderer,
        tag,
        frame,
    }: UiContext,
) -> Vec<Object> {
    // calculate range
    let grid_size = grid_cache.get_actual_size();
    let x_range = [
        grid_cache.column_range[item.column[0]][0],
        grid_cache.column_range[item.column[1]][1],
    ];
    let y_range = [
        grid_cache.row_range[item.row[0]][0],
        grid_cache.row_range[item.row[1]][1],
    ];

    let bg_range_x = background_range.x_range();
    let bg_range_y = background_range.y_range();

    // calculate background range
    let background_range_x_start =
        interpolate(0.0, bg_range_x[0], grid_size[0], bg_range_x[1], x_range[0]);

    let background_range_x_end =
        interpolate(0.0, bg_range_x[0], grid_size[0], bg_range_x[1], x_range[1]);

    let background_range_y_start =
        interpolate(0.0, bg_range_y[0], grid_size[1], bg_range_y[1], y_range[0]);

    let background_range_y_end =
        interpolate(0.0, bg_range_y[0], grid_size[1], bg_range_y[1], y_range[1]);

    // render
    item.item
        .render(
            UiBackground {
                parent_size: [Some(x_range[1] - x_range[0]), Some(y_range[1] - y_range[0])],
                background_view,
                background_range: Range2D::new(
                    [background_range_x_start, background_range_x_end],
                    [background_range_y_start, background_range_y_end],
                )
                .unwrap(),
            },
            UiContext {
                context,
                renderer,
                tag,
                frame,
            },
        )
        .into_iter()
        .map(|mut object| {
            object.translate(nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(
                x_range[0], y_range[0], 0.0,
            )));
            object
        })
        .collect()
}

// MARK: interpolate fn

fn interpolate<T: Float>(a_pos: T, a_val: T, b_pos: T, b_val: T, position: T) -> T {
    a_val + (position - a_pos) * ((b_val - a_val) / (b_pos - a_pos))
}

// MARK: calc_px_size

/// returns ([[column_start, column_end]; num_columns], [[row_start, row_end]; num_rows])
fn calc_px_siz(
    parent_size: [Option<f32>; 2],
    template_columns: &[DivSize],
    column_gap: DivSize,
    template_rows: &[DivSize],
    row_gap: DivSize,
    context: &SharedContext,
) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    // convert to standard size

    let std_template_columns = template_columns
        .iter()
        .map(|size| size.to_std_div_size(parent_size[0], context))
        .collect::<Vec<_>>();

    let std_template_rows = template_rows
        .iter()
        .map(|size| size.to_std_div_size(parent_size[1], context))
        .collect::<Vec<_>>();

    let std_column_gap = column_gap.to_std_div_size(parent_size[0], context);
    let std_row_gap = row_gap.to_std_div_size(parent_size[1], context);

    // sum up pixels and grows

    let (column_px_sum, column_grow_sum) = std_template_columns
        .iter()
        .chain([std_column_gap].iter())
        .fold((0.0, 0.0), |(sum, grow_sum), size| match size {
            StdDivSize::Pixel(px) => (sum + px, grow_sum),
            StdDivSize::Grow(grow) => (sum, grow_sum + grow),
        });

    let (row_px_sum, row_grow_sum) = std_template_rows.iter().chain([std_row_gap].iter()).fold(
        (0.0, 0.0),
        |(sum, grow_sum), size| match size {
            StdDivSize::Pixel(px) => (sum + px, grow_sum),
            StdDivSize::Grow(grow) => (sum, grow_sum + grow),
        },
    );

    // calculate pixel per grow unit

    let column_px_per_grow =
        ((parent_size[0].unwrap_or(0.0) - column_px_sum) / column_grow_sum).max(0.0);
    let row_px_per_grow = ((parent_size[1].unwrap_or(0.0) - row_px_sum) / row_grow_sum).max(0.0);

    // accumulate template

    // column

    let mut column_accumulate_template: Vec<[f32; 2]> =
        Vec::with_capacity(std_template_columns.len());
    let mut column_accumulate = 0.0;

    let column_gap = match std_column_gap {
        StdDivSize::Pixel(px) => px,
        StdDivSize::Grow(grow) => column_px_per_grow * grow,
    };

    for size in std_template_columns {
        let start = column_accumulate;
        let end = match size {
            StdDivSize::Pixel(px) => start + px,
            StdDivSize::Grow(grow) => start + column_px_per_grow * grow,
        };

        column_accumulate_template.push([start, end]);

        column_accumulate = end + column_gap;
    }

    // row

    let mut row_accumulate_template: Vec<[f32; 2]> = Vec::with_capacity(std_template_rows.len());
    let mut row_accumulate = 0.0;

    let row_gap = match std_row_gap {
        StdDivSize::Pixel(px) => px,
        StdDivSize::Grow(grow) => row_px_per_grow * grow,
    };

    for size in std_template_rows {
        let start = row_accumulate;
        let end = match size {
            StdDivSize::Pixel(px) => start + px,
            StdDivSize::Grow(grow) => start + row_px_per_grow * grow,
        };

        row_accumulate_template.push([start, end]);

        row_accumulate = end + row_gap;
    }

    (column_accumulate_template, row_accumulate_template)
}
