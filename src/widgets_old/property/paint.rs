use crate::types::color::Color;

#[derive(Debug, Clone, Copy)]
pub enum Paint {
    Blur(f32),
    Solid(Color),
    // Gradient(Gradient),
    // Image(Image),
}

impl Paint {
    pub fn is_opaque(&self) -> bool {
        match self {
            Paint::Blur(px) => *px == 0.0,
            Paint::Solid(color) => color.is_opaque(),
        }
    }
}
