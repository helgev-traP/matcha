use crate::types::color::Color;

/// # Border
/// **Individual border radius are currently NOT supported.**
#[derive(Debug, Clone)]
pub struct Border {
    pub px: f32,
    pub color: Color,
    pub top_left_radius: f32,
    pub top_right_radius: f32,
    pub bottom_left_radius: f32,
    pub bottom_right_radius: f32,
}

impl Default for Border {
    fn default() -> Self {
        Self {
            px: 0.0,
            color: Color::Rgba8USrgb { r: 0, g: 0, b: 0, a: 0 },
            top_left_radius: 0.0,
            top_right_radius: 0.0,
            bottom_left_radius: 0.0,
            bottom_right_radius: 0.0,
        }
    }
}

/// **Individual border radius are currently NOT supported.**
#[macro_export] macro_rules! border {
    () => {
        Border {
            px: 0.0,
            color: Color::Rgba8USrgb { r: 0, g: 0, b: 0, a: 0 },
            top_left_radius: 0.0,
            top_right_radius: 0.0,
            bottom_left_radius: 0.0,
            bottom_right_radius: 0.0,
        }
    };

    ($px:expr, $color:expr) => {
        Border {
            px: $px,
            color: $color,
            top_left_radius: 0.0,
            top_right_radius: 0.0,
            bottom_left_radius: 0.0,
            bottom_right_radius: 0.0,
        }
    };

    ($px:expr, $color:expr, $radius:expr) => {
        Border {
            px: $px,
            color: $color,
            top_left_radius: $radius,
            top_right_radius: $radius,
            bottom_left_radius: $radius,
            bottom_right_radius: $radius,
        }
    };

    ($px:expr, $color:expr, $top_left:expr, $top_right:expr, $bottom_left:expr, $bottom_right:expr) => {
        Border {
            px: $px,
            color: $color,
            top_left_radius: $top_left,
            top_right_radius: $top_right,
            bottom_left_radius: $bottom_left,
            bottom_right_radius: $bottom_right,
        }
    };
}