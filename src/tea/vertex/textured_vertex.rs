use wgpu::util::DeviceExt;

use super::vertex_generator::{rectangle, RectangleDescriptor};
use crate::application_context::ApplicationContext;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TexturedVertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl TexturedVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TexturedVertex>() as wgpu::BufferAddress,
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
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }

    /// Create a rectangle with the given position of upper-left corner, width, and height.
    pub fn atomic_rectangle(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> ([TexturedVertex; 4], [u16; 6]) {
        // 0-------3
        // | \     |
        // |   \   |
        // |     \ |
        // 1-------2
        (
            [
                TexturedVertex {
                    position: [x, y, 0.0],
                    tex_coords: [0.0, 0.0],
                },
                TexturedVertex {
                    position: [x, y - height, 0.0],
                    tex_coords: [0.0, 1.0],
                },
                TexturedVertex {
                    position: [x + width, y - height, 0.0],
                    tex_coords: [1.0, 1.0],
                },
                TexturedVertex {
                    position: [x + width, y, 0.0],
                    tex_coords: [1.0, 0.0],
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
        compute: bool,
    ) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let (vertices, indices) = TexturedVertex::atomic_rectangle(x, y, width, height);

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
    ) -> (Vec<TexturedVertex>, Vec<u16>) {
        let (raw_vertex, index) = rectangle(desc);

        let mut vertex = Vec::with_capacity(raw_vertex.len());

        for raw_vertex in raw_vertex {
            vertex.push(TexturedVertex {
                position: raw_vertex.position,
                tex_coords: [
                    (raw_vertex.position[0] - desc.x) / desc.width,
                    (raw_vertex.position[1] + desc.y) / desc.height,
                ],
            });
        }

        (vertex, index)
    }

    pub fn rectangle_buffer(
        context: &ApplicationContext,
        desc: RectangleDescriptor,
        compute: bool,
    ) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let (vertices, indices) =
            TexturedVertex::rectangle(desc);

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
