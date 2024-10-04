use std::sync::Arc;

use super::{application_context, component::Component, types::color::Color, ui::RenderNode};
mod gpu_state;

pub struct Window<'a, Model, Message: 'static> {
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

    render: Option<crate::render::Render>,

    // render tree
    render_tree: Option<Box<dyn RenderNode<Message>>>,

    // root component
    root_component: Component<Model, Message, Message>,

    // debug
    frame: u64,
}

// setup
impl<Model, Message: 'static> Window<'_, Model, Message> {
    pub fn new(component: Component<Model, Message, Message>) -> Self {
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
            render: None,
            render_tree: None,
            root_component: component,
            frame: 0,
        }
    }

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
}

impl<Model, Message: 'static> Window<'_, Model, Message> {
    fn render(&mut self) {
        // surface
        let surface = self.gpu_state.as_ref().unwrap().get_current_texture();
        let surface_texture_view = surface
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let viewport_size = self.gpu_state.as_ref().unwrap().get_viewport_size();

        // render
        let render = self.render.as_ref().unwrap();
        let render_tree = self.render_tree.as_mut().unwrap();

        render.render(
            surface_texture_view,
            &viewport_size,
            &self.base_color,
            render_tree,
        );

        // present
        surface.present();

        // frame
        println!("frame: {}", self.frame);
        self.frame += 1;
    }
}

// winit event handler
impl<Model, Message: 'static> winit::application::ApplicationHandler<Message>
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
        self.gpu_state = Some(gpu_state);

        // set winit control flow
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        // crate render
        self.render = Some(crate::render::Render::new(
            self.gpu_state.as_ref().unwrap().get_app_context(),
        ));

        // crate render tree
        self.render_tree = Some(self.root_component.view().unwrap().build_render_tree());
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::Resized(size) => {
                self.gpu_state.as_mut().unwrap().resize(size);
            }
            _ => {}
        }
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
