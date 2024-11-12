use wgpu::util::DeviceExt;

use crate::context::SharedContext;

use super::vertex_generator::{border, rectangle, BorderDescriptor, RectangleDescriptor};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColoredVertex {
    position: [f32; 3],
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
            ],
        }
    }

    pub fn atomic_rectangle(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> ([ColoredVertex; 4], [u16; 6]) {
        // 0-------3
        // | \     |
        // |   \   |
        // |     \ |
        // 1-------2
        (
            [
                ColoredVertex {
                    position: [x, y, 0.0],
                },
                ColoredVertex {
                    position: [x, y - height, 0.0],
                },
                ColoredVertex {
                    position: [x + width, y - height, 0.0],
                },
                ColoredVertex {
                    position: [x + width, y, 0.0],
                },
            ],
            [0, 1, 2, 0, 2, 3],
        )
    }

    pub fn atomic_rectangle_buffer(
        context: &SharedContext,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        compute: bool,
    ) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let (vertices, indices) = ColoredVertex::atomic_rectangle(x, y, width, height);

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
    ) -> (Vec<ColoredVertex>, Vec<u16>) {
        let (raw_vertex, index) = rectangle(desc);

        let mut vertex = Vec::with_capacity(raw_vertex.len());

        for raw_vertex in raw_vertex {
            vertex.push(ColoredVertex {
                position: raw_vertex.position,
            });
        }

        (vertex, index)
    }

    pub fn rectangle_buffer(
        context: &SharedContext,
        desc: RectangleDescriptor,
        compute: bool,
    ) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let (vertices, indices) = ColoredVertex::rectangle(desc);

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

    pub fn border(
        desc: BorderDescriptor,
    ) -> (Vec<ColoredVertex>, Vec<u16>) {
        let (raw_vertex, index) = border(desc);

        let mut vertex = Vec::with_capacity(raw_vertex.len());

        for raw_vertex in raw_vertex {
            vertex.push(ColoredVertex {
                position: raw_vertex.position,
            });
        }

        (vertex, index)
    }

    pub fn border_buffer(
        context: &SharedContext,
        desc: BorderDescriptor,
        compute: bool,
    ) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let (vertices, indices) = ColoredVertex::border(desc);

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
