use crate::{
    application_context::ApplicationContext,
    cosmic::{RenderAttribute, TextureAttribute},
    ui::{Ui, WidgetRenderObject, Widgets},
    vertex::{self, TexturedVertex},
};

pub struct Text<'a> {
    // text
    text: String,
    render_attribute: RenderAttribute<'a>,

    // context
    app_context: Option<ApplicationContext>,

    // object
    size: crate::types::Size,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    index_len: u32,
    texture: Option<wgpu::Texture>,
}

impl<'a> Text<'a> {
    pub fn new(text: String, size: crate::types::Size) -> Self {
        Self {
            text: text,
            render_attribute: RenderAttribute::new(),

            app_context: None,

            size,
            vertex_buffer: None,
            index_buffer: None,
            index_len: 0,
            texture: None,
        }
    }

    pub fn font_size(mut self, font_size: f32) -> Self {
        self.render_attribute.set_font_size(font_size);
        self
    }

    pub fn line_height(mut self, line_height: f32) -> Self {
        self.render_attribute.set_line_height(line_height);
        self
    }

    pub fn offset_px(mut self, offset_px: [f32; 2]) -> Self {
        self.render_attribute.set_offset_px(offset_px);
        self
    }

    pub fn font_color(mut self, font_color: [u8; 4]) -> Self {
        self.render_attribute.set_font_color(font_color);
        self
    }

    pub fn family(mut self, family: crate::cosmic::Family<'a>) -> Self {
        self.render_attribute.set_family(family);
        self
    }

    pub fn weight(mut self, weight: u16) -> Self {
        self.render_attribute.set_weight(weight);
        self
    }
}

impl<'a> Ui for Text<'a> {
    fn set_application_context(&mut self, context: ApplicationContext) {
        // create texture
        let texture = context
            .get_wgpu_device()
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Text Texture"),
                size: wgpu::Extent3d {
                    width: self.size.width as u32,
                    height: self.size.height as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

        // create vertex/index buffer
        let (vertex_buffer, index_buffer, index_len) = TexturedVertex::rectangle_buffer(
            &context,
            0.0,
            0.0,
            self.size.width as f32,
            self.size.height as f32,
            false,
        );

        // set
        self.app_context = Some(context);
        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
        self.index_len = index_len;
        self.texture = Some(texture);

        // render
        self.app_context.as_mut().unwrap().text_render(
            self.text.as_str(),
            &self.render_attribute,
            &TextureAttribute {
                width: self.size.width as u32,
                height: self.size.height as u32,
                texture: self.texture.as_ref().unwrap(),
            },
        );
    }

    fn size(&self) -> crate::types::Size {
        self.size
    }

    fn resize(&mut self, size: crate::types::Size) {
        self.size = size;

        if self.texture.is_none() {
            return;
        }

        let texture_new = self
            .app_context
            .as_ref()
            .unwrap()
            .get_wgpu_device()
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Text Texture"),
                size: wgpu::Extent3d {
                    width: self.size.width as u32,
                    height: self.size.height as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

        let mut encoder = self
            .app_context
            .as_ref()
            .unwrap()
            .get_wgpu_device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Text Texture Encoder"),
            });
        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture {
                texture: self.texture.as_ref().unwrap(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyTexture {
                texture: &texture_new,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: self.size.width as u32,
                height: self.size.height as u32,
                depth_or_array_layers: 1,
            },
        );

        let (vertex_buffer, index_buffer, index_len) = TexturedVertex::rectangle_buffer(
            &self.app_context.as_ref().unwrap(),
            0.0,
            0.0,
            self.size.width as f32,
            self.size.height as f32,
            false,
        );

        self.texture = Some(texture_new);
        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
        self.index_len = index_len;
    }
}

impl<'a> Widgets for Text<'a> {
    fn render_object(&self) -> Option<Vec<WidgetRenderObject>> {
        Some(vec![WidgetRenderObject {
            size: &self.size,
            offset: [0.0, 0.0],
            vertex_buffer: self.vertex_buffer.as_ref()?,
            index_buffer: self.index_buffer.as_ref()?,
            index_count: self.index_len,
            texture: self.texture.as_ref()?,
        }])
    }
}
