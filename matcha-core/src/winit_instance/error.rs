use thiserror::Error;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Failed to initialize tokio runtime")]
    TokioRuntime,
    #[error("Failed to initialize GPU")]
    Gpu,
    #[error(transparent)]
    UiControl(#[from] super::ui_control::UiControlError),
    #[error(transparent)]
    WindowSurface(#[from] super::window_surface::WindowSurfaceError),
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Window surface error: {0}")]
    WindowSurface(&'static str),
    #[error(transparent)]
    Surface(#[from] wgpu::SurfaceError),
    #[error(transparent)]
    Render(#[from] super::render_control::RenderControlError),
}
