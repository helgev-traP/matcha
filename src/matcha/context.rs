use std::sync::{Arc, MutexGuard};

use vello::wgpu;

pub struct SharedContext {
    winit_window: Arc<winit::window::Window>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    vello_renderer: Arc<std::sync::Mutex<vello::Renderer>>,
}

impl Clone for SharedContext {
    fn clone(&self) -> Self {
        Self {
            device: self.device.clone(),
            queue: self.queue.clone(),
            winit_window: self.winit_window.clone(),
            vello_renderer: self.vello_renderer.clone(),
        }
    }
}

impl SharedContext {
    pub(crate) fn new(
        winit_window: Arc<winit::window::Window>,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        vello_renderer: Arc<std::sync::Mutex<vello::Renderer>>,
    ) -> Self {
        Self {
            winit_window,
            device,
            queue,
            vello_renderer,
        }
    }

    pub fn get_device(&self) -> &Arc<wgpu::Device> {
        &self.device
    }

    pub fn get_queue(&self) -> &Arc<wgpu::Queue> {
        &self.queue
    }

    pub fn get_encoder(&self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("App Context Command Encoder"),
            })
    }

    pub fn get_dpi(&self) -> f64 {
        self.winit_window.scale_factor()
    }

    pub fn get_viewport_size(&self) -> (u32, u32) {
        let size = self.winit_window.inner_size();
        (size.width, size.height)
    }

    pub fn get_vello_renderer<'a>(&'a self) -> MutexGuard<'a, vello::Renderer> {
        self.vello_renderer.lock().unwrap()
    }
}
