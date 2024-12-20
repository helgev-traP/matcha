use std::sync::Arc;

use crate::{
    context::SharedContext,
    events::UiEvent,
    renderer::Renderer,
    types::size::{Size, StdSize},
    ui::{Dom, DomComPareResult, Widget},
    vertex::uv_vertex::UvVertex,
};

pub struct TemplateDescriptor {
    pub label: Option<String>,
    pub size: [Size; 2],
}

impl Default for TemplateDescriptor {
    fn default() -> Self {
        Self {
            label: None,
            size: [Size::Pixel(100.0), Size::Pixel(100.0)],
        }
    }
}

pub struct Template {
    label: Option<String>,
    size: [Size; 2],
}

impl Template {
    pub fn new(disc: TemplateDescriptor) -> Box<Self> {
        Box::new(Self {
            label: disc.label,
            size: disc.size,
        })
    }
}

impl<T: Send + 'static> Dom<T> for Template {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
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
    size: [Size; 2],
}

impl<T: Send + 'static> Widget<T> for TemplateNode {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn update_widget_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), ()> {
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

    fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> bool {
        todo!()
    }

    fn size(&self) -> [Size; 2] {
        self.size
    }

    fn px_size(&self, parent_size: [StdSize; 2], context: &SharedContext) -> [f32; 2] {
        // todo !
        todo!()
    }

    fn default_size(&self) -> [f32; 2] {
        [0.0, 0.0]
    }

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> crate::events::UiEventResult<T> {
        todo!()
    }

    fn render(
        &mut self,
        // ui environment
        parent_size: [StdSize; 2],
        // context
        context: &SharedContext,
        renderer: &Renderer,
        frame: u64,
    ) -> Vec<(
        Arc<wgpu::Texture>,
        Arc<Vec<UvVertex>>,
        Arc<Vec<u16>>,
        nalgebra::Matrix4<f32>,
    )> {
        todo!()
    }
}
