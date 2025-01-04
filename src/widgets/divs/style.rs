use crate::types::color::Color;

#[derive(Debug, Clone, Copy, Default)]
pub struct Margin {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Margin {
    pub fn new_each(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Margin {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn new(px: f32) -> Self {
        Margin {
            top: px,
            right: px,
            bottom: px,
            left: px,
        }
    }

    pub fn vertical(mut self, px: f32) -> Self {
        self.top = px;
        self.bottom = px;
        self
    }

    pub fn horizontal(mut self, px: f32) -> Self {
        self.right = px;
        self.left = px;
        self
    }

    pub fn top(mut self, px: f32) -> Self {
        self.top = px;
        self
    }

    pub fn right(mut self, px: f32) -> Self {
        self.right = px;
        self
    }

    pub fn bottom(mut self, px: f32) -> Self {
        self.bottom = px;
        self
    }

    pub fn left(mut self, px: f32) -> Self {
        self.left = px;
        self
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Padding {
    pub fn new_each(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Padding {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn new(px: f32) -> Self {
        Padding {
            top: px,
            right: px,
            bottom: px,
            left: px,
        }
    }

    pub fn vertical(mut self, px: f32) -> Self {
        self.top = px;
        self.bottom = px;
        self
    }

    pub fn horizontal(mut self, px: f32) -> Self {
        self.right = px;
        self.left = px;
        self
    }

    pub fn top(mut self, px: f32) -> Self {
        self.top = px;
        self
    }

    pub fn right(mut self, px: f32) -> Self {
        self.right = px;
        self
    }

    pub fn bottom(mut self, px: f32) -> Self {
        self.bottom = px;
        self
    }

    pub fn left(mut self, px: f32) -> Self {
        self.left = px;
        self
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Border {
    pub px: f32,
    pub color: Color,
    pub top_left_radius: f32,
    pub top_right_radius: f32,
    pub bottom_left_radius: f32,
    pub bottom_right_radius: f32,
}

impl Border {
    pub fn new(px: f32, color: Color) -> Self {
        Border {
            px,
            color,
            ..Default::default()
        }
    }

    pub fn radius(mut self, r: f32) -> Self {
        self.top_left_radius = r;
        self.top_right_radius = r;
        self.bottom_left_radius = r;
        self.bottom_right_radius = r;
        self
    }

    pub fn top_radius(mut self, r: f32) -> Self {
        self.top_left_radius = r;
        self.top_right_radius = r;
        self
    }

    pub fn bottom_radius(mut self, r: f32) -> Self {
        self.bottom_left_radius = r;
        self.bottom_right_radius = r;
        self
    }

    pub fn left_radius(mut self, r: f32) -> Self {
        self.top_left_radius = r;
        self.bottom_left_radius = r;
        self
    }

    pub fn right_radius(mut self, r: f32) -> Self {
        self.top_right_radius = r;
        self.bottom_right_radius = r;
        self
    }

    pub fn top_left_radius(mut self, r: f32) -> Self {
        self.top_left_radius = r;
        self
    }

    pub fn top_right_radius(mut self, r: f32) -> Self {
        self.top_right_radius = r;
        self
    }

    pub fn bottom_left_radius(mut self, r: f32) -> Self {
        self.bottom_left_radius = r;
        self
    }

    pub fn bottom_right_radius(mut self, r: f32) -> Self {
        self.bottom_right_radius = r;
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BoxSizing {
    ContentBox,
    BorderBox,
}

impl Default for BoxSizing {
    fn default() -> Self {
        BoxSizing::BorderBox
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    Visible,
    Hidden,
    None,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Visible
    }
}
