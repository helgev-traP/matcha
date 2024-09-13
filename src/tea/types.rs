// Colors

pub struct Rgba8Uint {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub struct Rgba64Float {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

// size
pub enum SizeUnit {
    Pixel(f32),
    Percent(f32),
    WindowPercent(f32),
}

#[derive(Clone, Copy, Debug)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}