use std::any::Any;

use matcha_core::{
    device_event::DeviceEvent,
    render_node::RenderNode,
    types::range::{CoverRange, Range2D},
    ui::{
        Background, Constraints, Dom, DomComPareResult, UpdateWidgetError, Widget, WidgetContext,
    },
    update_flag::UpdateNotifier,
};

use crate::types::size::{ChildSize, Size};

// MARK: Dom

pub struct Grid<T: Send + 'static> {
    label: Option<String>,
    template_columns: Vec<Size>,
    template_rows: Vec<Size>,
    gap_columns: Size,
    gap_rows: Size,
    items: Vec<GridItem<T>>,
}

pub struct GridItem<T: Send + 'static> {
    pub item: Box<dyn Dom<T>>,
    pub column: [usize; 2],
    pub row: [usize; 2],
}

impl<T: Send + 'static> Grid<T> {
    pub fn new() -> Self {
        Self {
            label: None,
            template_columns: Vec::new(),
            template_rows: Vec::new(),
            gap_columns: Size::px(0.0),
            gap_rows: Size::px(0.0),
            items: Vec::new(),
        }
    }

    pub fn template_columns(mut self, columns: Vec<Size>) -> Self {
        self.template_columns = columns;
        self
    }

    pub fn template_rows(mut self, rows: Vec<Size>) -> Self {
        self.template_rows = rows;
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

#[async_trait::async_trait]
impl<T: Send + 'static> Dom<T> for Grid<T> {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(GridNode {
            label: self.label.clone(),
            template_columns: self.template_columns.clone(),
            template_rows: self.template_rows.clone(),
            gap_columns: self.gap_columns.clone(),
            gap_rows: self.gap_rows.clone(),
            items: self
                .items
                .iter()
                .map(|item| GridNodeItem {
                    column: item.column,
                    row: item.row,
                    item: item.item.build_widget_tree(),
                })
                .collect(),
            update_notifier: None,
            column_ranges: Vec::new(),
            row_ranges: Vec::new(),
        })
    }

    async fn set_update_notifier(&self, notifier: &UpdateNotifier) {
        for item in &self.items {
            item.item.set_update_notifier(notifier).await;
        }
    }
}

// MARK: Widget

pub struct GridNode<T: Send + 'static> {
    label: Option<String>,
    template_columns: Vec<Size>,
    template_rows: Vec<Size>,
    gap_columns: Size,
    gap_rows: Size,
    items: Vec<GridNodeItem<T>>,
    update_notifier: Option<UpdateNotifier>,
    column_ranges: Vec<[f32; 2]>,
    row_ranges: Vec<[f32; 2]>,
}

struct GridNodeItem<T: Send + 'static> {
    column: [usize; 2],
    row: [usize; 2],
    item: Box<dyn Widget<T>>,
}

#[async_trait::async_trait]
impl<T: Send + 'static> Widget<T> for GridNode<T> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    async fn update_widget_tree(
        &mut self,
        _component_updated: bool,
        dom: &dyn Dom<T>,
    ) -> Result<(), UpdateWidgetError> {
        if let Some(_dom) = (dom as &dyn Any).downcast_ref::<Grid<T>>() {
            // Simplified update logic
            Ok(())
        } else {
            Err(UpdateWidgetError::TypeMismatch)
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if (dom as &dyn Any).downcast_ref::<Grid<T>>().is_some() {
            DomComPareResult::Same // Simplified
        } else {
            DomComPareResult::Different
        }
    }

    fn device_event(&mut self, event: &DeviceEvent, context: &WidgetContext) -> Option<T> {
        self.items
            .iter_mut()
            .find_map(|item| item.item.device_event(event, context))
    }

    fn is_inside(&mut self, position: [f32; 2], context: &WidgetContext) -> bool {
        self.items
            .iter_mut()
            .any(|item| item.item.is_inside(position, context))
    }

    fn preferred_size(&mut self, constraints: &Constraints, context: &WidgetContext) -> [f32; 2] {
        // A proper implementation would calculate the preferred size based on content.
        // For now, we just use the constraints.
        [constraints.max_width, constraints.max_height]
    }

    fn arrange(&mut self, final_size: [f32; 2], context: &WidgetContext) {
        let (column_ranges, row_ranges) = calc_px_siz(
            [Some(final_size[0]), Some(final_size[1])],
            &self.template_columns,
            &self.gap_columns,
            &self.template_rows,
            &self.gap_rows,
            context,
        );
        self.column_ranges = column_ranges;
        self.row_ranges = row_ranges;

        for item in &mut self.items {
            let col_start = self.column_ranges[item.column[0]][0];
            let col_end = self.column_ranges[item.column[1] - 1][1];
            let row_start = self.row_ranges[item.row[0]][0];
            let row_end = self.row_ranges[item.row[1] - 1][1];
            let item_size = [col_end - col_start, row_end - row_start];
            item.item.arrange(item_size, context);
        }
    }

    fn cover_range(&mut self, _context: &WidgetContext) -> CoverRange<f32> {
        CoverRange::default()
    }

    fn need_rerendering(&self) -> bool {
        self.items.iter().any(|item| item.item.need_rerendering())
    }

    fn render(
        &mut self,
        background: Background,
        animation_update_flag_notifier: UpdateNotifier,
        ctx: &WidgetContext,
    ) -> RenderNode {
        self.update_notifier = Some(animation_update_flag_notifier);
        let mut render_node = RenderNode::new();

        for item in &mut self.items {
            let col_start = self.column_ranges[item.column[0]][0];
            let row_start = self.row_ranges[item.row[0]][0];
            let position = [col_start, row_start];

            let notifier = self.update_notifier.clone().unwrap();
            let transform = nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(
                position[0],
                position[1],
                0.0,
            ));
            let child_node = item
                .item
                .render(background.transition(position), notifier, ctx);
            render_node.add_child(child_node, transform);
        }

        render_node
    }
}

fn calc_px_siz(
    parent_size: [Option<f32>; 2],
    template_columns: &[Size],
    column_gap: &Size,
    template_rows: &[Size],
    row_gap: &Size,
    context: &WidgetContext,
) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let (column_px_sum, column_grow_sum) =
        template_columns
            .iter()
            .fold((0.0, 0.0), |(sum, grow_sum), size| match size {
                Size::Size(f) => (
                    sum + f(parent_size, &mut ChildSize::default(), context),
                    grow_sum,
                ),
                Size::Grow(f) => (
                    sum,
                    grow_sum + f(parent_size, &mut ChildSize::default(), context),
                ),
            });

    let (row_px_sum, row_grow_sum) =
        template_rows
            .iter()
            .fold((0.0, 0.0), |(sum, grow_sum), size| match size {
                Size::Size(f) => (
                    sum + f(parent_size, &mut ChildSize::default(), context),
                    grow_sum,
                ),
                Size::Grow(f) => (
                    sum,
                    grow_sum + f(parent_size, &mut ChildSize::default(), context),
                ),
            });

    let column_gap_px = match column_gap {
        Size::Size(f) => f(parent_size, &mut ChildSize::default(), context),
        Size::Grow(f) => f(parent_size, &mut ChildSize::default(), context), // Grow in gap is not well-defined, treat as px for now
    };
    let total_column_gap = column_gap_px * (template_columns.len().saturating_sub(1) as f32);

    let row_gap_px = match row_gap {
        Size::Size(f) => f(parent_size, &mut ChildSize::default(), context),
        Size::Grow(f) => f(parent_size, &mut ChildSize::default(), context),
    };
    let total_row_gap = row_gap_px * (template_rows.len().saturating_sub(1) as f32);

    let column_px_per_grow = if column_grow_sum > 0.0 {
        ((parent_size[0].unwrap_or(0.0) - column_px_sum - total_column_gap) / column_grow_sum)
            .max(0.0)
    } else {
        0.0
    };

    let row_px_per_grow = if row_grow_sum > 0.0 {
        ((parent_size[1].unwrap_or(0.0) - row_px_sum - total_row_gap) / row_grow_sum).max(0.0)
    } else {
        0.0
    };

    let mut column_ranges = Vec::with_capacity(template_columns.len());
    let mut current_x = 0.0;
    for size in template_columns {
        let start = current_x;
        let width = match size {
            Size::Size(f) => f(parent_size, &mut ChildSize::default(), context),
            Size::Grow(f) => {
                column_px_per_grow * f(parent_size, &mut ChildSize::default(), context)
            }
        };
        let end = start + width;
        column_ranges.push([start, end]);
        current_x = end + column_gap_px;
    }

    let mut row_ranges = Vec::with_capacity(template_rows.len());
    let mut current_y = 0.0;
    for size in template_rows {
        let start = current_y;
        let height = match size {
            Size::Size(f) => f(parent_size, &mut ChildSize::default(), context),
            Size::Grow(f) => row_px_per_grow * f(parent_size, &mut ChildSize::default(), context),
        };
        let end = start + height;
        row_ranges.push([start, end]);
        current_y = end + row_gap_px;
    }

    (column_ranges, row_ranges)
}
