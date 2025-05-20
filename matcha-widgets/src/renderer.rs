use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct VelloRenderer {
    scene: Option<Mutex<vello::Scene>>,
    renderer: Option<Mutex<vello::Renderer>>,
}

impl matcha_core::renderer::RendererSetup for VelloRenderer {
    fn setup(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) {
        self.scene = Some(Mutex::new(vello::Scene::new()));
        self.renderer = Some(Mutex::new(self.setup_vello(device)));
    }
}

impl VelloRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    fn setup_vello(&mut self, device: &wgpu::Device) -> vello::Renderer {
        vello::Renderer::new(
            device,
            vello::RendererOptions {
                // Set the desired options for the renderer
                ..Default::default()
            },
        )
        .unwrap()
    }
}
