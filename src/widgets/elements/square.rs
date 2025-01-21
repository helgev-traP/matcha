use std::sync::Arc;

use crate::{
    context::SharedContext,
    events::UiEvent,
    renderer::Renderer,
    types::{
        color::Color,
        size::{Size, StdSize},
    },
    ui::{Dom, DomComPareResult, Object, TextureObject, Widget},
    vertex::uv_vertex::UvVertex,
};

pub struct SquareDescriptor {
    pub label: Option<String>,
    pub size: [Size; 2],
    pub radius: f32,
    pub background_color: Color,

    pub border_width: f32,
    pub border_color: Color,
}

impl Default for SquareDescriptor {
    fn default() -> Self {
        Self {
            label: None,
            size: [Size::Pixel(100.0), Size::Pixel(100.0)],
            radius: 0.0,
            background_color: Color::Rgb8USrgb { r: 0, g: 0, b: 0 },
            border_width: 0.0,
            border_color: Color::Rgb8USrgb { r: 0, g: 0, b: 0 },
        }
    }
}

pub struct Square {
    label: Option<String>,
    size: [Size; 2],
    radius: f32,

    background_color: Color,

    border_width: f32,
    border_color: Color,
}

impl Square {
    pub fn new(disc: SquareDescriptor) -> Box<Self> {
        Box::new(Self {
            label: disc.label,
            size: disc.size,
            radius: disc.radius,
            background_color: disc.background_color,
            border_width: disc.border_width,
            border_color: disc.border_color,
        })
    }
}

impl<T: Copy + Send + 'static> Dom<T> for Square {
    fn build_widget_tree(&self) -> (Box<dyn Widget<T>>, bool) {
        (
            Box::new(SquareWidget {
                label: self.label.clone(),
                size: self.size,
                radius: self.radius,
                background_color: self.background_color,
                border_width: self.border_width,
                border_color: self.border_color,
                scene: vello::Scene::new(),
                texture: None,
                vertex: None,
                index: Arc::new(vec![0, 1, 2, 0, 2, 3]),
                redraw: true,
            }),
            false,
        )
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct SquareWidget {
    label: Option<String>,

    size: [Size; 2],
    radius: f32,
    background_color: Color,
    border_width: f32,
    border_color: Color,

    // rendering
    scene: vello::Scene,
    texture: Option<Arc<wgpu::Texture>>,
    vertex: Option<Arc<Vec<UvVertex>>>,
    index: Arc<Vec<u16>>,

    // caching
    redraw: bool,
}

impl<R: Send + 'static> Widget<R> for SquareWidget {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> crate::events::UiEventResult<R> {
        crate::events::UiEventResult::default()
    }

    fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> bool {
        let current_size = [
            self.size[0]
                .to_std_size(parent_size[0], context)
                .unwrap_or(0.0),
            self.size[1]
                .to_std_size(parent_size[1], context)
                .unwrap_or(0.0),
        ];

        if position[0] < 0.0
            || position[0] > current_size[0]
            || position[1] < 0.0
            || position[1] > current_size[1]
        {
            false
        } else {
            true
        }
    }

    fn update_widget_tree(&mut self, dom: &dyn Dom<R>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<Square>() {
            return Err(());
        }

        let dom = dom.as_any().downcast_ref::<Square>().unwrap();

        self.size = dom.size;
        self.background_color = dom.background_color;

        Ok(())
    }

    fn compare(&self, dom: &dyn Dom<R>) -> DomComPareResult {
        if let Some(super_simple_button) = dom.as_any().downcast_ref::<Square>() {
            if self.size == super_simple_button.size
                && self.background_color == super_simple_button.background_color
            {
                DomComPareResult::Same
            } else {
                DomComPareResult::Changed
            }
        } else {
            DomComPareResult::Different
        }
    }

    fn size(&self) -> [Size; 2] {
        self.size
    }

    fn px_size(&self, parent_size: [StdSize; 2], context: &SharedContext) -> [f32; 2] {
        [
            self.size[0]
                .to_std_size(parent_size[0], context)
                .unwrap_or(0.0),
            self.size[1]
                .to_std_size(parent_size[1], context)
                .unwrap_or(0.0),
        ]
    }

    fn drawing_range(&self, parent_size: [StdSize; 2], context: &SharedContext) -> [[f32; 2]; 2] {
        let px_size: [f32; 2] = Widget::<R>::px_size(self, parent_size, context);

        [[0.0, 0.0], px_size]
    }

    fn cover_area(
        &self,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> Option<[[f32; 2]; 2]> {
        let px_size = Widget::<R>::px_size(self, parent_size, context);

        Some([
            [self.radius, self.radius],
            [px_size[0] - self.radius, px_size[1] - self.radius],
        ])
    }

    fn has_dynamic(&self) -> bool {
        false
    }

    fn redraw(&self) -> bool {
        self.redraw
    }

    fn render(
        &mut self,
        // ui environment
        parent_size: [StdSize; 2],
        background_view: &wgpu::TextureView,
        background_position: [[f32; 2]; 2], // [{upper left x, y}, {lower right x, y}]
        // context
        context: &SharedContext,
        renderer: &Renderer,
        frame: u64,
    ) -> Vec<Object> {
        let size = [
            self.size[0]
                .to_std_size(parent_size[0], context)
                .unwrap_or(0.0),
            self.size[1]
                .to_std_size(parent_size[1], context)
                .unwrap_or(0.0),
        ];

        if self.texture.is_none() || self.redraw {
            let device = context.get_wgpu_device();

            // create texture
            self.texture = Some(Arc::new(device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: size[0] as u32 + 1,
                    height: size[1] as u32 + 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            })));

            // draw

            self.scene.reset();

            let c = self.background_color.to_rgba_f64();

            self.scene.fill(
                vello::peniko::Fill::EvenOdd,
                vello::kurbo::Affine::IDENTITY,
                vello::peniko::Color::rgba(c[0], c[1], c[2], c[3]),
                None,
                &vello::kurbo::RoundedRect::new(
                    0.0,
                    0.0,
                    size[0] as f64,
                    size[1] as f64,
                    self.radius as f64,
                ),
            );

            if self.border_width > 0.0 {
                let c = self.border_color.to_rgba_f64();

                self.scene.stroke(
                    &vello::kurbo::Stroke::new(self.border_width as f64),
                    vello::kurbo::Affine::IDENTITY,
                    vello::peniko::Color::rgba(c[0], c[1], c[2], c[3]),
                    None,
                    &vello::kurbo::RoundedRect::new(
                        self.border_width as f64 / 2.0,
                        self.border_width as f64 / 2.0,
                        size[0] as f64 - self.border_width as f64 / 2.0,
                        size[1] as f64 - self.border_width as f64 / 2.0,
                        self.radius as f64 - self.border_width as f64 / 2.0,
                    ),
                );
            }

            renderer
                .vello_renderer()
                .render_to_texture(
                    context.get_wgpu_device(),
                    context.get_wgpu_queue(),
                    &self.scene,
                    &self
                        .texture
                        .as_ref()
                        .unwrap()
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                    &vello::RenderParams {
                        base_color: vello::peniko::Color::TRANSPARENT,
                        width: size[0] as u32,
                        height: size[1] as u32,
                        antialiasing_method: vello::AaConfig::Area,
                    },
                )
                .unwrap();

            self.redraw = false;
        }

        vec![Object::TextureObject(TextureObject {
            texture: self.texture.as_ref().unwrap().clone(),
            uv_vertices: Arc::new(vec![
                UvVertex {
                    position: [0.0, 0.0, 0.0].into(),
                    tex_coords: [0.0, 0.0].into(),
                },
                UvVertex {
                    position: [0.0, -size[1], 0.0].into(),
                    tex_coords: [0.0, 1.0].into(),
                },
                UvVertex {
                    position: [size[0], -size[1], 0.0].into(),
                    tex_coords: [1.0, 1.0].into(),
                },
                UvVertex {
                    position: [size[0], 0.0, 0.0].into(),
                    tex_coords: [1.0, 0.0].into(),
                },
            ]),
            indices: self.index.clone(),
            transform: nalgebra::Matrix4::identity(),
        })]
    }
}
