use wgpu::util::DeviceExt;

use super::{application_context::ApplicationContext, types::Color};

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
    pub fn rectangle(x: f32, y: f32, width: f32, height: f32) -> ([TexturedVertex; 4], [u16; 6]) {
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

    pub fn rectangle_buffer(
        device_queue: &ApplicationContext,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        compute: bool,
    ) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let (vertices, indices) = TexturedVertex::rectangle(x, y, width, height);

        let vertex_buffer;

        if compute {
            vertex_buffer = device_queue.get_wgpu_device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
                },
            );
        } else {
            vertex_buffer = device_queue.get_wgpu_device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            );
        }

        let index_buffer =
            device_queue
                .get_wgpu_device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

        (vertex_buffer, index_buffer, indices.len() as u32)
    }

    pub fn radius_rectangle(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
        div: u16,
    ) -> (Vec<TexturedVertex>, Vec<u16>) {
        todo!()
    }

    pub fn radius_rectangle_buffer(
        device_queue: &ApplicationContext,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
        div: u16,
    ) -> (wgpu::Buffer, wgpu::Buffer) {
        todo!()
    }
}

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

    pub fn rectangle(
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
        let color =color.to_rgba_f32();
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

    pub fn rectangle_buffer(
        app_context: &ApplicationContext,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: &Color,
        compute: bool,
    ) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let (vertices, indices) = ColoredVertex::rectangle(x, y, width, height, color);

        let vertex_buffer;

        if compute {
            vertex_buffer = app_context.get_wgpu_device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
                },
            );
        } else {
            vertex_buffer = app_context.get_wgpu_device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            );
        }

        let index_buffer =
            app_context
                .get_wgpu_device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

        (vertex_buffer, index_buffer, indices.len() as u32)
    }
}
