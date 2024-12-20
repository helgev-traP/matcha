use nalgebra as na;
use std::any::Any;
use std::sync::Arc;
use wgpu::ImageCopyTextureBase;

use crate::events::UiEvent;
use crate::types::size::StdSize;
use crate::{
    context::SharedContext,
    events::UiEventResult,
    types::size::StdSize,
    ui::{Dom, DomComPareResult, Widget},
    vertex::uv_vertex::UvVertex,
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
                width: crate::types::size::Size::Pixel(100.0),
                height: crate::types::size::Size::Pixel(100.0),
            },
            frame_size: crate::types::size::Size {
                width: crate::types::size::Size::Pixel(100.0),
                height: crate::types::size::Size::Pixel(100.0),
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
    pub fn new(disc: TeacupDescriptor) -> Box<Self> {
        Box::new(Self {
            label: disc.label,
            size: disc.size,
            frame_size: disc.frame_size,
            position: disc.position,
            rotate_dig: disc.rotate,
            visible: disc.visible,
        })
    }
}

impl<R: 'static> Dom<R> for Teacup {
    fn build_widget_tree(&self) -> Box<dyn Widget<R>> {
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
            vertex: Arc::new(vec![
                UvVertex {
                    position: [0.0, 0.0, 0.0].into(),
                    tex_coords: [0.0, 0.0].into(),
                },
                UvVertex {
                    position: [0.0, -(height as f32), 0.0].into(),
                    tex_coords: [0.0, 1.0].into(),
                },
                UvVertex {
                    position: [width as f32, -(height as f32), 0.0].into(),
                    tex_coords: [1.0, 1.0].into(),
                },
                UvVertex {
                    position: [width as f32, 0.0, 0.0].into(),
                    tex_coords: [1.0, 0.0].into(),
                },
            ]),
            index: Arc::new(vec![0, 1, 2, 0, 2, 3]),
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

    texture: Option<Arc<wgpu::Texture>>,
    vertex: Arc<Vec<UvVertex>>,
    index: Arc<Vec<u16>>,
}

impl<R: 'static> Widget<R> for TeacupRenderNode {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn widget_event(&mut self, _: &UiEvent, _: PxSize, _: &SharedContext) -> UiEventResult<R> {
        Default::default()
    }

    fn is_inside(&self, position: [f32; 2], parent_size: PxSize, context: &SharedContext) -> bool {
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

    fn update_widget_tree(&mut self, dom: &dyn Dom<R>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<Teacup>() {
            return Err(());
        }

        let dom = dom.as_any().downcast_ref::<Teacup>().unwrap();

        self.size = dom.size;
        self.position = dom.position;
        self.rotate = dom.rotate_dig;

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

    fn render(
        &mut self,
        // ui environment
        parent_size: PxSize,
        // context
        context: &SharedContext,
        _: &crate::renderer::Renderer,
        _: u64,
    ) -> Vec<(
        Arc<wgpu::Texture>,
        Arc<Vec<UvVertex>>,
        Arc<Vec<u16>>,
        nalgebra::Matrix4<f32>,
    )> {
        let context = context;
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

            self.texture = Some(Arc::new(texture));
        }

        if self.visible {
            vec![(
                self.texture.as_ref().unwrap().clone(),
                self.vertex.clone(),
                self.index.clone(),
                na::Matrix4::new_translation(&na::Vector3::new(
                    self.position[0],
                    -self.position[1],
                    0.0,
                )),
            )]
        } else {
            vec![]
        }
    }

    fn size(&self) -> crate::types::size::Size {
        self.frame_size
    }

    fn px_size(&self, parent_size: PxSize, context: &SharedContext) -> PxSize {
        let mut size = StdSize::from_parent_size(self.frame_size, parent_size, context);
        if size.width.is_default() {
            size.width = StdSize::Pixel(self.picture_size.width);
            size.height = StdSize::Pixel(self.picture_size.height);
        }
        size.unwrap()
    }

    fn default_size(&self) -> PxSize {
        self.picture_size
    }
}
