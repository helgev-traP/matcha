#[derive(Debug, Clone, Copy)]
pub struct Margin {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Border {
    pub px: f32,
    pub color: [u8; 4],
    pub top_left_radius: f32,
    pub top_right_radius: f32,
    pub bottom_left_radius: f32,
    pub bottom_right_radius: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum BoxSizing {
    ContentBox,
    BorderBox,
}

#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    Visible,
    Hidden,
    None,
}
