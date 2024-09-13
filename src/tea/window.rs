use cgmath::prelude::*;
use std::sync::Arc;
use wgpu::{core::device, util::DeviceExt};
use winit::{self, event::Event, platform::run_on_demand::EventLoopExtRunOnDemand};

use crate::application_context::ApplicationContext;
use crate::cosmic::FontContext;
use crate::panels::panel::Panel;
use crate::types::Size;
use crate::ui::RenderArea;
use crate::ui::Ui;
use crate::ui::Widgets;

use super::widgets;
use super::widgets::teacup;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TextureCopyVertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

const VERTICES: &[TextureCopyVertex] = &[
    TextureCopyVertex {
        position: [-1.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    TextureCopyVertex {
        position: [-1.0, -1.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
    TextureCopyVertex {
        position: [1.0, -1.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
    TextureCopyVertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
];

impl TextureCopyVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TextureCopyVertex>() as wgpu::BufferAddress,
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
}

const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

struct WindowState<'a> {
    // winit
    winit_window: Arc<winit::window::Window>,
    // wgpu
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    surface: wgpu::Surface<'a>,
    app_context: ApplicationContext,
    config: wgpu::SurfaceConfiguration,
    // for texture copy
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    top_panel_texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl<'a> WindowState<'a> {
    async fn new(
        winit_window: Arc<winit::window::Window>,
        cosmic_context: Option<FontContext>,
    ) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(winit_window.clone()).unwrap();

        let adapter = instance
            .request_adapter(
                &(wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                }),
            )
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &(wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web, we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    memory_hints: wgpu::MemoryHints::default(),
                }),
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let size = winit_window.inner_size();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        // shader

        let texture_copy_shader = device.create_shader_module(wgpu::include_wgsl!("./window.wgsl"));

        // pipeline

        let top_panel_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Top Panel Texture Bind Group Layout"),
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
                label: Some("Window Render Pipeline Layout"),
                bind_group_layouts: &[&top_panel_texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Window Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &texture_copy_shader,
                entry_point: "vs_main",
                buffers: &[TextureCopyVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &texture_copy_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
                unclipped_depth: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let app_context;

        if cosmic_context.is_none() {
            app_context = ApplicationContext::new(device, queue);
        } else {
            app_context =
                ApplicationContext::new_with_context(device, queue, cosmic_context.unwrap());
        }

        Self {
            winit_window,
            instance,
            adapter,
            surface,
            app_context,
            config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            top_panel_texture_bind_group_layout,
        }
    }

    fn clone_device_queue(&self) -> ApplicationContext {
        self.app_context.clone()
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface
            .configure(&*self.app_context.get_wgpu_device(), &self.config);
    }

    fn render(&mut self, top_panel_texture: Option<&wgpu::Texture>) {
        let surface_texture = self.surface.get_current_texture().unwrap();
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // get texture from top panel

        let top_panel_texture_view = top_panel_texture
            .unwrap()
            .create_view(&wgpu::TextureViewDescriptor::default());

        let top_panel_texture_smapler =
            self.app_context
                .get_wgpu_device()
                .create_sampler(&wgpu::SamplerDescriptor {
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Nearest,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    ..Default::default()
                });

        let top_panel_texture_bind_group =
            self.app_context
                .get_wgpu_device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Top Panel Texture Bind Group"),
                    layout: &self.top_panel_texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&top_panel_texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&top_panel_texture_smapler),
                        },
                    ],
                });

        let mut encoder = self.app_context.get_wgpu_device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("WindowState encoder"),
            },
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &top_panel_texture_bind_group, &[]);
            render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }

        self.app_context
            .get_wgpu_queue()
            .submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }
}

pub struct Window<'a> {
    winit_window: Option<Arc<winit::window::Window>>,
    window: Option<WindowState<'a>>,
    top_panel: Panel,
    cosmic_context: Option<crate::cosmic::FontContext>,
}

impl<'a> Window<'a> {
    pub fn new() -> Self {
        Self {
            winit_window: None,
            window: None,
            top_panel: Panel::new(Size {
                width: -1.0,
                height: -1.0,
            }),
            cosmic_context: None,
        }
    }

    pub fn set_cosmic_context(&mut self, cosmic_context: crate::cosmic::FontContext) {
        self.cosmic_context = Some(cosmic_context);
    }

    pub fn get_top_panel(&mut self) -> &mut Panel {
        &mut self.top_panel
    }
}

impl<'a> winit::application::ApplicationHandler for Window<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.winit_window = Some(Arc::new(
            event_loop
                .create_window(winit::window::Window::default_attributes())
                .unwrap(),
        ));

        let context = std::mem::take(&mut self.cosmic_context);

        let window_state = pollster::block_on(WindowState::new(
            self.winit_window.as_ref().unwrap().clone(),
            context,
        ));

        self.window = Some(window_state);

        let size = self.winit_window.as_ref().unwrap().inner_size();

        self.top_panel.resize(Size {
            width: size.width as f32,
            height: size.height as f32,
        });

        self.top_panel
            .set_application_context(self.window.as_ref().unwrap().clone_device_queue());
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                self.window
                    .as_mut()
                    .unwrap()
                    .render(self.top_panel.render());
            }
            winit::event::WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    self.window.as_mut().unwrap().resize(new_size);
                    self.top_panel.resize(Size {
                        width: new_size.width as f32,
                        height: new_size.height as f32,
                    });
                }
            }
            _ => {}
        }
    }

    // ----------- The Optionals ------------

    fn new_events(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        let _ = (event_loop, cause);
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: ()) {
        let _ = (event_loop, event);
    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn memory_warning(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }
}
