use std::mem::swap;

use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::font::Font;
use font_kit::handle::Handle;
use font_kit::hinting::HintingOptions;
use font_kit::source::SystemSource;
use image::{ImageBuffer, Rgba};
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::vector::Vector2I;
use wgpu::core::device;
use wgpu::naga::proc::index;
use wgpu::util::DeviceExt;

use crate::types::Size;
use crate::ui::{Ui, WidgetRenderObject, Widgets};
use crate::vertex::{self, TexturedVertex};

pub struct Character {
    // text
    chr: char,
    font: Font,
    size: Size,
    color: [u8; 4],

    temp_canvas: Option<Canvas>,

    // wgpu device
    device_queue: Option<crate::application_context::ApplicationContext>,

    // text rendering to texture
    pipeline: Option<wgpu::RenderPipeline>,
    texture_bind_group_layout: Option<wgpu::BindGroupLayout>,
    texture: Option<wgpu::Texture>,

    // render object
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    index_len: u32,
}

impl Character {
    pub fn new(chr: char, size: f32, color: [u8; 4]) -> Self {
        let font = SystemSource::new()
            .select_by_postscript_name("ArialMT")
            .unwrap()
            .load()
            .unwrap();

        println!("{:?}", font);

        let glyph_id = font.glyph_for_char('A').unwrap();
        let mut canvas = Canvas::new(Vector2I::new(size as i32, size as i32), Format::A8);

        font.rasterize_glyph(
            &mut canvas,
            glyph_id,
            32.0,
            Transform2F::default(),
            HintingOptions::None,
            RasterizationOptions::GrayscaleAa,
        )
        .unwrap();

        Self {
            chr,
            font,
            size: Size {
                width: size,
                height: size,
            },
            color,
            temp_canvas: Some(canvas),
            device_queue: None,
            pipeline: None,
            texture_bind_group_layout: None,
            texture: None,
            vertex_buffer: None,
            index_buffer: None,
            index_len: 0,
        }
    }

    fn pipeline_texture_setup(
        &mut self,
        device_queue: &crate::application_context::ApplicationContext,
    ) {
        let device = device_queue.get_wgpu_device();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Panel Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
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

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Panel Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Character Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("character.wgsl").into()),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Character Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex::TexturedVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        self.texture_bind_group_layout = Some(texture_bind_group_layout);
        self.pipeline = Some(render_pipeline);
    }

    fn rasterize_to_texture(&mut self) -> Option<()> {
        // rasterize to canvas

        if let None = self.temp_canvas.as_ref() {
            let glyph_id = self.font.glyph_for_char(self.chr).unwrap();

            self.font
                .rasterize_glyph(
                    self.temp_canvas.as_mut()?,
                    glyph_id,
                    32.0,
                    Transform2F::default(),
                    HintingOptions::None,
                    RasterizationOptions::GrayscaleAa,
                )
                .unwrap();
        }

        // ! debug

        println!("{:?}", self.temp_canvas.as_ref().unwrap().pixels);

        for i in 40..60 {
            self.temp_canvas.as_mut().unwrap().pixels[i] = 255;
        }

        // copy canvas to texture

        let device = self.device_queue.as_ref()?.get_wgpu_device();
        let queue = self.device_queue.as_ref()?.get_wgpu_queue();

        let texture_r8unorm = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("Character Texture"),
                size: wgpu::Extent3d {
                    width: self.size.width as u32,
                    height: self.size.height as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R8Unorm,
                usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            self.temp_canvas.as_ref()?.pixels.as_slice(),
        );

        // render to texture

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Character Texture"),
            size: wgpu::Extent3d {
                width: self.size.width as u32,
                height: self.size.height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let texture_r8unorm_view =
            texture_r8unorm.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_r8unorm_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Character Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_r8unorm_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Character Texture Bind Group"),
            layout: self.texture_bind_group_layout.as_ref()?,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_r8unorm_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_r8unorm_sampler),
                },
            ],
        });

        let (vertex_buffer, index_buffer, index_len) = TexturedVertex::rectangle_buffer(
            self.device_queue.as_ref()?,
            -1.0,
            1.0,
            2.0,
            2.0,
            false,
        );

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Character Texture Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Character Texture Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(self.pipeline.as_ref()?);
            render_pass.set_bind_group(0, &texture_r8unorm_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..index_len, 0, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));

        self.temp_canvas = None;
        self.texture = Some(texture);

        Some(())
    }
}

impl Ui for Character {
    fn set_application_context(
        &mut self,
        device_queue: crate::application_context::ApplicationContext,
    ) {
        // set pipeline and texture

        self.pipeline_texture_setup(&device_queue);

        // set vertex buffer and index buffer

        let (vertex_buffer, index_buffer, index_len) = TexturedVertex::rectangle_buffer(
            &device_queue,
            0.0,
            0.0,
            self.size.width as f32,
            self.size.height as f32,
            false,
        );

        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
        self.index_len = index_len;

        self.device_queue = Some(device_queue);

        // render to texture

        self.rasterize_to_texture();
    }

    fn size(&self) -> crate::types::Size {
        self.size
    }

    fn resize(&mut self, size: crate::types::Size) {
        self.size = size;
    }
}

impl Widgets for Character {
    fn render_object(&self) -> Option<Vec<WidgetRenderObject>> {
        Some(vec![WidgetRenderObject {
            size: &self.size,
            offset: [0.0, 0.0],
            vertex_buffer: self.vertex_buffer.as_ref()?,
            index_buffer: self.index_buffer.as_ref()?,
            index_count: 6,
            texture: self.texture.as_ref()?,
        }])
    }
}
