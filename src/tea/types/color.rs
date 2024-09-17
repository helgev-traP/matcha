pub enum Color {
    Rgb8USrgb { r: u8, g: u8, b: u8 },
    Rgba8USrgb { r: u8, g: u8, b: u8, a: u8 },
    RgbF32 { r: f32, g: f32, b: f32 },
    RgbaF32 { r: f32, g: f32, b: f32, a: f32 },
    RgbF64 { r: f64, g: f64, b: f64 },
    RgbaF64 { r: f64, g: f64, b: f64, a: f64 },
}

impl Default for Color {
    fn default() -> Self {
        Color::Rgb8USrgb { r: 0, g: 0, b: 0 }
    }
}

// a macro to convert integer to float
macro_rules! linear_convert {
    ($x:expr, $type:ty) => {
        if *$x as $type / 255.0 <= 0.04045 {
            *$x as $type / 255.0 / 12.92
        } else {
            ((*$x as $type / 255.0 + 0.055) / 1.055).powf(2.4)
        }
    };
}

impl Color {
    pub fn to_rgb_f32(&self) -> [f32; 3] {
        match self {
            Color::Rgb8USrgb { r, g, b } | Color::Rgba8USrgb { r, g, b, .. } => [
                linear_convert!(r, f32),
                linear_convert!(g, f32),
                linear_convert!(b, f32),
            ],
            Color::RgbF32 { r, g, b } | Color::RgbaF32 { r, g, b, .. } => [*r, *g, *b],
            Color::RgbF64 { r, g, b } | Color::RgbaF64 { r, g, b, .. } => {
                [*r as f32, *g as f32, *b as f32]
            }
        }
    }

    pub fn to_rgba_f32(&self) -> [f32; 4] {
        match self {
            Color::Rgb8USrgb { r, g, b } => [
                linear_convert!(r, f32),
                linear_convert!(g, f32),
                linear_convert!(b, f32),
                1.0,
            ],
            Color::Rgba8USrgb { r, g, b, a } => [
                linear_convert!(r, f32),
                linear_convert!(g, f32),
                linear_convert!(b, f32),
                *a as f32 / 255.0,
            ],
            Color::RgbF32 { r, g, b } => [*r, *g, *b, 1.0],
            Color::RgbaF32 { r, g, b, a } => [*r, *g, *b, *a],
            Color::RgbF64 { r, g, b } => [*r as f32, *g as f32, *b as f32, 1.0],
            Color::RgbaF64 { r, g, b, a } => [*r as f32, *g as f32, *b as f32, *a as f32],
        }
    }

    pub fn to_rgb_f64(&self) -> [f64; 3] {
        match self {
            Color::Rgb8USrgb { r, g, b } | Color::Rgba8USrgb { r, g, b, .. } => [
                linear_convert!(r, f64),
                linear_convert!(g, f64),
                linear_convert!(b, f64),
            ],
            Color::RgbF32 { r, g, b } | Color::RgbaF32 { r, g, b, .. } => [*r as f64, *g as f64, *b as f64],
            Color::RgbF64 { r, g, b } | Color::RgbaF64 { r, g, b, .. } => [*r, *g, *b],
        }
    }

    pub fn to_rgba_f64(&self) -> [f64; 4] {
        match self {
            Color::Rgb8USrgb { r, g, b } => [
                linear_convert!(r, f64),
                linear_convert!(g, f64),
                linear_convert!(b, f64),
                1.0,
            ],
            Color::Rgba8USrgb { r, g, b, a } => [
                linear_convert!(r, f64),
                linear_convert!(g, f64),
                linear_convert!(b, f64),
                *a as f64 / 255.0,
            ],
            Color::RgbF32 { r, g, b } => [*r as f64, *g as f64, *b as f64, 1.0],
            Color::RgbaF32 { r, g, b, a } => [*r as f64, *g as f64, *b as f64, *a as f64],
            Color::RgbF64 { r, g, b } => [*r, *g, *b, 1.0],
            Color::RgbaF64 { r, g, b, a } => [*r, *g, *b, *a],
        }
    }
}
