use std::sync::Arc;

use wgpu::util::DeviceExt;

use super::{
    component::Component,
    context,
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

// todo: rename generic type as Model, ComponentMessage -> M (Message), UiResponse -> R (Response).
pub struct Window<'a, Model: Send + 'static, Message: 'static, Response: 'static> {
    // --- window boot settings ---
    performance: wgpu::PowerPreference,
    window_title: String,
    init_size: [u32; 2],
    maximized: bool,
    full_screen: bool,
    // font_context: crate::cosmic::FontContext,
    // base_color: Color,

    // --- rendering context ---

    // --- UI context ---
    root_component: Component<Model, Message, Response>,
    // --- raw event handling ---

    // --- benchmark / monitoring ---
}

// build chain
impl<Model: Send, Message: 'static, Response: 'static> Window<'_, Model, Response, Message> {
    pub fn new(component: Component<Model, Message, Response>) -> Self {
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

// winit event handler
impl<Model: Send, Message: 'static, Response: 'static> Window<'_, Model, Message, Response> {
    fn render(&mut self) {}
}

// winit event handler
impl<Model: Send, Message: 'static, Response: 'static>
    winit::application::ApplicationHandler<Message> for Window<'_, Model, Message, Response>
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // --- create window ---

        // --- prepare gpu ---

        // --- init input context ---
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        // --- process raw event ---

        // --- get response from widget tree ---

        // --- exec user defined program with response ---
    }

    fn new_events(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        // DOM and Widget updates, as well as re-rendering, will only execute in this function
        // as a result of observers catching component updates.

        // --- check observer ---

        {
            // rebuild dom tree

            // update widget tree
        }

        // --- re-rendering ---
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: Message) {
        // --- give message to component ---
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
