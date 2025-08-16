// application entry point. wrapper of winit_instance.
pub mod app;

// event loop, window handling and rendering
pub mod winit_instance;

// widget system
pub mod backend;
pub mod component;
pub mod render_node;
pub mod ui;
pub mod update_flag;

// winit event handling
pub mod device_event;

// resource management
pub mod any_resource;
// gpu preparation
pub mod gpu;
// renderer of object tree
pub mod renderer;

#[path = "renderer/texture_color_renderer.rs"]
pub mod texture_color_renderer;
#[path = "renderer/texture_copy.rs"]
pub mod texture_copy;
#[path = "renderer/vertex_color_renderer.rs"]
pub mod vertex_color_renderer;
// allocator for area in texture atlas
pub mod texture_allocator;

// types
pub mod types;
pub mod vertex;
