use layout::LayoutNode;
use vello::{kurbo, peniko, Scene};

use crate::{
    context::SharedContext,
    events::UiEvent,
    types::size::{PxSize, Size},
    ui::{Dom, DomComPareResult, LayerStack, Widget},
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
    pub fn new(disc: ContainerDescriptor<T>) -> Self {
        Self {
            label: disc.label,
            properties: disc.properties,
            layout: disc.layout,
        }
    }
}

impl<T: Send + 'static> Dom<T> for Container<T> {
    fn build_render_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(ContainerNode {
            label: self.label.clone(),
            properties: self.properties.clone(),
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
    properties: Style,
    layout: LayoutNode<T>,
}

impl<T: Send + 'static> Widget<T> for ContainerNode<T> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn event(
        &mut self,
        event: &UiEvent,
        parent_size: PxSize,
        context: &SharedContext,
    ) -> crate::events::UiEventResult<T> {
        todo!()
    }

    fn is_inside(&self, position: [f32; 2], parent_size: PxSize, context: &SharedContext) -> bool {
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
        scene: &mut Scene,
        texture_layer: &mut LayerStack,
        parent_size: PxSize,
        affine: vello::kurbo::Affine,
        context: &SharedContext,
    ) {
        if let Visibility::Visible = self.properties.visibility {
            let size = self.px_size(parent_size, context);

            // render box
            // fill background
            if !self.properties.background_color.is_transparent() {
                let color = peniko::Color::from(self.properties.background_color.to_rgba_u8());

                scene.fill(
                    peniko::Fill::NonZero,
                    affine,
                    color,
                    None,
                    &kurbo::RoundedRect::new(
                        0.0,
                        0.0,
                        size.width as f64,
                        size.height as f64,
                        (
                            self.properties.border.top_left_radius as f64,
                            self.properties.border.top_right_radius as f64,
                            self.properties.border.bottom_right_radius as f64,
                            self.properties.border.bottom_left_radius as f64,
                        ),
                    ),
                );
            }

            // draw border
            if self.properties.border.px > 0.0 {
                let color = peniko::Color::from(self.properties.border.color.to_rgba_u8());

                scene.stroke(
                    &kurbo::Stroke::new(self.properties.border.px as f64),
                    affine,
                    color,
                    None,
                    &kurbo::RoundedRect::new(
                        0.0,
                        0.0,
                        size.width as f64,
                        size.height as f64,
                        (
                            self.properties.border.top_left_radius as f64,
                            self.properties.border.top_right_radius as f64,
                            self.properties.border.bottom_right_radius as f64,
                            self.properties.border.bottom_left_radius as f64,
                        ),
                    ),
                );
            }

            // render children
            self.layout
                .render(scene, texture_layer, size, affine, context);
            todo!();
        }
    }
}
