use std::sync::Arc;

use wgpu::core::device;
use wgpu::util::DeviceExt;

use winit::{self, event::Event, platform::run_on_demand::EventLoopExtRunOnDemand};

use cgmath::prelude::*;

use super::types::Rgba8Uint;
use super::window::DeviceQueue;

pub struct Panel {
    pub device_queue: DeviceQueue,
    pub texture: wgpu::Texture,
    pub height: u32,
    pub width: u32,
    pub base_color: Rgba8Uint,
    // render_pipeline: wgpu::RenderPipeline,
}

impl Panel {
    pub fn new(
        device_queue: DeviceQueue,
        texture: wgpu::Texture,
        width: u32,
        height: u32,
        base_color: Rgba8Uint,
    ) -> Self {
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
            device_queue,
            texture,
            height,
            width,
            base_color,
            // render_pipeline,
        }
    }

    pub fn render(&self) -> &wgpu::Texture {
        let texture_view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device_queue.get_device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Panel Encoder"),
            },
        );

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Panel Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(
                            wgpu::Color {
                                r: self.base_color.0[0] as f64 / 255.0,
                                g: self.base_color.0[1] as f64 / 255.0,
                                b: self.base_color.0[2] as f64 / 255.0,
                                a: self.base_color.0[3] as f64 / 255.0,
                            },
                        ),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        self.device_queue
            .get_queue()
            .submit(std::iter::once(encoder.finish()));

        &self.texture
    }

    pub fn get_texture(&self) -> &wgpu::Texture {
        &self.texture
    }
}
