pub mod color;
pub use color::Color;

use super::application_context::ApplicationContext;

// size
pub enum SizeUnit {
    // Absolute units
    Pixel(f32),
    Inch(f32),
    Point(f32),

    // Relative units
    Percent(f32),
    Em(f32),
    Rem(f32),
    Vw(f32),
    Vh(f32),
    VMin(f32),
    VMax(f32),
}

impl SizeUnit {
    pub fn to_px(&self, parent_px_size: f32, app_context: ApplicationContext) -> f32 {
        match self {
            SizeUnit::Pixel(x) => *x,
            SizeUnit::Inch(x) => *x * app_context.get_dpi() as f32,
            SizeUnit::Point(x) => *x * app_context.get_dpi() as f32 / 72.0,
            SizeUnit::Percent(x) => *x * parent_px_size / 100.0,
            SizeUnit::Em(_) => todo!(),
            SizeUnit::Rem(_) => todo!(),
            SizeUnit::Vw(x) => *x * app_context.get_viewport_size().0 as f32 / 100.0,
            SizeUnit::Vh(x) => *x * app_context.get_viewport_size().1 as f32 / 100.0,
            SizeUnit::VMin(x) => {
                *x * app_context
                    .get_viewport_size()
                    .0
                    .min(app_context.get_viewport_size().1) as f32
                    / 100.0
            }
            SizeUnit::VMax(x) => {
                *x * app_context
                    .get_viewport_size()
                    .0
                    .max(app_context.get_viewport_size().1) as f32
                    / 100.0
            }
        }
    }
}

pub struct Size {
    pub width: SizeUnit,
    pub height: SizeUnit,
}

#[derive(Clone, Copy, Debug)]
pub struct PxSize {
    pub width: f32,
    pub height: f32,
}
