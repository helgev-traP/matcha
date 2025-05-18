#[derive(Default)]
pub struct Renderer {
    vello_renderer: Option<vello::Renderer>,
}

impl matcha_core::renderer::RendererSetup for Renderer {
    fn setup(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) {
        self.vello_renderer = Some(self.setup_vello(device));
    }
}

impl Renderer {
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
