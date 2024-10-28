use wgpu::util::DeviceExt;

use crate::{application_context::ApplicationContext, types::color::Color};

use super::vertex_generator::{rectangle, RectangleDescriptor};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColoredVertex {
    position: [f32; 3],
    color: [f32; 4],
}

impl ColoredVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ColoredVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }

    pub fn atomic_rectangle(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: &Color,
    ) -> ([ColoredVertex; 4], [u16; 6]) {
        // 0-------3
        // | \     |
        // |   \   |
        // |     \ |
        // 1-------2
        let color = color.to_rgba_f32();
        (
            [
                ColoredVertex {
                    position: [x, y, 0.0],
                    color,
                },
                ColoredVertex {
                    position: [x, y - height, 0.0],
                    color,
                },
                ColoredVertex {
                    position: [x + width, y - height, 0.0],
                    color,
                },
                ColoredVertex {
                    position: [x + width, y, 0.0],
                    color,
                },
            ],
            [0, 1, 2, 0, 2, 3],
        )
    }

    pub fn atomic_rectangle_buffer(
        context: &ApplicationContext,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: &Color,
        compute: bool,
    ) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let (vertices, indices) = ColoredVertex::atomic_rectangle(x, y, width, height, color);

        let vertex_buffer;

        if compute {
            vertex_buffer = context.get_wgpu_device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
                },
            );
        } else {
            vertex_buffer = context.get_wgpu_device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            );
        }

        let index_buffer =
            context
                .get_wgpu_device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

        (vertex_buffer, index_buffer, indices.len() as u32)
    }

    pub fn rectangle(
        desc: RectangleDescriptor,
        color: &Color,
    ) -> (Vec<ColoredVertex>, Vec<u16>) {
        let (raw_vertex, index) = rectangle(desc);

        let mut vertex = Vec::with_capacity(raw_vertex.len());

        for raw_vertex in raw_vertex {
            vertex.push(ColoredVertex {
                position: raw_vertex.position,
                color: color.to_rgba_f32(),
            });
        }

        (vertex, index)
    }

    pub fn rectangle_buffer(
        context: &ApplicationContext,
        desc: RectangleDescriptor,
        color: &Color,
        compute: bool,
    ) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let (vertices, indices) = ColoredVertex::rectangle(desc, color);

        let vertex_buffer;

        if compute {
            vertex_buffer = context.get_wgpu_device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
                },
            );
        } else {
            vertex_buffer = context.get_wgpu_device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            );
        }

        let index_buffer =
            context
                .get_wgpu_device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

        (vertex_buffer, index_buffer, indices.len() as u32)
    }
}
