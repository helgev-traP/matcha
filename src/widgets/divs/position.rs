use std::sync::Arc;

use crate::{
    context::SharedContext,
    events::UiEvent,
    renderer::Renderer,
    types::{
        color::Color,
        size::{Size, StdSize},
    },
    ui::{Dom, DomComPareResult, Widget},
    vertex::uv_vertex::UvVertex,
};

use super::style::{Border, BoxSizing, Padding, Visibility};

pub struct Position<T> {
    // label
    label: Option<String>,
    // style
    size: [Size; 2],
    padding: Padding,
    border: Border,
    box_sizing: BoxSizing,
    visibility: Visibility,
    background_color: Color,
    // items
    items: Vec<PositionItem<T>>,
}

pub struct PositionItem<T> {
    pub position: [Size; 2],
    pub item: Box<dyn Dom<T>>,
}

impl<T> Position<T> {
    // new

    pub fn new(label: Option<&str>) -> Box<Self> {
        Box::new(Self {
            label: label.map(|s| s.to_string()),
            size: [Size::Parent(1.0), Size::Parent(1.0)],
            padding: Padding::default(),
            border: Border::default(),
            box_sizing: BoxSizing::default(),
            visibility: Visibility::default(),
            background_color: [0, 0, 0, 0].into(),
            items: Vec::new(),
        })
    }

    // build chain

    pub fn size(mut self, width: Size, height: Size) -> Box<Self> {
        self.size = [width, height];
        Box::new(self)
    }

    pub fn padding(mut self, padding: Padding) -> Box<Self> {
        self.padding = padding;
        Box::new(self)
    }

    pub fn border(mut self, border: Border) -> Box<Self> {
        self.border = border;
        Box::new(self)
    }

    pub fn box_sizing(mut self, box_sizing: BoxSizing) -> Box<Self> {
        self.box_sizing = box_sizing;
        Box::new(self)
    }

    pub fn visibility(mut self, visibility: Visibility) -> Box<Self> {
        self.visibility = visibility;
        Box::new(self)
    }

    pub fn background_color(mut self, color: Color) -> Box<Self> {
        self.background_color = color;
        Box::new(self)
    }

    pub fn item(mut self, v: Vec<PositionItem<T>>) -> Box<Self> {
        self.items = v;
        Box::new(self)
    }

    // push item

    pub fn push(&mut self, position: [Size; 2], item: Box<dyn Dom<T>>) {
        self.items.push(PositionItem { position, item });
    }
}

impl<T: Send + 'static> Dom<T> for Position<T> {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(PositionNode {
            label: self.label.clone(),
            size: self.size,
            padding: self.padding,
            border: self.border,
            box_sizing: self.box_sizing,
            visibility: self.visibility,
            background_color: self.background_color,
            items: self
                .items
                .iter()
                .map(|item| PositionNodeItem {
                    position: item.position,
                    item: item.item.build_widget_tree(),
                })
                .collect(),
            cache: None,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct PositionNode<T> {
    // label
    label: Option<String>,
    // style
    size: [Size; 2],
    padding: Padding,
    border: Border,
    box_sizing: BoxSizing,
    visibility: Visibility,
    background_color: Color,
    // items
    items: Vec<PositionNodeItem<T>>,

    // cache
    cache: Option<Cache>,
}

struct PositionNodeItem<T> {
    position: [Size; 2],
    item: Box<dyn Widget<T>>,
}

struct Cache {
    redraw: bool,
    // vello
    scene: Option<vello::Scene>,
    // texture
    texture: Arc<wgpu::Texture>,
    vello_texture: Option<Arc<wgpu::Texture>>,
    // vertices
    vertices: Arc<Vec<UvVertex>>,
    // indices
    indices: Arc<Vec<u16>>,
}

impl<T: Send + 'static> Widget<T> for PositionNode<T> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn update_widget_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<Position<T>>() {
            Err(())
        } else {
            let dom = dom.as_any().downcast_ref::<Position<T>>().unwrap();
            todo!()
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(_) = dom.as_any().downcast_ref::<Position<T>>() {
            todo!()
        } else {
            DomComPareResult::Different
        }
    }

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> crate::events::UiEventResult<T> {
        crate::events::UiEventResult::default()
    }

    fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> bool {
        let size = self.px_size(parent_size, context);

        !(position[0] < 0.0 || position[0] > size[0] || position[1] < 0.0 || position[1] > size[1])
    }

    fn size(&self) -> [Size; 2] {
        self.size
    }

    fn px_size(&self, parent_size: [StdSize; 2], context: &SharedContext) -> [f32; 2] {
        match self.box_sizing {
            BoxSizing::ContentBox => [
                match self.size[0].to_std_size(parent_size[0], context) {
                    StdSize::Pixel(px) => {
                        px + self.padding.left + self.padding.right + self.border.px * 2.0
                    }
                    StdSize::Content(_) => {
                        self.padding.left + self.padding.right + self.border.px * 2.0
                    }
                },
                match self.size[1].to_std_size(parent_size[1], context) {
                    StdSize::Pixel(px) => {
                        px + self.padding.top + self.padding.bottom + self.border.px * 2.0
                    }
                    StdSize::Content(_) => {
                        self.padding.top + self.padding.bottom + self.border.px * 2.0
                    }
                },
            ],
            BoxSizing::BorderBox => [
                match self.size[0].to_std_size(parent_size[0], context) {
                    StdSize::Pixel(px) => px,
                    StdSize::Content(_) => 0.0,
                },
                match self.size[1].to_std_size(parent_size[1], context) {
                    StdSize::Pixel(px) => px,
                    StdSize::Content(_) => 0.0,
                },
            ],
        }
    }

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
        // calculate the size of content box
        // StdSize::Content(_) will be 0.0

        let texture_size = self.px_size(parent_size, context);

        let field_size = [
            texture_size[0] - self.padding.left - self.padding.right - self.border.px * 2.0,
            texture_size[1] - self.padding.top - self.padding.bottom - self.border.px * 2.0,
        ];

        // padding and border translate
        let content_translate = nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(
            self.padding.left + self.border.px,
            -self.padding.top - self.border.px,
            0.0,
        ));

        // render self

        let cache = self.cache.get_or_insert_with(|| {
            let texture = Arc::new(context.create_texture(
                texture_size[0] as u32 + 1,
                texture_size[1] as u32 + 1,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            ));

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

            let indices = Arc::new(vec![0, 1, 2, 0, 2, 3]);

            Cache {
                redraw: true,
                scene: None,
                texture,
                vello_texture: None,
                vertices,
                indices,
            }
        });

        if cache.redraw {
            // render self
            if !self.background_color.is_transparent()
                || (self.border.px > 0.0 && !self.border.color.is_transparent())
            {
                let vello_texture = cache.vello_texture.get_or_insert_with(|| {
                    Arc::new(context.create_texture(
                        texture_size[0] as u32 + 1,
                        texture_size[1] as u32 + 1,
                        wgpu::TextureFormat::Rgba8Unorm,
                        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
                    ))
                });

                let scene = cache.scene.get_or_insert(vello::Scene::new());

                scene.reset();

                // background
                let color = self.background_color.to_rgba_f32();
                if color[3] > 0.0 {
                    scene.fill(
                        vello::peniko::Fill::EvenOdd,
                        vello::kurbo::Affine::IDENTITY,
                        vello::peniko::Color::new(color),
                        None,
                        &vello::kurbo::RoundedRect::new(
                            0.0,
                            0.0,
                            texture_size[0] as f64,
                            texture_size[1] as f64,
                            vello::kurbo::RoundedRectRadii::new(
                                self.border.top_left_radius as f64,
                                self.border.top_right_radius as f64,
                                self.border.bottom_right_radius as f64,
                                self.border.bottom_left_radius as f64,
                            ),
                        ),
                    );
                }

                // border
                let color = self.border.color.to_rgba_f32();
                let px = self.border.px as f64;
                if px > 0.0 && color[3] > 0.0 {
                    scene.stroke(
                        &vello::kurbo::Stroke::new(px),
                        vello::kurbo::Affine::IDENTITY,
                        vello::peniko::Color::new(color),
                        None,
                        &vello::kurbo::RoundedRect::new(
                            px / 2.0,
                            px / 2.0,
                            texture_size[0] as f64 - px / 2.0,
                            texture_size[1] as f64 - px / 2.0,
                            vello::kurbo::RoundedRectRadii::new(
                                self.border.top_left_radius as f64 - px / 2.0,
                                self.border.top_right_radius as f64 - px / 2.0,
                                self.border.bottom_right_radius as f64 - px / 2.0,
                                self.border.bottom_left_radius as f64 - px / 2.0,
                            ),
                        ),
                    );
                }

                println!("vello_texture: {:?}", [vello_texture.width(), vello_texture.height()]);

                renderer
                    .vello_renderer()
                    .render_to_texture(
                        context.get_wgpu_device(),
                        context.get_wgpu_queue(),
                        &scene,
                        &vello_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                        &vello::RenderParams {
                            base_color: vello::peniko::Color::TRANSPARENT,
                            width: texture_size[0] as u32 + 1,
                            height: texture_size[1] as u32 + 1,
                            antialiasing_method: vello::AaConfig::Area,
                        },
                    )
                    .unwrap();
            }

            // render content
            let content_render_objects =
                self.items
                    .iter_mut()
                    .map(|item| {
                        let field_std_size =
                            [StdSize::Pixel(field_size[0]), StdSize::Pixel(field_size[1])];

                        let position = [
                            match item.position[0].to_std_size(field_std_size[0], context) {
                                StdSize::Pixel(px) => px,
                                StdSize::Content(_) => 0.0,
                            },
                            match item.position[1].to_std_size(field_std_size[1], context) {
                                StdSize::Pixel(px) => px,
                                StdSize::Content(_) => 0.0,
                            },
                        ];

                        let translate = nalgebra::Matrix4::new_translation(
                            &nalgebra::Vector3::new(position[0], -position[1], 0.0),
                        );

                        item.item
                            .render(field_std_size, context, renderer, frame)
                            .into_iter()
                            .map(|(texture, vertices, indices, matrix)| {
                                (
                                    texture,
                                    vertices,
                                    indices,
                                    content_translate * translate * matrix,
                                )
                            })
                            .collect::<Vec<_>>()
                    })
                    .flatten()
                    .collect::<Vec<_>>();

            renderer.render_to_texture(
                &cache
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default()),
                [texture_size[0] + 1.0, texture_size[1] + 1.0],
                content_render_objects,
            );

            cache.redraw = false;
        }

        let mut v = Vec::new();

        if let Some(texture) = &cache.vello_texture {
            v.push((
                texture.clone(),
                cache.vertices.clone(),
                cache.indices.clone(),
                nalgebra::Matrix4::identity(),
            ));
        }

        v.push((
            cache.texture.clone(),
            cache.vertices.clone(),
            cache.indices.clone(),
            nalgebra::Matrix4::identity(),
        ));

        v
    }
}
