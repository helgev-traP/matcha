use std::sync::Arc;

use crate::{
    context::SharedContext,
    events::UiEvent,
    renderer::Renderer,
    types::{
        color::Color,
        size::{Size, StdSize},
    },
    ui::{Dom, DomComPareResult, Widget},
    vertex::uv_vertex::UvVertex,
};

use super::{
    div_size::DivSize,
    layout::{AlignContent, FlexWrap, JustifyContent},
    style::{Border, BoxSizing, Padding, Visibility},
};

// todo: consider the style properties

pub struct RowDescriptor<R> {
    // label
    pub label: Option<String>,
    // style
    pub size: [Size; 2],
    pub padding: Padding,
    pub border: Border,
    pub box_sizing: BoxSizing,
    pub visibility: Visibility,
    pub background_color: Color,
    pub border_color: Color,
    // todo: border_radius: f32,
    // layout
    // direction -> row(not reverse)
    pub wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_content: AlignContent,
    // items
    pub items: Vec<Box<dyn Dom<R>>>,
}

impl<R> Default for RowDescriptor<R> {
    fn default() -> Self {
        Self {
            // label
            label: None,
            // style
            size: [Size::Content(1.0), Size::Content(1.0)],
            padding: Padding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            },
            border: Border {
                px: 0.0,
                color: [0, 0, 0, 0],
                top_left_radius: 0.0,
                top_right_radius: 0.0,
                bottom_left_radius: 0.0,
                bottom_right_radius: 0.0,
            },
            box_sizing: BoxSizing::BorderBox,
            visibility: Visibility::Visible,
            background_color: [0, 0, 0, 0].into(),
            border_color: [0, 0, 0, 0].into(),
            // layout
            wrap: FlexWrap::NoWrap,
            justify_content: JustifyContent::FlexStart {
                gap: DivSize::Pixel(0.0),
            },
            align_content: AlignContent::Start,
            // items
            items: vec![],
        }
    }
}

pub struct Row<R> {
    // label
    label: Option<String>,
    // style
    size: [Size; 2],
    padding: Padding,
    border: Border,
    box_sizing: BoxSizing,
    visibility: Visibility,
    background_color: Color,
    border_color: Color,
    // layout
    // direction -> row(not reverse)
    wrap: FlexWrap,
    justify_content: JustifyContent,
    align_content: AlignContent,
    // items
    items: Vec<Box<dyn Dom<R>>>,
}

impl<R> Row<R> {
    pub fn new(disc: RowDescriptor<R>) -> Box<Self> {
        Box::new(Self {
            label: disc.label,
            size: disc.size,
            padding: disc.padding,
            border: disc.border,
            box_sizing: disc.box_sizing,
            visibility: disc.visibility,
            background_color: disc.background_color,
            border_color: disc.border_color,
            wrap: disc.wrap,
            justify_content: disc.justify_content,
            align_content: disc.align_content,
            items: disc.items,
        })
    }

    pub fn push(&mut self, child: Box<dyn Dom<R>>) {
        self.items.push(child);
    }
}

impl<R: Send + 'static> Dom<R> for Row<R> {
    fn build_widget_tree(&self) -> Box<dyn Widget<R>> {
        Box::new(RowRenderNode {
            label: self.label.clone(),
            size: self.size,
            padding: self.padding,
            border: self.border,
            box_sizing: self.box_sizing,
            visibility: self.visibility,
            background_color: self.background_color,
            border_color: self.border_color,
            wrap: self.wrap,
            justify_content: self.justify_content,
            align_content: self.align_content,
            items: self
                .items
                .iter()
                .map(|child| Item {
                    item: child.build_widget_tree(),
                    position: None,
                    size: None,
                })
                .collect(),
            self_size: None,
            rendering_cache: None,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct RowRenderNode<T: 'static> {
    // label
    label: Option<String>,

    // style
    size: [Size; 2],
    padding: Padding,
    border: Border,
    box_sizing: BoxSizing,
    visibility: Visibility,
    background_color: Color,
    border_color: Color,

    // layout (direction: row(not reverse))
    wrap: FlexWrap,
    justify_content: JustifyContent,
    align_content: AlignContent,

    // items
    items: Vec<Item<T>>,

    // render status
    self_size: Option<[f32; 2]>,

    // rendering cache
    rendering_cache: Option<(Arc<wgpu::Texture>, Arc<Vec<UvVertex>>)>,
}

struct Item<T> {
    item: Box<dyn Widget<T>>,
    // cache
    position: Option<[f32; 2]>,
    size: Option<[f32; 2]>,
}

impl<T> Widget<T> for RowRenderNode<T> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> crate::events::UiEventResult<T> {
        // todo
        crate::events::UiEventResult::default()
    }

    fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> bool {
        todo!()
    }

    fn update_widget_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<Row<T>>() {
            Err(())
        } else {
            let dom = dom.as_any().downcast_ref::<Row<T>>().unwrap();
            // todo: differential update
            let mut i = 0;
            loop {
                match (self.items.get_mut(i), dom.items.get(i)) {
                    (Some(child), Some(new_child)) => {
                        child.item.update_widget_tree(&**new_child)?;
                        i += 1;
                    }
                    (Some(_), None) => {
                        self.items.pop();
                    }
                    (None, Some(new_child)) => {
                        self.items.push(Item {
                            item: new_child.build_widget_tree(),
                            position: None,
                            size: None,
                        });
                        i += 1;
                    }
                    (None, None) => break,
                }
            }
            Ok(())
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(_) = dom.as_any().downcast_ref::<Row<T>>() {
            // todo: calculate difference

            DomComPareResult::Different
        } else {
            DomComPareResult::Different
        }
    }

    fn size(&self) -> [Size; 2] {
        self.size
    }

    fn px_size(&self, parent_size: [StdSize; 2], context: &SharedContext) -> [f32; 2] {
        todo!()
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
        let std_size = [
            self.size[0].to_std_size(parent_size[0], context),
            self.size[1].to_std_size(parent_size[1], context),
        ];

        match self.wrap {
            FlexWrap::NoWrap => {
                // get the size of each child
                let mut sizes = Vec::with_capacity(self.items.len());
                let mut total_items_width: f32 = 0.0;
                let mut max_items_height: f32 = 0.0;

                for item in self.items.iter_mut() {
                    let px_size = item.item.px_size(std_size, context);

                    sizes.push(px_size);
                    total_items_width += px_size[0];
                    max_items_height = max_items_height.max(px_size[1]);
                }

                let height = if let StdSize::Pixel(height) = std_size[1] {
                    height
                } else {
                    max_items_height
                };

                // calculate the position of each child

                // y position

                let mut y_positions = vec![0.0; self.items.len()];

                match self.align_content {
                    AlignContent::Start => {
                        // all y positions are 0.0
                    }
                    AlignContent::End => {
                        for (i, y) in y_positions.iter_mut().enumerate() {
                            *y = height - sizes[i][1];
                        }
                    }
                    AlignContent::Center => {
                        for (i, y) in y_positions.iter_mut().enumerate() {
                            *y = (height - sizes[i][1]) / 2.0;
                        }
                    }
                }

                // x position and layout

                let (gap, mut x) = match std_size[0] {
                    StdSize::Content(_) => {
                        // No grow, no space, x left offset is 0.0

                        let gap = match self.justify_content {
                            JustifyContent::FlexStart { gap }
                            | JustifyContent::FlexEnd { gap }
                            | JustifyContent::Center { gap } => {
                                match gap.to_std_size(std_size[0], context) {
                                    crate::widgets::div_size::StdDivSize::Pixel(px) => px,
                                    crate::widgets::div_size::StdDivSize::Grow(_) => 0.0,
                                }
                            }
                            JustifyContent::SpaceBetween => 0.0,
                            JustifyContent::SpaceAround => 0.0,
                            JustifyContent::SpaceEvenly => 0.0,
                        };

                        // left offset will be 0.0

                        (gap, 0.0)
                    }
                    StdSize::Pixel(width) => {
                        let gap = match self.justify_content {
                            JustifyContent::FlexStart { gap }
                            | JustifyContent::FlexEnd { gap }
                            | JustifyContent::Center { gap } => {
                                match gap.to_std_size(std_size[0], context) {
                                    crate::widgets::div_size::StdDivSize::Pixel(px) => px,
                                    crate::widgets::div_size::StdDivSize::Grow(_) => {
                                        (width - total_items_width)
                                            / (self.items.len() as f32 - 1.0)
                                    }
                                }
                            }
                            JustifyContent::SpaceBetween => ((width - total_items_width)
                                / (self.items.len() as f32 - 1.0))
                                .max(0.0),
                            JustifyContent::SpaceAround => {
                                ((width - total_items_width) / (self.items.len() as f32)).max(0.0)
                            }
                            JustifyContent::SpaceEvenly => ((width - total_items_width)
                                / (self.items.len() as f32 + 1.0))
                                .max(0.0),
                        };

                        let x = match self.justify_content {
                            JustifyContent::FlexStart { .. } | JustifyContent::SpaceBetween => 0.0,
                            JustifyContent::FlexEnd { .. } => {
                                width - total_items_width - gap * (self.items.len() as f32 - 1.0)
                            }
                            JustifyContent::Center { .. } => {
                                (width - total_items_width - gap * (self.items.len() as f32 - 1.0))
                                    / 2.0
                            }
                            JustifyContent::SpaceAround => gap / 2.0,
                            JustifyContent::SpaceEvenly => gap,
                        };

                        (gap, x)
                    }
                };

                self.items
                    .iter_mut()
                    .enumerate()
                    .map(|(i, item)| {
                        let position = [x, y_positions[i]];
                        x += sizes[i][0] + gap;
                        let translate = nalgebra::Matrix4::new_translation(
                            &nalgebra::Vector3::new(position[0], -position[1], 0.0),
                        );
                        item.item
                            .render(
                                [std_size[0], StdSize::Pixel(height)],
                                context,
                                renderer,
                                frame,
                            )
                            .into_iter()
                            .map(move |(texture, vertices, indices, transform)| {
                                (texture, vertices, indices, transform * translate)
                            })
                    })
                    .flatten()
                    .collect()
            }
            FlexWrap::Wrap | FlexWrap::WrapReverse => todo!(),
        }
    }
}
