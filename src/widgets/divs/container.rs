use std::{default, sync::Arc};

use crate::{
    context::SharedContext,
    events::UiEvent,
    renderer::Renderer,
    types::size::{PxSize, Size},
    ui::{Dom, DomComPareResult, Widget},
    vertex::{
        colored_vertex::ColoredVertex, uv_vertex::UvVertex, vertex_generator::RectangleDescriptor,
    },
};

// todo: organize modules and public uses.

// style
pub mod style;
use style::{Style, Visibility};

// layout
pub mod layout;
use layout::{Layout, LayoutNode};

#[derive(Default)]
pub struct ContainerDescriptor<T: 'static> {
    pub label: Option<String>,
    // style of the container itself
    pub style: Style,
    // layout of the child elements
    pub layout: Layout<T>,
}
pub struct Container<T: 'static> {
    label: Option<String>,
    style: Style,
    layout: Layout<T>,
}

impl<T> Container<T> {
    pub fn new(disc: ContainerDescriptor<T>) -> Box<Self> {
        Box::new(Self {
            label: disc.label,
            style: disc.style,
            layout: disc.layout,
        })
    }
}

impl<T: Send + 'static> Dom<T> for Container<T> {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(ContainerNode {
            label: self.label.clone(),
            style: self.style.clone(),
            layout: self.layout.build(),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct ContainerNode<T> {
    // entity info
    label: Option<String>,
    style: Style,
    layout: LayoutNode<T>,
}

impl<T: Send + 'static> Widget<T> for ContainerNode<T> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn update_widget_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<Container<T>>() {
            Err(())
        } else {
            let dom = dom.as_any().downcast_ref::<Container<T>>().unwrap();
            todo!()
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(_) = dom.as_any().downcast_ref::<Container<T>>() {
            todo!()
        } else {
            DomComPareResult::Different
        }
    }

    fn is_inside(&self, position: [f32; 2], parent_size: PxSize, context: &SharedContext) -> bool {
        let px_size = self.px_size(parent_size, context);

        !(position[0] < 0.0
            || position[0] > px_size.width
            || position[1] < 0.0
            || position[1] > px_size.height)
    }

    fn size(&self) -> Size {
        self.style.size
    }

    fn px_size(&self, parent_size: PxSize, context: &SharedContext) -> PxSize {
        match self.style.visibility {
            Visibility::None => PxSize {
                width: 0.0,
                height: 0.0,
            },
            Visibility::Visible | Visibility::Hidden => {
                todo!()
            },
        }
    }

    fn default_size(&self) -> PxSize {
        PxSize {
            width: 0.0,
            height: 0.0,
        }
    }

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: PxSize,
        context: &SharedContext,
    ) -> crate::events::UiEventResult<T> {
        todo!()
    }

    fn render(
        &mut self,
        // ui environment
        parent_size: PxSize,
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
