use std::any::Any;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use matcha_core::{
    device_event::{DeviceEvent, DeviceEventData, ElementState, MouseInput, MouseLogicalButton},
    render_node::RenderNode,
    types::{color::Color, range::CoverRange},
    ui::{
        Background, Constraints, Dom, DomComPareResult, Style, UpdateWidgetError, Widget,
        WidgetContext,
    },
    update_flag::UpdateNotifier,
};

use crate::style::solid_box::SolidBox;

// MARK: DOM

pub struct Button<T> {
    label: Option<String>,
    content: Box<dyn Dom<T>>,
    on_click: Option<Arc<dyn Fn() -> T + Send + Sync>>,
}

impl<T: 'static> Button<T> {
    pub fn new(content: Box<dyn Dom<T>>) -> Box<Self> {
        Box::new(Self {
            label: None,
            content,
            on_click: None,
        })
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn on_click<F>(mut self, f: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.on_click = Some(Arc::new(f));
        self
    }
}

#[async_trait::async_trait]
impl<T: Send + 'static + Clone> Dom<T> for Button<T> {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(ButtonNode {
            label: self.label.clone(),
            content: self.content.build_widget_tree(),
            on_click: self.on_click.clone(),
            state: ButtonState::Normal,
            update_notifier: None,
            size: [0.0, 0.0],
        })
    }

    async fn set_update_notifier(&self, notifier: &UpdateNotifier) {
        self.content.set_update_notifier(notifier).await;
    }
}

// MARK: Widget

#[derive(Clone, Copy, PartialEq, Eq)]
enum ButtonState {
    Normal,
    Hovered,
    Pressed,
}

pub struct ButtonNode<T> {
    label: Option<String>,
    content: Box<dyn Widget<T>>,
    on_click: Option<Arc<dyn Fn() -> T + Send + Sync>>,
    state: ButtonState,
    update_notifier: Option<UpdateNotifier>,
    size: [f32; 2],
}

#[async_trait::async_trait]
impl<T: Send + 'static + Clone> Widget<T> for ButtonNode<T> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    async fn update_widget_tree(
        &mut self,
        component_updated: bool,
        dom: &dyn Dom<T>,
    ) -> Result<(), UpdateWidgetError> {
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Button<T>>() {
            self.label = dom.label.clone();
            self.on_click = dom.on_click.clone();
            self.content
                .update_widget_tree(component_updated, &*dom.content)
                .await?;
            Ok(())
        } else {
            Err(UpdateWidgetError::TypeMismatch)
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Button<T>>() {
            self.content.compare(&*dom.content)
        } else {
            DomComPareResult::Different
        }
    }

    fn device_event(&mut self, event: &DeviceEvent, context: &WidgetContext) -> Option<T> {
        let mut msg = None;
        let mut new_state = self.state;

        let is_inside = self.is_inside(
            match event.event() {
                DeviceEventData::MouseEvent {
                    current_position, ..
                } => *current_position,
                _ => [-1.0, -1.0], // Not a mouse event, so it's outside
            },
            context,
        );

        match event.event() {
            DeviceEventData::MouseEvent {
                event: Some(mouse_event),
                ..
            } => match mouse_event {
                MouseInput::Click {
                    click_state,
                    button,
                } => {
                    if *button == MouseLogicalButton::Primary {
                        if is_inside {
                            if matches!(click_state, ElementState::Pressed(_)) {
                                new_state = ButtonState::Pressed;
                            } else if matches!(click_state, ElementState::Released(_))
                                && self.state == ButtonState::Pressed
                            {
                                new_state = ButtonState::Hovered;
                                if let Some(f) = &self.on_click {
                                    msg = Some(f());
                                }
                            }
                        } else {
                            new_state = ButtonState::Normal;
                        }
                    }
                }
                _ => {
                    // CursorMoved, Wheel, etc.
                    if is_inside {
                        if self.state == ButtonState::Normal {
                            new_state = ButtonState::Hovered;
                        }
                    } else {
                        new_state = ButtonState::Normal;
                    }
                }
            },
            DeviceEventData::MouseEvent { event: None, .. } => {
                // Cursor just moved
                if is_inside {
                    if self.state == ButtonState::Normal {
                        new_state = ButtonState::Hovered;
                    }
                } else {
                    new_state = ButtonState::Normal;
                }
            }
            _ => {}
        }

        if new_state != self.state {
            self.state = new_state;
            if let Some(notifier) = &mut self.update_notifier {
                notifier.notify();
            }
        }

        if msg.is_some() {
            return msg;
        }

        let content_event = event.mouse_transition([0.0, 0.0]); // Pass event to content
        self.content.device_event(&content_event, context)
    }

    fn is_inside(&mut self, position: [f32; 2], _context: &WidgetContext) -> bool {
        position[0] >= 0.0
            && position[0] <= self.size[0]
            && position[1] >= 0.0
            && position[1] <= self.size[1]
    }

    fn preferred_size(&self, constraints: &Constraints, context: &WidgetContext) -> [f32; 2] {
        self.content.preferred_size(constraints, context)
    }

    fn arrange(&mut self, final_size: [f32; 2], context: &WidgetContext) {
        self.size = final_size;
        self.content.arrange(final_size, context);
    }

    fn need_rerendering(&self) -> bool {
        self.content.need_rerendering()
    }

    fn render(&mut self, background: Background, ctx: &WidgetContext) -> RenderNode {
        let bg_color = match self.state {
            ButtonState::Normal => Color::RgbaF32 {
                r: 0.8,
                g: 0.8,
                b: 0.8,
                a: 1.0,
            },
            ButtonState::Hovered => Color::RgbaF32 {
                r: 0.9,
                g: 0.9,
                b: 0.9,
                a: 1.0,
            },
            ButtonState::Pressed => Color::RgbaF32 {
                r: 0.7,
                g: 0.7,
                b: 0.7,
                a: 1.0,
            },
        };

        let texture_size = [self.size[0].ceil() as u32, self.size[1].ceil() as u32];
        if texture_size[0] == 0 || texture_size[1] == 0 {
            return self.content.render(background, ctx);
        }

        let style_region = ctx
            .texture_atlas()
            .allocate_color(ctx.device(), ctx.queue(), texture_size)
            .unwrap();

        let mut encoder = ctx
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Button BG Render Encoder"),
            });

        {
            let mut render_pass = style_region.begin_render_pass(&mut encoder).unwrap();
            let bg_style = SolidBox {
                color: bg_color.to_rgba_f32(),
            };
            bg_style.draw(
                &mut render_pass,
                texture_size,
                style_region.formats()[0],
                self.size,
                [0.0, 0.0],
                ctx,
            );
        }
        ctx.queue().submit(Some(encoder.finish()));

        let mut render_node = RenderNode::new();
        render_node.texture_and_position =
            Some((style_region.clone(), nalgebra::Matrix4::identity()));

        let content_node = self.content.render(background, ctx);
        render_node.add_child(content_node, nalgebra::Matrix4::identity());

        render_node
    }
}
