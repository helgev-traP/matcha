#[derive(Clone)]
pub struct WidgetContext<'a> {
    global_context: &'a super::window::gpu_context::GpuContext<'a>,
    root_font_size: f32,
    font_size: f32,
}

impl<'a> WidgetContext<'a> {
    pub(crate) const fn new(
        global_context: &'a super::window::gpu_context::GpuContext,
        font_size: f32,
    ) -> Self {
        Self {
            global_context,
            root_font_size: font_size,
            font_size,
        }
    }

    pub const fn device(&self) -> &wgpu::Device {
        self.global_context.device()
    }

    pub const fn queue(&self) -> &wgpu::Queue {
        self.global_context.queue()
    }

    pub fn make_encoder(&self) -> wgpu::CommandEncoder {
        self.global_context
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder created by WidgetContext"),
            })
    }

    pub const fn common_resource(&self) -> &super::common_resource::CommonResource {
        self.global_context.common_resource()
    }

    pub const fn surface_format(&self) -> wgpu::TextureFormat {
        self.global_context.surface_format()
    }

    pub const fn texture_format(&self) -> wgpu::TextureFormat {
        self.global_context.texture_format()
    }

    pub fn dpi(&self) -> f64 {
        self.global_context.dpi()
    }

    pub const fn viewport_size(&self) -> [u32; 2] {
        self.global_context.viewport_size()
    }
}

impl WidgetContext<'_> {
    pub const fn font_size(&self) -> f32 {
        self.font_size
    }

    pub const fn root_font_size(&self) -> f32 {
        self.root_font_size
    }
}

impl WidgetContext<'_> {
    pub const fn with_font_size(&self, font_size: f32) -> Self {
        Self {
            global_context: self.global_context,
            root_font_size: self.root_font_size,
            font_size,
        }
    }
}
