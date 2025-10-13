use fxhash::FxBuildHasher;
use parking_lot::{Mutex, RwLock};
use std::sync::{Arc, Weak};
use std::time::Duration;

use crate::debug_config::DebugConfig;
use crate::gpu::Gpu;
use crate::window_surface::WindowSurface;
use crate::{AnyResource, texture_allocator};

pub(crate) struct AllContext {
    commands: Arc<Mutex<Vec<ApplicationContextCommand>>>,

    // ui resources
    current_time: Arc<RwLock<std::time::Instant>>,
    debug_config: Arc<RwLock<DebugConfig>>,

    // render resources
    gpu: Arc<Gpu>,
    texture_allocator: Arc<texture_allocator::TextureAllocator>,
    any_resource: Arc<AnyResource>,
}

impl AllContext {
    pub fn new() -> Self {
        todo!()
    }

    pub fn drain_commands(&self) -> Vec<ApplicationContextCommand> {
        let mut guard = self.commands.lock();
        std::mem::take(&mut *guard)
    }

    pub fn current_time(&self) -> std::time::Duration {
        self.current_time.read().elapsed()
    }

    pub fn debug_config(&self) -> parking_lot::RwLockReadGuard<'_, DebugConfig> {
        self.debug_config.read()
    }

    pub fn gpu(&self) -> &Arc<Gpu> {
        &self.gpu
    }

    pub fn texture_allocator(&self) -> &Arc<texture_allocator::TextureAllocator> {
        &self.texture_allocator
    }

    pub fn any_resource(&self) -> &Arc<AnyResource> {
        &self.any_resource
    }
}

impl AllContext {
    pub fn application_context(
        &self,
        tokio_handle: &tokio::runtime::Handle,
        window_surface: &Arc<RwLock<WindowSurface>>,
    ) -> ApplicationContext {
        ApplicationContext {
            task_executor: tokio_handle.clone(),
            window_surface: Arc::downgrade(window_surface),
            debug_config: Arc::downgrade(&self.debug_config),
            current_time: Arc::downgrade(&self.current_time),
            commands: Arc::downgrade(&self.commands),
        }
    }

    pub fn widget_context(
        &self,
        tokio_handle: &tokio::runtime::Handle,
        window_surface: &Arc<RwLock<WindowSurface>>
    ) -> WidgetContext {
        WidgetContext {
            task_executor: tokio_handle.clone(),
            window_surface: window_surface.clone(),
            current_time: self.current_time.clone(),
            debug_config: self.debug_config.clone(),
            gpu: self.gpu.clone(),
            texture_allocator: self.texture_allocator.clone(),
            any_resource: self.any_resource.clone(),
            scoped_config: AnyConfig::new(),
            commands: self.commands.clone(),
        }
    }
}

/// Provides contextual information available to all widgets during their lifecycle.
///
/// This includes access to the GPU, window properties, shared resources, and timing information.
/// It is passed down the widget tree during layout and rendering.
#[derive(Clone)]
pub struct WidgetContext {
    // async runtime
    task_executor: tokio::runtime::Handle,

    // ui rendering
    window_surface: Arc<RwLock<WindowSurface>>,
    current_time: Arc<RwLock<std::time::Instant>>,
    debug_config: Arc<RwLock<DebugConfig>>,

    // gpu resources
    gpu: Arc<Gpu>,
    texture_allocator: Arc<texture_allocator::TextureAllocator>,
    any_resource: Arc<AnyResource>,

    // nested config
    scoped_config: AnyConfig,

    // commands (for create application context)
    commands: Arc<Mutex<Vec<ApplicationContextCommand>>>,
}

impl WidgetContext {
    pub(crate) fn application_context(&self) -> ApplicationContext {
        ApplicationContext {
            task_executor: self.task_executor.clone(),
            window_surface: Arc::downgrade(&self.window_surface),
            debug_config: Arc::downgrade(&self.debug_config),
            current_time: Arc::downgrade(&self.current_time),
            commands: Arc::downgrade(&self.commands),
        }
    }
}

impl WidgetContext {
    /// Returns a reference to the WGPU device.
    pub fn device(&self) -> &wgpu::Device {
        self.gpu.device()
    }

    /// Returns a reference to the WGPU queue.
    pub fn queue(&self) -> &wgpu::Queue {
        self.gpu.queue()
    }

    /// Provides access to a type-safe, shared resource storage.
    pub fn any_resource(&self) -> &AnyResource {
        &self.any_resource
    }

    /// Returns the texture format of the surface.
    pub fn surface_format(&self) -> Option<wgpu::TextureFormat> {
        self.window_surface.read().format()
    }

    /// Returns the texture format for color used by the texture atlas.
    pub fn texture_format(&self) -> wgpu::TextureFormat {
        self.texture_allocator.color_format()
    }

    /// Returns a reference to the texture allocator.
    pub fn texture_atlas(&self) -> &texture_allocator::TextureAllocator {
        &self.texture_allocator
    }

    /// Returns the texture format for stencil used by the texture atlas.
    pub fn stencil_format(&self) -> wgpu::TextureFormat {
        self.texture_allocator.stencil_format()
    }

    /// Returns the DPI scaling factor of the window.
    pub fn dpi(&self) -> Option<f64> {
        self.window_surface.read().dpi()
    }

    /// Returns the logical size of the viewport.
    pub fn viewport_size(&self) -> Option<[f32; 2]> {
        self.window_surface
            .read()
            .inner_size()
            .map(|s| [s.width as f32, s.height as f32])
    }

    /// Returns the current absolute time since the application started.
    pub fn current_time(&self) -> Duration {
        self.current_time.read().elapsed()
    }

    /// Returns a clone of the shared debug config.
    pub fn debug_config<'a>(&'a self) -> parking_lot::RwLockReadGuard<'a, DebugConfig> {
        self.debug_config.read()
    }
}

/// ApplicationHandler is owned by the window / WinitInstance and holds the
/// shared command buffer. Components receive `ApplicationHandle` clones to
/// enqueue commands.
#[derive(Clone)]
pub struct ApplicationContext {
    task_executor: tokio::runtime::Handle,

    window_surface: Weak<RwLock<WindowSurface>>,
    debug_config: Weak<RwLock<DebugConfig>>,
    // todo: replace this by `Ticker`
    current_time: Weak<RwLock<std::time::Instant>>,

    commands: Weak<Mutex<Vec<ApplicationContextCommand>>>,
}

/// Commands that can be enqueued from components / handlers.
/// Extend this enum when new application-level commands are needed.
pub(crate) enum ApplicationContextCommand {
    // Define events that the application handler will process
    Quit,
    // future: Custom(Box<dyn FnOnce(&mut AppState) + Send>), etc.
}

impl ApplicationContext {
    /// Enqueue a Quit command.
    pub fn quit(&self) {
        if let Some(command) = self.commands.upgrade() {
            let mut guard = command.lock();
            guard.push(ApplicationContextCommand::Quit);
        }
    }

    // future: push_custom, query_with_oneshot, etc.
}

#[derive(Default, Clone)]
pub(crate) struct AnyConfig {
    configs: std::collections::HashMap<
        std::any::TypeId,
        Arc<dyn std::any::Any + Send + Sync>,
        FxBuildHasher,
    >,
}

impl AnyConfig {
    pub fn new() -> Self {
        Self {
            configs: std::collections::HashMap::with_hasher(FxBuildHasher::default()),
        }
    }

    /// Insert a nested configuration of type T.
    pub fn set<T>(&mut self, config: T)
    where
        T: Send + Sync + 'static,
    {
        self.configs
            .insert(std::any::TypeId::of::<T>(), Arc::new(config));
    }

    /// Retrieve a reference to a nested configuration of type T, if it exists.
    pub fn get<T>(&self) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        self.configs
            .get(&std::any::TypeId::of::<T>())
            .and_then(|arc_any| arc_any.downcast_ref::<T>())
    }

    pub fn get_or_insert<T>(&mut self, v: T) -> &T
    where
        T: Send + Sync + 'static,
    {
        self.get_or_insert_with(|| v)
    }

    pub fn get_or_insert_default<T>(&mut self) -> &T
    where
        T: Default + Send + Sync + 'static,
    {
        self.get_or_insert_with(T::default)
    }

    pub fn get_or_insert_with<T, F>(&mut self, f: F) -> &T
    where
        T: Send + Sync + 'static,
        F: FnOnce() -> T,
    {
        self.configs
            .entry(std::any::TypeId::of::<T>())
            .or_insert_with(|| Arc::new(f()))
            .downcast_ref::<T>()
            .expect("Type map in `NestedConfig` should ensure `key == value.type_id()`")
    }
}
