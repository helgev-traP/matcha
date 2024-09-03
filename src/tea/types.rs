pub struct Rgb8Uint(pub [u8; 3]);

impl Rgb8Uint {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self([r, g, b])
    }
}

pub struct Rgb64Float(pub [f64; 3]);

impl Rgb64Float {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self([r, g, b])
    }
}

pub struct Rgba8Uint(pub [u8; 4]);

impl Rgba8Uint {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self([r, g, b, a])
    }
}

pub struct Rgba64Float(pub [f64; 4]);

impl Rgba64Float {
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self([r, g, b, a])
    }
}