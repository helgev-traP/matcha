use std::sync::Arc;

#[derive(Clone, Copy)]
pub struct DeviceQueue<'a> {
    pub(crate) device: &'a wgpu::Device,
    pub(crate) queue: &'a wgpu::Queue,
}

impl DeviceQueue<'_> {
    pub fn device(&self) -> &wgpu::Device {
        self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        self.queue
    }
}

pub struct Gpu {
    // gpu device
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
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
                    label: None,
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
            device: Arc::new(device),
            queue: Arc::new(queue),
        })
    }
}

impl Gpu {
    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    pub fn device(&self) -> &Arc<wgpu::Device> {
        &self.device
    }

    pub fn queue(&self) -> &Arc<wgpu::Queue> {
        &self.queue
    }

    pub fn device_queue(&self) -> DeviceQueue<'_> {
        DeviceQueue {
            device: &self.device,
            queue: &self.queue,
        }
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
