use std::sync::Arc;

use crate::{
    context::SharedContext,
    events::UiEvent,
    renderer::Renderer,
    types::{
        color::Color,
        size::{PxSize, Size, Size},
    },
    ui::{Dom, DomComPareResult, Widget},
    vertex::uv_vertex::UvVertex,
};

pub struct ButtonDescriptor<T>
where
    T: Send + Clone + 'static,
{
    pub label: Option<String>,

    // default
    pub size: Size,
    pub radius: f32,
    pub background_color: Color,
    pub border_width: f32,
    pub border_color: Color,

    // hover
    pub hover_background_color: Option<Color>,
    pub hover_border_width: Option<f32>,
    pub hover_border_color: Option<Color>,

    // logic
    pub onclick: Option<T>,

    // inner content
    pub content_position: Option<nalgebra::Matrix4<f32>>,
    pub content: Option<Box<dyn Dom<T>>>,
}

impl<T> Default for ButtonDescriptor<T>
where
    T: Send + Clone + 'static,
{
    fn default() -> Self {
        Self {
            label: None,
            size: Size {
                width: Size::Parent(100.0),
                height: Size::Parent(100.0),
            },
            radius: 0.0,
            background_color: Color::Rgba8USrgb {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            border_width: 0.0,
            border_color: Color::Rgba8USrgb {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            hover_background_color: None,
            hover_border_width: None,
            hover_border_color: None,
            onclick: None,
            content_position: None,
            content: None,
        }
    }
}

impl<T> ButtonDescriptor<T>
where
    T: Send + Clone + 'static,
{
    pub fn new(label: Option<&str>) -> Self {
        Self {
            label: label.map(|s| s.to_string()),
            ..Default::default()
        }
    }

    pub fn normal(
        mut self,
        size: Size,
        radius: f32,
        background_color: Color,
        border_width: f32,
        border_color: Color,
    ) -> Self {
        self.size = size;
        self.radius = radius;
        self.background_color = background_color;
        self.border_width = border_width;
        self.border_color = border_color;
        self
    }

    pub fn hover(
        mut self,
        hover_background_color: Color,
        hover_border_width: f32,
        hover_border_color: Color,
    ) -> Self {
        self.hover_background_color = Some(hover_background_color);
        self.hover_border_width = Some(hover_border_width);
        self.hover_border_color = Some(hover_border_color);
        self
    }

    pub fn onclick(self, onclick: T) -> Self {
        ButtonDescriptor {
            label: self.label,
            size: self.size,
            radius: self.radius,
            background_color: self.background_color,
            border_width: self.border_width,
            border_color: self.border_color,
            hover_background_color: self.hover_background_color,
            hover_border_width: self.hover_border_width,
            hover_border_color: self.hover_border_color,
            onclick: Some(onclick),
            content_position: self.content_position,
            content: self.content,
        }
    }

    pub fn content(mut self, content: Box<dyn Dom<T>>) -> Self {
        self.content = Some(content);
        self
    }
}

pub struct Button<T>
where
    T: Send + Clone + 'static,
{
    label: Option<String>,

    // default
    size: Size,
    radius: f32,
    background_color: Color,
    border_width: f32,
    border_color: Color,
    // hover
    hover_background_color: Option<Color>,
    hover_border_width: Option<f32>,
    hover_border_color: Option<Color>,
    // logic
    onclick: Option<T>,
    // inner content
    content_position: Option<nalgebra::Matrix4<f32>>,
    content: Option<Box<dyn Dom<T>>>,
}

impl<T> Button<T>
where
    T: Send + Clone + 'static,
{
    pub fn new(disc: ButtonDescriptor<T>) -> Box<Self> {
        Box::new(Self {
            label: disc.label,
            size: disc.size,
            radius: disc.radius,
            background_color: disc.background_color,
            border_width: disc.border_width,
            border_color: disc.border_color,
            hover_background_color: disc.hover_background_color,
            hover_border_width: disc.hover_border_width,
            hover_border_color: disc.hover_border_color,
            onclick: disc.onclick,
            content_position: disc.content_position,
            content: disc.content,
        })
    }
}

impl<T: Send + 'static> Dom<T> for Button<T>
where
    T: Send + Clone + 'static,
{
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(ButtonWidget {
            label: self.label.clone(),
            size: self.size,
            radius: self.radius,
            background_color: self.background_color,
            border_width: self.border_width,
            border_color: self.border_color,
            hover_background_color: self.hover_background_color,
            hover_border_width: self.hover_border_width,
            hover_border_color: self.hover_border_color,
            onclick: self.onclick.clone(),
            content_position: self.content_position,
            content: self.content.as_ref().map(|c| c.build_widget_tree()),
            is_hover: false,
            scene: vello::Scene::new(),
            texture: None,
            texture_hover: None,
            vertex: None,
            index: Arc::new(vec![0, 1, 2, 2, 3, 0]),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct ButtonWidget<T>
where
    T: Send + Clone + 'static,
{
    label: Option<String>,
    // default
    size: Size,
    radius: f32,
    background_color: Color,
    border_width: f32,
    border_color: Color,
    // hover
    hover_background_color: Option<Color>,
    hover_border_width: Option<f32>,
    hover_border_color: Option<Color>,
    // logic
    onclick: Option<T>,
    // inner content
    content_position: Option<nalgebra::Matrix4<f32>>,
    content: Option<Box<dyn Widget<T>>>,

    // input status
    is_hover: bool,

    // rendering
    scene: vello::Scene,
    texture: Option<Arc<wgpu::Texture>>,
    texture_hover: Option<Arc<wgpu::Texture>>,
    vertex: Option<Arc<Vec<UvVertex>>>,
    index: Arc<Vec<u16>>,
}

impl<T: Send + 'static> Widget<T> for ButtonWidget<T>
where
    T: Send + Clone + 'static,
{
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: PxSize,
        context: &SharedContext,
    ) -> crate::events::UiEventResult<T> {
        match &event.content {
            crate::events::UiEventContent::MouseClick {
                position,
                click_state,
                button,
            } => {
                if self.is_inside(*position, parent_size, context) {
                    if button == &crate::device::mouse::MouseButton::Primary {
                        if let crate::events::ElementState::Pressed(_) = click_state {
                            if let Some(onclick) = self.onclick.clone() {
                                return crate::events::UiEventResult {
                                    user_event: Some(onclick),
                                };
                            }
                        }
                    }
                }
                crate::events::UiEventResult::default()
            }
            crate::events::UiEventContent::CursorEntered => {
                self.is_hover = true;
                crate::events::UiEventResult::default()
            }
            crate::events::UiEventContent::CursorLeft => {
                self.is_hover = false;
                crate::events::UiEventResult::default()
            }
            _ => crate::events::UiEventResult::default(),
        }
    }

    fn is_inside(&self, position: [f32; 2], parent_size: PxSize, context: &SharedContext) -> bool {
        let current_size = self.size.unwrap_to_px(parent_size, context);

        !(position[0] < 0.0
            || position[0] > current_size.width
            || position[1] < 0.0
            || position[1] > current_size.height)
    }

    fn update_widget_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<Button<T>>() {
            Err(())
        } else {
            let dom = dom.as_any().downcast_ref::<Button<T>>().unwrap();

            if let Some(content) = self.content.as_mut() {
                if let Some(dom_content) = dom.content.as_ref() {
                    content.update_widget_tree(&**dom_content)?;
                } else {
                    return Err(());
                }
            }

            // check content change
            if self.size != dom.size
                || self.radius != dom.radius
                || self.background_color != dom.background_color
                || self.border_width != dom.border_width
                || self.border_color != dom.border_color
                || self.hover_background_color != dom.hover_background_color
                || self.hover_border_width != dom.hover_border_width
                || self.hover_border_color != dom.hover_border_color
            {
                return Err(());
            }

            // update
            self.label = dom.label.clone();
            self.onclick = dom.onclick.clone();
            self.content_position = dom.content_position;

            Ok(())
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(_) = dom.as_any().downcast_ref::<Button<T>>() {
            todo!()
        } else {
            DomComPareResult::Different
        }
    }

    fn size(&self) -> Size {
        self.size
    }

    fn px_size(&self, parent_size: PxSize, context: &SharedContext) -> PxSize {
        self.size.unwrap_to_px(parent_size, context)
    }

    fn default_size(&self) -> PxSize {
        todo!()
    }

    fn render(
        &mut self,
        // ui environment
        parent_size: PxSize,
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
        let size = self.size.unwrap_to_px(parent_size, context);

        if self.vertex.is_none() {
            let vertex = vec![
                UvVertex {
                    position: [0.0, 0.0, 0.0].into(),
                    tex_coords: [0.0, 0.0].into(),
                },
                UvVertex {
                    position: [0.0, -size.height, 0.0].into(),
                    tex_coords: [0.0, 1.0].into(),
                },
                UvVertex {
                    position: [size.width, -size.height, 0.0].into(),
                    tex_coords: [1.0, 1.0].into(),
                },
                UvVertex {
                    position: [size.width, 0.0, 0.0].into(),
                    tex_coords: [1.0, 0.0].into(),
                },
            ];

            self.vertex = Some(Arc::new(vertex));
        }

        if self.is_hover {
            if self.texture_hover.is_none() {
                let device = context.get_wgpu_device();

                // create texture
                self.texture_hover =
                    Some(Arc::new(device.create_texture(&wgpu::TextureDescriptor {
                        label: None,
                        size: wgpu::Extent3d {
                            width: size.width as u32,
                            height: size.height as u32,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING
                            | wgpu::TextureUsages::STORAGE_BINDING,
                        view_formats: &[],
                    })));

                // draw

                self.scene.reset();

                let c = self
                    .hover_background_color
                    .as_ref()
                    .unwrap_or(&self.background_color)
                    .to_rgba_f64();

                self.scene.fill(
                    vello::peniko::Fill::EvenOdd,
                    vello::kurbo::Affine::IDENTITY,
                    vello::peniko::Color::rgba(c[0], c[1], c[2], c[3]),
                    None,
                    &vello::kurbo::RoundedRect::new(
                        0.0,
                        0.0,
                        size.width as f64,
                        size.height as f64,
                        self.radius as f64,
                    ),
                );

                if self.hover_border_width.unwrap_or(self.border_width) > 0.0 {
                    let c = self
                        .hover_border_color
                        .unwrap_or(self.border_color)
                        .to_rgba_f64();

                    self.scene.stroke(
                        &vello::kurbo::Stroke::new(
                            self.hover_border_width.unwrap_or(self.border_width) as f64,
                        )
                        .with_join(vello::kurbo::Join::Bevel),
                        vello::kurbo::Affine::IDENTITY,
                        vello::peniko::Color::rgba(c[0], c[1], c[2], c[3]),
                        None,
                        &vello::kurbo::RoundedRect::new(
                            self.hover_border_width.unwrap_or(self.border_width) as f64 / 2.0,
                            self.hover_border_width.unwrap_or(self.border_width) as f64 / 2.0,
                            size.width as f64
                                - self.hover_border_width.unwrap_or(self.border_width) as f64 / 2.0,
                            size.height as f64
                                - self.hover_border_width.unwrap_or(self.border_width) as f64 / 2.0,
                            self.radius as f64
                                - self.hover_border_width.unwrap_or(self.border_width) as f64 / 2.0,
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
                            .texture_hover
                            .as_ref()
                            .unwrap()
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                        &vello::RenderParams {
                            base_color: vello::peniko::Color::TRANSPARENT,
                            height: size.height as u32,
                            width: size.width as u32,
                            antialiasing_method: vello::AaConfig::Area,
                        },
                    )
                    .unwrap();
            }

            let mut content_render_data = self
                .content
                .as_mut()
                .map(|c| c.render(size, context, renderer, frame))
                .unwrap_or_default();

            for (_, _, _, matrix) in content_render_data.iter_mut() {
                *matrix = self
                    .content_position
                    .unwrap_or(nalgebra::Matrix4::identity())
                    * *matrix;
            }

            vec![
                vec![(
                    self.texture_hover.as_ref().unwrap().clone(),
                    self.vertex.as_ref().unwrap().clone(),
                    self.index.clone(),
                    nalgebra::Matrix4::identity(),
                )],
                content_render_data,
            ]
            .concat()
        } else {
            if self.texture.is_none() {
                let device = context.get_wgpu_device();

                // create texture
                self.texture = Some(Arc::new(device.create_texture(&wgpu::TextureDescriptor {
                    label: None,
                    size: wgpu::Extent3d {
                        width: size.width as u32,
                        height: size.height as u32,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::STORAGE_BINDING,
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
                        size.width as f64,
                        size.height as f64,
                        self.radius as f64,
                    ),
                );

                if self.border_width > 0.0 {
                    let c = self.border_color.to_rgba_f64();

                    self.scene.stroke(
                        &vello::kurbo::Stroke::new(self.border_width as f64)
                            .with_join(vello::kurbo::Join::Bevel),
                        vello::kurbo::Affine::IDENTITY,
                        vello::peniko::Color::rgba(c[0], c[1], c[2], c[3]),
                        None,
                        &vello::kurbo::RoundedRect::new(
                            self.border_width as f64 / 2.0,
                            self.border_width as f64 / 2.0,
                            size.width as f64 - self.border_width as f64 / 2.0,
                            size.height as f64 - self.border_width as f64 / 2.0,
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
                            height: size.height as u32,
                            width: size.width as u32,
                            antialiasing_method: vello::AaConfig::Area,
                        },
                    )
                    .unwrap();
            }

            let mut content_render_data = self
                .content
                .as_mut()
                .map(|c| c.render(size, context, renderer, frame))
                .unwrap_or_default();

            for (_, _, _, matrix) in content_render_data.iter_mut() {
                *matrix = self
                    .content_position
                    .unwrap_or(nalgebra::Matrix4::identity())
                    * *matrix;
            }

            vec![
                vec![(
                    self.texture.as_ref().unwrap().clone(),
                    self.vertex.as_ref().unwrap().clone(),
                    self.index.clone(),
                    nalgebra::Matrix4::identity(),
                )],
                content_render_data,
            ]
            .concat()
        }
    }
}
