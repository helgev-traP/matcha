use core::panic;
use std::sync::Arc;

use gpu_utils::gpu::Gpu;
use parking_lot::RwLock;
use renderer::RenderNode;
use utils::{back_prop_dirty::BackPropDirty, update_flag::UpdateFlag};
use winit::dpi::{PhysicalPosition, PhysicalSize};

use crate::{
    context::GlobalResources,
    device_input::{
        DeviceInput, DeviceInputData, KeyboardState, MouseState,
        mouse_state::{MousePrimaryButton, MouseStateConfig},
        window_state::WindowState,
    },
    metrics::Constraints,
    ui::{AnyWidgetFrame, Background, component::AnyComponent},
    window_surface::WindowSurface,
};

pub struct WindowUi<Message: 'static, Event: 'static> {
    // window
    window: Arc<RwLock<WindowSurface>>,

    // ui
    component: Box<dyn AnyComponent<Message, Event>>,
    widget: Option<Box<dyn AnyWidgetFrame<Event>>>,
    model_update_detecter: UpdateFlag,

    // input handling
    window_state: WindowState,
    mouse_state_config: MouseStateConfig,
    mouse_state: MouseState,
    keyboard_state: KeyboardState,
}

pub struct RenderResult {
    pub render_node: Arc<RenderNode>,
    pub viewport_size: [f32; 2],
    pub surface_texture: wgpu::SurfaceTexture,
    pub surface_format: wgpu::TextureFormat,
}

#[derive(Debug, thiserror::Error)]
pub enum WindowUiError {
    #[error("combo_duration must be less than or equal to long_press_duration")]
    InvalidDuration,
}

pub enum WindowUiRenderError {
    WindowNotStarted,
    WgpuSurfaceError(wgpu::SurfaceError),
}

impl<Message: 'static, Event: 'static> WindowUi<Message, Event> {
    pub fn new(
        component: Box<dyn AnyComponent<Message, Event>>,
        mouse_state_config: MouseStateConfig,
    ) -> Result<Self, WindowUiError> {
        Ok(Self {
            window: Arc::new(RwLock::new(WindowSurface::new())),
            component,
            model_update_detecter: UpdateFlag::new(),
            widget: None,
            window_state: WindowState::default(),
            mouse_state_config,
            mouse_state: mouse_state_config
                .init()
                .ok_or(WindowUiError::InvalidDuration)?,
            keyboard_state: KeyboardState::new(),
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
}

impl<Message: 'static, Event: 'static> WindowUi<Message, Event> {
    pub fn resize_window(&self, new_size: PhysicalSize<u32>, device: &wgpu::Device) {
        self.window.write().set_surface_size(new_size, device);
    }

    pub fn request_redraw(&self) {
        self.window.read().request_redraw();
    }
}

impl<Message: 'static, Event: 'static> WindowUi<Message, Event> {
    pub async fn start_window(
        &self,
        winit_event_loop: &winit::event_loop::ActiveEventLoop,
        gpu: &Gpu,
    ) -> Result<(), crate::window_surface::WindowSurfaceError> {
        self.window.write().start_window(winit_event_loop, gpu)
    }

    // start component setup function
    // TODO: This is provisional implementation. Refactor this after organizing async execution flow.
    pub async fn setup(&self, tokio_handle: &tokio::runtime::Handle, resource: &GlobalResources) {
        self.component
            .setup(&resource.application_context(tokio_handle, &self.window))
    }

    /// Returns true if a render should be performed.
    /// Render is required when the model update flag or animation update flag is true,
    /// or when the widget is not yet initialized.
    pub fn needs_render(&self) -> bool {
        self.model_update_detecter.is_true() || self.widget.as_ref().is_none_or(|w| w.need_redraw())
    }

    pub async fn render<'a>(
        &'a mut self,
        tokio_handle: &tokio::runtime::Handle,
        winit_event_loop: &winit::event_loop::ActiveEventLoop,
        resource: &GlobalResources,
        benchmark: &mut utils::benchmark::Benchmark,
    ) -> Option<RenderResult> {
        let viewport_size;
        let surface_texture;
        let surface_format;
        let background;
        {
            let mut window = self.window.upgradable_read();

            // check window existence
            if window.window().is_none() {
                window.with_upgraded(|window| {
                    // reset widget and states
                    self.widget = None;
                    self.model_update_detecter = UpdateFlag::new();
                    self.mouse_state = self.mouse_state_config.init().expect(
                        "already checked mouse state config is valid when WindowUi is created or updated so this should not fail"
                    );
                    self.window_state = WindowState::default();

                    // start window
                    window.start_window(
                        winit_event_loop,
                        &resource.gpu(),
                    ).expect("failed to start window");
                })
            }

            let viewport_size_physical = window.inner_size().expect("we checked window existence");
            viewport_size = [
                viewport_size_physical.width as f32,
                viewport_size_physical.height as f32,
            ];

            surface_texture = match window
                .current_texture()
                .expect("we checked window existence")
            {
                Ok(texture) => texture,
                Err(e) => {
                    match e {
                        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated => {
                            // reconfigure the surface
                            window.with_upgraded(|w| {
                                w.reconfigure_surface(&resource.gpu().device());
                            });

                            // call rerender event
                            window.request_redraw();

                            return None;
                        }
                        wgpu::SurfaceError::Timeout => {
                            // skip this frame
                            return None;
                        }
                        wgpu::SurfaceError::OutOfMemory => {
                            panic!("out of memory");
                        }
                        wgpu::SurfaceError::Other => {
                            panic!("unknown error at wgpu surface");
                        }
                    }
                }
            };

            surface_format = window.format().expect("we checked window existence");

            // placeholder background
            // TODO: support background image
            background = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        }

        Some(
            self.render_inner(
                viewport_size,
                surface_texture,
                surface_format,
                background,
                tokio_handle,
                resource,
                benchmark,
            )
            .await,
        )
    }

    // if this method is called, it means we already have a current surface texture so we must re-render it to prevent flickering.
    async fn render_inner<'a>(
        &'a mut self,
        viewport_size: [f32; 2],
        surface_texture: wgpu::SurfaceTexture,
        surface_format: wgpu::TextureFormat,
        background: Background<'a>,
        tokio_handle: &tokio::runtime::Handle,
        resource: &GlobalResources,
        benchmark: &mut utils::benchmark::Benchmark,
    ) -> RenderResult {
        let ctx = resource.widget_context(tokio_handle, &self.window);

        if self.widget.is_none() {
            // directly build widget tree from dom
            let dom = benchmark
                .with_async("create_dom", self.component.view())
                .await;
            let widget = self
                .widget
                .insert(benchmark.with("create_widget", || dom.build_widget_tree()));

            // set model update notifier
            self.model_update_detecter = UpdateFlag::new();
            widget
                .set_model_update_notifier(&self.model_update_detecter.notifier())
                .await;
            // set dirty flags
            widget.update_dirty_flags(BackPropDirty::new(true), BackPropDirty::new(true));
        } else if self.model_update_detecter.is_true() {
            // Widget update is required
            let dom = benchmark
                .with_async("create_dom", self.component.view())
                .await;

            if let Some(widget) = self.widget.as_mut()
                && benchmark
                    .with_async("update_widget", widget.update_widget_tree(&*dom))
                    .await
                    .is_err()
            {
                self.widget = None;
            }

            let widget = self.widget.get_or_insert_with(|| dom.build_widget_tree());

            // set model update notifier
            self.model_update_detecter = UpdateFlag::new();
            widget
                .set_model_update_notifier(&self.model_update_detecter.notifier())
                .await;
            // set dirty flags
            widget.update_dirty_flags(BackPropDirty::new(true), BackPropDirty::new(true));
        }

        let widget = self.widget.as_mut().expect("widget initialized above");

        let constraints: Constraints =
            Constraints::new([0.0, viewport_size[0]], [0.0, viewport_size[1]]);

        let preferred_size =
            benchmark.with("layout_measure", || widget.measure(&constraints, &ctx));
        let final_size = [
            preferred_size[0].clamp(0.0, viewport_size[0]),
            preferred_size[1].clamp(0.0, viewport_size[1]),
        ];

        benchmark.with("layout_arrange", || widget.arrange(final_size, &ctx));
        let render_node = benchmark.with("widget_render", || widget.render(background, &ctx));

        RenderResult {
            render_node,
            viewport_size,
            surface_texture,
            surface_format,
        }
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
        tokio_handle: &tokio::runtime::Handle,
        resource: &GlobalResources,
    ) -> Option<Event> {
        // check window existence
        if self.window.read().window().is_none() {
            return None;
        }

        let ctx = resource.widget_context(tokio_handle, &self.window);

        let window_clone = self.window.clone();
        let get_window_size = || {
            let window = window_clone.read();
            (
                window.inner_size().expect("we checked window existence"),
                window.outer_size().expect("we checked window existence"),
            )
        };
        let window_clone = self.window.clone();
        let get_window_position = || {
            let window = window_clone.read();
            (
                window
                    .inner_position()
                    .expect("we checked window existence")
                    .expect("window should be there and when Android / Wayland window moving event should not be called"),
                window
                    .outer_position()
                    .expect("we checked window existence")
                    .expect("window should be there and when Android / Wayland window moving event should not be called"),
            )
        };

        let event =
            self.convert_winit_to_window_event(window_event, get_window_size, get_window_position);

        if let (Some(widget), Some(event)) = (&mut self.widget, event) {
            widget.device_input(&event, &ctx)
        } else {
            None
        }
    }

    pub fn user_event(
        &self,
        user_event: &Message,
        tokio_runtime: &tokio::runtime::Handle,
        resource: &GlobalResources,
    ) {
        let widget_ctx = &resource.widget_context(tokio_runtime, &self.window);

        let app_ctx = widget_ctx.application_context();

        self.component.update(user_event, &app_ctx);
    }
}
