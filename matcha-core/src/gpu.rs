// TODO NOTE:
// In this framework, GPU Device Lost have not been handled yet.
// This must be handled in the future.

const DEFAULT_PREFERRED_SURFACE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

pub struct Gpu {
    // gpu device
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,

    preferred_surface_format: wgpu::TextureFormat,
}

impl Gpu {
    pub async fn new(power_preference: wgpu::PowerPreference) -> Result<Self, GpuError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(
                &(wgpu::RequestAdapterOptions {
                    power_preference,
                    compatible_surface: None,
                    force_fallback_adapter: false,
                }),
            )
            .await
            .ok_or(GpuError::AdapterRequestFailed)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Gpu: request device"),
                    required_features: wgpu::Features::PUSH_CONSTANTS
                        | wgpu::Features::VERTEX_WRITABLE_STORAGE,
                    required_limits: adapter.limits(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            preferred_surface_format: DEFAULT_PREFERRED_SURFACE_FORMAT,
        })
    }

    pub fn with_preferred_surface_format(
        mut self,
        format: wgpu::TextureFormat,
    ) -> Self {
        self.preferred_surface_format = format;
        self
    }

    pub fn preferred_surface_format(&self) -> wgpu::TextureFormat {
        self.preferred_surface_format
    }
}

impl Gpu {
    pub(crate) fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    pub(crate) fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

impl Gpu {
    pub fn max_texture_dimension_1d(&self) -> u32 {
        self.adapter.limits().max_texture_dimension_1d
    }

    pub fn max_texture_dimension_2d(&self) -> u32 {
        self.adapter.limits().max_texture_dimension_2d
    }

    pub fn max_texture_dimension_3d(&self) -> u32 {
        self.adapter.limits().max_texture_dimension_3d
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GpuError {
    #[error("Failed to request adapter")]
    AdapterRequestFailed,
    #[error("Failed to request device")]
    DeviceRequestFailed(#[from] wgpu::RequestDeviceError),
}
