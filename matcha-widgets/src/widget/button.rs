use std::any::Any;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use matcha_core::{
    device_input::{DeviceInput, DeviceInputData, ElementState, MouseInput, MouseLogicalButton},
    types::color::Color,
    ui::Background,
    ui::{
        AnyWidgetFrame, Arrangement, Constraints, Dom, Style, Widget, WidgetContext, WidgetFrame,
        widget::{AnyWidget, InvalidationHandle},
    },
    update_flag::UpdateNotifier,
};
use renderer::render_node::RenderNode;

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
impl<T: Send + Sync + 'static> Dom<T> for Button<T> {
    fn build_widget_tree(&self) -> Box<dyn AnyWidgetFrame<T>> {
        Box::new(WidgetFrame::new(
            self.label.clone(),
            vec![(self.content.build_widget_tree(), ())],
            vec![0], // Use a fixed ID for the single child
            ButtonNode {
                on_click: self.on_click.clone(),
                state: ButtonState::Normal,
            },
        ))
    }

    async fn set_update_notifier(&self, notifier: &UpdateNotifier) {
        self.content.set_update_notifier(notifier).await;
    }
}

// MARK: Widget

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ButtonState {
    Normal,
    Hovered,
    Pressed,
}

pub struct ButtonNode<T> {
    on_click: Option<Arc<dyn Fn() -> T + Send + Sync>>,
    state: ButtonState,
}

impl<T: Send + Sync + 'static> Widget<Button<T>, T, ()> for ButtonNode<T> {
    fn update_widget<'a>(
        &mut self,
        dom: &'a Button<T>,
        _cache_invalidator: Option<InvalidationHandle>,
    ) -> Vec<(&'a dyn Dom<T>, (), u128)> {
        self.on_click = dom.on_click.clone();
        vec![(&*dom.content, (), 0)]
    }

    fn measure(
        &self,
        constraints: &Constraints,
        children: &[(&dyn AnyWidget<T>, &())],
        ctx: &WidgetContext,
    ) -> [f32; 2] {
        if let Some((content, _)) = children.first() {
            content.measure(constraints, ctx)
        } else {
            [0.0, 0.0]
        }
    }

    fn arrange(
        &self,
        final_size: [f32; 2],
        _children: &[(&dyn AnyWidget<T>, &())],
        _ctx: &WidgetContext,
    ) -> Vec<Arrangement> {
        vec![Arrangement::new(final_size, nalgebra::Matrix4::identity())]
    }

    fn device_input(
        &mut self,
        bounds: [f32; 2],
        event: &DeviceInput,
        children: &mut [(&mut dyn AnyWidget<T>, &mut (), &Arrangement)],
        cache_invalidator: InvalidationHandle,
        ctx: &WidgetContext,
    ) -> Option<T> {
        let mut msg = None;
        let mut new_state = self.state;

        let position = event.mouse_position().unwrap_or([-1.0, -1.0]);

        let is_inside = position[0] >= 0.0
            && position[0] <= bounds[0]
            && position[1] >= 0.0
            && position[1] <= bounds[1];

        match event.event() {
            DeviceInputData::MouseInput {
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
            DeviceInputData::MouseInput { event: None, .. } => {
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
            cache_invalidator.redraw_next_frame();
        }

        if msg.is_some() {
            return msg;
        }

        if let Some((content, _, arrangement)) = children.first_mut() {
            let content_event = event.transform(arrangement.affine);
            return content.device_event(&content_event, ctx);
        }

        None
    }

    fn render(
        &self,
        background: Background,
        children: &[(&dyn AnyWidget<T>, &(), &Arrangement)],
        ctx: &WidgetContext,
    ) -> RenderNode {
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

        let mut render_node = RenderNode::new();

        if let Some((content, _, arrangement)) = children.first() {
            let texture_size = [
                arrangement.size[0].ceil() as u32,
                arrangement.size[1].ceil() as u32,
            ];
            if texture_size[0] > 0 && texture_size[1] > 0 {
                // This is inefficient and should be replaced with a direct color rendering in the renderer.
                // For now, we replicate the old behavior of drawing to a texture atlas.
                if let Ok(style_region) =
                    ctx.texture_atlas()
                        .allocate_color(ctx.device(), ctx.queue(), texture_size)
                {
                    let mut encoder =
                        ctx.device()
                            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: Some("Button BG Render Encoder"),
                            });

                    let bg_style = SolidBox {
                        color: bg_color.to_rgba_f32(),
                    };
                    bg_style.draw(
                        &mut encoder,
                        &style_region,
                        arrangement.size,
                        [0.0, 0.0],
                        ctx,
                    );

                    ctx.queue().submit(Some(encoder.finish()));
                    render_node = render_node.with_texture(
                        style_region,
                        arrangement.size,
                        arrangement.affine,
                    );
                }
            }

            let content_node = content.render(arrangement.size, background, ctx);
            render_node.push_child(content_node, arrangement.affine);
        }

        render_node
    }
}
