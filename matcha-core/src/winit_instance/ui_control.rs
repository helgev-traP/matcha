use std::time::Duration;

use thiserror::Error;
use winit::dpi::{PhysicalPosition, PhysicalSize};

use crate::{
    UpdateNotifier,
    component::Component,
    device_event::{
        DeviceEvent, DeviceEventData,
        key_state::KeyboardState,
        mouse_state::{MousePrimaryButton, MouseState},
        window_state::WindowState,
    },
    render_node::RenderNode,
    ui::{Background, Constraints, Widget, WidgetContext},
    update_flag::UpdateFlag,
};

pub struct UiControl<
    Model: Send + Sync + 'static,
    Message: 'static,
    Event: 'static,
    InnerEvent: 'static = Event,
> {
    component: Component<Model, Message, Event, InnerEvent>,
    model_update_flag: UpdateFlag,
    widget: Option<Box<dyn Widget<Event>>>,
    animation_update_flag: UpdateFlag,

    window_state: WindowState,
    mouse_state: MouseState,
    keyboard_state: KeyboardState,

    default_font_size: f32,
}

#[derive(Debug, Error)]
pub enum UiControlError {
    #[error("combo_duration must be less than or equal to long_press_duration")]
    InvalidDuration,
}

impl<Model: Send + Sync + 'static, Message: 'static, Event: 'static, InnerEvent: 'static>
    UiControl<Model, Message, Event, InnerEvent>
{
    pub fn new(
        component: Component<Model, Message, Event, InnerEvent>,
        double_click_duration: Duration,
        long_press_duration: Duration,
        primary_button: MousePrimaryButton,
        scroll_pixel_per_line: f32,
        default_font_size: f32,
    ) -> Result<Self, UiControlError> {
        Ok(Self {
            component,
            model_update_flag: UpdateFlag::new_true(),
            widget: None,
            animation_update_flag: UpdateFlag::new(),
            window_state: WindowState::default(),
            mouse_state: MouseState::new(
                double_click_duration,
                long_press_duration,
                primary_button,
                scroll_pixel_per_line,
            )
            .ok_or(UiControlError::InvalidDuration)?,
            keyboard_state: KeyboardState::new(),
            default_font_size,
        })
    }

    pub fn set_mouse_primary_button(&mut self, button: MousePrimaryButton) {
        self.mouse_state.set_primary_button(button);
    }

    pub fn mouse_primary_button(&self) -> MousePrimaryButton {
        self.mouse_state.primary_button()
    }

    pub fn set_scroll_pixel_per_line(&mut self, pixel: f32) {
        self.mouse_state.set_scroll_pixel_per_line(pixel);
    }

    pub fn scroll_pixel_per_line(&self) -> f32 {
        self.mouse_state.scroll_pixel_per_line()
    }

    pub fn set_default_font_size(&mut self, font_size: f32) {
        self.default_font_size = font_size;
    }

    pub fn default_font_size(&self) -> f32 {
        self.default_font_size
    }
}

impl<Model: Send + Sync + 'static, Message: 'static, Event: 'static, InnerEvent: 'static>
    UiControl<Model, Message, Event, InnerEvent>
{
    /// Returns true if a render should be performed.
    /// Render is required when the model update flag or animation update flag is true,
    /// or when the widget is not yet initialized.
    pub fn needs_render(&self) -> bool {
        self.model_update_flag.is_true()
            || self.animation_update_flag.is_true()
            || self.widget.is_none()
    }

    // if this method is called, it means we already have a current surface texture so we must re-render it to prevent flickering.
    pub async fn render<'a>(
        &mut self,
        size: [f32; 2],
        background: Background<'a>,
        ctx: &WidgetContext<'a>,
    ) -> RenderNode {
        if self.model_update_flag.is_true() || self.widget.is_none() {
            // Dom update is required
            let dom = self.component.view().await;

            self.model_update_flag = UpdateFlag::new();
            dom.set_update_notifier(&self.model_update_flag.notifier())
                .await;

            if let Some(widget) = self.widget.as_mut() {
                if widget.update_widget_tree(false, &*dom).await.is_err() {
                    self.widget = None;
                }
            }
            if self.widget.is_none() {
                // Initialize widget
                self.widget = Some(dom.build_widget_tree());
            }

            // trigger rendering
            self.animation_update_flag = UpdateFlag::new_true();
        }

        let widget = self.widget.as_mut().expect("widget initialized above");

        self.animation_update_flag = UpdateFlag::new();

        Self::render_current_widget(
            &mut **widget,
            size,
            background,
            self.animation_update_flag.notifier(),
            ctx,
        )
    }

    fn render_current_widget(
        widget: &mut dyn Widget<Event>,
        size: [f32; 2],
        background: Background,
        animation_update_flag_notifier: UpdateNotifier,
        ctx: &WidgetContext,
    ) -> RenderNode {
        let constraints = Constraints {
            min_width: 0.0,
            max_width: size[0],
            min_height: 0.0,
            max_height: size[1],
        };
        let preferred_size = widget.preferred_size(&constraints, ctx);
        widget.arrange(
            [
                preferred_size[0].min(size[0]),
                preferred_size[1].min(size[1]),
            ],
            ctx,
        );
        widget.render(background, animation_update_flag_notifier, ctx)
    }

    fn convert_winit_to_window_event(
        &mut self,
        window_event: winit::event::WindowEvent,
        get_window_size: impl Fn() -> (PhysicalSize<u32>, PhysicalSize<u32>),
        get_window_position: impl Fn() -> (PhysicalPosition<i32>, PhysicalPosition<i32>),
    ) -> Option<DeviceEvent> {
        match window_event {
            // we don't handle these events here
            winit::event::WindowEvent::ScaleFactorChanged { .. }
            | winit::event::WindowEvent::Occluded(_)
            | winit::event::WindowEvent::ActivationTokenDone { .. }
            | winit::event::WindowEvent::RedrawRequested
            | winit::event::WindowEvent::Destroyed => None,

            // window interactions
            winit::event::WindowEvent::Resized(_) => {
                let (inner_size, outer_size) = get_window_size();
                Some(DeviceEvent::new(
                    self.window_state
                        .resized(inner_size.into(), outer_size.into()),
                ))
            }
            winit::event::WindowEvent::Moved(_) => {
                let (inner_position, outer_position) = get_window_position();
                Some(DeviceEvent::new(
                    self.window_state
                        .moved(inner_position.into(), outer_position.into()),
                ))
            }
            winit::event::WindowEvent::CloseRequested => {
                Some(DeviceEvent::new(DeviceEventData::CloseRequested))
            }
            winit::event::WindowEvent::Focused(focused) => {
                Some(DeviceEvent::new(DeviceEventData::WindowFocus(focused)))
            }
            winit::event::WindowEvent::ThemeChanged(theme) => {
                Some(DeviceEvent::new(DeviceEventData::Theme(theme)))
            }

            // file drop events
            winit::event::WindowEvent::DroppedFile(path_buf) => {
                let mouse_position = self.mouse_state.position();
                Some(DeviceEvent::new(DeviceEventData::FileDrop {
                    mouse_position,
                    path_buf,
                }))
            }
            winit::event::WindowEvent::HoveredFile(path_buf) => {
                let mouse_position = self.mouse_state.position();
                Some(DeviceEvent::new(DeviceEventData::FileHover {
                    mouse_position,
                    path_buf,
                }))
            }
            winit::event::WindowEvent::HoveredFileCancelled => {
                let mouse_position = self.mouse_state.position();
                Some(DeviceEvent::new(DeviceEventData::FileHoverCancelled {
                    mouse_position,
                }))
            }

            // keyboard events
            winit::event::WindowEvent::KeyboardInput { event, .. } => {
                self.keyboard_state.keyboard_input(event)
            }
            winit::event::WindowEvent::ModifiersChanged(modifiers) => {
                self.keyboard_state.modifiers_changed(modifiers.state());
                None
            }
            winit::event::WindowEvent::Ime(_) => Some(DeviceEvent::new(DeviceEventData::Ime)),

            // mouse events
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                Some(self.mouse_state.cursor_moved(position))
            }
            winit::event::WindowEvent::CursorEntered { .. } => {
                Some(self.mouse_state.cursor_entered())
            }
            winit::event::WindowEvent::CursorLeft { .. } => Some(self.mouse_state.cursor_left()),
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                Some(self.mouse_state.mouse_wheel(delta))
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                self.mouse_state.mouse_input(button, state)
            }

            // touch events
            winit::event::WindowEvent::PinchGesture { .. }
            | winit::event::WindowEvent::PanGesture { .. }
            | winit::event::WindowEvent::DoubleTapGesture { .. }
            | winit::event::WindowEvent::RotationGesture { .. }
            | winit::event::WindowEvent::TouchpadPressure { .. }
            | winit::event::WindowEvent::Touch(..)
            | winit::event::WindowEvent::AxisMotion { .. } => {
                Some(DeviceEvent::new(DeviceEventData::Touch))
            }
        }
    }

    pub fn window_event(
        &mut self,
        viewport_size: [f32; 2],
        window_event: winit::event::WindowEvent,
        get_window_size: impl Fn() -> (PhysicalSize<u32>, PhysicalSize<u32>),
        get_window_position: impl Fn() -> (PhysicalPosition<i32>, PhysicalPosition<i32>),
        ctx: &WidgetContext,
    ) -> Option<Event> {
        let event =
            self.convert_winit_to_window_event(window_event, get_window_size, get_window_position);

        if let (Some(widget), Some(event)) = (&mut self.widget, event) {
            let constraints = Constraints {
                min_width: 0.0,
                max_width: viewport_size[0],
                min_height: 0.0,
                max_height: viewport_size[1],
            };
            let preferred_size = widget.preferred_size(&constraints, ctx);
            widget.arrange(
                [
                    preferred_size[0].min(viewport_size[0]),
                    preferred_size[1].min(viewport_size[1]),
                ],
                ctx,
            );
            widget.device_event(&event, ctx)
        } else {
            None
        }
    }

    pub fn user_event(&self, user_event: &Message) {
        self.component.update(user_event);
    }
}
