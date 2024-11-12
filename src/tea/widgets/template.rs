use crate::{
    context::SharedContext,
    events::UiEvent,
    types::size::{PxSize, Size, SizeUnit},
    ui::{Dom, DomComPareResult, TextureSet, Widget},
};

pub struct TemplateDescriptor {
    pub label: Option<String>,
    pub size: Size,
}

impl Default for TemplateDescriptor {
    fn default() -> Self {
        Self {
            label: None,
            size: Size {
                width: SizeUnit::Pixel(100.0),
                height: SizeUnit::Pixel(100.0),
            },
        }
    }
}

pub struct Template {
    label: Option<String>,
    size: Size,
}

impl Template {
    pub fn new(disc: TemplateDescriptor) -> Self {
        Self {
            label: disc.label,
            size: disc.size,
        }
    }
}

impl<T: Send + 'static> Dom<T> for Template {
    fn build_render_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(TemplateNode {
            label: self.label.clone(),
            size: self.size,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct TemplateNode {
    label: Option<String>,
    size: Size,
}

impl<T: Send + 'static> Widget<T> for TemplateNode {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: PxSize,
        context: &SharedContext,
    ) -> crate::events::UiEventResult<T> {
        todo!()
    }

    fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: PxSize,
        context: &SharedContext,
    ) -> bool {
        todo!()
    }

    fn update_render_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<Template>() {
            Err(())
        } else {
            let dom = dom.as_any().downcast_ref::<Template>().unwrap();
            todo!()
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(_) = dom.as_any().downcast_ref::<Template>() {
            todo!()
        } else {
            DomComPareResult::Different
        }
    }

    fn size(&self) -> Size {
        self.size
    }

    fn px_size(&self, parent_size: PxSize, context: &SharedContext) -> PxSize {
        self.size.to_px(parent_size, context)
    }

    fn default_size(&self) -> PxSize {
        PxSize {
            width: 0.0,
            height: 0.0,
        }
    }

    fn render(
        &mut self,
        texture: Option<&TextureSet>,
        parent_size: PxSize,
        affine: nalgebra::Matrix4<f32>,
        context: &SharedContext,
    )
    {
        todo!()
    }
}
