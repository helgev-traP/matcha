use crate::{
    context::SharedContext,
    cosmic,
    device::keyboard,
    events::{ElementState, UiEvent},
    types::{
        color::Color,
        size::{PxSize, Size, SizeUnit},
    },
    ui::{Dom, DomComPareResult, TextureSet, Widget},
    vertex::textured_vertex::TexturedVertex,
};

pub struct TextDescriptor {
    pub label: Option<String>,
    pub size: Size,

    pub font_size: f32,
    pub font_color: Color,
    pub text: String,
}

impl Default for TextDescriptor {
    fn default() -> Self {
        Self {
            label: None,
            size: Size {
                width: SizeUnit::Pixel(100.0),
                height: SizeUnit::Pixel(100.0),
            },
            font_size: 16.0,
            font_color: Color::Rgb8USrgb {
                r: 255,
                g: 255,
                b: 255,
            },
            text: "".to_string(),
        }
    }
}

pub struct Text {
    label: Option<String>,
    size: Size,

    font_size: f32,
    font_color: Color,
    text: String,
}

impl Text {
    pub fn new(disc: TextDescriptor) -> Self {
        Self {
            label: disc.label,
            size: disc.size,
            font_size: disc.font_size,
            font_color: disc.font_color,
            text: disc.text,
        }
    }
}

impl<T: Send + 'static> Dom<T> for Text {
    fn build_render_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(TextNode {
            label: self.label.clone(),
            size: self.size,
            font_size: self.font_size,
            font_color: self.font_color,
            text: self.text.clone(),
            text_cursor: self.text.len(),
            texture: None,
            redraw_texture: true,
            vertex_buffer: None,
            index_buffer: None,
            index_len: 0,
            reset_vertex: true,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct TextNode {
    label: Option<String>,
    size: Size,

    font_size: f32,
    font_color: Color,
    text: String,

    text_cursor: usize,

    texture: Option<wgpu::Texture>,
    redraw_texture: bool,

    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    index_len: u32,
    reset_vertex: bool,
}

impl<T: Send + 'static> Widget<T> for TextNode {
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
            crate::events::UiEventContent::KeyboardInput { key, element_state } => {
                if let ElementState::Pressed(_) = element_state {
                    match key {
                        keyboard::Key::Character(c) => {
                            self.text.insert(self.text_cursor, *c);
                            self.text_cursor += 1;
                            self.redraw_texture = true;
                            self.reset_vertex = true;
                        }
                        keyboard::Key::Spacial(keyboard::NamedKey::Space) => {
                            self.text.insert(self.text_cursor, ' ');
                            self.text_cursor += 1;
                            self.redraw_texture = true;
                            self.reset_vertex = true;
                        }
                        keyboard::Key::Spacial(keyboard::NamedKey::Return) => {
                            self.text.insert(self.text_cursor, '\n');
                            self.text_cursor += 1;
                            self.redraw_texture = true;
                            self.reset_vertex = true;
                        }
                        keyboard::Key::Spacial(keyboard::NamedKey::Backspace) => {
                            if self.text_cursor > 0 {
                                self.text_cursor -= 1;
                                self.text.remove(self.text_cursor);
                                self.redraw_texture = true;
                                self.reset_vertex = true;
                            }
                        }
                        keyboard::Key::Spacial(keyboard::NamedKey::Delete) => {
                            if self.text_cursor < self.text.len() {
                                self.text.remove(self.text_cursor);
                                self.redraw_texture = true;
                                self.reset_vertex = true;
                            }
                        }
                        keyboard::Key::Spacial(keyboard::NamedKey::ArrowLeft) => {
                            if self.text_cursor > 0 {
                                self.text_cursor -= 1;
                            }
                        }
                        keyboard::Key::Spacial(keyboard::NamedKey::ArrowRight) => {
                            if self.text_cursor < self.text.len() {
                                self.text_cursor += 1;
                            }
                        }
                        _ => {}
                    }
                }
                crate::events::UiEventResult::default()
            }
            _ => crate::events::UiEventResult::default(),
        }
    }

    fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: PxSize,
        context: &SharedContext,
    ) -> bool {
        let current_size = self.size.to_px(parent_size, context);

        if position[0] >= 0.0
            && position[0] <= current_size.width
            && position[1] >= 0.0
            && position[1] <= current_size.height
        {
            true
        } else {
            false
        }
    }

    fn update_render_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<Text>() {
            Err(())
        } else {
            let dom = dom.as_any().downcast_ref::<Text>().unwrap();
            todo!()
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(_) = dom.as_any().downcast_ref::<Text>() {
            todo!()
        } else {
            DomComPareResult::Different
        }
    }

    fn size(&self) -> Size {
        self.size
    }

    fn px_size(&self, parent_size: PxSize, context: &SharedContext) -> PxSize {
        self.size.to_px(parent_size, context)
    }

    fn default_size(&self) -> PxSize {
        PxSize {
            width: 0.0,
            height: 0.0,
        }
    }

    fn render(
        &mut self,
        texture: Option<&TextureSet>,
        parent_size: PxSize,
        affine: nalgebra::Matrix4<f32>,
        context: &SharedContext,
    )
    {
        if self.redraw_texture {
            let context = context;
            let current_size = self.size.to_px(parent_size, context);

            // allocate texture
            if self.texture.is_none() {
                let texture = context
                    .get_wgpu_device()
                    .create_texture(&wgpu::TextureDescriptor {
                        size: wgpu::Extent3d {
                            width: current_size.width as u32,
                            height: current_size.height as u32,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
                        label: Some("Text Texture"),
                        view_formats: &[],
                    });

                self.texture = Some(texture.into());
            }

            let texture = self.texture.as_ref().unwrap();

            context.text_render(
                &self.text,
                cosmic::RenderAttribute {
                    font_size: self.font_size,
                    font_color: self.font_color.to_rgba_u8(),
                    offset_px: [0.0, 0.0],
                    text_attr: cosmic_text::Attrs::new(),
                    line_height: self.font_size,
                },
                cosmic::TextureAttribute {
                    width: current_size.width as u32,
                    height: current_size.height as u32,
                    texture,
                },
            );

            self.redraw_texture = false;
        }

        if self.reset_vertex {
            let context = context;
            let current_size = self.size.to_px(parent_size, context);

            let (vertex_buffer, index_buffer, index_len) = TexturedVertex::atomic_rectangle_buffer(
                &context,
                0.0,
                0.0,
                current_size.width,
                current_size.height,
                false,
            );

            self.vertex_buffer = Some(vertex_buffer);
            self.index_buffer = Some(index_buffer);
            self.index_len = index_len;

            self.reset_vertex = false;
        }

        // render texture

        todo!()
    }
}
