use std::sync::Arc;

use crate::{
    context::SharedContext,
    events::UiEvent,
    types::{
        color::Color,
        size::{PxSize, Size, SizeUnit},
    },
    ui::{Dom, DomComPareResult, Widget},
    vertex::{
        colored_vertex::ColoredVertex, textured_vertex::TexturedVertex, vertex_generator::{BorderDescriptor, RectangleDescriptor}
    },
};

pub struct SquareDescriptor {
    pub label: Option<String>,
    pub size: Size,
    pub radius: f32,
    pub background_color: Color,

    pub border_width: f32,
    pub border_color: Color,

    pub div: u16,
}

impl Default for SquareDescriptor {
    fn default() -> Self {
        Self {
            label: None,
            size: Size {
                width: SizeUnit::Pixel(100.0),
                height: SizeUnit::Pixel(100.0),
            },
            radius: 0.0,
            background_color: Color::Rgb8USrgb { r: 0, g: 0, b: 0 },
            border_width: 0.0,
            border_color: Color::Rgb8USrgb { r: 0, g: 0, b: 0 },
            div: 0,
        }
    }
}

pub struct Square {
    label: Option<String>,
    size: Size,
    radius: f32,

    background_color: Color,

    border_width: f32,
    border_color: Color,

    div: u16,
}

impl Square {
    pub fn new(disc: SquareDescriptor) -> Self {
        Self {
            label: disc.label,
            size: disc.size,
            radius: disc.radius,
            background_color: disc.background_color,
            border_width: disc.border_width,
            border_color: disc.border_color,
            div: disc.div,
        }
    }
}

impl<R: Copy + Send + 'static> Dom<R> for Square {
    fn build_render_tree(&self) -> Box<dyn Widget<R>> {
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
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct SquareWidget {
    label: Option<String>,

    size: Size,
    radius: f32,
    background_color: Color,
    border_width: f32,
    border_color: Color,

    scene: vello::Scene,
    texture: Option<Arc<wgpu::Texture>>,
    vertex: Option<Arc<Vec<TexturedVertex>>>,
    index: Arc<Vec<u16>>,
}

impl<R: Copy + Send + 'static> Widget<R> for SquareWidget {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: PxSize,
        context: &SharedContext,
    ) -> crate::events::UiEventResult<R> {
        crate::events::UiEventResult::default()
    }

    fn is_inside(&self, position: [f32; 2], parent_size: PxSize, context: &SharedContext) -> bool {
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

    fn size(&self) -> Size {
        self.size
    }

    fn px_size(
        &self,
        parent_size: crate::types::size::PxSize,
        context: &crate::context::SharedContext,
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
        // ui environment
        parent_size: PxSize,
        // context
        context: &SharedContext,
    ) -> Vec<(
        Arc<wgpu::Texture>,
        Arc<Vec<crate::vertex::textured_vertex::TexturedVertex>>,
        Arc<Vec<u16>>,
        nalgebra::Matrix4<f32>,
    )> {
        let context = context;

        let size = self.size.to_px(parent_size, context);

        if self.texture.is_none() {
            let device = context.get_wgpu_device();
            let queue = context.get_wgpu_queue();

            // create texture
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: size.width as u32,
                    height: size.height as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });

            self.texture = Some(Arc::new(texture));

            // draw to texture

            let stroke = vello::kurbo::Stroke::new(6.0);
            let rect = vello::kurbo::RoundedRect::new(10.0, 10.0, 240.0, 240.0, 20.0);
            let rect_stroke_color = vello::peniko::Color::rgb(0.9804, 0.702, 0.5294);
            self.scene.stroke(
                &stroke,
                vello::kurbo::Affine::IDENTITY,
                rect_stroke_color,
                None,
                &rect,
            );
        }

        vec![(
            self.texture.clone().unwrap(),
            vec![],
            vec![],
            nalgebra::Matrix4::identity(),
        )]
    }
}
