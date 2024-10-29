use std::sync::Arc;

use crate::{
    application_context::ApplicationContext,
    events::UiEvent,
    types::{
        color::Color,
        size::{PxSize, Size, SizeUnit},
    },
    ui::{Dom, Widget},
    vertex::{
        colored_vertex::ColoredVertex,
        vertex_generator::{BorderDescriptor, RectangleDescriptor},
    },
};

pub struct SquareDescriptor {
    pub label: Option<String>,
    pub size: Size,
    pub radius: f32,
    pub background_color: Color,

    pub border_width: f32,
    pub border_color: Color,
}

impl Default for SquareDescriptor {
    fn default() -> Self {
        Self {
            label: None,
            size: Size {
                width: SizeUnit::Pixel(100.0),
                height: SizeUnit::Pixel(100.0),
            },
            radius: 0.0,
            background_color: Color::Rgb8USrgb { r: 0, g: 0, b: 0 },
            border_width: 0.0,
            border_color: Color::Rgb8USrgb { r: 0, g: 0, b: 0 },
        }
    }
}

pub struct Square {
    label: Option<String>,
    size: Size,
    radius: f32,

    background_color: Color,

    border_width: f32,
    border_color: Color,
}

impl Square {
    pub fn new(disc: SquareDescriptor) -> Self {
        Self {
            label: disc.label,
            size: disc.size,
            radius: disc.radius,
            background_color: disc.background_color,
            border_width: disc.border_width,
            border_color: disc.border_color,
        }
    }
}

impl<R: Copy + Send + 'static> Dom<R> for Square {
    fn build_render_tree(&self) -> Box<dyn Widget<R>> {
        Box::new(SquareNode {
            label: self.label.clone(),
            size: self.size,
            radius: self.radius,
            background_color: self.background_color,
            border_width: self.border_width,
            border_color: self.border_color,
            vertex_buffer: None,
            index_buffer: None,
            index_len: 0,
            border_vertex_buffer: None,
            border_index_buffer: None,
            border_index_len: 0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct SquareNode {
    label: Option<String>,

    size: Size,
    radius: f32,
    background_color: Color,
    border_width: f32,
    border_color: Color,

    // box
    vertex_buffer: Option<Arc<wgpu::Buffer>>,
    index_buffer: Option<Arc<wgpu::Buffer>>,
    index_len: u32,

    // border
    border_vertex_buffer: Option<Arc<wgpu::Buffer>>,
    border_index_buffer: Option<Arc<wgpu::Buffer>>,
    border_index_len: u32,
}

impl<R: Copy + Send + 'static> super::WidgetTrait<R> for SquareNode {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> crate::events::UiEventResult<R> {
        crate::events::UiEventResult::default()
    }

    fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> bool {
        let current_size = self.size.to_px(parent_size, context);

        if position[0] < 0.0
            || position[0] > current_size.width
            || position[1] < 0.0
            || position[1] > current_size.height
        {
            false
        } else {
            true
        }
    }

    fn update_render_tree(&mut self, dom: &dyn Dom<R>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<Square>() {
            return Err(());
        }

        let dom = dom.as_any().downcast_ref::<Square>().unwrap();

        self.size = dom.size;
        self.background_color = dom.background_color;

        Ok(())
    }

    fn compare(&self, dom: &dyn Dom<R>) -> super::DomComPareResult {
        if let Some(super_simple_button) = dom.as_any().downcast_ref::<Square>() {
            if self.size == super_simple_button.size
                && self.background_color == super_simple_button.background_color
            {
                super::DomComPareResult::Same
            } else {
                super::DomComPareResult::Changed
            }
        } else {
            super::DomComPareResult::Different
        }
    }
}

impl super::RenderingTrait for SquareNode {
    fn size(&self) -> Size {
        self.size
    }

    fn px_size(
        &self,
        parent_size: crate::types::size::PxSize,
        context: &crate::application_context::ApplicationContext,
    ) -> crate::types::size::PxSize {
        self.size.to_px(parent_size, context)
    }

    fn default_size(&self) -> crate::types::size::PxSize {
        crate::types::size::PxSize {
            width: 0.0,
            height: 0.0,
        }
    }

    fn render(
        &mut self,
        _: &rayon::Scope,
        parent_size: crate::types::size::PxSize,
        affine: nalgebra::Matrix4<f32>,
        encoder: &mut crate::renderer::RendererCommandEncoder,
    ) {
        let context = encoder.get_context();

        let size = self.size.to_px(parent_size, context);

        if self.vertex_buffer.is_none() || self.index_buffer.is_none() || self.index_len == 0 {
            let (vertex, index, index_len) = ColoredVertex::rectangle_buffer(
                context,
                RectangleDescriptor::new(size.width, size.height).radius(self.radius),
                false,
            );

            self.vertex_buffer = Some(Arc::new(vertex));
            self.index_buffer = Some(Arc::new(index));
            self.index_len = index_len;
        }

        if self.border_vertex_buffer.is_none() {
            let (vertex, index, index_len) = ColoredVertex::border_buffer(
                context,
                BorderDescriptor::new(size.width, size.height, self.border_width)
                    .radius(self.radius)
                    .border_width(self.border_width),
                false,
            );

            self.border_vertex_buffer = Some(Arc::new(vertex));
            self.border_index_buffer = Some(Arc::new(index));
            self.border_index_len = index_len;
        }

        encoder.draw(
            super::RenderItem {
                object: vec![
                    crate::ui::Object::Colored {
                        object_affine: nalgebra::Matrix4::identity(),
                        vertex_buffer: self.vertex_buffer.as_ref().unwrap().clone(),
                        index_buffer: self.index_buffer.as_ref().unwrap().clone(),
                        index_len: self.index_len,
                        color: self.background_color,
                    },
                    crate::ui::Object::Colored {
                        object_affine: nalgebra::Matrix4::identity(),
                        vertex_buffer: self.border_vertex_buffer.as_ref().unwrap().clone(),
                        index_buffer: self.border_index_buffer.as_ref().unwrap().clone(),
                        index_len: self.border_index_len,
                        color: self.border_color,
                    },
                ],
            },
            affine,
        );
    }
}
