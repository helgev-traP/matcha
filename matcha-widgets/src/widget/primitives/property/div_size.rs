use matcha_core::context::WidgetContext;

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
    pub fn to_std_div_size(
        &self,
        parent_px_size: Option<f32>,
        context: &WidgetContext,
    ) -> StdDivSize {
        match self {
            DivSize::Pixel(x) => StdDivSize::Pixel(*x),
            DivSize::Inch(x) => StdDivSize::Pixel(*x * context.dpi() as f32),
            DivSize::Point(x) => StdDivSize::Pixel(*x * context.dpi() as f32 / 72.0),
            DivSize::Parent(x) => match parent_px_size {
                Some(px) => StdDivSize::Pixel(px * x),
                None => StdDivSize::Pixel(0.0),
            },
            DivSize::Em(_) => todo!(),
            DivSize::Rem(_) => todo!(),
            DivSize::Vw(x) => StdDivSize::Pixel(*x * context.viewport_size()[0] as f32),
            DivSize::Vh(x) => StdDivSize::Pixel(*x * context.viewport_size()[1] as f32),
            DivSize::VMin(x) => StdDivSize::Pixel(
                *x * context.viewport_size()[0].min(context.viewport_size()[1]) as f32,
            ),
            DivSize::VMax(x) => StdDivSize::Pixel(
                *x * context.viewport_size()[0].max(context.viewport_size()[1]) as f32,
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
