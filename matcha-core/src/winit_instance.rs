use renderer::{core_renderer, CoreRenderer};
use std::fmt::Debug;
use thiserror::Error;

use crate::{
    backend::Backend,
    ui,
    window_surface::{self},
};

// MARK: modules

mod benchmark;
mod builder;
mod ticker;
mod window_ui;

pub(crate) use builder::WinitInstanceBuilder;

// todo: move this to more appropriate place
pub trait WindowManager<Message> {
    fn message_routing(&self, message: &Message) -> Option<winit::window::WindowId>;
}

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

    // --- Context ---
    context: ui::context::AllContext,
    preferred_surface_format: wgpu::TextureFormat,
    // ticker: ticker::Ticker,

    // --- ui ---
    windows: std::collections::HashMap<
        winit::window::WindowId,
        window_ui::WindowUi<Model, Message, Event, InnerEvent>,
        fxhash::FxBuildHasher,
    >,
    window_manager: Box<dyn WindowManager<Message> + Send + Sync>,

    // --- render control ---
    base_color: wgpu::Color,
    renderer: CoreRenderer,

    // --- backend ---
    backend: B,

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

impl<
    Model: Send + Sync + 'static,
    Message: 'static,
    B: Backend<Event> + Clone + 'static,
    Event: Send + 'static,
    InnerEvent: 'static,
> WinitInstance<Model, Message, B, Event, InnerEvent>
{
    fn render(
        &mut self,
        window_id: winit::window::WindowId,
        winit_event_loop: &winit::event_loop::ActiveEventLoop,
        force: bool,
    ) -> Result<(), RenderError> {
        let Some(window_ui) = self.windows.get_mut(&window_id) else {
            return Err(RenderError::WindowNotFound);
        };

        // Check if the UI needs to be re-rendered before getting the surface texture
        if !window_ui.needs_render() && !force {
            return Ok(());
        }

        let object = {
            self.tokio_runtime.block_on(window_ui.render(
                self.tokio_runtime.handle(),
                &self.context,
                winit_event_loop,
                &mut self.benchmarker,
            ))
        };

        let Some(window_ui::RenderResult {
            render_node: object,
            viewport_size,
            surface_texture,
            surface_format,
        }) = object
        else {
            // Nothing to render
            return Ok(());
        };

        let device = self.context.gpu().device();
        let queue = self.context.gpu().queue();

        self.benchmarker
            .with_gpu_driven_render(|| -> Result<(), RenderError> {
                self.renderer
                    .render(
                        device,
                        queue,
                        surface_format,
                        &surface_texture
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                        viewport_size,
                        &object,
                        self.base_color,
                        &self.context.texture_allocator().color_texture(),
                        &self.context.texture_allocator().stencil_texture(),
                    )
                    .map_err(RenderError::Render)?;

                Ok(())
            })?;

        // clear terminal line and print benchmark info
        print!(
            "\r({:.3}) | (frame: {}) | ",
            self.context.current_time().as_secs_f32(),
            self.frame,
        );
        self.benchmarker.print();
        println!();
        std::io::Write::flush(&mut std::io::stdout()).ok();

        self.frame += 1;

        surface_texture.present();

        Ok(())
    }

    fn handle_commands(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        for command in self.context.drain_commands() {
            match command {
                ui::ApplicationContextCommand::Quit => {
                    event_loop.exit();
                }
            }
        }
    }
}

// MARK: Winit Event Loop

// TODO: Use TokioRuntime::spawn() instead of blocking on as much as possible.

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
        self.tokio_runtime.block_on(async {
            // start window
            for window_ui in self.windows.values_mut() {
                window_ui.start_window(event_loop, &self.context).await;
            }

            // call setup function
            for window_ui in self.windows.values_mut() {
                window_ui
                    .setup(self.tokio_runtime.handle(), &self.context)
                    .await;
            }
        });
    }

    // MARK: window_event

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        // events which are to be handled by render system
        match event {
            winit::event::WindowEvent::RedrawRequested => {
                if let Err(e) = self.render(window_id, event_loop, false) {
                    todo!("Render error: {:?}", e);
                }
            }
            winit::event::WindowEvent::Resized(physical_size) => {
                if let Some(window_ui) = self.windows.get_mut(&window_id) {
                    window_ui.resize_window(physical_size, self.context.gpu().device());
                    window_ui.request_redraw();
                }
            }
            _ => {}
        }

        // convert window event to Event

        let Some(window_ui) = self.windows.get_mut(&window_id) else {
            return;
        };

        let event = window_ui.window_event(event, self.tokio_runtime.handle(), &self.context);

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
                for window_ui in self.windows.values_mut() {
                    window_ui.request_redraw();
                }
            }
        }
    }

    // MARK: user_event

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: Message) {
        if let Some(window_id) = self.window_manager.message_routing(&event) {
            if let Some(window_ui) = self.windows.get_mut(&window_id) {
                window_ui.user_event(&event, self.tokio_runtime.handle(), &self.context);
                window_ui.request_redraw();
            }
        }

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
    WindowUi(#[from] window_ui::WindowUiError),
    #[error(transparent)]
    WindowSurface(#[from] window_surface::WindowSurfaceError),
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Window not found")]
    WindowNotFound,
    #[error("Window surface error: {0}")]
    WindowSurface(&'static str),
    #[error(transparent)]
    Surface(#[from] wgpu::SurfaceError),
    #[error(transparent)]
    Render(#[from] core_renderer::TextureValidationError),
}
