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
pub struct ParentPxSize {
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl ParentPxSize {
    pub fn current_size(&self, size: &Size, app: &ApplicationContext) -> ParentPxSize {
        ParentPxSize {
            width: size.width.to_px(self.width, app),
            height: size.height.to_px(self.height, app),
        }
    }
}

impl Into<ParentPxSize> for &PxSize {
    fn into(self) -> ParentPxSize {
        ParentPxSize {
            width: Some(self.width),
            height: Some(self.height),
        }
    }
}
