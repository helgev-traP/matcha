use nalgebra as na;
use std::{any::Any, cell::Cell, sync::Arc};

use crate::{
    context::SharedContext,
    events::{UiEvent, UiEventResult},
    renderer::Renderer,
    types::size::{PxSize, Size, SizeUnit, StdSize, StdSizeUnit},
    ui::{Dom, DomComPareResult, Widget},
    vertex::uv_vertex::UvVertex,
};

pub struct ColumnDescriptor<R> {
    pub label: Option<String>,
    pub vec: Vec<Box<dyn Dom<R>>>,
}

impl<R> Default for ColumnDescriptor<R> {
    fn default() -> Self {
        Self {
            label: None,
            vec: Vec::new(),
        }
    }
}

pub struct Column<R: 'static> {
    label: Option<String>,
    children: Vec<Box<dyn Dom<R>>>,
}

impl<R: 'static> Column<R> {
    pub fn new(disc: ColumnDescriptor<R>) -> Box<Self> {
        Box::new(Self {
            label: disc.label,
            children: disc.vec,
        })
    }

    pub fn push(&mut self, child: Box<dyn Dom<R>>) {
        self.children.push(child);
    }
}

impl<R: 'static> Dom<R> for Column<R> {
    fn build_widget_tree(&self) -> Box<dyn Widget<R>> {
        Box::new(ColumnRenderNode {
            label: self.label.clone(),
            redraw: true,
            children: self
                .children
                .iter()
                .map(|child| Child {
                    item: child.build_widget_tree(),
                    position: None,
                    size: None,
                })
                .collect(),
            cache_self_size: Cell::new(None),
            mouse_hovering_index: None,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct ColumnRenderNode<T: 'static> {
    label: Option<String>,
    redraw: bool,
    children: Vec<Child<T>>,
    cache_self_size: Cell<Option<PxSize>>,
    mouse_hovering_index: Option<usize>,
}

struct Child<T> {
    item: Box<dyn Widget<T>>,
    // cache
    position: Option<[f32; 2]>,
    size: Option<PxSize>,
}

impl<R: 'static> Widget<R> for ColumnRenderNode<R> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: PxSize,
        context: &SharedContext,
    ) -> UiEventResult<R> {
        match &event.content {
            // mouse event with position
            crate::events::UiEventContent::MouseClick { position, .. }
            | crate::events::UiEventContent::CursorMove { position, .. }
            | crate::events::UiEventContent::MouseScroll { position, .. } => {
                let current_size = self.px_size(parent_size, context);

                let mut accumulated_height: f32 = 0.0;

                let mut mouse_pointer_on = None;

                for (i, child) in &mut self.children.iter_mut().enumerate() {
                    let child_position = child.position.get_or_insert([0.0, accumulated_height]);

                    let child_px_size = child
                        .size
                        .get_or_insert_with(|| child.item.px_size(current_size, context));

                    accumulated_height += child_px_size.height;

                    let is_inside = child.item.is_inside(
                        [
                            position[0] - child_position[0],
                            position[1] - child_position[1],
                        ],
                        current_size,
                        context,
                    );

                    if is_inside {
                        mouse_pointer_on = Some(i);
                    }
                }

                // handle mouse enter or leave
                if mouse_pointer_on != self.mouse_hovering_index {
                    match (self.mouse_hovering_index, mouse_pointer_on) {
                        (None, Some(hover_into)) => {
                            // cursor entered
                            self.mouse_hovering_index = Some(hover_into);

                            self.children[hover_into].item.widget_event(
                                &crate::events::UiEvent {
                                    frame: event.frame,
                                    content: crate::events::UiEventContent::CursorEntered,
                                    diff: (),
                                },
                                current_size,
                                context,
                            );
                        }
                        (Some(hover_out_from), None) => {
                            // cursor left
                            self.mouse_hovering_index = None;

                            self.children[hover_out_from].item.widget_event(
                                &crate::events::UiEvent {
                                    frame: event.frame,
                                    content: crate::events::UiEventContent::CursorLeft,
                                    diff: (),
                                },
                                current_size,
                                context,
                            );
                        }
                        (Some(hover_out_from), Some(hover_into)) => {
                            // cursor shifted
                            self.mouse_hovering_index = Some(hover_into);

                            self.children[hover_out_from].item.widget_event(
                                &crate::events::UiEvent {
                                    frame: event.frame,
                                    content: crate::events::UiEventContent::CursorLeft,
                                    diff: (),
                                },
                                current_size,
                                context,
                            );

                            self.children[hover_into].item.widget_event(
                                &crate::events::UiEvent {
                                    frame: event.frame,
                                    content: crate::events::UiEventContent::CursorEntered,
                                    diff: (),
                                },
                                current_size,
                                context,
                            );
                        }
                        (None, None) => (),
                    }
                }

                // send mouse move event
                if let Some(i) = mouse_pointer_on {
                    let child = &mut self.children[i];
                    // update position
                    match &event.content {
                        crate::events::UiEventContent::MouseClick {
                            position,
                            click_state,
                            button,
                        } => child.item.widget_event(
                            &crate::events::UiEvent {
                                frame: event.frame,
                                content: crate::events::UiEventContent::MouseClick {
                                    position: [
                                        position[0] - child.position.unwrap()[0],
                                        position[1] - child.position.unwrap()[1],
                                    ],
                                    click_state: click_state.clone(),
                                    button: button.clone(),
                                },
                                diff: (),
                            },
                            current_size,
                            context,
                        ),
                        crate::events::UiEventContent::CursorMove {
                            position,
                            primary_dragging_from,
                            secondary_dragging_from,
                            middle_dragging_from,
                        } => child.item.widget_event(
                            &crate::events::UiEvent {
                                frame: event.frame,
                                content: crate::events::UiEventContent::CursorMove {
                                    position: [
                                        position[0] - child.position.unwrap()[0],
                                        position[1] - child.position.unwrap()[1],
                                    ],
                                    primary_dragging_from: primary_dragging_from.map(|from| {
                                        [
                                            from[0] - child.position.unwrap()[0],
                                            from[1] - child.position.unwrap()[1],
                                        ]
                                    }),
                                    secondary_dragging_from: secondary_dragging_from.map(|from| {
                                        [
                                            from[0] - child.position.unwrap()[0],
                                            from[1] - child.position.unwrap()[1],
                                        ]
                                    }),
                                    middle_dragging_from: middle_dragging_from.map(|from| {
                                        [
                                            from[0] - child.position.unwrap()[0],
                                            from[1] - child.position.unwrap()[1],
                                        ]
                                    }),
                                },
                                diff: (),
                            },
                            current_size,
                            context,
                        ),
                        crate::events::UiEventContent::MouseScroll { position, delta } => {
                            child.item.widget_event(
                                &crate::events::UiEvent {
                                    frame: event.frame,
                                    content: crate::events::UiEventContent::MouseScroll {
                                        position: [
                                            position[0] - child.position.unwrap()[0],
                                            position[1] - child.position.unwrap()[1],
                                        ],
                                        delta: *delta,
                                    },
                                    diff: (),
                                },
                                current_size,
                                context,
                            )
                        }
                        _ => unreachable!(),
                    }
                } else {
                    crate::events::UiEventResult::default()
                }
            }
            // others:
            // todo
            crate::events::UiEventContent::CursorEntered => Default::default(),
            crate::events::UiEventContent::CursorLeft => Default::default(),
            // todo
            _ => Default::default(),
        }
    }

    fn is_inside(&self, position: [f32; 2], parent_size: PxSize, context: &SharedContext) -> bool {
        let current_size = self.px_size(parent_size, context);

        !(position[0] < 0.0
            || position[0] > current_size.width
            || position[1] < 0.0
            || position[1] > current_size.height)
    }

    fn update_widget_tree(&mut self, dom: &dyn Dom<R>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<Column<R>>() {
            Err(())
        } else {
            let dom = dom.as_any().downcast_ref::<Column<R>>().unwrap();
            // todo: differential update
            let mut i = 0;
            loop {
                match (self.children.get_mut(i), dom.children.get(i)) {
                    (Some(child), Some(new_child)) => {
                        child.item.update_widget_tree(&**new_child)?;
                        i += 1;
                    }
                    (Some(_), None) => {
                        self.children.pop();
                    }
                    (None, Some(new_child)) => {
                        self.children.push(Child {
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

    fn compare(&self, dom: &dyn Dom<R>) -> DomComPareResult {
        if let Some(_) = dom.as_any().downcast_ref::<Column<R>>() {
            // todo: calculate difference

            DomComPareResult::Different
        } else {
            DomComPareResult::Different
        }
    }

    fn size(&self) -> Size {
        Size {
            width: SizeUnit::Content(1.0),
            height: SizeUnit::Content(1.0),
        }
    }

    fn px_size(&self, _: PxSize, context: &SharedContext) -> PxSize {
        let mut width: f32 = 0.0;
        let mut height_px: f32 = 0.0;
        let mut height_percent: f32 = 0.0;

        for child in &self.children {
            let child_std_size = StdSize::from_size(child.item.size(), context);

            match child_std_size.width {
                StdSizeUnit::None => width = width.max(child.item.default_size().width),
                StdSizeUnit::Pixel(px) => width = width.max(px),
                StdSizeUnit::Percent(_) => (),
            }

            match child_std_size.height {
                StdSizeUnit::None => height_px += child.item.default_size().height,
                StdSizeUnit::Pixel(px) => height_px += px,
                StdSizeUnit::Percent(percent) => height_percent += percent,
            }
        }

        let height = height_px / (1.0 - height_percent);

        self.cache_self_size.set(Some(PxSize { width, height }));
        PxSize { width, height }
    }

    fn default_size(&self) -> PxSize {
        PxSize {
            width: 0.0,
            height: 0.0,
        }
    }

    fn render(
        &mut self,
        // ui environment
        parent_size: PxSize,
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
        let current_size = self.px_size(parent_size, context);

        let mut accumulated_height: f32 = 0.0;

        self.children
            .iter_mut()
            .map(|child| {
                let child_px_size = child.item.px_size(current_size, context);
                let child_affine =
                    na::Matrix4::new_translation(&na::Vector3::new(0.0, -accumulated_height, 0.0));

                accumulated_height += child_px_size.height;

                child
                    .item
                    .render(child_px_size, context, renderer, frame)
                    .into_iter()
                    .map(|(texture, vertices, indices, matrix)| {
                        (texture, vertices, indices, matrix * child_affine)
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }
}
