// application entry point. wrapper of winit_instance.
pub mod app;

// event loop, window handling and rendering
pub mod winit_instance;

// widget system
pub mod backend;
pub mod component;
pub mod observer;
pub mod ui;

// winit event handling
pub mod device_event;

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
