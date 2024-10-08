use crate::application_context::ApplicationContext;

#[derive(Debug, Clone, Copy)]
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
        parent_px_size: Option<f32>,
        app_context: &ApplicationContext,
    ) -> Option<f32> {
        let parent_px_size = parent_px_size?;
        match self {
            SizeUnit::Pixel(x) => Some(*x),
            SizeUnit::Inch(x) => Some(*x * app_context.get_dpi() as f32),
            SizeUnit::Point(x) => Some(*x * app_context.get_dpi() as f32 / 72.0),
            SizeUnit::Percent(x) => Some(*x * parent_px_size / 100.0),
            SizeUnit::Em(_) => todo!(),
            SizeUnit::Rem(_) => todo!(),
            SizeUnit::Vw(x) => Some(*x * app_context.get_viewport_size().0 as f32 / 100.0),
            SizeUnit::Vh(x) => Some(*x * app_context.get_viewport_size().1 as f32 / 100.0),
            SizeUnit::VMin(x) => Some(
                *x * app_context
                    .get_viewport_size()
                    .0
                    .min(app_context.get_viewport_size().1) as f32
                    / 100.0,
            ),
            SizeUnit::VMax(x) => Some(
                *x * app_context
                    .get_viewport_size()
                    .0
                    .max(app_context.get_viewport_size().1) as f32
                    / 100.0,
            ),
            SizeUnit::Content(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: SizeUnit,
    pub height: SizeUnit,
}

#[derive(Clone, Copy, Debug)]
pub struct PxSize {
    pub width: f32,
    pub height: f32,
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

#[derive(Clone, Copy, Debug)]
pub struct OptionPxSize {
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl OptionPxSize {
    pub fn current_size(&self, size: &Size, app: &ApplicationContext) -> OptionPxSize {
        OptionPxSize {
            width: size.width.to_px(self.width, app),
            height: size.height.to_px(self.height, app),
        }
    }

    pub fn from_size(size: Size, app: &ApplicationContext) -> OptionPxSize {
        OptionPxSize {
            width: size.width.to_px(None, app),
            height: size.height.to_px(None, app),
        }
    }

    pub fn from_parent_size(size: Size, parent_size: OptionPxSize, app: &ApplicationContext) -> OptionPxSize {
        OptionPxSize {
            width: size.width.to_px(parent_size.width, app),
            height: size.height.to_px(parent_size.height, app),
        }
    }
}

impl From<PxSize> for OptionPxSize {
    fn from(size: PxSize) -> Self {
        OptionPxSize {
            width: Some(size.width),
            height: Some(size.height),
        }
    }
}

impl From<(Size, &ApplicationContext)> for OptionPxSize {
    fn from(size: (Size, &ApplicationContext)) -> Self {
        OptionPxSize {
            width: size.0.width.to_px(None, &size.1),
            height: size.0.height.to_px(None, &size.1),
        }
    }
}
