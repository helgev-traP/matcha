use crate::{
    gpu::Gpu,
    renderer::{Renderer, TextureValidationError},
    texture_allocator::TextureAllocator,
    ui::Object,
};

const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
const STENCIL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

pub struct DeviceQueue<'a> {
    pub(crate) device: &'a wgpu::Device,
    pub(crate) queue: &'a wgpu::Queue,
}

impl DeviceQueue<'_> {
    pub fn device(&self) -> &wgpu::Device {
        self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        self.queue
    }
}

pub struct RenderControl {
    gpu: Gpu,
    base_color: wgpu::Color,
    renderer: Renderer,
    texture_allocator: TextureAllocator,
}

impl RenderControl {
    pub async fn new(power_preferences: wgpu::PowerPreference, base_color: wgpu::Color) -> Self {
        let gpu = Gpu::new(power_preferences).await.unwrap();
        let renderer = Renderer::new(gpu.device());

        let texture_allocator = TextureAllocator::new(&gpu, COLOR_FORMAT, STENCIL_FORMAT);

        Self {
            gpu,
            base_color,
            renderer,
            texture_allocator,
        }
    }

    pub fn device(&self) -> &wgpu::Device {
        self.gpu.device()
    }

    pub fn queue(&self) -> &wgpu::Queue {
        self.gpu.queue()
    }

    pub fn device_queue(&self) -> DeviceQueue {
        DeviceQueue {
            device: self.gpu.device(),
            queue: self.gpu.queue(),
        }
    }

    pub fn texture_allocator(&self) -> () {
        todo!()
    }

    pub fn render(
        &self,
        object: Object,
        target_view: &wgpu::TextureView,
        viewport_size: [f32; 2],
        surface_format: wgpu::TextureFormat,
    ) -> Result<(), TextureValidationError> {
        self.renderer.render(
            self.device(),
            self.queue(),
            surface_format,
            target_view,
            viewport_size,
            object,
            self.base_color,
            self.texture_allocator.color_texture(),
            self.texture_allocator.stencil_texture(),
        )
    }
}
