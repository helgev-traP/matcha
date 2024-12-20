use nalgebra as na;

use crate::context::SharedContext;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
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

    // Relative units that can not calculate to pixel from shared context or parent size
    Content(f32),
}

impl Size {
    /// standardize `Size` to `StdSize` with dependent on parent `StdSize`.
    pub fn to_std_size(&self, parent_px_size: StdSize, app_context: &SharedContext) -> StdSize {
        match self {
            Size::Pixel(x) => StdSize::Pixel(*x),
            Size::Inch(x) => StdSize::Pixel(*x * app_context.get_dpi() as f32),
            Size::Point(x) => StdSize::Pixel(*x * app_context.get_dpi() as f32 / 72.0),
            Size::Parent(x) => match parent_px_size {
                StdSize::Pixel(px) => StdSize::Pixel(px * x),
                StdSize::Content(_) => StdSize::Content(1.0), // todo: consider is this correct?
            },
            Size::Em(_) => todo!(),
            Size::Rem(_) => todo!(),
            Size::Vw(x) => StdSize::Pixel(*x * app_context.get_viewport_size().0 as f32),
            Size::Vh(x) => StdSize::Pixel(*x * app_context.get_viewport_size().1 as f32),
            Size::VMin(x) => StdSize::Pixel(
                *x * app_context
                    .get_viewport_size()
                    .0
                    .min(app_context.get_viewport_size().1) as f32,
            ),
            Size::VMax(x) => StdSize::Pixel(
                *x * app_context
                    .get_viewport_size()
                    .0
                    .max(app_context.get_viewport_size().1) as f32,
            ),
            Size::Content(x) => StdSize::Content(*x),
        }
    }
}

impl<T> From<T> for Size
where
    T: Into<f32>,
{
    fn from(x: T) -> Self {
        Size::Pixel(x.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StdSize {
    Pixel(f32),
    Content(f32),
}

impl StdSize {
    pub fn unwrap_as_pixel(self) -> f32 {
        match self {
            StdSize::Pixel(x) => x,
            _ => panic!("unwrap failed: not pixel"),
        }
    }

    pub fn unwrap_as_percent(self) -> f32 {
        match self {
            StdSize::Content(x) => x,
            _ => panic!("unwrap failed: not percent"),
        }
    }
}

impl StdSize {
    /// Convert `StdSize` to pixel size with content size(Option<f32>).
    pub fn from_content_size_to_px(&self, content_size: Option<f32>) -> Option<f32> {
        match self {
            StdSize::Pixel(x) => Some(*x),
            StdSize::Content(x) => content_size.map(|content_size| content_size * x),
        }
    }
}

impl<T> From<T> for StdSize
where
    T: Into<f32>,
{
    fn from(x: T) -> Self {
        StdSize::Pixel(x.into())
    }
}

pub fn make_normalize_matrix<T>(size: T) -> na::Matrix4<f32>
where
    T: Into<[f32; 2]>,
{
    let size = size.into();
    na::Matrix4::new(
        // -
        2.0 / size[0],
        0.0,
        0.0,
        -1.0,
        // -
        0.0,
        2.0 / size[1],
        0.0,
        1.0,
        // -
        0.0,
        0.0,
        1.0,
        0.0,
        // -
        0.0,
        0.0,
        0.0,
        1.0,
    )
}
