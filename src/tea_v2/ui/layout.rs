use nalgebra as na;

use crate::{
    affine,
    types::{ParentPxSize, PxSize, SizeUnit},
    ui::property::BoxSizing,
    vertex::ColoredVertex,
};

use super::{EventHandle, Object, Property, RenderObject, Style, TeaUi, UiRendering};

pub struct Container {
    pub items: Vec<Box<dyn TeaUi>>,
    pub property: Property,
    app_context: Option<crate::application_context::ApplicationContext>,
}

impl Container {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            property: Property::default(),
            app_context: None,
        }
    }

    pub fn vec(items: Vec<Box<dyn TeaUi>>) -> Self {
        Self {
            items,
            property: Property::default(),
            app_context: None,
        }
    }
}

impl UiRendering for Container {
    fn set_application_context(
        &mut self,
        app_context: crate::tea::application_context::ApplicationContext,
    ) {
        todo!()
    }

    fn get_style(&self) -> &Property {
        &self.property
    }

    fn set_style(&mut self, property: Property) {
        self.property = property;
    }

    // fn render_necessity(&self) -> bool {
    //     self.items.iter().any(|item| item.render_necessity())
    // }

    fn render_object(&self, parent_size: ParentPxSize) -> Result<RenderObject, ()> {
        // calculate the size of the container

        let mut current_size = ParentPxSize {
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

        match &self.property.display {
            super::property::Display::None => {
                // err じゃなくしたい
                return Err(());
            }
            super::property::Display::Flex {
                direction,
            } => {
                let sub_objects: Vec<RenderObject> = self
                    .items
                    .iter()
                    .filter_map(|item| {
                        if let Ok(item) = item.render_object(current_size) {
                            Some(item)
                        } else {
                            None
                        }
                    })
                    .collect();

                // calculate the size of the container

                let mut items_width_sum: f32 = 0.0;
                let mut items_width_max: f32 = 0.0;
                let mut items_height_sum: f32 = 0.0;
                let mut items_height_max: f32 = 0.0;

                for item in sub_objects.iter() {
                    let width = item.px_size.width
                        + if let BoxSizing::ContentBox = item.property.box_sizing {
                            item.property.border.px * 2.0
                        } else {
                            0.0
                        };
                    let height = item.px_size.height
                        + if let BoxSizing::ContentBox = item.property.box_sizing {
                            item.property.border.px * 2.0
                        } else {
                            0.0
                        };
                    items_width_sum += width;
                    items_width_max = items_width_max.max(width);
                    items_height_sum += height;
                    items_height_max = items_height_max.max(height);
                }

                match direction {
                    crate::ui::property::FlexDirection::Row => {
                    }
                    crate::ui::property::FlexDirection::RowReverse => todo!(),
                    crate::ui::property::FlexDirection::Column => {}
                    crate::ui::property::FlexDirection::ColumnReverse => todo!(),
                }

                //
            }
            super::property::Display::Grid {
                template_columns,
                template_rows,
                gap,
            } => todo!(),
        }

        todo!();
    }
}

impl EventHandle for Container {}

impl Style for Container {}
