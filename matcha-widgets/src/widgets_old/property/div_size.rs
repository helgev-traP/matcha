use crate::{context::SharedContext, types::size::StdSize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DivSize {
    // Absolute units
    Pixel(f32),
    Inch(f32),
    Point(f32),

    // Relative units that can calculate to pixel from shared context or parent size
    Parent(f32),
    Em(f32),
    Rem(f32),
    Vw(f32),
    Vh(f32),
    VMin(f32),
    VMax(f32),

    // filling space
    Grow(f32),
}

impl DivSize {
    pub fn to_std_size(&self, parent_px_size: StdSize, app_context: &SharedContext) -> StdDivSize {
        match self {
            DivSize::Pixel(x) => StdDivSize::Pixel(*x),
            DivSize::Inch(x) =>  StdDivSize::Pixel(*x * app_context.get_dpi() as f32),
            DivSize::Point(x) => StdDivSize::Pixel(*x * app_context.get_dpi() as f32 / 72.0),
            DivSize::Parent(x) => match parent_px_size {
                StdSize::Pixel(px) => StdDivSize::Pixel(px * x),
                StdSize::Content(_) => StdDivSize::Pixel(0.0), // todo: care about this in the future
            },
            DivSize::Em(_) => todo!(),
            DivSize::Rem(_) => todo!(),
            DivSize::Vw(x) => StdDivSize::Pixel(*x * app_context.get_viewport_size().0 as f32),
            DivSize::Vh(x) => StdDivSize::Pixel(*x * app_context.get_viewport_size().1 as f32),
            DivSize::VMin(x) => StdDivSize::Pixel(
                *x * app_context
                    .get_viewport_size()
                    .0
                    .min(app_context.get_viewport_size().1) as f32,
            ),
            DivSize::VMax(x) => StdDivSize::Pixel(
                *x * app_context
                    .get_viewport_size()
                    .0
                    .max(app_context.get_viewport_size().1) as f32,
            ),
            DivSize::Grow(x) => StdDivSize::Grow(*x),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StdDivSize {
    Pixel(f32),
    Grow(f32),
}
