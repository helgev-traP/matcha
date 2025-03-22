use std::sync::Arc;

use crate::{
    context::SharedContext,
    events::UiEvent,
    renderer::Renderer,
    types::{
        double_cache_set::DoubleSetCache,
        size::{Size, StdSize},
    },
    ui::{Dom, DomComPareResult, Widget},
    vertex::uv_vertex::UvVertex,
};

// todo: organize modules and public uses.

// style
pub mod style;
use style::{BoxSizing, Style, Visibility};

// layout
pub mod layout;
use layout::{Layout, LayoutNode};

const CACHE_ACCURACY: f32 = 10.0;

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
            indices: Arc::new(vec![0, 1, 2, 2, 3, 0]),
            cache: DoubleSetCache::new(),
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

    // texture, vertices, iF32SizeHashKey
    indices: Arc<Vec<u16>>,
    // pixel accuracy is `CACHE_ACCURACY`
    cache: DoubleSetCache<[u32; 2], (Arc<wgpu::Texture>, Arc<Vec<UvVertex>>)>,
}

/// methods used internally
impl<T> ContainerNode<T> {
    // calculate the size of the box except margin.
    // This will be the size of the texture.
    fn box_size(&self, parent_size: [StdSize; 2], context: &SharedContext) -> [f32; 2] {
        let std_size = [
            self.style.size[0].to_std_size(parent_size[0], context),
            self.style.size[1].to_std_size(parent_size[1], context),
        ];

        match std_size {
            [StdSize::Pixel(width), StdSize::Pixel(height)] => match self.style.box_sizing {
                BoxSizing::BorderBox => [width, height],
                BoxSizing::ContentBox => [
                    width
                        + self.style.border.px * 2.0
                        + self.style.padding.left
                        + self.style.padding.right,
                    height
                        + self.style.border.px * 2.0
                        + self.style.padding.top
                        + self.style.padding.bottom,
                ],
            },
            _ => {
                // need to query children size
                let content_size = self.layout.px_size(std_size, context);

                let width = match std_size[0] {
                    StdSize::Pixel(x) => x,
                    StdSize::Content(p) => content_size[0] * p,
                };

                let width = match self.style.box_sizing {
                    BoxSizing::BorderBox => width,
                    BoxSizing::ContentBox => {
                        width
                            + self.style.border.px * 2.0
                            + self.style.padding.left
                            + self.style.padding.right
                    }
                };

                let height = match std_size[1] {
                    StdSize::Pixel(x) => x,
                    StdSize::Content(p) => content_size[1] * p,
                };

                let height = match self.style.box_sizing {
                    BoxSizing::BorderBox => height,
                    BoxSizing::ContentBox => {
                        height
                            + self.style.border.px * 2.0
                            + self.style.padding.top
                            + self.style.padding.bottom
                    }
                };

                [width, height]
            }
        }
    }
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
        let box_size = self.box_size(parent_size, context);

        let position = [
            position[0] - self.style.margin.left,
            position[1] + self.style.margin.top,
        ];

        !(position[0] < 0.0
            || position[0] > box_size[0]
            || position[1] < 0.0
            || position[1] > box_size[1])
    }

    fn size(&self) -> [Size; 2] {
        self.style.size
    }

    fn px_size(&self, parent_size: [StdSize; 2], context: &SharedContext) -> [f32; 2] {
        match self.style.visibility {
            Visibility::None => [0.0, 0.0],
            Visibility::Visible | Visibility::Hidden => {
                let box_size = self.box_size(parent_size, context);

                [
                    box_size[0] + self.style.margin.left + self.style.margin.right,
                    box_size[1] + self.style.margin.top + self.style.margin.bottom,
                ]
            }
        }
    }

    // todo: cache the children
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

        // children that will overflow is visible

        let box_size = self.box_size(parent_size, context);

        let mut render_items: Vec<(
            Arc<wgpu::Texture>,
            Arc<Vec<UvVertex>>,
            Arc<Vec<u16>>,
            nalgebra::Matrix4<f32>,
        )> = vec![];

        let content_affine = nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(
            self.style.margin.left + self.style.padding.left + self.style.border.px,
            -self.style.margin.top - self.style.padding.top - self.style.border.px,
            0.0,
        ));

        // hash
        let hash_key = [(box_size[0] * CACHE_ACCURACY) as u32, (box_size[1] * CACHE_ACCURACY) as u32];

        let (texture, vertex) = self.cache.get_or_insert_with(hash_key, frame, || {
            let device = context.get_wgpu_device();
            let queue = context.get_wgpu_queue();

            // create texture
            let texture = Arc::new(device.create_texture(&wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: box_size[0] as u32,
                    height: box_size[1] as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
                label: None,
                view_formats: &[],
            }));

            // create vertex
            let vertex = Arc::new(vec![
                UvVertex {
                    position: [0.0, 0.0, 0.0].into(),
                    tex_coords: [0.0, 0.0].into(),
                },
                UvVertex {
                    position: [0.0, -box_size[1], 0.0].into(),
                    tex_coords: [0.0, 1.0].into(),
                },
                UvVertex {
                    position: [box_size[0], -box_size[1], 0.0].into(),
                    tex_coords: [1.0, 1.0].into(),
                },
                UvVertex {
                    position: [box_size[0], 0.0, 0.0].into(),
                    tex_coords: [1.0, 0.0].into(),
                },
            ]);

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
                        (box_size[0] - self.style.border.px) as f64,
                        (box_size[1] - self.style.border.px) as f64,
                        vello::kurbo::RoundedRectRadii::new(
                            (self.style.border.top_left_radius - self.style.border.px) as f64,
                            (self.style.border.top_right_radius - self.style.border.px) as f64,
                            (self.style.border.bottom_right_radius - self.style.border.px) as f64,
                            (self.style.border.bottom_left_radius - self.style.border.px) as f64,
                        ),
                    ),
                );
            }

            // border
            let border_color = self.style.border.color.to_rgba_f64();
            let border_width = self.style.border.px;
            let half_border_width = border_width / 2.0;
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
                        (0.0 + half_border_width) as f64,
                        (0.0 + half_border_width) as f64,
                        (box_size[0] - half_border_width) as f64,
                        (box_size[1] - half_border_width) as f64,
                        vello::kurbo::RoundedRectRadii::new(
                            (self.style.border.top_left_radius - half_border_width) as f64,
                            (self.style.border.top_right_radius - half_border_width) as f64,
                            (self.style.border.bottom_right_radius - half_border_width) as f64,
                            (self.style.border.bottom_left_radius - half_border_width) as f64,
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
                    &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    &vello::RenderParams {
                        base_color: vello::peniko::Color::TRANSPARENT,
                        height: box_size[1] as u32,
                        width: box_size[0] as u32,
                        antialiasing_method: vello::AaConfig::Area,
                    },
                )
                .unwrap();

            // todo: render children to current texture

            render_items.extend(
                self.layout
                    .render(
                        [box_size[0].into(), box_size[1].into()],
                        context,
                        renderer,
                        frame,
                    )
                    .into_iter()
                    .map(|(texture, vertex, index, affine)| {
                        (texture, vertex, index, content_affine * affine)
                    }),
            );

            // return
            (texture, vertex)
        });

        let mut v = vec![(
            texture.clone(),
            vertex.clone(),
            self.indices.clone(),
            nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(
                self.style.margin.left,
                -self.style.margin.top,
                0.0,
            )),
        )];

        v.append(&mut render_items);

        v
    }
}
