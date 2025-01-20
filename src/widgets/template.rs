use crate::{
    context::SharedContext,
    events::UiEvent,
    renderer::Renderer,
    types::size::{Size, StdSize},
    ui::{Dom, DomComPareResult, Object, Widget},
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
    // label
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    // for dom handling
    fn update_widget_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<Template>() {
            Err(())
        } else {
            let dom = dom.as_any().downcast_ref::<Template>().unwrap();
            let _ = dom;
            todo!()
        }
    }

    // comparing dom
    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(_) = dom.as_any().downcast_ref::<Template>() {
            todo!()
        } else {
            DomComPareResult::Different
        }
    }

    // widget event
    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> crate::events::UiEventResult<T> {
        let _ = (event, parent_size, context);
        todo!()
    }

    // inside / outside check
    fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> bool {
        let _ = (position, parent_size, context);
        todo!()
    }

    // The size configuration of the widget.
    fn size(&self) -> [Size; 2] {
        self.size
    }

    // Actual size including its sub widgets with pixel value.
    fn px_size(&self, parent_size: [StdSize; 2], context: &SharedContext) -> [f32; 2] {
        let _ = (parent_size, context);
        todo!()
    }

    // The drawing range of the whole widget.
    fn drawing_range(&self) -> [[f32; 2]; 2] {
        todo!()
    }

    // The area that the widget always covers.
    fn cover_area(&self) -> Option<[[f32; 2]; 2]> {
        todo!()
    }

    // if there is any dynamic widget in children
    fn has_dynamic(&self) -> bool {
        todo!()
    }

    // if redraw is needed
    fn redraw(&self) -> bool {
        todo!()
    }

    // render
    fn render(
        &mut self,
        // ui environment
        parent_size: [StdSize; 2],
        background_view: &wgpu::TextureView,
        background_position: [[f32; 2]; 2], // [{upper left x, y}, {lower right x, y}]
        // context
        context: &SharedContext,
        renderer: &Renderer,
        frame: u64,
    ) -> Vec<Object> {
        let _ = (parent_size, background_view, background_position, context, renderer, frame);
        todo!()
    }
}
