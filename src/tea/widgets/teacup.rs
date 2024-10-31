use nalgebra as na;
use std::any::Any;
use wgpu::util::DeviceExt;
use wgpu::ImageCopyTextureBase;

use crate::events::UiEvent;
use crate::renderer::RendererCommandEncoder;
use crate::types::size::StdSizeUnit;
use crate::{
    application_context::ApplicationContext,
    events::UiEventResult,
    types::size::{PxSize, StdSize},
    ui::{Dom, DomComPareResult, RenderItem, RenderingTrait, Widget, WidgetTrait},
    vertex::textured_vertex::TexturedVertex,
};

pub struct TeacupDescriptor {
    pub label: Option<String>,
    pub size: crate::types::size::Size,
    pub frame_size: crate::types::size::Size,
    pub position: [f32; 2],
    pub rotate: f32,
    pub visible: bool,
}

impl Default for TeacupDescriptor {
    fn default() -> Self {
        Self {
            label: None,
            size: crate::types::size::Size {
                width: crate::types::size::SizeUnit::Pixel(100.0),
                height: crate::types::size::SizeUnit::Pixel(100.0),
            },
            frame_size: crate::types::size::Size {
                width: crate::types::size::SizeUnit::Pixel(100.0),
                height: crate::types::size::SizeUnit::Pixel(100.0),
            },
            position: [0.0, 0.0],
            rotate: 0.0,
            visible: true,
        }
    }
}

pub struct Teacup {
    label: Option<String>,
    size: crate::types::size::Size,
    frame_size: crate::types::size::Size,
    position: [f32; 2],
    rotate_dig: f32,
    visible: bool,
}

impl Teacup {
    pub fn new(disc: TeacupDescriptor) -> Self {
        Self {
            label: disc.label,
            size: disc.size,
            frame_size: disc.frame_size,
            position: disc.position,
            rotate_dig: disc.rotate,
            visible: disc.visible,
        }
    }
}

impl<R: 'static> Dom<R> for Teacup {
    fn build_render_tree(&self) -> Box<dyn Widget<R>> {
        let teacup_bytes = include_bytes!("./teacup.png");
        let teacup_image = image::load_from_memory(teacup_bytes).unwrap();
        let teacup_rgba = teacup_image.to_rgba8();
        let (width, height) = teacup_rgba.dimensions();

        Box::new(TeacupRenderNode {
            label: self.label.clone(),
            teacup_rgba,
            picture_size: crate::types::size::PxSize {
                width: width as f32,
                height: height as f32,
            },
            position: self.position,
            rotate: self.rotate_dig,
            size: self.size,
            frame_size: self.frame_size,
            visible: self.visible,
            texture: None,
            vertex_buffer: None,
            index_buffer: None,
            index_len: 0,
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct TeacupRenderNode {
    label: Option<String>,

    teacup_rgba: image::RgbaImage,
    picture_size: crate::types::size::PxSize,
    position: [f32; 2],
    rotate: f32,

    size: crate::types::size::Size,
    frame_size: crate::types::size::Size,

    visible: bool,

    // previous_size: Option<PxSize>,
    texture: Option<wgpu::Texture>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    index_len: u32,
}

impl<R: 'static> WidgetTrait<R> for TeacupRenderNode {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn widget_event(&mut self, _: &UiEvent, _: PxSize, _: &ApplicationContext) -> UiEventResult<R> {
        Default::default()
    }

    fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> bool {
        let size = PxSize::from_size_parent_size(self.size, parent_size, context);

        if position[0] < self.position[0]
            || position[0] > self.position[0] + size.width
            || position[1] < self.position[1]
            || position[1] > self.position[1] + size.height
        {
            false
        } else {
            true
        }
    }

    fn update_render_tree(&mut self, dom: &dyn Dom<R>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<Teacup>() {
            return Err(());
        }

        let dom = dom.as_any().downcast_ref::<Teacup>().unwrap();

        self.size = dom.size;
        self.position = dom.position;
        self.rotate = dom.rotate_dig;

        // clear cache

        self.vertex_buffer = None;
        self.index_buffer = None;
        self.index_len = 0;

        Ok(())
    }

    fn compare(&self, dom: &dyn Dom<R>) -> DomComPareResult {
        if let Some(teacup) = dom.as_any().downcast_ref::<Teacup>() {
            if teacup.size == self.size
                && teacup.position == self.position
                && teacup.rotate_dig == self.rotate
            {
                DomComPareResult::Same
            } else {
                DomComPareResult::Changed
            }
        } else {
            DomComPareResult::Different
        }
    }
}

impl RenderingTrait for TeacupRenderNode {
    fn render(
        &mut self,
        _: &rayon::Scope,
        parent_size: PxSize,
        affine: na::Matrix4<f32>,
        encoder: &RendererCommandEncoder,
    ) {
        let context = encoder.get_context();
        let device = context.get_wgpu_device();

        // calculate actual size

        let size = PxSize::from_size_parent_size(self.size, parent_size, context);

        // create texture

        if self.texture.is_none() {
            let texture_size = wgpu::Extent3d {
                width: self.picture_size.width as u32,
                height: self.picture_size.height as u32,
                depth_or_array_layers: 1,
            };

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Teacup Texture"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Texture Buffer"),
                contents: &self.teacup_rgba,
                usage: wgpu::BufferUsages::COPY_SRC,
            });

            context.get_wgpu_queue().write_texture(
                ImageCopyTextureBase {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &self.teacup_rgba,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * self.picture_size.width as u32),
                    rows_per_image: None,
                },
                texture_size,
            );

            // let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            //     label: Some("Texture Command Encoder"),
            // });
            // encoder.copy_buffer_to_texture(
            //     wgpu::ImageCopyBuffer {
            //         buffer: &buffer,
            //         layout: wgpu::ImageDataLayout {
            //             offset: 0,
            //             bytes_per_row: Some(((4 * self.picture_size.width as u32) / 256 + 1) * 256),
            //             rows_per_image: None,
            //         },
            //     },
            //     wgpu::ImageCopyTexture {
            //         texture: &texture,
            //         mip_level: 0,
            //         origin: wgpu::Origin3d::ZERO,
            //         aspect: wgpu::TextureAspect::All,
            //     },
            //     texture_size,
            // );
            // context.get_wgpu_queue().submit(Some(encoder.finish()));

            self.texture = Some(texture);
        }

        // create / update vertex buffer

        if self.vertex_buffer.is_none() || self.index_buffer.is_none() || self.index_len == 0 {
            let (vertex, index, index_len) = TexturedVertex::atomic_rectangle_buffer(
                context,
                0.0,
                0.0,
                size.width,
                size.height,
                false,
            );

            self.vertex_buffer = Some(vertex);
            self.index_buffer = Some(index);
            self.index_len = index_len;
        }

        // draw

        if self.visible {
            encoder.draw(
                RenderItem {
                    object: vec![crate::ui::Object::Textured {
                        vertex_buffer: self.vertex_buffer.as_ref().unwrap(),
                        index_buffer: self.index_buffer.as_ref().unwrap(),
                        index_len: self.index_len,
                        texture: self.texture.as_ref().unwrap().clone(),
                        object_affine: na::Matrix4::identity(),
                    }],
                },
                nalgebra::Matrix4::new_translation(&na::Vector3::new(
                    self.position[0],
                    -self.position[1],
                    0.0,
                )) * affine
                    * nalgebra::Matrix4::new_translation(&na::Vector3::new(
                        size.width / 2.0,
                        -size.height / 2.0,
                        0.0,
                    ))
                    * nalgebra::Matrix4::new_rotation(nalgebra::Vector3::new(
                        0.0,
                        0.0,
                        self.rotate.to_radians(),
                    ))
                    * nalgebra::Matrix4::new_translation(&na::Vector3::new(
                        -size.width / 2.0,
                        size.height / 2.0,
                        0.0,
                    )),
            );
        }
    }

    fn size(&self) -> crate::types::size::Size {
        self.frame_size
    }

    fn px_size(&self, parent_size: PxSize, context: &ApplicationContext) -> PxSize {
        let mut size = StdSize::from_parent_size(self.frame_size, parent_size, context);
        if size.width.is_none() {
            size.width = StdSizeUnit::Pixel(self.picture_size.width);
            size.height = StdSizeUnit::Pixel(self.picture_size.height);
        }
        size.unwrap()
    }

    fn default_size(&self) -> PxSize {
        self.picture_size
    }
}
