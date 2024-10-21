use crate::{
    application_context::ApplicationContext,
    device::mouse::MouseButton,
    events::UiEvent,
    renderer::RendererCommandEncoder,
    types::size::{PxSize, Size, SizeUnit},
    ui::{Dom, Widget},
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
    pub fn new(disc: DragFieldDescriptor<T>) -> Self {
        Self {
            label: disc.label,
            size: disc.size,
            item: disc.item,
        }
    }
}

impl<T: Send + 'static> Dom<T> for DragField<T> {
    fn build_render_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(DragFieldNode {
            label: self.label.clone(),
            size: self.size,

            item_position: [0.0, 0.0],
            drag_delta: None,
            item: self.item.build_render_tree(),
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

impl<T: Send + 'static> super::WidgetTrait<T> for DragFieldNode<T> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: PxSize,
        context: &ApplicationContext,
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
                                [position[0] - self.item_position[0], position[1] - self.item_position[1]],
                                parent_size,
                                context
                            ) {
                                self.drag_delta = Some([0.0, 0.0]);
                            }
                        }
                        crate::events::ElementState::Released(_) => {
                            if let Some(drag_delta) = self.drag_delta {
                                self.item_position[0] += drag_delta[0];
                                self.item_position[1] += drag_delta[1];
                                self.drag_delta = None;
                            }
                        }
                        _ => (),
                    }
                }
            }
            crate::events::UiEventContent::CursorMove {
                position,
                primary_dragging_from,
                secondary_dragging_from,
                middle_dragging_from,
            } => {
                if let Some(drag_from) = primary_dragging_from {
                    if let Some(drag_delta) = self.drag_delta {
                        self.drag_delta =
                            Some([position[0] - drag_from[0], position[1] - drag_from[1]]);
                    }
                }
            }
            _ => (),
        }

        crate::events::UiEventResult::default()
    }

    fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> bool {
        let current_size = self.size.to_px(parent_size, context);

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

    fn update_render_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<DragField<T>>() {
            Err(())
        } else {
            let dom = dom.as_any().downcast_ref::<DragField<T>>().unwrap();
            todo!()
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> super::DomComPareResult {
        if let Some(_) = dom.as_any().downcast_ref::<DragField<T>>() {
            todo!()
        } else {
            super::DomComPareResult::Different
        }
    }
}

impl<T> super::RenderingTrait for DragFieldNode<T> {
    fn size(&self) -> Size {
        self.size
    }

    fn px_size(&self, parent_size: PxSize, context: &ApplicationContext) -> PxSize {
        self.size.to_px(parent_size, context)
    }

    fn default_size(&self) -> PxSize {
        PxSize {
            width: 0.0,
            height: 0.0,
        }
    }

    fn render(
        &mut self,
        s: &rayon::Scope,
        parent_size: PxSize,
        affine: nalgebra::Matrix4<f32>,
        encoder: &mut RendererCommandEncoder,
    ) {
        let current_size = self.size.to_px(parent_size, encoder.get_context());

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

        self.item
            .render(s, current_size, item_position_matrix * affine, encoder);
    }
}
