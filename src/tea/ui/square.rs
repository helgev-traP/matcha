use std::sync::Arc;

use crate::{
    application_context::ApplicationContext,
    events::UiEvent,
    types::{
        color::Color,
        size::{PxSize, Size, SizeUnit},
    },
    ui::{Dom, Widget},
    vertex::colored_vertex::ColoredVertex,
};

pub struct SquareDescriptor {
    pub label: Option<String>,
    pub size: Size,
    pub background_color: Color,
}

impl Default for SquareDescriptor {
    fn default() -> Self {
        Self {
            label: None,
            size: Size {
                width: SizeUnit::Pixel(100.0),
                height: SizeUnit::Pixel(100.0),
            },
            background_color: Color::Rgb8USrgb { r: 0, g: 0, b: 0 },
        }
    }
}

pub struct Square {
    label: Option<String>,
    size: Size,
    background_color: Color,
}

impl Square {
    pub fn new(disc: SquareDescriptor) -> Self {
        Self {
            label: disc.label,
            size: disc.size,
            background_color: disc.background_color,
        }
    }
}

impl<R: Copy + Send + 'static> Dom<R> for Square {
    fn build_render_tree(&self) -> Box<dyn Widget<R>> {
        Box::new(SquareNode {
            label: self.label.clone(),
            size: self.size,
            background_color: self.background_color,
            vertex_buffer: None,
            index_buffer: None,
            index_len: 0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct SquareNode {
    label: Option<String>,

    size: Size,
    background_color: Color,

    vertex_buffer: Option<Arc<wgpu::Buffer>>,
    index_buffer: Option<Arc<wgpu::Buffer>>,
    index_len: u32,
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

    fn is_inside(&self, position: [f32; 2], parent_size: PxSize, context: &ApplicationContext) -> bool {
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
                0.0,
                0.0,
                size.width,
                size.height,
                &self.background_color,
                false,
            );

            self.vertex_buffer = Some(Arc::new(vertex));
            self.index_buffer = Some(Arc::new(index));
            self.index_len = index_len;
        }

        encoder.draw(
            super::RenderItem {
                object: vec![crate::ui::Object::Colored {
                    vertex_buffer: self.vertex_buffer.as_ref().unwrap().clone(),
                    index_buffer: self.index_buffer.as_ref().unwrap().clone(),
                    index_len: self.index_len,
                    instance_affine: nalgebra::Matrix4::identity(),
                }],
            },
            affine,
        );
    }
}
