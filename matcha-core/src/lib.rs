// application entry point. wrapper of winit_instance.
pub mod app;

// event loop, window handling and rendering
pub mod winit_instance;

// widget system
pub mod backend;
pub mod ui;
pub mod update_flag;

// winit event handling
pub mod device_input;

// resource management
pub mod any_resource;
// gpu preparation
pub mod gpu;

// allocator for area in texture atlas
pub mod texture_allocator;

// types
pub mod types;

// Re-export key components
pub use app::App;
pub use device_input::DeviceInput;
pub use ui::{
    Background, Component, ComponentDom, ComponentWidget, Constraints, Dom, Style, Widget,
    WidgetContext,
};
pub use update_flag::UpdateNotifier;
