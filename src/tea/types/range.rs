#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range2D<T: Copy> {
    pub x: [T; 2],
    pub y: [T; 2],
}

impl<T: Copy> From<[[T; 2]; 2]> for Range2D<T> {
    fn from(range: [[T; 2]; 2]) -> Self {
        Range2D {
            x: range[0],
            y: range[1],
        }
    }
}

impl<T: Copy> Range2D<T> {
    /// [x, y]
    pub fn upper_left(&self) -> [T; 2] {
        [self.x[0], self.y[0]]
    }

    /// [x, y]
    pub fn lower_right(&self) -> [T; 2] {
        [self.x[1], self.y[1]]
    }
}
