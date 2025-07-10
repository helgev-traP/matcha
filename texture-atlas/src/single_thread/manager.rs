use core::panic;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use thiserror::Error;

use super::atlas::{Texture, TextureAtlas, TextureAtlasError};

pub struct MemoryAllocateStrategy {
    pub initial_pages: u32,
    pub resize_threshold: Option<f32>,
    pub resize_factor: f32,
    pub shrink_threshold: f32,
    pub shrink_factor: f32,
}

pub struct AtlasManager {
    device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,

    max_size_of_3d_texture: wgpu::Extent3d,
    memory_strategy: MemoryAllocateStrategy,

    atlases: HashMap<Vec<wgpu::TextureFormat>, Rc<RefCell<TextureAtlas>>>,
}

impl AtlasManager {
    pub fn new(
        device: Rc<wgpu::Device>,
        queue: Rc<wgpu::Queue>,
        memory_strategy: MemoryAllocateStrategy,
        max_size_of_3d_texture: wgpu::Extent3d,
    ) -> Self {
        Self {
            device,
            queue,
            max_size_of_3d_texture,
            memory_strategy,
            atlases: HashMap::new(),
        }
    }

    pub fn add_format_set(
        &mut self,
        formats: Vec<wgpu::TextureFormat>,
    ) -> Result<(), AtlasManagerError> {
        if formats.is_empty() {
            return Err(AtlasManagerError::EmptyFormatSet);
        }
        if self.atlases.contains_key(&formats) {
            return Err(AtlasManagerError::FormatSetAlreadyExists);
        }

        let atlas = TextureAtlas::new(
            &self.device,
            wgpu::Extent3d {
                width: self.max_size_of_3d_texture.width,
                height: self.max_size_of_3d_texture.height,
                depth_or_array_layers: self.memory_strategy.initial_pages,
            },
            &formats,
        );
        self.atlases.insert(formats, atlas);

        Ok(())
    }

    pub fn allocate(
        &mut self,
        size: [u32; 2],
        formats: &[wgpu::TextureFormat],
    ) -> Result<Texture, AtlasManagerError> {
        if size[0] == 0 || size[1] == 0 {
            return Err(AtlasManagerError::InvalidTextureSize);
        }
        if size[0] > self.max_size_of_3d_texture.width
            || size[1] > self.max_size_of_3d_texture.height
        {
            return Err(AtlasManagerError::InvalidTextureSize);
        }

        let atlas_rc = self
            .atlases
            .get_mut(formats)
            .ok_or(AtlasManagerError::FormatSetNotFound)?;

        // Check if proactive resize is needed
        let allocation_area = (size[0] * size[1]) as usize;
        let mut resize = None;
        if let Some(resize_threshold) = self.memory_strategy.resize_threshold {
            let atlas = atlas_rc.borrow();
            let atlas_size = atlas.size();
            let capacity = atlas.capacity();
            let usage = atlas.usage();

            if ((usage + allocation_area) as f32 / capacity as f32 > resize_threshold)
                && (atlas_size.depth_or_array_layers
                    < self.max_size_of_3d_texture.depth_or_array_layers)
            {
                let new_page_size = ((atlas_size.depth_or_array_layers as f32
                    * self.memory_strategy.resize_factor)
                    .ceil() as u32)
                    .min(self.max_size_of_3d_texture.depth_or_array_layers);

                resize = Some(wgpu::Extent3d {
                    width: atlas_size.width,
                    height: atlas_size.height,
                    depth_or_array_layers: new_page_size,
                });
            }
        }

        if resize.is_none() {
            // No resize needed yet, try to allocate.
            let mut atlas = atlas_rc.borrow_mut();
            if let Ok(texture) = atlas.allocate(size) {
                return Ok(texture);
            } else {
                // Allocation failed, so we must resize.
                let atlas_size = atlas.size();
                if atlas_size.depth_or_array_layers
                    >= self.max_size_of_3d_texture.depth_or_array_layers
                {
                    return Err(AtlasManagerError::from(
                        TextureAtlasError::AllocationFailedNotEnoughSpace,
                    ));
                }
                let new_page_size = ((atlas_size.depth_or_array_layers as f32
                    * self.memory_strategy.resize_factor)
                    .ceil() as u32)
                    .min(self.max_size_of_3d_texture.depth_or_array_layers);

                resize = Some(wgpu::Extent3d {
                    width: atlas_size.width,
                    height: atlas_size.height,
                    depth_or_array_layers: new_page_size,
                });
            }
        }

        let Some(new_size) = resize else {
            panic!(
                "Resize was not triggered when it should have been. This is a bug in the atlas manager logic."
            );
        };

        // Resize the atlas and allocate the texture.
        let mut atlas = atlas_rc.borrow_mut();
        match atlas.resize(&self.device, &self.queue, new_size, false, Some(size)) {
            Ok(Some(texture)) => Ok(texture),
            Ok(None) => {
                panic!(
                    "expected a texture to be returned after resizing when we give a size as `new_allocation`. This is a bug in the atlas manager logic."
                );
            }
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(Debug, Error)]
pub enum AtlasManagerError {
    #[error("Format set cannot be empty")]
    EmptyFormatSet,
    #[error("Format set already exists in the manager")]
    FormatSetAlreadyExists,
    #[error(
        "Requested texture size is invalid (width or height is zero, or exceeds max texture dimension)"
    )]
    InvalidTextureSize,
    #[error("The specified format set was not found in the manager")]
    FormatSetNotFound,
    #[error("Failed to allocate texture, even after attempting to resize the atlas")]
    AllocationFailed,
    #[error("An error occurred in the texture atlas")]
    AtlasError(#[from] TextureAtlasError),
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_wgpu() -> (wgpu::Device, wgpu::Queue) {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: true,
            })
            .await
            .unwrap();
        adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap()
    }

    #[cfg(test)]
    impl AtlasManager {
        fn atlas_count(&self) -> usize {
            self.atlases.len()
        }

        fn get_atlas_size(&self, formats: &[wgpu::TextureFormat]) -> Option<wgpu::Extent3d> {
            self.atlases.get(formats).map(|atlas| atlas.borrow().size())
        }

        fn get_atlas_usage(&self, formats: &[wgpu::TextureFormat]) -> Option<usize> {
            self.atlases
                .get(formats)
                .map(|atlas| atlas.borrow().usage())
        }
    }

    #[test]
    fn test_manager_new() {
        pollster::block_on(async {
            let (device, queue) = setup_wgpu().await;
            let memory_strategy = MemoryAllocateStrategy {
                initial_pages: 1,
                resize_threshold: Some(0.8),
                resize_factor: 2.0,
                shrink_threshold: 0.2,
                shrink_factor: 0.5,
            };
            let max_size = wgpu::Extent3d {
                width: 1024,
                height: 1024,
                depth_or_array_layers: 8,
            };
            let manager =
                AtlasManager::new(Rc::new(device), Rc::new(queue), memory_strategy, max_size);

            assert_eq!(manager.atlas_count(), 0);
        });
    }

    #[test]
    fn test_add_format_set() {
        pollster::block_on(async {
            let (device, queue) = setup_wgpu().await;
            let memory_strategy = MemoryAllocateStrategy {
                initial_pages: 2,
                resize_threshold: Some(0.8),
                resize_factor: 2.0,
                shrink_threshold: 0.2,
                shrink_factor: 0.5,
            };
            let max_size = wgpu::Extent3d {
                width: 1024,
                height: 1024,
                depth_or_array_layers: 8,
            };
            let mut manager =
                AtlasManager::new(Rc::new(device), Rc::new(queue), memory_strategy, max_size);

            let formats = vec![wgpu::TextureFormat::Rgba8UnormSrgb];
            manager.add_format_set(formats.clone()).unwrap();
            assert_eq!(manager.atlas_count(), 1);
            assert_eq!(
                manager
                    .get_atlas_size(&formats)
                    .unwrap()
                    .depth_or_array_layers,
                2
            );

            let result = manager.add_format_set(vec![]);
            assert!(matches!(result, Err(AtlasManagerError::EmptyFormatSet)));

            let result = manager.add_format_set(formats.clone());
            assert!(matches!(
                result,
                Err(AtlasManagerError::FormatSetAlreadyExists)
            ));
        });
    }

    #[test]
    fn test_allocate_basic() {
        pollster::block_on(async {
            let (device, queue) = setup_wgpu().await;
            let memory_strategy = MemoryAllocateStrategy {
                initial_pages: 1,
                resize_threshold: Some(0.8),
                resize_factor: 2.0,
                shrink_threshold: 0.2,
                shrink_factor: 0.5,
            };
            let max_size = wgpu::Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            };
            let mut manager =
                AtlasManager::new(Rc::new(device), Rc::new(queue), memory_strategy, max_size);

            let formats = vec![wgpu::TextureFormat::Rgba8UnormSrgb];
            manager.add_format_set(formats.clone()).unwrap();

            let texture = manager.allocate([32, 32], &formats).unwrap();
            assert_eq!(texture.size(), [32, 32]);
            assert_eq!(manager.get_atlas_usage(&formats).unwrap(), 32 * 32);
        });
    }

    #[test]
    fn test_allocate_invalid_size() {
        pollster::block_on(async {
            let (device, queue) = setup_wgpu().await;
            let memory_strategy = MemoryAllocateStrategy {
                initial_pages: 1,
                resize_threshold: Some(0.8),
                resize_factor: 2.0,
                shrink_threshold: 0.2,
                shrink_factor: 0.5,
            };
            let max_size = wgpu::Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            };
            let mut manager =
                AtlasManager::new(Rc::new(device), Rc::new(queue), memory_strategy, max_size);

            let formats = vec![wgpu::TextureFormat::Rgba8UnormSrgb];
            manager.add_format_set(formats.clone()).unwrap();

            let result = manager.allocate([0, 32], &formats);
            assert!(matches!(result, Err(AtlasManagerError::InvalidTextureSize)));

            let result = manager.allocate([32, 0], &formats);
            assert!(matches!(result, Err(AtlasManagerError::InvalidTextureSize)));

            let result = manager.allocate([257, 32], &formats);
            assert!(matches!(result, Err(AtlasManagerError::InvalidTextureSize)));

            let result = manager.allocate([32, 257], &formats);
            assert!(matches!(result, Err(AtlasManagerError::InvalidTextureSize)));
        });
    }

    #[test]
    fn test_allocate_format_set_not_found() {
        pollster::block_on(async {
            let (device, queue) = setup_wgpu().await;
            let memory_strategy = MemoryAllocateStrategy {
                initial_pages: 1,
                resize_threshold: Some(0.8),
                resize_factor: 2.0,
                shrink_threshold: 0.2,
                shrink_factor: 0.5,
            };
            let max_size = wgpu::Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            };
            let mut manager =
                AtlasManager::new(Rc::new(device), Rc::new(queue), memory_strategy, max_size);

            let formats = vec![wgpu::TextureFormat::Rgba8UnormSrgb];
            let result = manager.allocate([32, 32], &formats);
            assert!(matches!(result, Err(AtlasManagerError::FormatSetNotFound)));
        });
    }

    #[test]
    fn test_proactive_resize_only() {
        pollster::block_on(async {
            let (device, queue) = setup_wgpu().await;
            let memory_strategy = MemoryAllocateStrategy {
                initial_pages: 1,
                resize_threshold: Some(0.1),
                resize_factor: 2.0,
                shrink_threshold: 0.2,
                shrink_factor: 0.5,
            };
            let max_size = wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 4,
            };
            let mut manager =
                AtlasManager::new(Rc::new(device), Rc::new(queue), memory_strategy, max_size);

            let formats = vec![wgpu::TextureFormat::Rgba8UnormSrgb];
            manager.add_format_set(formats.clone()).unwrap();

            assert_eq!(
                manager
                    .get_atlas_size(&formats)
                    .unwrap()
                    .depth_or_array_layers,
                1
            );

            let _texture = manager.allocate([32, 32], &formats).unwrap();

            assert_eq!(
                manager
                    .get_atlas_size(&formats)
                    .unwrap()
                    .depth_or_array_layers,
                2
            );
        });
    }

    #[test]
    fn test_allocate_max_size_reached() {
        pollster::block_on(async {
            let (device, queue) = setup_wgpu().await;
            let memory_strategy = MemoryAllocateStrategy {
                initial_pages: 1,
                resize_threshold: Some(0.1),
                resize_factor: 2.0,
                shrink_threshold: 0.2,
                shrink_factor: 0.5,
            };
            let max_size = wgpu::Extent3d {
                width: 32,
                height: 32,
                depth_or_array_layers: 1,
            };
            let mut manager =
                AtlasManager::new(Rc::new(device), Rc::new(queue), memory_strategy, max_size);

            let formats = vec![wgpu::TextureFormat::Rgba8UnormSrgb];
            manager.add_format_set(formats.clone()).unwrap();

            let _texture1 = manager.allocate([32, 32], &formats).unwrap();
            assert_eq!(
                manager
                    .get_atlas_size(&formats)
                    .unwrap()
                    .depth_or_array_layers,
                1
            );
            assert_eq!(manager.get_atlas_usage(&formats).unwrap(), 32 * 32);

            let result = manager.allocate([1, 1], &formats);
            assert!(matches!(
                result,
                Err(AtlasManagerError::AtlasError(
                    TextureAtlasError::AllocationFailedNotEnoughSpace
                ))
            ));
        });
    }
}
