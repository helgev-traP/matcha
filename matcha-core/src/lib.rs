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
pub mod types;

// Re-export key components
pub use app::App;
pub use device_input::DeviceInput;
pub use ui::{
    AnyWidget, AnyWidgetFrame, Arrangement, Background, Component, ComponentDom, ComponentWidget,
    Constraints, Dom, InvalidationHandle, LayoutSizeKey, ModelAccessor, Style, UpdateWidgetError,
    Widget, WidgetContext, WidgetFrame, component,
};
pub use update_flag::UpdateNotifier;
