use std::{fmt::Debug, sync::Arc};

use wgpu::util::DeviceExt;

use crate::principle_renderer::PrincipleRenderer;

use super::{
    component::Component,
    context,
    events::Event,
    observer::Observer,
    types::color::Color,
    ui::{Background, Widget},
};

// MARK: modules

mod benchmark;
mod error;
pub(crate) mod gpu_context;
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
    base_color: Color,

    // --- rendering context ---
    winit_window: Option<Arc<winit::window::Window>>,
    gpu_state: Option<gpu_context::GpuContext<'a>>,
    background_texture: Option<wgpu::Texture>,

    // --- UI context ---
    root_component: Component<Model, Message, Response, IR>,
    root_widget: Option<Box<dyn Widget<Response>>>,
    observer: Observer,

    // --- raw event handling ---
    mouse_state: Option<mouse_state::MouseState>,
    keyboard_state: Option<keyboard_state::KeyboardState>,

    // --- event handling settings ---
    scroll_pixel_per_line: f32,

    // --- widget context ---
    default_font_size: f32,

    // frame
    frame: u64,

    // --- benchmark / monitoring ---
    benchmarker: Option<benchmark::Benchmark>,
}

// build chain
impl<Model: Send + Sync + 'static, Message: 'static, Response: 'static, IR: 'static>
    Window<'_, Model, Message, Response, IR>
{
    pub fn new(component: Component<Model, Message, Response, IR>) -> Self {
        let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .unwrap();

        Self {
            tokio_runtime,
            performance: wgpu::PowerPreference::HighPerformance,
            window_title: String::from("matcha"),
            init_size: [800, 600],
            maximized: false,
            full_screen: false,
            base_color: Color::default(),
            winit_window: None,
            gpu_state: None,
            background_texture: None,
            root_component: component,
            root_widget: None,
            observer: Observer::default(),
            mouse_state: None,
            keyboard_state: None,
            scroll_pixel_per_line: 40.0,
            default_font_size: 16.0,
            frame: 0,
            benchmarker: None,
        }
    }

    // design

    pub fn base_color(&mut self, color: Color) {}

    pub fn performance(&mut self, performance: wgpu::PowerPreference) {}

    pub fn title(&mut self, title: &str) {}

    pub fn init_size(&mut self, size: [u32; 2]) {}

    pub fn maximized(&mut self, maximized: bool) {}

    pub fn full_screen(&mut self, full_screen: bool) {}

    // input

    pub fn mouse_primary_button(&mut self, button: crate::device::mouse::MousePhysicalButton) {}

    pub fn scroll_pixel_per_line(&mut self, pixel: f32) {}
}

// MARK: render

impl<Model: Send + Sync + 'static, Message: 'static, Response: 'static, IR: 'static>
    Window<'_, Model, Message, Response, IR>
{
    fn render(&mut self) -> Result<(), error::RenderError> {
        // ensure that rendering context is available.
        let Some(gpu_state) = self.gpu_state.as_ref() else {
            return Err(error::RenderError::Gpu);
        };

        let Some(root_widget) = self.root_widget.as_mut() else {
            return Err(error::RenderError::RootWidget);
        };

        // let Some(window_renderer) = self.window_renderer.as_mut() else {
        //     return Err(error::RenderError::Renderer);
        // };

        let Some(benchmarker) = self.benchmarker.as_mut() else {
            return Err(error::RenderError::Benchmarker);
        };

        // rendering

        // get surface texture
        let surface_texture = gpu_state.get_current_texture();
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // viewport size
        let viewport_size = gpu_state.viewport_size();
        let viewport_size = [viewport_size[0] as f32, viewport_size[1] as f32];

        // prepare background color (1x1) texture
        let background_texture = self.background_texture.get_or_insert_with(|| {
            let device = gpu_state.device();
            let queue = gpu_state.queue();

            device.create_texture_with_data(
                queue,
                &wgpu::TextureDescriptor {
                    size: wgpu::Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
                    label: None,
                    view_formats: &[],
                },
                wgpu::util::TextureDataOrder::MipMajor,
                &self.base_color.to_rgba_u8(),
            )
        });
        let background_view =
            background_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // start benchmark
        benchmarker.with_benchmark(|| {
            let ctx = gpu_state.widget_context(self.default_font_size);

            let render_result = root_widget.render(
                [Some(viewport_size[0]), Some(viewport_size[1])],
                Background::new(&background_view, [0.0, 0.0]),
                &ctx,
            );

            gpu_state
                .common_resource()
                .get_or_insert_with::<PrincipleRenderer, _>(|| {
                    PrincipleRenderer::new(&ctx)
                })
                .render_to_surface(
                    gpu_state.device(),
                    gpu_state.queue(),
                    &surface_view,
                    viewport_size,
                    render_result,
                    None,
                );
        });

        // present to screen
        surface_texture.present();

        // print debug info
        {
            print!(
                "\rframe rendering time: {}, average: {}, max in second: {} | frame: {}",
                benchmarker.last_time(),
                benchmarker.average_time(),
                benchmarker.max_time(),
                self.frame,
            );
            // flush
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }

        // increment frame count for input handling
        self.frame += 1;

        // return
        Ok(())
    }
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
    ) -> Event {
        let _ = event_loop;
        let _ = window_id;

        match event {
            // window
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
                Event::default()
            }
            winit::event::WindowEvent::Resized(size) => {
                self.gpu_state.as_mut().unwrap().resize(size);
                Event::default()
            }
            // mouse
            winit::event::WindowEvent::CursorMoved {
                device_id,
                position,
            } => self
                .mouse_state
                .as_mut()
                .unwrap()
                .mouse_move(self.frame, position.into()),
            winit::event::WindowEvent::CursorEntered { device_id } => self
                .mouse_state
                .as_mut()
                .unwrap()
                .cursor_entered(self.frame),
            winit::event::WindowEvent::CursorLeft { device_id } => {
                self.mouse_state.as_mut().unwrap().cursor_left(self.frame)
            }
            winit::event::WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
            } => match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => {
                    self.mouse_state.as_mut().unwrap().mouse_scroll(
                        self.frame,
                        [
                            x * self.scroll_pixel_per_line,
                            y * self.scroll_pixel_per_line,
                        ],
                    )
                }
                winit::event::MouseScrollDelta::PixelDelta(position) => self
                    .mouse_state
                    .as_mut()
                    .unwrap()
                    .mouse_scroll(self.frame, [position.x as f32, position.y as f32]),
            },
            winit::event::WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                let button = match button {
                    winit::event::MouseButton::Left => crate::device::mouse::MouseButton::Primary,
                    winit::event::MouseButton::Right => {
                        crate::device::mouse::MouseButton::Secondary
                    }
                    winit::event::MouseButton::Middle => crate::device::mouse::MouseButton::Middle,
                    winit::event::MouseButton::Back => return Event::default(),
                    winit::event::MouseButton::Forward => return Event::default(),
                    winit::event::MouseButton::Other(_) => return Event::default(),
                };
                match state {
                    winit::event::ElementState::Pressed => self
                        .mouse_state
                        .as_mut()
                        .unwrap()
                        .button_pressed(self.frame, button),
                    winit::event::ElementState::Released => self
                        .mouse_state
                        .as_mut()
                        .unwrap()
                        .button_released(self.frame, button),
                }
            }
            // keyboard
            winit::event::WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => self
                .keyboard_state
                .as_mut()
                .unwrap()
                .key_event(self.frame, event)
                .unwrap_or_default(),
            _ => {
                // ignore other events
                Event::default()
            }
        }
    }
}

// MARK: polling to render

impl<Model: Send + Sync + 'static, Message: 'static, Response: 'static, IR: 'static>
    Window<'_, Model, Message, Response, IR>
{
    pub fn poll(&mut self) {
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

                if let Some(root_widget) = self.root_widget.as_mut() {
                    if (root_widget.update_widget_tree(true, &*dom).await).is_err() {
                        self.root_widget = Some(dom.build_widget_tree());
                    }
                } else {
                    self.root_widget = Some(dom.build_widget_tree());
                }
            });
        }

        // re-rendering
        self.render().unwrap();
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

        let gpu_state = self.tokio_runtime.block_on(gpu_context::GpuContext::new(
            self.winit_window.as_ref().unwrap().clone(),
            self.performance,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        ));
        self.gpu_state = Some(gpu_state);

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
            self.benchmarker = Some(benchmark::Benchmark::new((rate / 1000) as usize));
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
        // process raw event

        let event = self.process_raw_event(event_loop, window_id, event);

        // when root widget exists
        if let Some(root_widget) = self.root_widget.as_mut() {
            // get response from widget tree
            let window_size = self.gpu_state.as_ref().unwrap().viewport_size();
            let window_size = [window_size[0] as f32, window_size[1] as f32];

            let response = root_widget.widget_event(
                &event,
                [Some(window_size[0]), Some(window_size[1])],
                &self
                    .gpu_state
                    .as_ref()
                    .unwrap()
                    .widget_context(self.default_font_size),
            );

            if let Some(user_event) = response {
                // send response to backend
                todo!(
                    "Response to backend: {:?}\nBut sending to backend is not implemented yet",
                    user_event
                );
            }
        }

        // polling
        self.poll();
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
                self.poll()
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
