use std::{fmt::Debug, sync::Arc};

use super::{
    component::Component,
    context,
    events::UiEvent,
    observer::Observer,
    renderer::Renderer,
    types::{color::Color, range::Range2D},
    ui::Widget,
};

// MARK: modules

mod benchmark;
mod gpu_state;
mod keyboard_state;
mod mouse_state;

// MARK: Window

pub struct Window<
    'a,
    Model: Send + Sync + 'static,
    Message: 'static,
    Response: 'static,
    IR: 'static = Response,
> {
    // --- tokio runtime ---
    tokio_runtime: tokio::runtime::Runtime,

    // --- window boot settings ---
    performance: wgpu::PowerPreference,
    window_title: String,
    init_size: [u32; 2],
    maximized: bool,
    full_screen: bool,
    font_context: Option<crate::cosmic::FontContext>,
    // base_color: Color,

    // --- rendering context ---
    winit_window: Option<Arc<winit::window::Window>>,
    gpu_state: Option<gpu_state::GpuState<'a>>,
    // renderer ?

    // --- UI context ---
    root_component: Component<Model, Message, Response, IR>,
    root_widget: Option<Box<dyn Widget<Response>>>,
    observer: Observer,

    // --- raw event handling ---
    mouse_state: Option<mouse_state::MouseState>,
    keyboard_state: Option<keyboard_state::KeyboardState>,

    // --- benchmark / monitoring ---
    benchmark: Option<benchmark::Benchmark>,
}

// build chain
impl<Model: Send + Sync + 'static, Message: 'static, Response: 'static, IR: 'static>
    Window<'_, Model, Message, Response, IR>
{
    pub fn new(component: Component<Model, Message, Response, IR>) -> Self {
        todo!()
    }

    // design

    pub fn base_color(&mut self, color: Color) {}

    pub fn performance(&mut self, performance: wgpu::PowerPreference) {}

    pub fn title(&mut self, title: &str) {}

    pub fn init_size(&mut self, size: [u32; 2]) {}

    pub fn maximized(&mut self, maximized: bool) {}

    pub fn full_screen(&mut self, full_screen: bool) {}

    pub fn font_context(&mut self, font_context: crate::cosmic::FontContext) {}

    // input

    pub fn mouse_primary_button(&mut self, button: crate::device::mouse::MousePhysicalButton) {}

    pub fn scroll_pixel_per_line(&mut self, pixel: f32) {}
}

// MARK: render

impl<Model: Send + Sync + 'static, Message: 'static, Response: 'static, IR: 'static>
    Window<'_, Model, Message, Response, IR>
{
    fn render(&mut self) {}
}

// MARK: process_raw_event

impl<Model: Send + Sync + 'static, Message: 'static, Response: 'static, IR: 'static>
    Window<'_, Model, Message, Response, IR>
{
    fn process_raw_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) -> UiEvent {
        todo!()
    }
}

// MARK: Winit Event Loop

// winit event handler
impl<Model: Send + Sync + 'static, Message: 'static, Response: Debug + 'static, IR: 'static>
    winit::application::ApplicationHandler<Message> for Window<'_, Model, Message, Response, IR>
{
    // MARK: resumed

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // --- create window ---
        let winit_window = Arc::new(
            event_loop
                .create_window(winit::window::WindowAttributes::default())
                .unwrap(),
        );

        // set window initial settings
        winit_window.set_title(&self.window_title);
        let _ = winit_window.request_inner_size(winit::dpi::PhysicalSize::new(
            self.init_size[0],
            self.init_size[1],
        ));
        if self.maximized {
            winit_window.set_maximized(true);
        }
        if self.full_screen {
            winit_window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        }
        self.winit_window = Some(winit_window);

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        // --- prepare gpu ---

        // todo: refactor.
        // separate font_context from the gpu state

        let font_context = self.font_context.take();
        let gpu_state = self.tokio_runtime.block_on(gpu_state::GpuState::new(
            self.winit_window.as_ref().unwrap().clone(),
            self.performance,
            font_context,
        ));
        self.gpu_state = Some(gpu_state);
        // renderer preparation.

        // --- init input context ---
        {
            // todo: calculate double click and long press duration from monitor refresh rate

            self.mouse_state = Some(mouse_state::MouseState::new(12, 60).unwrap());
            self.keyboard_state = Some(keyboard_state::KeyboardState::new());
        }

        // --- prepare benchmark monitoring ---

        {
            let rate = self
                .winit_window
                .as_ref()
                .unwrap()
                .current_monitor()
                .unwrap()
                .refresh_rate_millihertz()
                .unwrap();
            println!("Monitor refresh rate: {}.{} Hz", rate / 1000, rate % 1000);
            self.benchmark = Some(benchmark::Benchmark::new((rate / 1000) as usize));
        }

        // --- trigger first frame rendering ---
        {
            // set observer that returns true
            self.observer = Observer::new_render_trigger();
        }
    }

    // MARK: window_event

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if self.root_widget.is_none() {
            return;
        }

        // process raw event

        let event = self.process_raw_event(event_loop, window_id, event);

        // get response from widget tree

        let window_size = self.gpu_state.as_ref().unwrap().get_viewport_size();

        let response = self
            .root_widget
            .as_mut()
            .expect("never panic")
            .widget_event(
                &event,
                [Some(window_size[0]), Some(window_size[1])],
                &self.gpu_state.as_ref().unwrap().get_app_context(),
            );

        if let Some(user_event) = response {
            // send response to backend
            todo!(
                "Response to backend: {:?}\nBut sending to backend is not implemented yet",
                user_event
            );
        }
    }

    // MARK: new_events

    fn new_events(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        let _ = event_loop;

        match cause {
            winit::event::StartCause::Init => {}
            winit::event::StartCause::WaitCancelled { .. } => {}
            winit::event::StartCause::ResumeTimeReached { .. } | winit::event::StartCause::Poll => {
                // DOM and Widget updates, as well as re-rendering, will only execute in this function
                // as a result of observers catching component updates.

                // check observer
                if self.observer.is_updated() {
                    self.tokio_runtime.block_on(async {
                        // rebuild dom tree
                        let dom = self.root_component.view().await;

                        // re-collect observers
                        self.observer = dom.collect_observer().await;

                        // update widget tree
                        self.root_widget
                            .as_mut()
                            // todo
                            .expect("todo: ensure that the widget tree is not empty")
                            // todo: try remove `&*`
                            .update_widget_tree(true, &*dom)
                            .await
                            .unwrap();
                    });
                }

                // re-rendering
                self.render();
            }
        }
    }

    // MARK: user_event

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: Message) {
        let _ = event_loop;
        // --- send message to component ---
        self.root_component.update(event);
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
