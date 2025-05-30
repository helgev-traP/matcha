use num::Float;

/// This struct is used to manage the position
/// in both UI space(y-axis is downward) and
/// 3D space(y-axis is upward).
pub struct Position<T: Float> {
    /// y-axis is upward internally,
    position: [T; 3],
}

impl Position<f32> {
    pub const fn new_y_down(position: [f32; 3]) -> Self {
        Self {
            position: [
                position[0],
                -position[1], // Invert y-axis for UI space
                position[2],
            ],
        }
    }

    pub const fn new_y_up(position: [f32; 3]) -> Self {
        Self { position }
    }

    pub const fn to_y_down(&self) -> [f32; 3] {
        [
            self.position[0],
            -self.position[1], // Invert y-axis for UI space
            self.position[2],
        ]
    }

    pub const fn to_y_up(&self) -> [f32; 3] {
        self.position
    }
}

impl Position<f64> {
    pub const fn new_y_down(position: [f64; 3]) -> Self {
        Self {
            position: [
                position[0],
                -position[1], // Invert y-axis for UI space
                position[2],
            ],
        }
    }

    pub const fn new_y_up(position: [f64; 3]) -> Self {
        Self { position }
    }

    pub const fn to_y_down(&self) -> [f64; 3] {
        [
            self.position[0],
            -self.position[1], // Invert y-axis for UI space
            self.position[2],
        ]
    }

    pub const fn to_y_up(&self) -> [f64; 3] {
        self.position
    }
}
