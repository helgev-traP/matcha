use nalgebra as na;
use std::{cell::Cell, sync::Arc};

use crate::{
    context::SharedContext,
    events::{UiEvent, UiEventResult},
    renderer::Renderer,
    types::size::{PxSize, Size, SizeUnit, StdSize},
    ui::{Dom, DomComPareResult, Widget},
    vertex::uv_vertex::UvVertex,
};

pub struct RowDescriptor<R> {
    pub label: Option<String>,
    pub vec: Vec<Box<dyn Dom<R>>>,
}

impl<R> Default for RowDescriptor<R> {
    fn default() -> Self {
        Self {
            label: None,
            vec: Vec::new(),
        }
    }
}

pub struct Row<R: 'static> {
    label: Option<String>,
    children: Vec<Box<dyn Dom<R>>>,
}

impl<R> Row<R> {
    pub fn new(disc: RowDescriptor<R>) -> Box<Self> {
        Box::new(Self {
            label: disc.label,
            children: disc.vec,
        })
    }

    pub fn push(&mut self, child: Box<dyn Dom<R>>) {
        self.children.push(child);
    }
}

impl<R: Send + 'static> Dom<R> for Row<R> {
    fn build_widget_tree(&self) -> Box<dyn Widget<R>> {
        Box::new(RowRenderNode {
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
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct RowRenderNode<T: 'static> {
    label: Option<String>,
    redraw: bool,
    children: Vec<Child<T>>,
    cache_self_size: Cell<Option<PxSize>>,
}

struct Child<T> {
    item: Box<dyn Widget<T>>,
    // cache
    position: Option<[f32; 2]>,
    size: Option<PxSize>,
}

impl<T> Widget<T> for RowRenderNode<T> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: PxSize,
        context: &SharedContext,
    ) -> crate::events::UiEventResult<T> {
        match &event.content {
            // mouse event with position
            crate::events::UiEventContent::MouseClick { position, .. }
            | crate::events::UiEventContent::CursorMove { position, .. }
            | crate::events::UiEventContent::MouseScroll { position, .. } => {
                let current_size = self.px_size(parent_size, context);

                let mut accumulated_width: f32 = 0.0;

                let mut where_event_will_be_sent = None;

                for child in &mut self.children {
                    let child_position = child.position.get_or_insert([accumulated_width, 0.0]);

                    let child_px_size = child
                        .size
                        .get_or_insert_with(|| child.item.px_size(current_size, context));

                    accumulated_width += child_px_size.width;

                    // event handling
                    if child.item.is_inside(
                        [
                            position[0] - child_position[0],
                            position[1] - child_position[1],
                        ],
                        current_size,
                        context,
                    ) {
                        where_event_will_be_sent = Some(child);
                    }
                }

                if let Some(child) = where_event_will_be_sent {
                    // updata position
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
            // crate::events::UiEventContent::CursorEntered => todo!(),
            // crate::events::UiEventContent::CursorLeft => todo!(),
            // todo
            _ => crate::events::UiEventResult::default(),
        }
    }

    fn is_inside(&self, position: [f32; 2], parent_size: PxSize, context: &SharedContext) -> bool {
        if let Some(size) = self.cache_self_size.get() {
            position[0] < 0.0
                || position[0] > size.width
                || position[1] < 0.0
                || position[1] > size.height
        } else {
            let current_size = self.px_size(parent_size, context);

            let mut accumulated_width: f32 = 0.0;
            let mut max_height: f32 = 0.0;

            for child in &self.children {
                let child_px_size = child.item.px_size(current_size, context);

                accumulated_width += child_px_size.width;
                max_height = max_height.max(child_px_size.height);
            }

            self.cache_self_size.set(Some(PxSize {
                width: accumulated_width,
                height: max_height,
            }));

            position[0] < 0.0
                || position[0] > accumulated_width
                || position[1] < 0.0
                || position[1] > max_height
        }
    }

    fn update_widget_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<Row<T>>() {
            Err(())
        } else {
            let dom = dom.as_any().downcast_ref::<Row<T>>().unwrap();
            // todo: differential update
            self.children.clear();
            self.children.extend(dom.children.iter().map(|child| Child {
                item: child.build_widget_tree(),
                position: None,
                size: None,
            }));
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

    fn size(&self) -> crate::types::size::Size {
        Size {
            width: SizeUnit::Content(1.0),
            height: SizeUnit::Content(1.0),
        }
    }

    fn px_size(&self, _: PxSize, context: &SharedContext) -> PxSize {
        let mut width_px: f32 = 0.0;
        let mut width_percent: f32 = 0.0;
        let mut height: f32 = 0.0;

        for child in &self.children {
            let child_std_size = StdSize::from_size(child.item.size(), context);

            match child_std_size.width {
                crate::types::size::StdSizeUnit::None => {
                    width_px += child.item.default_size().width
                }
                crate::types::size::StdSizeUnit::Pixel(px) => width_px += px,
                crate::types::size::StdSizeUnit::Percent(percent) => width_percent += percent,
            }

            match child_std_size.height {
                crate::types::size::StdSizeUnit::None => {
                    height = height.max(child.item.default_size().height)
                }
                crate::types::size::StdSizeUnit::Pixel(px) => height = height.max(px),
                crate::types::size::StdSizeUnit::Percent(_) => (),
            }
        }

        let width = width_px / (1.0 - width_percent);

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

        let mut accumulated_width: f32 = 0.0;

        self.children
            .iter_mut()
            .map(|child| {
                let child_px_size = child.item.px_size(current_size, context);
                let child_affine =
                    na::Matrix4::new_translation(&na::Vector3::new(accumulated_width, 0.0, 0.0));

                accumulated_width += child_px_size.width;

                child
                    .item
                    .render(current_size, context, renderer, frame)
                    .into_iter()
                    .map(|(texture, vertices, indices, affine)| {
                        (texture, vertices, indices, child_affine * affine)
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }
}
