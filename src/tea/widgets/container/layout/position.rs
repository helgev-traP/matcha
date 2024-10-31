use nalgebra as na;

#[derive(Debug, Clone)]
pub enum Position {
    Static,
    Relative(na::Matrix3<f32>),
    Absolute(na::Matrix3<f32>),
    Fixed(na::Matrix3<f32>),
    // Sticky,
}

impl Default for Position {
    fn default() -> Self {
        Self::Static
    }
}