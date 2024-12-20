use std::{default, sync::Arc};

use crate::{
    context::SharedContext,
    events::UiEvent,
    renderer::Renderer,
    types::size::{Size, StdSize},
    ui::{Dom, DomComPareResult, Widget},
    vertex::{
        colored_vertex::ColoredVertex, uv_vertex::UvVertex, vertex_generator::RectangleDescriptor,
    },
};

// todo: organize modules and public uses.

// style
pub mod style;
use style::{border, BoxSizing, Style, Visibility};

// layout
pub mod layout;
use layout::{Layout, LayoutNode};
use wgpu::naga::back;

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

    // todo
    fn update_widget_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<Container<T>>() {
            Err(())
        } else {
            let dom = dom.as_any().downcast_ref::<Container<T>>().unwrap();
            todo!()
        }
    }

    // todo
    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(_) = dom.as_any().downcast_ref::<Container<T>>() {
            todo!()
        } else {
            DomComPareResult::Different
        }
    }

    // todo
    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> crate::events::UiEventResult<T> {
        todo!()
    }

    fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> bool {
        let px_size = self.px_size(parent_size, context);

        !(position[0] < 0.0
            || position[0] > px_size[0]
            || position[1] < 0.0
            || position[1] > px_size[1])
    }

    fn size(&self) -> [Size; 2] {
        self.style.size
    }

    fn px_size(&self, parent_size: [StdSize; 2], context: &SharedContext) -> [f32; 2] {
        match self.style.visibility {
            Visibility::None => [0.0, 0.0],
            Visibility::Visible | Visibility::Hidden => {
                // calculate children size
                let std_size = [
                    self.style.size[0].to_std_size(parent_size[0], context),
                    self.style.size[1].to_std_size(parent_size[1], context),
                ];

                let px = match std_size {
                    [StdSize::Pixel(width), StdSize::Pixel(height)] => [width, height],
                    _ => {
                        // need to query children size
                        self.layout.px_size(std_size, context)
                    }
                };

                // add padding, margin, border.
                [
                    // width
                    px[0]
                        + self.style.padding.left
                        + self.style.padding.right
                        + self.style.margin.left
                        + self.style.margin.right
                        + match self.style.box_sizing {
                            BoxSizing::ContentBox => 0.0,
                            BoxSizing::BorderBox => self.style.border.px * 2.0,
                        },
                    // height
                    px[1]
                        + self.style.padding.top
                        + self.style.padding.bottom
                        + self.style.margin.top
                        + self.style.margin.bottom
                        + match self.style.box_sizing {
                            BoxSizing::ContentBox => 0.0,
                            BoxSizing::BorderBox => self.style.border.px * 2.0,
                        },
                ]
            }
        }
    }

    fn default_size(&self) -> [f32; 2] {
        [
            self.style.padding.left
                + self.style.padding.right
                + self.style.margin.left
                + self.style.margin.right
                + match self.style.box_sizing {
                    BoxSizing::ContentBox => 0.0,
                    BoxSizing::BorderBox => self.style.border.px * 2.0,
                },
            self.style.padding.top
                + self.style.padding.bottom
                + self.style.margin.top
                + self.style.margin.bottom
                + match self.style.box_sizing {
                    BoxSizing::ContentBox => 0.0,
                    BoxSizing::BorderBox => self.style.border.px * 2.0,
                },
        ]
    }

    // todo
    fn render(
        &mut self,
        // ui environment
        parent_size: [StdSize; 2],
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
        // check visibility
        if self.style.visibility == Visibility::None || self.style.visibility == Visibility::Hidden
        {
            return vec![];
        }

        // generally, leave the process to the layout system.

        let mut render_items = vec![];
        let px_size = self.px_size(parent_size, context);

        {
            // make the container itself.
            let border_affine_translation =
                nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(
                    match self.style.box_sizing {
                        BoxSizing::BorderBox => 0.0,
                        BoxSizing::ContentBox => -self.style.border.px,
                    },
                    match self.style.box_sizing {
                        BoxSizing::BorderBox => 0.0,
                        BoxSizing::ContentBox => self.style.border.px,
                    },
                    0.0,
                ));

            // prepare texture and vertices
            // todo

            // fill box
            let background_color = self.style.background_color.to_rgba_f32();
            if background_color[3] == 0.0 {
                todo!()
            }

            // border
            let border_color = self.style.border.color.to_rgba_f32();
            let border_width = self.style.border.px;
            if border_width > 0.0 && border_color[3] > 0.0 {
                todo!()
            }
        }

        {
            // render children
            let margin_affine_translation =
                nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(
                    self.style.margin.left
                        + self.style.padding.left
                        + match self.style.box_sizing {
                            BoxSizing::ContentBox => 0.0,
                            BoxSizing::BorderBox => self.style.border.px,
                        },
                    -self.style.margin.top
                        - self.style.padding.top
                        - match self.style.box_sizing {
                            BoxSizing::ContentBox => 0.0,
                            BoxSizing::BorderBox => self.style.border.px,
                        },
                    0.0,
                ));

            render_items.append(
                &mut self
                    .layout
                    .render(px_size, context, renderer, frame)
                    .into_iter()
                    .map(|(texture, vertices, indices, affine)| {
                        // apply margin translation
                        let affine = margin_affine_translation * affine;
                        (texture, vertices, indices, affine)
                    })
                    .collect::<Vec<_>>(),
            );
        }

        render_items
    }
}
