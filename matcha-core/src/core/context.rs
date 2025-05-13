#[derive(Clone)]
pub struct WidgetContext<'a> {
    global_context: &'a super::window::gpu_state::GlobalContext<'a>,
    root_font_size: f32,
    font_size: f32,
}

impl<'a> WidgetContext<'a> {
    pub(crate) fn new(
        global_context: &'a super::window::gpu_state::GlobalContext,
        font_size: f32,
    ) -> Self {
        Self {
            global_context,
            root_font_size: font_size,
            font_size,
        }
    }

    pub fn device(&self) -> &wgpu::Device {
        self.global_context.device()
    }

    pub fn queue(&self) -> &wgpu::Queue {
        self.global_context.queue()
    }

    pub fn make_encoder(&self) -> wgpu::CommandEncoder {
        self.global_context
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Widget Context Command Encoder"),
            })
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.global_context.surface_format()
    }

    pub fn dpi(&self) -> f64 {
        self.global_context.dpi()
    }

    pub fn viewport_size(&self) -> [u32; 2] {
        self.global_context.viewport_size()
    }
}

impl WidgetContext<'_> {
    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    pub fn root_font_size(&self) -> f32 {
        self.root_font_size
    }
}

impl WidgetContext<'_> {
    pub fn with_font_size(&self, font_size: f32) -> Self {
        Self {
            global_context: self.global_context,
            root_font_size: self.root_font_size,
            font_size,
        }
    }
}
