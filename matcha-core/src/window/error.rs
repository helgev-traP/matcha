use std::error::Error;

#[derive(Debug)]
pub enum RenderError {
    Gpu,
    RootWidget,
    Renderer,
    Benchmarker,
}
impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::Gpu => write!(f, "GPU is not ready"),
            RenderError::RootWidget => write!(f, "Root Widget is not ready"),
            RenderError::Renderer => write!(f, "Renderer is not ready"),
            RenderError::Benchmarker => write!(f, "Benchmarker is not ready"),
        }
    }
}

impl Error for RenderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
