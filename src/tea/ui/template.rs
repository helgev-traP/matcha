use crate::{
    application_context::ApplicationContext,
    events::WidgetEvent,
    render::RenderCommandEncoder,
    types::size::{PxSize, Size},
};

use super::{Dom, Widget};

pub struct Template {
    size: Size,
}

impl<R: Send + 'static> Dom<R> for Template {
    fn build_render_tree(&self) -> Box<dyn Widget<R>> {
        Box::new(TemplateRenderNode {
            size: self.size,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct TemplateRenderNode {
    size: Size,
}

impl<R: Send + 'static> super::WidgetTrait<R> for TemplateRenderNode {
    fn widget_event(
        &self,
        event: &WidgetEvent,
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> crate::events::WidgetEventResult<R> {
        todo!()
    }

    fn update_render_tree(&mut self, dom: &dyn Dom<R>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<Template>() {
            return Err(());
        }

        let dom = dom.as_any().downcast_ref::<Template>().unwrap();

        todo!()
    }

    fn compare(&self, dom: &dyn Dom<R>) -> super::DomComPareResult {
        if let Some(_) = dom.as_any().downcast_ref::<Template>() {
            todo!()
        } else {
            super::DomComPareResult::Different
        }
    }
}

impl super::RenderingTrait for TemplateRenderNode {
    fn size(&self) -> Size {
        self.size
    }

    fn px_size(&self, parent_size: PxSize, context: &ApplicationContext) -> PxSize {
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
        s: &rayon::Scope,
        parent_size: PxSize,
        affine: nalgebra::Matrix4<f32>,
        encoder: &mut RenderCommandEncoder,
    ) {
        todo!()
    }
}
