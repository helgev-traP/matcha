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
use vello::skrifa::color;
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
            scene: vello::Scene::new(),
            texture: None,
            vertices: None,
            indices: None,
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

    // vello scene
    scene: vello::Scene,

    // texture, vertices, indices
    texture: Option<Arc<wgpu::Texture>>,
    vertices: Option<Arc<Vec<UvVertex>>>,
    indices: Option<Arc<Vec<u16>>>,
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

        // todo: use cache.
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

            // same to the size of the border.
            let texture_size = [
                px_size[0]
                    + match self.style.box_sizing {
                        BoxSizing::BorderBox => 0.0,
                        BoxSizing::ContentBox => self.style.border.px * 2.0,
                    },
                px_size[1]
                    + match self.style.box_sizing {
                        BoxSizing::BorderBox => 0.0,
                        BoxSizing::ContentBox => self.style.border.px * 2.0,
                    },
            ];

            let fill_box_size = [
                px_size[0]
                    + match self.style.box_sizing {
                        BoxSizing::BorderBox => -self.style.border.px * 2.0,
                        BoxSizing::ContentBox => 0.0,
                    },
                px_size[1]
                    + match self.style.box_sizing {
                        BoxSizing::BorderBox => -self.style.border.px * 2.0,
                        BoxSizing::ContentBox => 0.0,
                    },
            ];

            // prepare texture and vertices
            if self.texture.is_none() {
                let device = context.get_wgpu_device();
                let queue = context.get_wgpu_queue();

                // create texture
                self.texture = Some(Arc::new(device.create_texture(&wgpu::TextureDescriptor {
                    label: None,
                    size: wgpu::Extent3d {
                        width: texture_size[0] as u32,
                        height: texture_size[1] as u32,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })));
            }

            if self.vertices.is_none() {
                // create vertices
                let vertices = Arc::new(vec![
                    UvVertex {
                        position: [0.0, 0.0, 0.0].into(),
                        tex_coords: [0.0, 0.0].into(),
                    },
                    UvVertex {
                        position: [0.0, -texture_size[1], 0.0].into(),
                        tex_coords: [0.0, 1.0].into(),
                    },
                    UvVertex {
                        position: [texture_size[0], -texture_size[1], 0.0].into(),
                        tex_coords: [1.0, 1.0].into(),
                    },
                    UvVertex {
                        position: [texture_size[0], 0.0, 0.0].into(),
                        tex_coords: [1.0, 0.0].into(),
                    },
                ]);

                self.vertices = Some(vertices);
                self.indices = Some(Arc::new(vec![0, 1, 2, 0, 2, 3]));
            }

            // draw
            self.scene.reset();

            // fill box
            let background_color = self.style.background_color.to_rgba_f64();
            if background_color[3] > 0.0 {
                self.scene.fill(
                    vello::peniko::Fill::EvenOdd,
                    vello::kurbo::Affine::IDENTITY,
                    vello::peniko::Color::rgba(
                        background_color[0],
                        background_color[1],
                        background_color[2],
                        background_color[3],
                    ),
                    None,
                    &vello::kurbo::RoundedRect::new(
                        self.style.border.px as f64,
                        self.style.border.px as f64,
                        (self.style.border.px + fill_box_size[0]) as f64,
                        (self.style.border.px + fill_box_size[1]) as f64,
                        vello::kurbo::RoundedRectRadii::new(
                            (self.style.border.top_left_radius - self.style.border.px / 2.0) as f64,
                            (self.style.border.top_right_radius - self.style.border.px / 2.0)
                                as f64,
                            (self.style.border.bottom_right_radius - self.style.border.px / 2.0)
                                as f64,
                            (self.style.border.bottom_left_radius - self.style.border.px / 2.0)
                                as f64,
                        ),
                    ),
                );
            }

            // border
            let border_color = self.style.border.color.to_rgba_f64();
            let border_width = self.style.border.px;
            if border_width > 0.0 && border_color[3] > 0.0 {
                self.scene.stroke(
                    &vello::kurbo::Stroke::new(border_width as f64),
                    vello::kurbo::Affine::IDENTITY,
                    vello::peniko::Color::rgba(
                        border_color[0],
                        border_color[1],
                        border_color[2],
                        border_color[3],
                    ),
                    None,
                    &vello::kurbo::RoundedRect::new(
                        0.0,
                        0.0,
                        texture_size[0] as f64,
                        texture_size[1] as f64,
                        vello::kurbo::RoundedRectRadii::new(
                            self.style.border.top_left_radius as f64,
                            self.style.border.top_right_radius as f64,
                            self.style.border.bottom_right_radius as f64,
                            self.style.border.bottom_left_radius as f64,
                        ),
                    ),
                );
            }

            // render to texture
            renderer
                .vello_renderer()
                .render_to_texture(
                    context.get_wgpu_device(),
                    context.get_wgpu_queue(),
                    &self.scene,
                    &self.texture.as_ref().unwrap().create_view(
                        &wgpu::TextureViewDescriptor::default(),
                    ),
                    &vello::RenderParams {
                        base_color: vello::peniko::Color::TRANSPARENT,
                        height: texture_size[1] as u32,
                        width: texture_size[0] as u32,
                        antialiasing_method: vello::AaConfig::Area,
                    },
                )
                .unwrap();

            // todo: ここから
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
