use layout::LayoutNode;
use nalgebra as na;

use crate::{
    context::SharedContext,
    events::UiEvent,
    types::size::{PxSize, Size},
    ui::{Dom, DomComPareResult, TextureSet, Widget},
    vertex::{colored_vertex::ColoredVertex, vertex_generator::RectangleDescriptor},
};

pub mod style;
pub use style::{Style, Visibility};

pub mod layout;
pub use layout::Layout;

pub struct ContainerDescriptor<T: 'static> {
    pub label: Option<String>,
    pub properties: Style,
    pub layout: Layout<T>,
}

impl<T> Default for ContainerDescriptor<T> {
    fn default() -> Self {
        Self {
            label: None,
            properties: Style::default(),
            layout: Layout::default(),
        }
    }
}

pub struct Container<T: 'static> {
    label: Option<String>,
    properties: Style,
    layout: Layout<T>,
}

impl<T> Container<T> {
    pub fn new(disc: ContainerDescriptor<T>) -> Box<Self> {
        Box::new(Self {
            label: disc.label,
            properties: disc.properties,
            layout: disc.layout,
        })
    }
}

impl<T: Send + 'static> Dom<T> for Container<T> {
    fn build_render_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(ContainerNode {
            label: self.label.clone(),
            properties: self.properties.clone(),
            layout: self.layout.build(),
            box_vertex_buffer: None,
            box_index_buffer: None,
            box_index_len: 0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct ContainerNode<T> {
    // entity info
    label: Option<String>,
    properties: Style,
    layout: LayoutNode<T>,

    // rendering
    box_vertex_buffer: Option<wgpu::Buffer>,
    box_index_buffer: Option<wgpu::Buffer>,
    box_index_len: u32,
}

impl<T: Send + 'static> Widget<T> for ContainerNode<T> {
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

    fn size(&self) -> Size {
        self.properties.size
    }

    fn px_size(&self, parent_size: PxSize, context: &SharedContext) -> PxSize {
        self.properties.size.to_px(parent_size, context)
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
        affine: na::Matrix4<f32>,
        context: &SharedContext,
    )
    {
        if let Visibility::Visible = self.properties.visibility {
            // render box
            if !self.properties.background_color.is_transparent() {
                if self.box_vertex_buffer.is_none() {
                    let (vertex_buffer, index_buffer, index_len) = ColoredVertex::rectangle_buffer(
                        context,
                        RectangleDescriptor {
                            x: 0.0,
                            y: 0.0,
                            width: parent_size.width,
                            height: parent_size.height,
                            radius: self.properties.border.top_left_radius,
                            div: (self.properties.border.top_left_radius as u16).min(16),
                        },
                        false,
                    );
                    self.box_vertex_buffer = Some(vertex_buffer);
                    self.box_index_buffer = Some(index_buffer);
                    self.box_index_len = index_len;
                }
            }

            todo!()
        }
    }
}
