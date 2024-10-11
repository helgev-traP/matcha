use nalgebra as na;

use crate::application_context::ApplicationContext;

#[derive(Debug, Clone, Copy, PartialEq)]
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
    Content(f32),
}

impl SizeUnit {
    pub fn to_px(
        &self,
        parent_px_size: StdSizeUnit,
        app_context: &ApplicationContext,
    ) -> StdSizeUnit {
        match self {
            SizeUnit::Pixel(x) => StdSizeUnit::Pixel(*x),
            SizeUnit::Inch(x) => StdSizeUnit::Pixel(*x * app_context.get_dpi() as f32),
            SizeUnit::Point(x) => StdSizeUnit::Pixel(*x * app_context.get_dpi() as f32 / 72.0),
            SizeUnit::Percent(x) => match parent_px_size {
                StdSizeUnit::Pixel(px) => StdSizeUnit::Pixel(px * x / 100.0),
                _ => StdSizeUnit::None,
            },
            SizeUnit::Em(_) => todo!(),
            SizeUnit::Rem(_) => todo!(),
            SizeUnit::Vw(x) => {
                StdSizeUnit::Pixel(*x * app_context.get_viewport_size().0 as f32 / 100.0)
            }
            SizeUnit::Vh(x) => {
                StdSizeUnit::Pixel(*x * app_context.get_viewport_size().1 as f32 / 100.0)
            }
            SizeUnit::VMin(x) => StdSizeUnit::Pixel(
                *x * app_context
                    .get_viewport_size()
                    .0
                    .min(app_context.get_viewport_size().1) as f32
                    / 100.0,
            ),
            SizeUnit::VMax(x) => StdSizeUnit::Pixel(
                *x * app_context
                    .get_viewport_size()
                    .0
                    .max(app_context.get_viewport_size().1) as f32
                    / 100.0,
            ),
            SizeUnit::Content(_) => StdSizeUnit::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: SizeUnit,
    pub height: SizeUnit,
}

#[derive(Clone, Copy, Debug)]
pub struct PxSize {
    pub width: f32,
    pub height: f32,
}

impl PxSize {
    pub fn from_size_parent_size(
        size: Size,
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> PxSize {
        PxSize {
            width: size
                .width
                .to_px(StdSizeUnit::Pixel(parent_size.width), context)
                .unwrap_as_pixel(),
            height: size
                .height
                .to_px(StdSizeUnit::Pixel(parent_size.height), context)
                .unwrap_as_pixel(),
        }
    }

    pub fn make_normalizer(&self) -> na::Matrix4<f32> {
        na::Matrix4::new(
            // -
            2.0 / self.width,
            0.0,
            0.0,
            -1.0,
            // -
            0.0,
            2.0 / self.height,
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
}

impl<T> From<[T; 2]> for PxSize
where
    T: Into<f32>,
{
    fn from([width, height]: [T; 2]) -> Self {
        PxSize {
            width: width.into(),
            height: height.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StdSizeUnit {
    None,
    Pixel(f32),
    Percent(f32),
}

impl StdSizeUnit {
    pub fn is_none(&self) -> bool {
        match self {
            StdSizeUnit::None => true,
            _ => false,
        }
    }

    pub fn is_pixel(&self) -> bool {
        match self {
            StdSizeUnit::Pixel(_) => true,
            _ => false,
        }
    }

    pub fn is_percent(&self) -> bool {
        match self {
            StdSizeUnit::Percent(_) => true,
            _ => false,
        }
    }
}

impl StdSizeUnit {
    pub fn unwrap_as_pixel(self) -> f32 {
        match self {
            StdSizeUnit::Pixel(x) => x,
            _ => panic!("unwrap failed: not pixel"),
        }
    }

    pub fn unwrap_as_percent(self) -> f32 {
        match self {
            StdSizeUnit::Percent(x) => x,
            _ => panic!("unwrap failed: not percent"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StdSize {
    pub width: StdSizeUnit,
    pub height: StdSizeUnit,
}

// todo v---- ここから

impl StdSize {
    pub fn standardize(&self, parent_size: &Size, context: &ApplicationContext) -> StdSize {
        StdSize {
            width: parent_size.width.to_px(self.width, context),
            height: parent_size.height.to_px(self.height, context),
        }
    }

    pub fn from_size(size: Size, context: &ApplicationContext) -> StdSize {
        StdSize {
            width: size.width.to_px(StdSizeUnit::None, context),
            height: size.height.to_px(StdSizeUnit::None, context),
        }
    }

    pub fn from_parent_size(
        size: Size,
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> StdSize {
        StdSize {
            width: size
                .width
                .to_px(StdSizeUnit::Pixel(parent_size.width), context),
            height: size
                .height
                .to_px(StdSizeUnit::Pixel(parent_size.height), context),
        }
    }

    pub fn unwrap(self) -> PxSize {
        PxSize {
            width: self.width.unwrap_as_pixel(),
            height: self.height.unwrap_as_pixel(),
        }
    }
}

impl From<PxSize> for StdSize {
    fn from(size: PxSize) -> Self {
        StdSize {
            width: StdSizeUnit::Pixel(size.width),
            height: StdSizeUnit::Pixel(size.height),
        }
    }
}

impl From<(Size, &ApplicationContext)> for StdSize {
    fn from(size: (Size, &ApplicationContext)) -> Self {
        StdSize {
            width: size.0.width.to_px(StdSizeUnit::None, &size.1),
            height: size.0.height.to_px(StdSizeUnit::None, &size.1),
        }
    }
}
