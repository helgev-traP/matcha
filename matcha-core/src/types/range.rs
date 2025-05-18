use num::Float;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range2D<T: Float = f32> {
    x: [T; 2],
    y: [T; 2],
}

// MARK: Initialization

impl<T: Float> Range2D<T> {
    pub fn new(x: [T; 2], y: [T; 2]) -> Option<Self> {
        if x[0] > x[1] || y[0] > y[1] {
            None
        } else {
            Some(Range2D { x, y })
        }
    }

    pub const fn new_unchecked(x: [T; 2], y: [T; 2]) -> Self {
        Range2D { x, y }
    }

    pub fn left_side(mut self, x: T) -> Result<Self, Self> {
        if x <= self.x[1] {
            self.x[0] = x;
            Ok(self)
        } else {
            Err(self)
        }
    }

    pub fn right_side(mut self, x: T) -> Result<Self, Self> {
        if self.x[0] <= x {
            self.x[1] = x;
            Ok(self)
        } else {
            Err(self)
        }
    }

    pub fn top_side(mut self, y: T) -> Result<Self, Self> {
        if y <= self.y[1] {
            self.y[0] = y;
            Ok(self)
        } else {
            Err(self)
        }
    }

    pub fn bottom_side(mut self, y: T) -> Result<Self, Self> {
        if self.y[0] <= y {
            self.y[1] = y;
            Ok(self)
        } else {
            Err(self)
        }
    }
}

// MARK: Operation

impl<T: Float> Range2D<T> {
    pub fn slide(self, s: [T; 2]) -> Self {
        Range2D {
            x: [self.x[0] + s[0], self.x[1] + s[0]],
            y: [self.y[0] + s[1], self.y[1] + s[1]],
        }
    }

    pub fn reduction(self, r: T) -> Option<Self> {
        if r * T::from(2).unwrap() > self.short_side() {
            None
        } else {
            let x = [self.x[0] + r, self.x[1] - r];
            let y = [self.y[0] + r, self.y[1] - r];
            Some(Range2D { x, y })
        }
    }

    pub fn expansion(self, e: T) -> Self {
        let x = [self.x[0] - e, self.x[1] + e];
        let y = [self.y[0] - e, self.y[1] + e];
        Range2D { x, y }
    }
}

// MARK: Getter

impl<T: Float> Range2D<T> {
    pub fn width(&self) -> T {
        self.x[1] - self.x[0]
    }

    pub fn height(&self) -> T {
        self.y[1] - self.y[0]
    }

    pub fn long_side(&self) -> T {
        self.width().max(self.height())
    }

    pub fn short_side(&self) -> T {
        self.width().min(self.height())
    }

    pub fn center(&self) -> [T; 2] {
        [
            (self.x[0] + self.x[1]) / T::from(2).unwrap(),
            (self.y[0] + self.y[1]) / T::from(2).unwrap(),
        ]
    }

    pub fn contains(&self, p: [T; 2]) -> bool {
        self.x[0] <= p[0] && p[0] <= self.x[1] && self.y[0] <= p[1] && p[1] <= self.y[1]
    }

    pub fn area(&self) -> T {
        self.width() * self.height()
    }

    pub fn left(&self) -> T {
        self.x[0]
    }

    pub fn right(&self) -> T {
        self.x[1]
    }

    pub fn top(&self) -> T {
        self.y[0]
    }

    pub fn bottom(&self) -> T {
        self.y[1]
    }

    pub fn upper_left(&self) -> [T; 2] {
        [self.x[0], self.y[0]]
    }

    pub fn lower_right(&self) -> [T; 2] {
        [self.x[1], self.y[1]]
    }

    pub fn x_range(&self) -> [T; 2] {
        self.x
    }

    pub fn y_range(&self) -> [T; 2] {
        self.y
    }
}

// MARK: Utils

impl<T: Float> Range2D<T> {
    pub fn interpolate(value: Self, mix: Self) -> Self {
        Range2D {
            x: [
                value.x[0] + (value.x[1] - value.x[0]) * mix.x[0],
                value.x[0] + (value.x[1] - value.x[0]) * mix.x[1],
            ],
            y: [
                value.y[0] + (value.y[1] - value.y[0]) * mix.y[0],
                value.y[0] + (value.y[1] - value.y[0]) * mix.y[1],
            ],
        }
    }
}

// MARK: CoverRange

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct CoverRange<T: Float = f32> {
    pub inscribed: Option<Range2D<T>>,
    pub circumscribed: Option<Range2D<T>>,
}

impl<T: Float> From<[Option<Range2D<T>>; 2]> for CoverRange<T> {
    fn from(ranges: [Option<Range2D<T>>; 2]) -> Self {
        CoverRange {
            inscribed: ranges[0],
            circumscribed: ranges[1],
        }
    }
}

impl<T: Float> CoverRange<T> {
    pub fn new(inscribed: Option<Range2D<T>>, circumscribed: Option<Range2D<T>>) -> Self {
        CoverRange {
            inscribed,
            circumscribed,
        }
    }

    pub fn inscribed(&self) -> Option<Range2D<T>> {
        self.inscribed
    }

    pub fn circumscribed(&self) -> Option<Range2D<T>> {
        self.circumscribed
    }
}

impl<T: Float> CoverRange<T> {
    pub fn slide(self, s: [T; 2]) -> Self {
        CoverRange {
            inscribed: self.inscribed.map(|r| r.slide(s)),
            circumscribed: self.circumscribed.map(|r| r.slide(s)),
        }
    }
}
