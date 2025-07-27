// application entry point. wrapper of winit_instance.
pub mod app;

// event loop, window handling and rendering
pub mod winit_instance;

// widget system
pub mod component;
pub mod observer;
pub mod ui;

// core modules
pub mod device;
pub mod events;

// resource management
pub mod any_resource;
// gpu preparation
pub mod gpu;
// renderer of object tree
pub mod renderer;
// allocator for area in texture atlas
pub mod texture_allocator;

// types
pub mod types;
