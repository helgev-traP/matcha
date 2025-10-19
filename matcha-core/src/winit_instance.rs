use renderer::{CoreRenderer, core_renderer};
use std::fmt::Debug;
use thiserror::Error;

use crate::{
    backend::Backend,
    context::{ApplicationCommand, GlobalResources},
    ui::component::AnyComponent,
    window_surface::{self},
};

// MARK: modules

mod builder;
mod window_ui;

pub(crate) use builder::WinitInstanceBuilder;

// MARK: Winit

pub struct WinitInstance<Message: 'static, Event: Send + 'static, B: Backend<Event> + 'static> {
    // --- tokio runtime ---
    tokio_runtime: tokio::runtime::Runtime,

    // --- Context ---
    resource: GlobalResources,
    // ticker: ticker::Ticker,

    // --- ui ---
    windows: std::collections::HashMap<
        winit::window::WindowId,
        window_ui::WindowUi<Message, Event>,
        fxhash::FxBuildHasher,
    >,

    // --- render control ---
    base_color: wgpu::Color,
    renderer: CoreRenderer,

    // --- backend ---
    backend: B,

    // --- benchmark / monitoring ---
    benchmarker: utils::benchmark::Benchmark,
    frame: u128,
}

impl<Message, Event: Send + 'static, B: Backend<Event> + 'static> WinitInstance<Message, Event, B> {
    pub fn builder(
        component: impl AnyComponent<Message, Event> + 'static,
        backend: B,
    ) -> WinitInstanceBuilder<Message, Event, B> {
        WinitInstanceBuilder::new(component, backend)
    }
}

// MARK: render

impl<Message: 'static, Event: Send + 'static, B: Backend<Event> + Clone + 'static>
    WinitInstance<Message, Event, B>
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
                winit_event_loop,
                &self.resource,
                &self.renderer,
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

        let device = self.resource.gpu().device();
        let queue = self.resource.gpu().queue();

        self.benchmarker
            .with("gpu_driven_render", || -> Result<(), RenderError> {
                self.renderer
                    .render(
                        &device,
                        &queue,
                        surface_format,
                        &surface_texture
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                        viewport_size,
                        &object,
                        self.base_color,
                        &self.resource.texture_atlas().lock().texture(),
                        &self.resource.stencil_atlas().lock().texture(),
                    )
                    .map_err(RenderError::Render)?;

                Ok(())
            })?;

        // clear terminal line and print benchmark info
        print!(
            "\r({:.3}) | (frame: {}) | ",
            self.resource.current_time().as_secs_f32(),
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
        while let Ok(command) = self.resource.command_receiver().try_recv() {
            match command {
                ApplicationCommand::Quit => {
                    event_loop.exit();
                }
            }
        }
    }
}

// MARK: Winit Event Loop

// TODO: Use TokioRuntime::spawn() instead of blocking on as much as possible.

// winit event handler
impl<Message: 'static, Event: Send + 'static, B: Backend<Event> + Clone + 'static>
    winit::application::ApplicationHandler<Message> for WinitInstance<Message, Event, B>
{
    // MARK: resumed

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.tokio_runtime.block_on(async {
            // start window
            for window_ui in self.windows.values_mut() {
                window_ui
                    .start_window(event_loop, self.resource.gpu())
                    .await;
            }

            // call setup function
            for window_ui in self.windows.values_mut() {
                window_ui
                    .setup(self.tokio_runtime.handle(), &self.resource)
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
                    window_ui.resize_window(physical_size, &self.resource.gpu().device());
                    window_ui.request_redraw();
                }
            }
            _ => {}
        }

        // convert window event to Event

        let Some(window_ui) = self.windows.get_mut(&window_id) else {
            return;
        };

        let event = window_ui.window_event(event, self.tokio_runtime.handle(), &self.resource);

        if let Some(event) = event {
            self.tokio_runtime.block_on(self.backend.send_event(event));
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
        for window_ui in self.windows.values_mut() {
            window_ui.user_event(&event, self.tokio_runtime.handle(), &self.resource);
            window_ui.request_redraw();
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
