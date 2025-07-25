use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("GPU is not ready")]
    Gpu,
    #[error("Window surface is not ready")]
    WindowSurface,
    #[error("Texture allocator is not ready")]
    TextureAllocator,
    #[error("Application context is not ready")]
    AnyResource,
    #[error("Root Widget is not ready")]
    RootWidget,
    #[error("Renderer is not ready")]
    Renderer,
    #[error("Benchmarker is not ready")]
    Benchmarker,
}
