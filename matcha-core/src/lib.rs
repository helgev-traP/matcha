// application entry point. wrapper of winit_instance.
pub mod app;

// event loop, window handling and rendering
mod winit_instance;

// widget system
pub mod backend;
pub mod ui;
pub mod update_flag;

// winit event handling
pub mod device_input;

// resource management
pub mod any_resource;
pub use any_resource::AnyResource;
// gpu preparation
mod gpu;

// allocator for area in texture atlas
mod texture_allocator;

// types
pub mod color;
pub mod metrics;
