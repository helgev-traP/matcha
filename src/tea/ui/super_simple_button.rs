use std::sync::Arc;

use crate::{
    application_context::ApplicationContext,
    types::{
        color::Color,
        size::{PxSize, Size, SizeUnit},
    },
};

use super::{Dom, Widget};

pub struct SuperSimpleButton<R: Copy + Send + 'static> {
    size: Size,
    background_color: Color,
    on_click: R,
}

impl<R: Copy + Send> SuperSimpleButton<R> {
    pub fn new(on_click: R) -> Self {
        Self {
            size: Size {
                width: SizeUnit::Pixel(100.0),
                height: SizeUnit::Pixel(100.0),
            },
            background_color: Color::Rgb8USrgb {
                r: 128,
                g: 128,
                b: 128,
            },
            on_click,
        }
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn background_color(mut self, background_color: Color) -> Self {
        self.background_color = background_color;
        self
    }

    pub fn on_click(self, on_click: R) -> SuperSimpleButton<R> {
        SuperSimpleButton {
            size: self.size,
            background_color: self.background_color,
            on_click,
        }
    }
}

impl<R: Copy + Send> Dom<R> for SuperSimpleButton<R> {
    fn build_render_tree(&self) -> Box<dyn Widget<R>> {
        Box::new(SuperSimpleButtonRenderNode {
            size: self.size,
            background_color: self.background_color,
            on_click: self.on_click,
            vertex_buffer: None,
            index_buffer: None,
            index_len: 0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct SuperSimpleButtonRenderNode<R: Copy + Send> {
    size: Size,
    background_color: Color,
    on_click: R,

    vertex_buffer: Option<Arc<wgpu::Buffer>>,
    index_buffer: Option<Arc<wgpu::Buffer>>,
    index_len: u32,
}

impl<R: Copy + Send + 'static> super::WidgetTrait<R> for SuperSimpleButtonRenderNode<R> {
    fn widget_event(
        &self,
        event: &crate::events::WidgetEvent,
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> crate::events::WidgetEventResult<R> {
        match event {
            crate::events::WidgetEvent::MouseLeftClick { x, y } => {
                let actual_size = self.size.to_px(parent_size, &context);
                if *x >= 0.0 && *x <= actual_size.width && *y >= 0.0 && *y <= actual_size.height {
                    crate::events::WidgetEventResult {
                        user_event: Some(self.on_click),
                    }
                } else {
                    crate::events::WidgetEventResult::default()
                }
            }
        }
    }

    fn update_render_tree(&mut self, dom: &dyn Dom<R>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<SuperSimpleButton<R>>() {
            return Err(());
        }

        let dom = dom.as_any().downcast_ref::<SuperSimpleButton<R>>().unwrap();

        self.size = dom.size;
        self.background_color = dom.background_color;

        Ok(())
    }

    fn compare(&self, dom: &dyn Dom<R>) -> super::DomComPareResult {
        if let Some(super_simple_button) = dom.as_any().downcast_ref::<SuperSimpleButton<R>>() {
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

impl<R: Copy + Send> super::RenderingTrait for SuperSimpleButtonRenderNode<R> {
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
        encoder: &mut crate::render::RenderCommandEncoder,
    ) {
        let context = encoder.get_context();

        let size = self.size.to_px(parent_size, context);

        if self.vertex_buffer.is_none() || self.index_buffer.is_none() || self.index_len == 0 {
            let (vertex, index, index_len) = crate::vertex::ColoredVertex::rectangle_buffer(
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
