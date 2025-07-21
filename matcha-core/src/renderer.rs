use std::{collections::HashMap, hash::Hash};

use crate::{ui::Object, ui::WidgetContext, vertex::UvVertex};

pub mod texture_copy;

mod texture_color_renderer;
use texture_color_renderer::TextureObjectRenderer;
mod vertex_color_renderer;
use vertex_color_renderer::VertexColorRenderer;
use wgpu::core::instance;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceData {
    viewport_position: [[f32; 2]; 2], // [[x0, x1], [y0, y1]]
    atlas_page: u32,
    atlas_position: [[f32; 2]; 2], // [[x0, x1], [y0, y1]]

    stencil_group: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct StencilData {
    viewport_position: [[f32; 2]; 2], // [[x0, x1], [y0, y1]]
    atlas_page: u32,
    atlas_position: [[f32; 2]; 2], // [[x0, x1], [y0, y1]]
}

pub struct ObjectRenderer {
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_sampler: wgpu::Sampler,

    // instance_buffer: wgpu::Buffer,
    // stencil_buffer: wgpu::Buffer,
    // visible_instances_buffer: wgpu::Buffer,
    // instances_atomic_counter: wgpu::Buffer,

    culling_pipeline_layout: wgpu::PipelineLayout,
    culling_pipeline: wgpu::ComputePipeline,
    command_pipeline_layout: wgpu::PipelineLayout,
    command_pipeline: wgpu::ComputePipeline,
    render_pipeline_layout: wgpu::PipelineLayout,
    render_pipeline: wgpu::RenderPipeline,
}

impl ObjectRenderer {
    pub fn new(ctx: &WidgetContext) -> Self {
        let device = &ctx.device();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("TextureObjectRenderer: Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let affine_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("TextureObjectRenderer: Affine Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("TextureObjectRenderer: Pipeline Layout"),
            bind_group_layouts: &[&affine_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("TextureObjectRenderer: Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("renderer.wgsl").into()),
        });

        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("TextureObjectRenderer: Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        todo!()
    }

    fn create_culling_pipeline(
        &self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> wgpu::ComputePipeline {
        todo!()
    }

    fn create_command_pipeline(
        &self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> wgpu::ComputePipeline {
        todo!()
    }

    fn create_render_pipeline(
        &self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        todo!()
    }

    pub fn render(
        &self,
        // gpu
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        // surface format
        surface_format: wgpu::TextureFormat,
        // destination
        destination_view: &wgpu::TextureView,
        destination_size: [f32; 2],
        // objects
        objects: Object,
        // texture atlas
        texture_atlas: wgpu::Texture,
        stencil_atlas: wgpu::Texture,
    ) {
        // integrate objects into a instance array
        let mut instances: Vec<InstanceData> = Vec::new();
        todo!();

        // update or create buffers and bind groups
        todo!();

        // start command encoder

        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ObjectRenderer: Command Encoder"),
        });

        // culling compute pass
        {
            let mut culling_pass =
                command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("ObjectRenderer: Culling Pass"),
                    ..Default::default()
                });
            culling_pass.set_pipeline(&self.culling_pipeline);
            culling_pass.set_bind_group(0, &self.culling_bind_group, &[]);
            culling_pass.dispatch_workgroups(0, 1, 1);
        }

        // command encoding pass
        {
            let mut command_pass =
                command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("ObjectRenderer: Command Pass"),
                    ..Default::default()
                });
            command_pass.set_pipeline(&self.command_pipeline);
            command_pass.set_bind_group(0, &self.command_bind_group, &[]);
            command_pass.dispatch_workgroups(1, 1, 1);
        }

        // render pass
        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ObjectRenderer: Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: destination_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            todo!();
        }
    }
}

fn mesh_integrate(objects: Vec<Object>) -> Vec<(wgpu::TextureView, Vec<UvVertex>, Vec<u16>)> {
    todo!()
}

#[rustfmt::skip]
fn make_normalize_matrix(destination_size: [f32; 2]) -> nalgebra::Matrix4<f32> {
    nalgebra::Matrix4::new(
        2.0 / destination_size[0], 0.0, 0.0, -1.0,
        0.0, 2.0 / destination_size[1], 0.0, 1.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    )
}
