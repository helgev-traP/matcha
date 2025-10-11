use std::{sync::Arc, time::Duration};

use thiserror::Error;
use utils::back_prop_dirty::BackPropDirty;
use winit::dpi::{PhysicalPosition, PhysicalSize};

use crate::{
    device_input::{
        DeviceInput, DeviceInputData,
        key_state::KeyboardState,
        mouse_state::{MousePrimaryButton, MouseState},
        window_state::WindowState,
    },
    metrics::Constraints,
    ui::{AnyWidgetFrame, ApplicationContext, Background, WidgetContext, component::Component},
    update_flag::UpdateFlag,
};
use renderer::render_node::RenderNode;

pub struct UiControl<
    Model: Send + Sync + 'static,
    Message: 'static,
    Event: 'static,
    InnerEvent: 'static = Event,
> {
    component: Component<Model, Message, Event, InnerEvent>,
    model_update_flag: UpdateFlag,
    widget: Option<Box<dyn AnyWidgetFrame<Event>>>,

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
    // start component setup function
    // TODO: This is provisional implementation. Refactor this after organizing async execution flow.
    pub async fn setup(&self, app_handler: &ApplicationContext) {
        self.component.setup(app_handler);
    }

    /// Returns true if a render should be performed.
    /// Render is required when the model update flag or animation update flag is true,
    /// or when the widget is not yet initialized.
    pub fn needs_render(&self) -> bool {
        self.model_update_flag.is_true() || self.widget.as_ref().is_none_or(|w| w.need_redraw())
    }

    // if this method is called, it means we already have a current surface texture so we must re-render it to prevent flickering.
    pub async fn render<'a>(
        &'a mut self,
        viewport_size: [f32; 2],
        background: Background<'a>,
        ctx: &WidgetContext<'a>,
        benchmark: &mut super::benchmark::Benchmark,
    ) -> Arc<RenderNode> {
        if self.widget.is_none() {
            // directly build widget tree from dom
            let dom = benchmark.with_create_dom(self.component.view()).await;
            let widget = self
                .widget
                .insert(benchmark.with_create_widget(|| dom.build_widget_tree()));

            // set model update notifier
            self.model_update_flag = UpdateFlag::new();
            widget
                .set_model_update_notifier(&self.model_update_flag.notifier())
                .await;
            // set dirty flags
            widget.update_dirty_flags(BackPropDirty::new(true), BackPropDirty::new(true));
        } else if self.model_update_flag.is_true() {
            // Widget update is required
            let dom = benchmark.with_create_dom(self.component.view()).await;

            if let Some(widget) = self.widget.as_mut()
                && benchmark
                    .with_update_widget(widget.update_widget_tree(&*dom))
                    .await
                    .is_err()
            {
                self.widget = None;
            }

            let widget = self.widget.get_or_insert_with(|| dom.build_widget_tree());

            // set model update notifier
            self.model_update_flag = UpdateFlag::new();
            widget
                .set_model_update_notifier(&self.model_update_flag.notifier())
                .await;
            // set dirty flags
            widget.update_dirty_flags(BackPropDirty::new(true), BackPropDirty::new(true));
        }

        let widget = self.widget.as_mut().expect("widget initialized above");

        let constraints: Constraints =
            Constraints::new([0.0, viewport_size[0]], [0.0, viewport_size[1]]);

        let preferred_size = benchmark.with_layout_measure(|| widget.measure(&constraints, ctx));
        let final_size = [
            preferred_size[0].clamp(0.0, viewport_size[0]),
            preferred_size[1].clamp(0.0, viewport_size[1]),
        ];

        benchmark.with_layout_arrange(|| widget.arrange(final_size, ctx));
        benchmark.with_widget_render(|| widget.render(background, ctx))
    }

    fn convert_winit_to_window_event(
        &mut self,
        window_event: winit::event::WindowEvent,
        get_window_size: impl Fn() -> (PhysicalSize<u32>, PhysicalSize<u32>),
        get_window_position: impl Fn() -> (PhysicalPosition<i32>, PhysicalPosition<i32>),
    ) -> Option<DeviceInput> {
        let device_input_data = match &window_event {
            // we don't handle these events here
            winit::event::WindowEvent::ScaleFactorChanged { .. }
            | winit::event::WindowEvent::Occluded(_)
            | winit::event::WindowEvent::ActivationTokenDone { .. }
            | winit::event::WindowEvent::RedrawRequested
            | winit::event::WindowEvent::Destroyed => None,

            // window interactions
            winit::event::WindowEvent::Resized(_) => {
                let (inner_size, outer_size) = get_window_size();
                Some(
                    self.window_state
                        .resized(inner_size.into(), outer_size.into()),
                )
            }
            winit::event::WindowEvent::Moved(_) => {
                let (inner_position, outer_position) = get_window_position();
                Some(
                    self.window_state
                        .moved(inner_position.into(), outer_position.into()),
                )
            }
            winit::event::WindowEvent::CloseRequested => Some(DeviceInputData::CloseRequested),
            winit::event::WindowEvent::Focused(focused) => {
                Some(DeviceInputData::WindowFocus(*focused))
            }
            winit::event::WindowEvent::ThemeChanged(theme) => Some(DeviceInputData::Theme(*theme)),

            // file drop events
            winit::event::WindowEvent::DroppedFile(path_buf) => Some(DeviceInputData::FileDrop {
                path_buf: path_buf.clone(),
            }),
            winit::event::WindowEvent::HoveredFile(path_buf) => Some(DeviceInputData::FileHover {
                path_buf: path_buf.clone(),
            }),
            winit::event::WindowEvent::HoveredFileCancelled => {
                Some(DeviceInputData::FileHoverCancelled)
            }

            // keyboard events
            winit::event::WindowEvent::KeyboardInput { event, .. } => {
                self.keyboard_state.keyboard_input(event.clone())
            }
            winit::event::WindowEvent::ModifiersChanged(modifiers) => {
                self.keyboard_state.modifiers_changed(modifiers.state());
                None
            }
            winit::event::WindowEvent::Ime(_) => Some(DeviceInputData::Ime),

            // mouse events
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                Some(self.mouse_state.cursor_moved(*position))
            }
            winit::event::WindowEvent::CursorEntered { .. } => {
                Some(self.mouse_state.cursor_entered())
            }
            winit::event::WindowEvent::CursorLeft { .. } => Some(self.mouse_state.cursor_left()),
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                Some(self.mouse_state.mouse_wheel(*delta))
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                self.mouse_state.mouse_input(*button, *state)
            }

            // touch events
            winit::event::WindowEvent::PinchGesture { .. }
            | winit::event::WindowEvent::PanGesture { .. }
            | winit::event::WindowEvent::DoubleTapGesture { .. }
            | winit::event::WindowEvent::RotationGesture { .. }
            | winit::event::WindowEvent::TouchpadPressure { .. }
            | winit::event::WindowEvent::Touch(..)
            | winit::event::WindowEvent::AxisMotion { .. } => Some(DeviceInputData::Touch),
        };

        device_input_data.map(|device_input_data| {
            let mouse_position = self.mouse_state.position();
            DeviceInput::new(mouse_position, device_input_data, window_event)
        })
    }

    pub fn window_event(
        &mut self,
        window_event: winit::event::WindowEvent,
        get_window_size: impl Fn() -> (PhysicalSize<u32>, PhysicalSize<u32>),
        get_window_position: impl Fn() -> (PhysicalPosition<i32>, PhysicalPosition<i32>),
        ctx: &WidgetContext,
        app_handler: &ApplicationContext,
    ) -> Option<Event> {
        let event =
            self.convert_winit_to_window_event(window_event, get_window_size, get_window_position);

        if let (Some(widget), Some(event)) = (&mut self.widget, event) {
            widget.device_input(&event, ctx, app_handler)
        } else {
            None
        }
    }

    pub fn user_event(&self, user_event: &Message, app_handler: &ApplicationContext) {
        self.component.update(user_event, app_handler);
    }
}
