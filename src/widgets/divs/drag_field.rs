use std::sync::Arc;

use crate::{
    context::SharedContext,
    device::mouse::MouseButton,
    events::UiEvent,
    types::size::{self, PxSize, Size},
    ui::{Dom, DomComPareResult, Widget},
};

use nalgebra as na;

pub struct DragFieldDescriptor<R> {
    pub label: Option<String>,
    pub size: Size,

    // item
    pub item: Box<dyn Dom<R>>,
}

pub struct DragField<T> {
    label: Option<String>,
    size: Size,

    item: Box<dyn Dom<T>>,
}

impl<T> DragField<T> {
    pub fn new(disc: DragFieldDescriptor<T>) -> Box<Self> {
        Box::new(Self {
            label: disc.label,
            size: disc.size,
            item: disc.item,
        })
    }
}

impl<T: Send + 'static> Dom<T> for DragField<T> {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(DragFieldNode {
            label: self.label.clone(),
            size: self.size,

            item_position: [0.0, 0.0],
            drag_delta: None,
            item: self.item.build_widget_tree(),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct DragFieldNode<T> {
    label: Option<String>,
    size: Size,

    item_position: [f32; 2],
    drag_delta: Option<[f32; 2]>,
    item: Box<dyn Widget<T>>,
}

impl<T: Send + 'static> Widget<T> for DragFieldNode<T> {
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
            crate::events::UiEventContent::MouseClick {
                position,
                click_state,
                button,
            } => {
                if let MouseButton::Primary = button {
                    match &click_state {
                        crate::events::ElementState::Pressed(_) => {
                            // todo: check if the cursor is inside the item
                            if self.item.is_inside(
                                [
                                    position[0] - self.item_position[0],
                                    position[1] - self.item_position[1],
                                ],
                                parent_size,
                                context,
                            ) {
                                self.drag_delta = Some([0.0, 0.0]);
                            }
                        }
                        crate::events::ElementState::Released(_) => {
                            if let Some(drag_delta) = self.drag_delta {
                                // drag release
                                self.item_position[0] += drag_delta[0];
                                self.item_position[1] += drag_delta[1];
                                self.drag_delta = None;
                            } else {
                                // click event
                                let size = self.px_size(parent_size, context);

                                return self.item.widget_event(
                                    &UiEvent {
                                        frame: event.frame,
                                        content: crate::events::UiEventContent::MouseClick {
                                            position: [
                                                position[0] - self.item_position[0],
                                                position[1] - self.item_position[1],
                                            ],
                                            click_state: click_state.clone(),
                                            button: button.clone(),
                                        },
                                        diff: event.diff,
                                    },
                                    size,
                                    context,
                                );
                            }
                        }
                        _ => (),
                    }
                }
            }
            crate::events::UiEventContent::CursorMove {
                position,
                primary_dragging_from,
                ..
            } => {
                if let Some(drag_from) = primary_dragging_from {
                    self.drag_delta =
                        Some([position[0] - drag_from[0], position[1] - drag_from[1]]);
                }
            }
            _ => (),
        }

        crate::events::UiEventResult::default()
    }

    fn is_inside(&self, position: [f32; 2], parent_size: PxSize, context: &SharedContext) -> bool {
        let current_size = self.size.unwrap_to_px(parent_size, context);

        if position[0] < 0.0
            || position[0] > current_size.width
            || position[1] < 0.0
            || position[1] > current_size.height
        {
            false
        } else {
            true
        }
    }

    fn update_widget_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<DragField<T>>() {
            Err(())
        } else {
            println!("update_widget_tree");
            let dom = dom.as_any().downcast_ref::<DragField<T>>().unwrap();
            todo!()
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(_) = dom.as_any().downcast_ref::<DragField<T>>() {
            todo!()
        } else {
            DomComPareResult::Different
        }
    }

    fn size(&self) -> Size {
        self.size
    }

    fn px_size(&self, parent_size: PxSize, context: &SharedContext) -> PxSize {
        self.size.unwrap_to_px(parent_size, context)
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
        renderer: &crate::renderer::Renderer,
        frame: u64,
    ) -> Vec<(
        Arc<wgpu::Texture>,
        Arc<Vec<crate::vertex::uv_vertex::UvVertex>>,
        Arc<Vec<u16>>,
        nalgebra::Matrix4<f32>,
    )> {
        let current_size = self.size.unwrap_to_px(parent_size, context);

        let item_position_matrix = if let Some(drag_delta) = self.drag_delta {
            na::Matrix4::new_translation(&na::Vector3::new(
                self.item_position[0] + drag_delta[0],
                -self.item_position[1] - drag_delta[1],
                0.0,
            ))
        } else {
            na::Matrix4::new_translation(&na::Vector3::new(
                self.item_position[0],
                -self.item_position[1],
                0.0,
            ))
        };

        let item = self.item.render(current_size, context, renderer, frame);

        item.into_iter()
            .map(|(texture, vertices, indices, matrix)| {
                (texture, vertices, indices, item_position_matrix * matrix)
            })
            .collect()
    }
}
