use crate::{
    application_context::ApplicationContext,
    events::UiEvent,
    renderer::RendererCommandEncoder,
    types::size::{PxSize, Size, SizeUnit},
    ui::{Dom, Widget},
};

pub struct ContainerDescriptor<T> {
    pub label: Option<String>,
    pub size: Size,
    pub properties: (),
    pub children: Vec<Box<dyn Dom<T>>>,
}

impl<T> Default for ContainerDescriptor<T> {
    fn default() -> Self {
        Self {
            label: None,
            size: Size {
                width: SizeUnit::Pixel(100.0),
                height: SizeUnit::Pixel(100.0),
            },
            properties: (),
            children: vec![],
        }
    }
}

pub struct Container<T> {
    label: Option<String>,
    size: Size,
    properties: (),
    children: Vec<Box<dyn Dom<T>>>,
}

impl<T> Container<T> {
    pub fn new(disc: ContainerDescriptor<T>) -> Self {
        Self {
            label: disc.label,
            size: disc.size,
            properties: disc.properties,
            children: disc.children,
        }
    }
}

impl<T: Send + 'static> Dom<T> for Container<T> {
    fn build_render_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(ContainerNode {
            label: self.label.clone(),
            size: self.size,
            properties: self.properties,
            children: self
                .children
                .iter()
                .map(|child| child.build_render_tree())
                .collect(),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct ContainerNode<T> {
    label: Option<String>,
    size: Size,
    properties: (),
    children: Vec<Box<dyn Widget<T>>>,
}

impl<T: Send + 'static> super::WidgetTrait<T> for ContainerNode<T> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> crate::events::UiEventResult<T> {
        todo!()
    }

    fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> bool {
        todo!()
    }

    fn update_render_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<Container<T>>() {
            Err(())
        } else {
            let dom = dom.as_any().downcast_ref::<Container<T>>().unwrap();
            todo!()
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> super::DomComPareResult {
        if let Some(_) = dom.as_any().downcast_ref::<Container<T>>() {
            todo!()
        } else {
            super::DomComPareResult::Different
        }
    }
}

impl<T> super::RenderingTrait for ContainerNode<T> {
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
        encoder: &mut RendererCommandEncoder,
    ) {
        todo!()
    }
}
