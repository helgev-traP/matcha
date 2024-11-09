#[derive(Debug, Clone,)]
pub struct Margin {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Default for Margin {
    fn default() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }
}

#[macro_export] macro_rules! margin {
    ($all:expr) => {
        Margin {
            top: $all,
            right: $all,
            bottom: $all,
            left: $all,
        }
    };

    ($vertical:expr, $horizontal:expr) => {
        Margin {
            top: $vertical,
            right: $horizontal,
            bottom: $vertical,
            left: $horizontal,
        }
    };

    ($top:expr, $horizontal:expr, $bottom:expr) => {
        Margin {
            top: $top,
            right: $horizontal,
            bottom: $bottom,
            left: $horizontal,
        }
    };

    ($top:expr, $right:expr, $bottom:expr, $left:expr) => {
        Margin {
            top: $top,
            right: $right,
            bottom: $bottom,
            left: $left,
        }
    };
}