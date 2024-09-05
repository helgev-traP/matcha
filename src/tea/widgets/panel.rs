use std::sync::Arc;

use wgpu::core::device;
use wgpu::util::DeviceExt;

use winit::{self, event::Event, platform::run_on_demand::EventLoopExtRunOnDemand};

use cgmath::prelude::*;

use super::{super::window::DeviceQueue, Elements};

pub struct Panel {
    pub height: u32,
    pub width: u32,
    pub base_color: [f64; 4],
    // render_pipeline: wgpu::RenderPipeline,
    device_queue: Option<crate::window::DeviceQueue>,
    texture: Option<wgpu::Texture>,

    inner_elements: Option<Box<dyn Elements>>,
}

impl Panel {
    pub fn new(width: u32, height: u32, base_color: [f64; 4]) -> Self {
        /*
        let device = device_queue.get_device();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Panel Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Panel Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &device_queue
                    .get_device()
                    .create_shader_module(&wgpu::include_spirv!("../shaders/panel.vert.spv")),
                entry_point: "main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &device_queue
                    .get_device()
                    .create_shader_module(&wgpu::include_spirv!("../shaders/panel.frag.spv")),
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: texture.format(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });
        */
        Self {
            height,
            width,
            base_color,
            // render_pipeline,
            device_queue: None,
            texture: None,
            inner_elements: None,
        }
    }

    pub fn set_base_color(&mut self, base_color: [f64; 4]) {
        self.base_color = base_color;
    }

    pub fn set_inner_elements(&mut self, inner_elements: Box<dyn Elements>) {
        self.inner_elements = Some(inner_elements);
    }

    pub fn set_inner_elements_device_queue(&mut self, device_queue: DeviceQueue) {
        self.inner_elements.as_mut().unwrap().set_device_queue(device_queue);
    }
}

impl Elements for Panel {
    fn set_device_queue(&mut self, device_queue: DeviceQueue) {
        let texture = device_queue
            .get_device()
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Teacup Texture"),
                size: wgpu::Extent3d {
                    width: self.width,
                    height: self.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

        self.set_inner_elements_device_queue(device_queue.clone());

        self.device_queue = Some(device_queue);
        self.texture = Some(texture);
    }

    fn size(&self) -> &crate::types::Size {
        todo!()
    }

    fn resize(&mut self, size: crate::types::Size) {
        self.width = size.width;
        self.height = size.height;
    }

    fn render(&self) -> Option<&wgpu::Texture> {
        let texture_view = self
            .texture
            .as_ref()?
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device_queue
            .as_ref()?
            .get_device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Panel Encoder"),
            });

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Panel Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.base_color[0],
                            g: self.base_color[1],
                            b: self.base_color[2],
                            a: self.base_color[3],
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        if self.inner_elements.is_some() {
            let inner_texture = self.inner_elements.as_ref()?.render();

            let size = self.inner_elements.as_ref()?.size();

            encoder.copy_texture_to_texture(
                wgpu::ImageCopyTexture {
                    texture: inner_texture.as_ref()?,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyTexture {
                    texture: self.texture.as_ref()?,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
            )
        }

        self.device_queue
            .as_ref()?
            .get_queue()
            .submit(std::iter::once(encoder.finish()));

        self.texture.as_ref()
    }
}
