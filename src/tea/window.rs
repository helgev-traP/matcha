use std::sync::Arc;

use wgpu::{core::device, util::DeviceExt};

use winit::{self, event::Event, platform::run_on_demand::EventLoopExtRunOnDemand};

use cgmath::prelude::*;

use crate::widgets::panel::Panel;
use crate::widgets::Widget;

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

pub struct DeviceQueue {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
}

impl DeviceQueue {
    pub fn get_device(&self) -> &Arc<wgpu::Device> {
        &self.device
    }

    pub fn get_queue(&self) -> &Arc<wgpu::Queue> {
        &self.queue
    }
}

impl Clone for DeviceQueue {
    fn clone(&self) -> Self {
        Self {
            device: self.device.clone(),
            queue: self.queue.clone(),
        }
    }
}

struct WindowState<'a> {
    // winit
    winit_window: Arc<winit::window::Window>,
    // wgpu
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    surface: wgpu::Surface<'a>,
    device_queue: DeviceQueue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    top_panel_texture_bind_group_layout: wgpu::BindGroupLayout,
    // panel
    top_panel: Panel,
}

impl<'a> WindowState<'a> {
    async fn new(winit_window: Arc<winit::window::Window>) -> Self {
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
                    power_preference: wgpu::PowerPreference::default(),
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

        let device_queue = DeviceQueue {
            device: Arc::new(device),
            queue: Arc::new(queue),
        };

        // shader

        let texture_copy_shader = device_queue
            .get_device()
            .create_shader_module(wgpu::include_wgsl!("./window.wgsl"));

        // pipeline

        let top_panel_texture_bind_group_layout = device_queue
            .get_device()
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            device_queue
                .get_device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Window Render Pipeline Layout"),
                    bind_group_layouts: &[&top_panel_texture_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            device_queue
                .get_device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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

        let vertex_buffer =
            device_queue
                .get_device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(VERTICES),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer =
            device_queue
                .get_device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(INDICES),
                    usage: wgpu::BufferUsages::INDEX,
                });

        // make texture for top panel

        let size = winit_window.inner_size();

        let top_panel_texture =
            device_queue
                .get_device()
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("Top Panel Texture"),
                    size: wgpu::Extent3d {
                        width: size.width,
                        height: size.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::COPY_SRC
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });

        let top_panel = Panel::new(
            size.width,
            size.height,
            [128, 0, 0, 255],
        );

        Self {
            winit_window,
            instance,
            adapter,
            surface,
            device_queue,
            config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            top_panel_texture_bind_group_layout,
            top_panel,
        }
    }

    fn clone_device_queue(&self) -> DeviceQueue {
        self.device_queue.clone()
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface
            .configure(&*self.device_queue.get_device(), &self.config);
    }

    fn render(&mut self) {
        let surface_texture = self.surface.get_current_texture().unwrap();
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // set top panel inner elements

        let mut teacup = super::widgets::teacup::Teacup::new();
        teacup.set_device_queue(self.clone_device_queue());

        self.top_panel.set_device_queue(self.clone_device_queue());
        self.top_panel.set_inner_elements(Box::new(teacup));

        // get texture from top panel

        let top_panel_texture = self.top_panel.render();

        let top_panel_texture_view =
            top_panel_texture.as_ref().unwrap().create_view(&wgpu::TextureViewDescriptor::default());

        let top_panel_texture_smapler =
            self.device_queue
                .get_device()
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
            self.device_queue
                .get_device()
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

        let mut encoder = self.device_queue.get_device().create_command_encoder(
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

        self.device_queue
            .get_queue()
            .submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }
}

pub struct Window<'a> {
    winit_window: Option<Arc<winit::window::Window>>,
    window: Option<WindowState<'a>>,
}

impl<'a> Window<'a> {
    pub fn new() -> Self {
        Self {
            winit_window: None,
            window: None,
        }
    }
}

impl<'a> winit::application::ApplicationHandler for Window<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.winit_window = Some(Arc::new(
            event_loop
                .create_window(winit::window::Window::default_attributes())
                .unwrap(),
        ));

        let window_state = pollster::block_on(WindowState::new(
            self.winit_window.as_ref().unwrap().clone(),
        ));

        self.window = Some(window_state);
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
                self.window.as_mut().unwrap().render();
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
