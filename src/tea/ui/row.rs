use super::EventHandle;
use super::Property;
use super::Style;
use super::TeaUi;
use super::UiRendering;

use crate::affine;
use crate::event::Event;
use crate::ui::SubObject;
use crate::vertex;
use crate::vertex::ColoredVertex;

pub struct Row {
    pub items: Vec<Box<dyn TeaUi>>,

    // currently supports: size, position, background_color.
    pub property: Property,

    app_context: Option<crate::application_context::ApplicationContext>,
}

impl UiRendering for Row {
    fn set_application_context(
        &mut self,
        app_context: crate::tea::application_context::ApplicationContext,
    ) {
        for item in &mut self.items {
            item.set_application_context(app_context.clone());
        }
    }

    fn get_style(&self) -> &super::Property {
        todo!()
    }

    fn set_style(&mut self, property: super::Property) {
        todo!()
    }

    fn render_object(
        &self,
        parent_size: crate::types::ParentPxSize,
    ) -> Result<super::RenderObject, ()> {
        // current size of the row.
        let current_size = crate::types::ParentPxSize {
            width: self
                .property
                .size
                .width
                .to_px(parent_size.width, self.app_context.as_ref().unwrap()),
            height: self
                .property
                .size
                .height
                .to_px(parent_size.height, self.app_context.as_ref().unwrap()),
        };

        // get all render objects from the items.

        let mut render_objects = Vec::new();
        for item in &self.items {
            let render_object = item.render_object(current_size);
            if render_object.is_err() {
                return Err(());
            }
            render_objects.push(render_object.unwrap());
        }

        // calculate the sum of the width and height of the row items.
        let mut row_items_width: f32 = 0.0;
        let mut row_items_height: f32 = 0.0;

        for render_object in &render_objects {
            row_items_width += render_object.px_size.width;
            row_items_height = row_items_height.max(render_object.px_size.height);
        }

        // position the items.
        let mut sub_objects = Vec::new();

        let mut x = 0.0;
        for render_object in render_objects {
            let temp = render_object.px_size.width;
            sub_objects.push(SubObject {
                affine: affine::translate_2d(x, 0.0) * affine::init_2d(),
                object: render_object,
            });
            x += temp;
        }

        // return.
        let width = if let Some(width) = current_size.width {
            width
        } else {
            row_items_width
        };

        let height = if let Some(height) = current_size.height {
            height
        } else {
            row_items_height
        };

        let (vertex_buffer, index_buffer, index_len) = ColoredVertex::rectangle_buffer(
            self.app_context.as_ref().unwrap(),
            0.0,
            0.0,
            width,
            height,
            &self.property.background_color,
            false,
        );

        Ok(
            super::RenderObject {
                object: super::Object::Colored {
                    vertex_buffer: Box::new(vertex_buffer),
                    index_buffer: Box::new(index_buffer),
                    index_len,
                },
                px_size: crate::types::PxSize { width, height },
                property: self.property.clone(),
                sub_objects,
            }
        )
    }
}

impl EventHandle for Row {}

impl Style for Row {}
