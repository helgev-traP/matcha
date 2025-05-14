use matcha_core::{
    context::WidgetContext, renderer::Renderer, vertex::colored_vertex::ColorVertex
};

// todo: more documentation

// MARK: DOM

pub struct SolidBox {
    label: Option<String>,

    // box settings
    radius: f32,

    // background settings
    background_color: [f32; 4],
    background_blur: f32,

    // border settings
    border_width: f32,
    border_color: [f32; 4],
    border_blur: f32,

    // render context
    vertices: Option<Vec<ColorVertex>>,
    indices: Option<Vec<u16>>,
}

impl SolidBox {
    pub fn new(label: Option<&str>) -> Box<Self> {
        Box::new(Self {
            label: label.map(|s| s.to_string()),
            radius: 0.0,
            background_color: [0.0, 0.0, 0.0, 0.0],
            background_blur: 0.0,
            border_width: 0.0,
            border_color: [0.0, 0.0, 0.0, 0.0],
            border_blur: 0.0,
            vertices: None,
            indices: None,
        })
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn background_color(mut self, color: [f32; 4]) -> Self {
        self.background_color = color;
        self
    }

    pub fn background_blur(mut self, blur: f32) -> Self {
        self.background_blur = blur;
        self
    }

    pub fn border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    pub fn border_color(mut self, color: [f32; 4]) -> Self {
        self.border_color = color;
        self
    }

    pub fn border_blur(mut self, blur: f32) -> Self {
        self.border_blur = blur;
        self
    }
}

impl SolidBox {
    pub fn render(target: wgpu::TextureView, renderer: &Renderer) {
        todo!()
    }
}



