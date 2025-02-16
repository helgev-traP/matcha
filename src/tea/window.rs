use std::sync::Arc;

use wgpu::util::DeviceExt;

use super::{
    component::Component,
    context,
    renderer::Renderer,
    types::{color::Color, range::Range2D},
    ui::Widget,
};

mod benchmark;
mod gpu_state;
mod keyboard_state;
mod mouse_state;

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
    context: Option<context::SharedContext>,
    renderer: Option<Renderer>,

    // root component
    root_component: Component<Model, Message, Message, Message>,
    root_widget: Option<Box<dyn Widget<Message>>>,
    root_widget_has_dynamic: bool,
    background_texture: Option<wgpu::Texture>,

    // frame
    frame: u64,

    // --- input and event handling ---

    // mouse
    mouse_state: Option<mouse_state::MouseState>,
    mouse_primary_button: winit::event::MouseButton,
    scroll_pixel_per_line: f32,

    // keyboard
    keyboard_state: Option<keyboard_state::KeyboardState>,

    // --- benchmark ---
    benchmark: Option<benchmark::Benchmark>,
}

// build chain
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
            renderer: None,
            root_component: component,
            root_widget: None,
            root_widget_has_dynamic: false,
            background_texture: None,
            frame: 0,
            mouse_state: None,
            mouse_primary_button: winit::event::MouseButton::Left,
            scroll_pixel_per_line: 40.0,
            keyboard_state: None,
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

// winit event handler
impl<Model: Send, Message: 'static> Window<'_, Model, Message> {
    fn render(&mut self) {
        // --- get surface texture ---
        // surface
        let surface = self.gpu_state.as_ref().unwrap().get_current_texture();
        let surface_texture_view = surface
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // viewport size
        let viewport_size = self.gpu_state.as_ref().unwrap().get_viewport_size();

        // --- rendering ---

        // prepare background texture

        let background_texture = self.background_texture.get_or_insert_with(|| {
            let device = self.context.as_ref().unwrap().get_wgpu_device();
            let queue = self.context.as_ref().unwrap().get_wgpu_queue();

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
        let background_texture_view =
            background_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let background_range = Range2D {
            x: [0.0, 1.0],
            y: [0.0, 1.0],
        };

        // benchmark timer start ----------------------------------
        self.benchmark.as_mut().unwrap().start();

        // get root component's render result
        let render_result = self.root_widget.as_mut().unwrap().render(
            [viewport_size[0].into(), viewport_size[1].into()],
            &background_texture_view,
            background_range,
            self.context.as_ref().unwrap(),
            self.renderer.as_ref().unwrap(),
            self.frame,
        );

        // project to screen
        self.renderer.as_mut().unwrap().render_to_surface(
            &surface_texture_view,
            viewport_size,
            render_result,
        );

        // benchmark timer stop -----------------------------------
        self.benchmark.as_mut().unwrap().stop();

        // present
        surface.present();

        // print frame (debug)
        if let Some(benchmark) = &self.benchmark {
            print!(
                "\rframe rendering time: {}, average: {}, max in second: {} | frame: {}",
                benchmark.last_time(),
                benchmark.average_time(),
                benchmark.max_time(),
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
        // --- crate window ---

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

        // --- create gpu state / shared context ---

        let context = std::mem::take(&mut self.font_context);
        let gpu_state = pollster::block_on(gpu_state::GpuState::new(
            self.winit_window.as_ref().unwrap().clone(),
            self.performance,
            context,
        ));
        self.context = Some(gpu_state.get_app_context());
        self.gpu_state = Some(gpu_state);
        self.renderer = Some(Renderer::new(self.context.as_ref().unwrap().clone()));

        // set winit control flow

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        // crate input states

        // todo: calculate double click and long press duration from monitor refresh rate

        self.mouse_state = Some(mouse_state::MouseState::new(12, 60).unwrap());
        self.keyboard_state = Some(keyboard_state::KeyboardState::new());

        // prepare benchmark

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

        // --- render ---
        let (root_widget, root_widget_has_dynamic) = self.root_component.view().build_widget_tree();

        self.root_widget = Some(root_widget);
        self.root_widget_has_dynamic = root_widget_has_dynamic;

        self.render();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let ui_event = match event {
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

        // event handling
        let viewport_size = self.gpu_state.as_ref().unwrap().get_viewport_size();

        let result = self.root_widget.as_mut().unwrap().widget_event(
            &ui_event,
            [viewport_size[0].into(), viewport_size[1].into()],
            self.context.as_ref().unwrap(),
        );

        // update component
        if let Some(message) = result.user_event {
            println!("received message");
            self.root_component.update(message);
        }
        self.root_component.view();
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
