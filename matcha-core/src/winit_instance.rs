use std::fmt::Debug;
use thiserror::Error;

use crate::{any_resource::AnyResource, backend::Backend, debug_config::SharedDebugConfig, ui};

// MARK: modules

mod benchmark;
mod builder;
mod render_control;
mod ticker;
mod ui_control;
mod window_surface;

pub(crate) use builder::WinitInstanceBuilder;

// MARK: Winit

pub struct WinitInstance<
    Model: Send + Sync + 'static,
    Message: 'static,
    B: Backend<Event> + Clone + 'static,
    Event: Send + 'static,
    InnerEvent: 'static = Event,
> {
    // --- tokio runtime ---
    tokio_runtime: tokio::runtime::Runtime,
    // --- window ---
    window: window_surface::WindowSurface,
    surface_preferred_format: wgpu::TextureFormat,
    // --- rendering context ---
    any_resource: AnyResource,
    debug_config: SharedDebugConfig,
    // --- render control ---
    render_control: render_control::RenderControl,
    // --- UI control ---
    ui_control: ui_control::UiControl<Model, Message, Event, InnerEvent>,
    app_handler: ui::ApplicationHandler,
    // --- backend ---
    backend: B,
    // --- ticker ---
    ticker: ticker::Ticker,
    // --- benchmark / monitoring ---
    benchmarker: benchmark::Benchmark,
    frame: u128,
}

impl<
    Model: Send + Sync + 'static,
    Message: 'static,
    B: Backend<Event> + Clone + 'static,
    Event: Send + 'static,
    InnerEvent: 'static,
> WinitInstance<Model, Message, B, Event, InnerEvent>
{
    pub fn builder(
        component: ui::Component<Model, Message, Event, InnerEvent>,
        backend: B,
    ) -> WinitInstanceBuilder<Model, Message, B, Event, InnerEvent> {
        WinitInstanceBuilder::new(component, backend)
    }
}

// MARK: render

fn create_widget_context<
    'a,
    Model: Send + Sync + 'static,
    Message: 'static,
    Event: 'static,
    InnerEvent: 'static,
>(
    window: &window_surface::WindowSurface,
    render_control: &'a render_control::RenderControl,
    ui_control: &ui_control::UiControl<Model, Message, Event, InnerEvent>,
    any_resource: &'a AnyResource,
    debug_config: SharedDebugConfig,
    current_time: std::time::Duration,
) -> Option<ui::WidgetContext<'a>> {
    let size = window.inner_size()?;
    let size = [size.width as f32, size.height as f32];
    let dpi = window.dpi()?;

    let format = window.format()?;

    Some(ui::WidgetContext::new(
        render_control.device_queue(),
        format,
        size,
        dpi,
        render_control.texture_allocator(),
        any_resource,
        ui_control.default_font_size(),
        debug_config,
        current_time,
    ))
}

impl<
    Model: Send + Sync + 'static,
    Message: 'static,
    B: Backend<Event> + Clone + 'static,
    Event: Send + 'static,
    InnerEvent: 'static,
> WinitInstance<Model, Message, B, Event, InnerEvent>
{
    fn render(&mut self, force: bool) -> Result<(), RenderError> {
        // Check if the UI needs to be re-rendered before getting the surface texture
        if !self.ui_control.needs_render() && !force {
            return Ok(());
        }

        let surface_texture = self
            .window
            .get_current_texture()
            .ok_or(RenderError::WindowSurface("Failed to get current texture"))??;

        let target_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let ctx = create_widget_context(
            &self.window,
            &self.render_control,
            &self.ui_control,
            &self.any_resource,
            self.debug_config.clone(),
            self.ticker.current_time(),
        )
        .expect("Window must exist when render is called, as it is only called after resumed");

        let object = {
            let background = ui::Background::new(&target_view, [0.0, 0.0]);
            self.tokio_runtime.block_on(self.ui_control.render(
                ctx.viewport_size(),
                background,
                &ctx,
                &mut self.benchmarker,
            ))
        };

        let size = self
            .window
            .inner_size()
            .expect("Window must exist when render is called, as it is only called after resumed");
        let size = [size.width as f32, size.height as f32];

        self.benchmarker
            .with_gpu_driven_render(|| -> Result<(), RenderError> {
                self.render_control
                    .render(
                        &object,
                        &target_view,
                        size,
                        self.window
                            .format()
                            .expect("Surface must be configured when render is called"),
                    )
                    .map_err(RenderError::Render)?;

                Ok(())
            })?;

        // clear terminal line and print benchmark info
        print!(
            "\r({:.3}) | (frame: {}) | ",
            self.ticker.current_time().as_secs_f32(),
            self.frame,
        );
        self.benchmarker.print();
        println!();
        std::io::Write::flush(&mut std::io::stdout()).ok();

        self.frame += 1;

        surface_texture.present();

        Ok(())
    }

    fn try_render(&mut self, force: bool) {
        if let Err(e) = self.render(force) {
            match e {
                RenderError::Surface(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    // reconfigure the surface
                    let size = self
                        .window
                        .inner_size()
                        .expect("Window must exist to render");
                    self.window
                        .set_size(size, self.render_control.device_queue().device);
                    // request redraw in the next frame
                    self.window.request_redraw();
                }
                _ => {
                    eprintln!("Render error: {e:?}");
                }
            }
        }
    }

    fn handle_commands(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        for command in self.app_handler.drain_commands() {
            match command {
                ui::ApplicationHandlerCommand::Quit => {
                    event_loop.exit();
                }
            }
        }
    }
}

// MARK: Winit Event Loop

// winit event handler
impl<
    Model: Send + Sync + 'static,
    Message: 'static,
    B: Backend<Event> + Clone + 'static,
    Event: Debug + Send + 'static,
    InnerEvent: 'static,
> winit::application::ApplicationHandler<Message>
    for WinitInstance<Model, Message, B, Event, InnerEvent>
{
    // MARK: resumed

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // create a window
        if let Err(e) = self.window.start_window(
            event_loop,
            self.surface_preferred_format,
            self.render_control.gpu(),
        ) {
            eprintln!("Failed to start window: {e:?}");
            event_loop.exit();
            return;
        }

        // call setup function
        self.tokio_runtime
            .block_on(self.ui_control.setup(&self.app_handler));

        self.window.request_redraw();
    }

    // MARK: window_event

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        self.ticker.tick();

        // events which are to be handled by render system
        match event {
            winit::event::WindowEvent::RedrawRequested => {
                self.try_render(false);
            }
            winit::event::WindowEvent::Resized(physical_size) => {
                // update the window size
                self.window
                    .set_size(physical_size, self.render_control.device_queue().device);
                self.try_render(true);
            }
            _ => {}
        }

        // convert window event to Event

        let Some(ctx) = create_widget_context(
            &self.window,
            &self.render_control,
            &self.ui_control,
            &self.any_resource,
            self.debug_config.clone(),
            self.ticker.current_time(),
        ) else {
            return;
        };

        let event = self.ui_control.window_event(
            event,
            || {
                (
                    self.window.inner_size().expect("window should be there when window event is called"),
                    self.window.outer_size().expect("window should be there when window event is called"),
                )
            },
            || {
                (
                    self.window.inner_position().expect("").expect("window should be there and when Android / Wayland window moving event should not be called"),
                    self.window.outer_position().expect("").expect("window should be there and when Android / Wayland window moving event should not be called"),
                )
            },
            &ctx,
            &self.app_handler,
        );

        if let Some(event) = event {
            let backend = self.backend.clone();
            self.tokio_runtime
                .spawn(async move { backend.send_event(event).await });
        }

        self.handle_commands(event_loop);
    }

    // MARK: new_events

    fn new_events(
        &mut self,
        _: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        match cause {
            winit::event::StartCause::Init => {}
            winit::event::StartCause::WaitCancelled { .. } => {}
            winit::event::StartCause::ResumeTimeReached { .. } | winit::event::StartCause::Poll => {
                self.window.request_redraw();
            }
        }
    }

    // MARK: user_event

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: Message) {
        self.ui_control.user_event(&event, &self.app_handler);
        self.window.request_redraw();

        self.handle_commands(event_loop);
    }

    // MARK: other

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn memory_warning(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }
}

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Failed to initialize tokio runtime")]
    TokioRuntime,
    #[error("Failed to initialize GPU")]
    Gpu,
    #[error(transparent)]
    UiControl(#[from] ui_control::UiControlError),
    #[error(transparent)]
    WindowSurface(#[from] window_surface::WindowSurfaceError),
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Window surface error: {0}")]
    WindowSurface(&'static str),
    #[error(transparent)]
    Surface(#[from] wgpu::SurfaceError),
    #[error(transparent)]
    Render(#[from] render_control::RenderControlError),
}
