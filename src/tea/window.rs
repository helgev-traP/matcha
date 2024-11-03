use nalgebra as na;
use std::{sync::Arc, time::Instant};

use super::{
    application_context,
    component::Component,
    events::{self, UiEventContent},
    types::{color::Color, size::PxSize},
    ui::{RenderingTrait, Widget, WidgetTrait},
};

mod gpu_state;
mod keyboard_state;
mod mouse_state;

mod benchmark;

pub struct Window<'a, Model: Send + 'static, Message: 'static> {
    // --- rendering context ---
    // boot status
    performance: wgpu::PowerPreference,
    title: String,
    init_size: [u32; 2],
    maximized: bool,
    full_screen: bool,

    font_context: Option<crate::cosmic::FontContext>,

    base_color: Color,

    // rendering
    winit_window: Option<Arc<winit::window::Window>>,
    gpu_state: Option<gpu_state::GpuState<'a>>,
    context: Option<application_context::ApplicationContext>,

    render: Option<crate::renderer::Renderer>,

    // render tree
    render_tree: Option<Box<dyn Widget<Message>>>,

    // root component
    root_component: Component<Model, Message, Message, Message>,

    // frame
    frame: u64,

    // --- input and event handling ---
    // mouse
    mouse_state: Option<mouse_state::MouseState>,

    // mouse settings
    mouse_primary_button: winit::event::MouseButton,
    scroll_pixel_per_line: f32,

    // keyboard
    keyboard_state: Option<keyboard_state::KeyboardState>,

    // --- benchmark ---
    #[cfg(debug_assertions)]
    benchmark: Option<benchmark::Benchmark>,
}

// setup
impl<Model: Send, Message: 'static> Window<'_, Model, Message> {
    pub fn new(component: Component<Model, Message, Message, Message>) -> Self {
        Self {
            performance: wgpu::PowerPreference::default(),
            title: "Tea".to_string(),
            init_size: [800, 600],
            maximized: false,
            full_screen: false,
            font_context: None,
            base_color: Color::Rgb8USrgb { r: 0, g: 0, b: 0 },
            winit_window: None,
            gpu_state: None,
            context: None,
            render: None,
            render_tree: None,
            root_component: component,
            frame: 0,
            mouse_state: None,
            mouse_primary_button: winit::event::MouseButton::Left,
            scroll_pixel_per_line: 40.0,
            keyboard_state: None,
            // #[cfg(debug_assertions)]
            benchmark: None,
        }
    }

    // design

    pub fn base_color(&mut self, color: Color) {
        self.base_color = color;
    }

    pub fn performance(&mut self, performance: wgpu::PowerPreference) {
        self.performance = performance;
    }

    pub fn title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    pub fn init_size(&mut self, size: [u32; 2]) {
        self.init_size = size;
    }

    pub fn maximized(&mut self, maximized: bool) {
        self.maximized = maximized;
    }

    pub fn full_screen(&mut self, full_screen: bool) {
        self.full_screen = full_screen;
    }

    pub fn font_context(&mut self, font_context: crate::cosmic::FontContext) {
        self.font_context = Some(font_context);
    }

    // input

    pub fn mouse_primary_button(&mut self, button: crate::device::mouse::MousePhysicalButton) {
        match button {
            crate::device::mouse::MousePhysicalButton::Left => {
                self.mouse_primary_button = winit::event::MouseButton::Left;
            }
            crate::device::mouse::MousePhysicalButton::Right => {
                self.mouse_primary_button = winit::event::MouseButton::Right;
            }
            crate::device::mouse::MousePhysicalButton::Middle => {
                self.mouse_primary_button = winit::event::MouseButton::Middle;
            }
        }
    }

    pub fn scroll_pixel_per_line(&mut self, pixel: f32) {
        self.scroll_pixel_per_line = pixel;
    }
}

impl<Model: Send, Message: 'static> Window<'_, Model, Message> {
    fn render(&mut self) {
        // surface
        let surface = self.gpu_state.as_ref().unwrap().get_current_texture();
        let surface_texture_view = surface
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // depth texture
        let depth_texture = self.gpu_state.as_ref().unwrap().get_depth_texture();
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // multisample texture
        let multisampled_texture = self.gpu_state.as_ref().unwrap().get_multisampled_texture();
        let multisampled_texture_view =
            multisampled_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // viewport size
        let viewport_size = self.gpu_state.as_ref().unwrap().get_viewport_size();

        // render

        // make encoder
        let render = self.render.as_mut().unwrap();
        let encoder = render.encoder(
            &surface_texture_view,
            &multisampled_texture_view,
            &depth_texture_view,
            viewport_size,
        );

        // encode render tree
        let render_tree = self.render_tree.as_mut().unwrap().for_rendering();
        encoder.clear(self.base_color);
        render_tree.render(viewport_size, na::Matrix4::identity(), encoder.clone());

        #[cfg(debug_assertions)] // benchmark timer start ----------------------------------
        self.benchmark.as_mut().unwrap().start();

        encoder.finish().unwrap();

        #[cfg(debug_assertions)] // benchmark timer stop -----------------------------------
        self.benchmark.as_mut().unwrap().stop();

        // present
        surface.present();

        // print frame (debug)
        #[cfg(debug_assertions)]
        {
            print!(
                "\rframe rendering time: {}, average: {}, max in second: {} | frame: {}",
                self.benchmark.as_ref().unwrap().last_time(),
                self.benchmark.as_ref().unwrap().average_time(),
                self.benchmark.as_ref().unwrap().max_time(),
                self.frame,
            );
            // flush
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }

        self.frame += 1;
    }
}

// winit event handler
impl<Model: Send, Message: 'static> winit::application::ApplicationHandler<Message>
    for Window<'_, Model, Message>
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // crate window

        let winit_window = Arc::new(
            event_loop
                .create_window(winit::window::Window::default_attributes())
                .unwrap(),
        );
        winit_window.set_title(self.title.as_str());
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

        let context = std::mem::take(&mut self.font_context);
        let gpu_state = pollster::block_on(gpu_state::GpuState::new(
            self.winit_window.as_ref().unwrap().clone(),
            self.performance,
            context,
        ));
        self.context = Some(gpu_state.get_app_context());
        self.gpu_state = Some(gpu_state);

        // set winit control flow

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        // crate renderer

        self.render = Some(crate::renderer::Renderer::new(
            self.gpu_state.as_ref().unwrap().get_app_context(),
        ));

        // crate render tree

        self.render_tree = Some(self.root_component.view().unwrap().build_render_tree());

        // crate input states

        // todo: calculate double click and long press duration from monitor refresh rate
        self.mouse_state = Some(mouse_state::MouseState::new(12, 60).unwrap());
        self.keyboard_state = Some(keyboard_state::KeyboardState::new());

        // prepare benchmark

        #[cfg(debug_assertions)]
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

        // render

        self.render();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let mut ui_event = match event {
            // window
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
                return;
            }
            winit::event::WindowEvent::Resized(size) => {
                self.gpu_state.as_mut().unwrap().resize(size);
                return;
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
                    winit::event::MouseButton::Back => return,
                    winit::event::MouseButton::Forward => return,
                    winit::event::MouseButton::Other(_) => return,
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
            } => {
                if let Some(event) = self
                    .keyboard_state
                    .as_mut()
                    .unwrap()
                    .key_event(self.frame, event)
                {
                    event
                } else {
                    return;
                }
            }
            _ => return,
        };

        self.render_tree.as_mut().unwrap().widget_event(
            &ui_event,
            self.gpu_state.as_ref().unwrap().get_viewport_size(),
            self.context.as_ref().unwrap(),
        );
    }

    fn new_events(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        match cause {
            winit::event::StartCause::Init => {}
            winit::event::StartCause::ResumeTimeReached {
                start,
                requested_resume,
            } => {}
            winit::event::StartCause::WaitCancelled {
                start,
                requested_resume,
            } => {}
            winit::event::StartCause::Poll => {
                // poll events
                let event = self
                    .mouse_state
                    .as_mut()
                    .unwrap()
                    .long_pressing_detection(self.frame);

                // todo: give event to root component

                // render
                self.render();
            }
        }
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: Message) {
        self.root_component.update(event);
    }

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
