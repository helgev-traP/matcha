use std::collections::HashMap;
use std::sync::{Arc, Weak};

use euclid::Box2D;
use guillotiere::{AllocId, AtlasAllocator, Size, euclid};
use parking_lot::Mutex;
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone)]
pub struct Texture {
    inner: Arc<TextureInner>,
}

// We only store the texture id and reference to the atlas,
// to make `Texture` remain valid after `TextureAtlas` resizes or changes,
// except for data loss when the atlas shrinks.
struct TextureInner {
    // allocation info
    texture_id: TextureId,
    // interaction with the atlas
    atlas: Weak<Mutex<TextureAtlas>>,
    // It may be useful to store some information about the texture that will not change during atlas resizing
    size: [u32; 2],                    // size of the texture in pixels
    formats: Vec<wgpu::TextureFormat>, // formats of the texture
}

/// Public API to interact with a texture.
/// User code should not need to know about its id, location, or atlas.
impl Texture {
    pub fn area(&self) -> u32 {
        self.inner.size[0] * self.inner.size[1]
    }

    pub fn size(&self) -> [u32; 2] {
        self.inner.size
    }

    pub fn formats(&self) -> &[wgpu::TextureFormat] {
        &self.inner.formats
    }

    pub fn atlas_pointer(&self) -> Option<usize> {
        self.inner
            .atlas
            .upgrade()
            .map(|arc| Arc::as_ptr(&arc) as usize)
    }

    pub fn translate_uv(&self, uvs: &[[f32; 2]]) -> Result<Vec<[f32; 2]>, TextureError> {
        // Get the texture location in the atlas
        let Some(atlas) = self.inner.atlas.upgrade() else {
            return Err(TextureError::AtlasGone);
        };
        let atlas = atlas.lock();
        let Some(location) = atlas.get_location(self.inner.texture_id) else {
            return Err(TextureError::TextureNotFoundInAtlas);
        };
        let x_max = location.uv.max.x;
        let y_max = location.uv.max.y;
        let x_min = location.uv.min.x;
        let y_min = location.uv.min.y;

        // Translate the vertices to the texture area
        let translated_vertices = uvs
            .iter()
            .map(|&[x, y]| {
                [
                    (x_min + (x * (x_max - x_min))).clamp(0.0, 1.0),
                    (y_min + (y * (y_max - y_min))).clamp(0.0, 1.0),
                ]
            })
            .collect::<Vec<_>>();

        Ok(translated_vertices)
    }

    pub fn write_data(&self, queue: &wgpu::Queue, data: &[&[u8]]) -> Result<(), TextureError> {
        // Check data consistency
        if data.len() != self.inner.formats.len() {
            return Err(TextureError::DataConsistencyError(
                "Data length does not match formats length".to_string(),
            ));
        }
        for (i, format) in self.inner.formats.iter().enumerate() {
            let bytes_per_pixel = format
                .block_copy_size(None)
                .ok_or(TextureError::InvalidFormatBlockCopySize)?;
            let expected_size = self.inner.size[0] * self.inner.size[1] * bytes_per_pixel;
            if data[i].len() as u32 != expected_size {
                return Err(TextureError::DataConsistencyError(format!(
                    "Data size for format {i} does not match expected size"
                )));
            }
        }

        // Get the texture in the atlas and location
        let Some(atlas) = self.inner.atlas.upgrade() else {
            return Err(TextureError::AtlasGone);
        };
        let atlas = atlas.lock();

        let textures = atlas.textures();
        let Some(location) = atlas.get_location(self.inner.texture_id) else {
            return Err(TextureError::TextureNotFoundInAtlas);
        };

        for (i, texture) in textures.iter().enumerate() {
            let data = data[i];

            let bytes_per_pixel = texture
                .format()
                .block_copy_size(None)
                .ok_or(TextureError::InvalidFormatBlockCopySize)?;
            let bytes_per_row = self.inner.size[0] * bytes_per_pixel;

            let origin = wgpu::Origin3d {
                x: location.bounds.min.x as u32,
                y: location.bounds.min.y as u32,
                z: location.page_index,
            };

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin,
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width: self.inner.size[0],
                    height: self.inner.size[1],
                    depth_or_array_layers: 1,
                },
            );
        }

        Ok(())
    }

    pub fn read_data(&self) -> Result<(), TextureError> {
        todo!()
    }

    pub fn copy_from_texture(&self) -> Result<(), TextureError> {
        todo!()
    }

    pub fn copy_to_texture(&self) -> Result<(), TextureError> {
        todo!()
    }

    pub fn copy_from_buffer(&self) -> Result<(), TextureError> {
        todo!()
    }

    pub fn copy_to_buffer(&self) -> Result<(), TextureError> {
        todo!()
    }

    pub fn set_viewport(&self, render_pass: &mut wgpu::RenderPass<'_>) -> Result<(), TextureError> {
        // Get the texture location in the atlas
        let Some(atlas) = self.inner.atlas.upgrade() else {
            return Err(TextureError::AtlasGone);
        };
        let atlas = atlas.lock();
        let Some(location) = atlas.get_location(self.inner.texture_id) else {
            return Err(TextureError::TextureNotFoundInAtlas);
        };

        // Set the viewport to the texture area
        render_pass.set_viewport(
            location.bounds.min.x as f32,
            location.bounds.min.y as f32,
            location.size()[0] as f32,
            location.size()[1] as f32,
            0.0,
            1.0,
        );

        Ok(())
    }

    pub fn begin_render_pass<'a>(
        &'a self,
        encoder: &'a mut wgpu::CommandEncoder,
    ) -> Result<wgpu::RenderPass<'a>, TextureError> {
        // Get the texture location in the atlas
        let Some(atlas) = self.inner.atlas.upgrade() else {
            return Err(TextureError::AtlasGone);
        };
        let atlas = atlas.lock();
        let texture_views = atlas.texture_views();
        let Some(location) = atlas.get_location(self.inner.texture_id) else {
            return Err(TextureError::TextureNotFoundInAtlas);
        };

        // Create a render pass for the texture area
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Texture Atlas Render Pass"),
            color_attachments: texture_views
                .iter()
                .map(|view| wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })
                .map(Option::Some)
                .collect::<Vec<_>>()
                .as_slice(),
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Set the viewport to the texture area
        render_pass.set_viewport(
            location.bounds.min.x as f32,
            location.bounds.min.y as f32,
            location.size()[0] as f32,
            location.size()[1] as f32,
            0.0,
            1.0,
        );

        Ok(render_pass)
    }

    pub fn uv(&self) -> Result<Box2D<f32, euclid::UnknownUnit>, TextureError> {
        // Get the texture location in the atlas
        let Some(atlas) = self.inner.atlas.upgrade() else {
            return Err(TextureError::AtlasGone);
        };
        let atlas = atlas.lock();
        let Some(location) = atlas.get_location(self.inner.texture_id) else {
            return Err(TextureError::TextureNotFoundInAtlas);
        };

        Ok(location.uv)
    }

    // pub fn with_data<Init, F>(&self, init: Init, f: F) -> Result<(), TextureError>
    // where
    //     Init: FnOnce(&Texture) -> Result<(), TextureError>,
    //     F: FnOnce(&Texture) -> Result<(), TextureError>,
    // {
    //     todo!()
    // }
}

// Ensure the texture area will be deallocated when the texture is dropped.
impl Drop for TextureInner {
    fn drop(&mut self) {
        if let Some(atlas) = self.atlas.upgrade() {
            match atlas.lock().deallocate(self.texture_id) {
                Ok(_) => {
                    // Successfully deallocated
                }
                Err(DeallocationErrorTextureNotFound) => {
                    // We do not need to handle this error because this means the texture was already deallocated.
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TextureId {
    texture_uuid: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TextureLocation {
    page_index: u32,
    bounds: euclid::Box2D<i32, euclid::UnknownUnit>,
    uv: euclid::Box2D<f32, euclid::UnknownUnit>,
}

impl TextureLocation {
    fn area(&self) -> u32 {
        self.bounds.area() as u32
    }

    fn size(&self) -> [u32; 2] {
        [
            (self.bounds.max.x - self.bounds.min.x) as u32,
            (self.bounds.max.y - self.bounds.min.y) as u32,
        ]
    }
}

pub struct TextureAtlas {
    textures: Vec<wgpu::Texture>,
    texture_views: Vec<wgpu::TextureView>,
    size: wgpu::Extent3d,
    formats: Vec<wgpu::TextureFormat>,

    state: TextureAtlasState,

    weak_self: Weak<Mutex<Self>>,
}

struct TextureAtlasState {
    allocators: Vec<AtlasAllocator>,
    texture_id_to_location: HashMap<TextureId, TextureLocation>,
    texture_id_to_alloc_id: HashMap<TextureId, AllocId>,
    usage: usize,
}

/// Constructor and information methods.
impl TextureAtlas {
    pub fn new(
        device: &wgpu::Device,
        size: wgpu::Extent3d,
        formats: &[wgpu::TextureFormat],
    ) -> Arc<Mutex<Self>> {
        let (textures, texture_views) = Self::create_texture_and_view(device, formats, size);

        // Initialize the state with an empty allocator and allocation map.
        let state = TextureAtlasState {
            allocators: (0..size.depth_or_array_layers)
                .map(|_| Size::new(size.width as i32, size.height as i32))
                .map(AtlasAllocator::new)
                .collect(),
            texture_id_to_location: HashMap::new(),
            texture_id_to_alloc_id: HashMap::new(),
            usage: 0,
        };

        Arc::new_cyclic(|weak_self| {
            Mutex::new(Self {
                textures,
                texture_views,
                size,
                formats: formats.to_vec(),
                state,
                weak_self: weak_self.clone(),
            })
        })
    }

    pub fn size(&self) -> wgpu::Extent3d {
        self.size
    }

    pub fn formats(&self) -> &[wgpu::TextureFormat] {
        &self.formats
    }

    pub fn capacity(&self) -> usize {
        self.size.width as usize
            * self.size.height as usize
            * self.size.depth_or_array_layers as usize
    }

    pub fn usage(&self) -> usize {
        self.state.usage
    }

    // todo: we can optimize this performance.
    pub fn max_allocation_size(&self) -> [u32; 2] {
        let mut max_size = [0; 2];

        for location in self.state.texture_id_to_location.values() {
            let size = location.size();
            max_size[0] = max_size[0].max(size[0]);
            max_size[1] = max_size[1].max(size[1]);
        }
        max_size
    }
}

/// TextureAtlas allocation and deallocation
impl TextureAtlas {
    /// Allocate a texture in the atlas.
    pub fn allocate(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: [u32; 2],
    ) -> Result<Texture, TextureAtlasError> {
        // Check if size is smaller than the atlas size
        if size[0] > self.size.width || size[1] > self.size.height {
            return Err(TextureAtlasError::AllocationFailedTooLarge);
        }

        let size = Size::new(size[0] as i32, size[1] as i32);

        for (page_index, allocator) in self.state.allocators.iter_mut().enumerate() {
            if let Some(alloc) = allocator.allocate(size) {
                let bounds = alloc.rectangle;
                let uvs = euclid::Box2D::new(
                    euclid::Point2D::new(
                        (bounds.min.x as f32) / (self.size.width as f32),
                        (bounds.min.y as f32) / (self.size.height as f32),
                    ),
                    euclid::Point2D::new(
                        (bounds.max.x as f32) / (self.size.width as f32),
                        (bounds.max.y as f32) / (self.size.height as f32),
                    ),
                );
                let location = TextureLocation {
                    page_index: page_index as u32,
                    bounds,
                    uv: uvs,
                };

                // Create a new TextureId and Texture
                let texture_id = TextureId {
                    texture_uuid: Uuid::new_v4(),
                };
                let texture_inner = TextureInner {
                    texture_id,
                    atlas: self.weak_self.clone(),
                    size: [size.width as u32, size.height as u32],
                    formats: self.formats.clone(),
                };
                let texture = Texture {
                    inner: Arc::new(texture_inner),
                };

                // Store the texture location and allocation id in the atlas state.
                self.state
                    .texture_id_to_location
                    .insert(texture_id, location);
                self.state
                    .texture_id_to_alloc_id
                    .insert(texture_id, alloc.id);
                // Update usage
                self.state.usage += location.bounds.area() as usize;

                // Return the allocated texture
                return Ok(texture);
            }
        }

        self.add_one_page(device, queue);

        // Retry allocation after adding a new page
        let page_index = self.state.allocators.len() - 1;
        let allocator = &mut self.state.allocators[page_index];
        if let Some(alloc) = allocator.allocate(size) {
            let bounds = alloc.rectangle;
            let uvs = euclid::Box2D::new(
                euclid::Point2D::new(
                    (bounds.min.x as f32) / (self.size.width as f32),
                    (bounds.min.y as f32) / (self.size.height as f32),
                ),
                euclid::Point2D::new(
                    (bounds.max.x as f32) / (self.size.width as f32),
                    (bounds.max.y as f32) / (self.size.height as f32),
                ),
            );
            let location = TextureLocation {
                page_index: page_index as u32,
                bounds,
                uv: uvs,
            };

            // Create a new TextureId and Texture
            let texture_id = TextureId {
                texture_uuid: Uuid::new_v4(),
            };
            let texture_inner = TextureInner {
                texture_id,
                atlas: self.weak_self.clone(),
                size: [size.width as u32, size.height as u32],
                formats: self.formats.clone(),
            };
            let texture = Texture {
                inner: Arc::new(texture_inner),
            };

            // Store the texture location and allocation id in the atlas state.
            self.state
                .texture_id_to_location
                .insert(texture_id, location);
            self.state
                .texture_id_to_alloc_id
                .insert(texture_id, alloc.id);
            // Update usage
            self.state.usage += location.bounds.area() as usize;

            // Return the allocated texture
            Ok(texture)
        } else {
            panic!("We checked for enough space at the beginning, so this should never happen.");
        }
    }

    /// Deallocate a texture from the atlas.
    /// This will be called automatically when the `TextureInner` is dropped.
    fn deallocate(&mut self, id: TextureId) -> Result<(), DeallocationErrorTextureNotFound> {
        // Find the texture location and remove it from the id-to-location map.
        let location = self
            .state
            .texture_id_to_location
            .remove(&id)
            .ok_or(DeallocationErrorTextureNotFound)?;

        // Find the allocation id and remove it from the id-to-alloc-id map.
        let alloc_id = self
            .state
            .texture_id_to_alloc_id
            .remove(&id)
            .ok_or(DeallocationErrorTextureNotFound)?;

        // Deallocate the texture from the allocator.
        self.state.allocators[location.page_index as usize].deallocate(alloc_id);

        // Update usage
        self.state.usage -= location.area() as usize;

        Ok(())
    }
}

/// Resize the atlas to a new size.
impl TextureAtlas {
    fn add_one_page(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let new_size = wgpu::Extent3d {
            width: self.size.width,
            height: self.size.height,
            depth_or_array_layers: self.size.depth_or_array_layers + 1,
        };

        let (new_textures, new_texture_views) =
            Self::create_texture_and_view(device, &self.formats, new_size);

        self.state.allocators.push(AtlasAllocator::new(Size::new(
            new_size.width as i32,
            new_size.height as i32,
        )));

        // Copy existing texture data to the new textures.
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("TextureAtlas Resize Encoder"),
        });

        for (old_texture, new_texture) in self.textures.iter().zip(new_textures.iter()) {
            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: old_texture,
                    mip_level: 0,
                    aspect: wgpu::TextureAspect::All,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                },
                wgpu::TexelCopyTextureInfo {
                    texture: new_texture,
                    mip_level: 0,
                    aspect: wgpu::TextureAspect::All,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                },
                wgpu::Extent3d {
                    width: self.size.width,
                    height: self.size.width,
                    depth_or_array_layers: self.size.depth_or_array_layers,
                },
            );
        }

        queue.submit(Some(encoder.finish()));

        // Update the atlas state with the new textures and views.
        self.textures.extend(new_textures);
        self.texture_views.extend(new_texture_views);
        self.size = new_size;
    }
}

// for internal use only
impl TextureAtlas {
    fn get_location(&self, id: TextureId) -> Option<TextureLocation> {
        self.state.texture_id_to_location.get(&id).copied()
    }

    fn textures(&self) -> &[wgpu::Texture] {
        &self.textures
    }

    fn texture_views(&self) -> &[wgpu::TextureView] {
        &self.texture_views
    }
}

// helper functions
impl TextureAtlas {
    fn create_texture_and_view(
        device: &wgpu::Device,
        formats: &[wgpu::TextureFormat],
        page_size: wgpu::Extent3d,
    ) -> (Vec<wgpu::Texture>, Vec<wgpu::TextureView>) {
        let mut textures = Vec::with_capacity(formats.len());
        let mut texture_views = Vec::with_capacity(formats.len());

        for &format in formats {
            let texture_label = format!("texture_atlas_texture_{format:?}");
            let texture_view_label = format!("texture_atlas_texture_view_{format:?}");

            let texture_descriptor = wgpu::TextureDescriptor {
                label: Some(&texture_label),
                size: page_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            };
            let texture = device.create_texture(&texture_descriptor);
            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(&texture_view_label),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..wgpu::TextureViewDescriptor::default()
            });
            textures.push(texture);
            texture_views.push(texture_view);
        }

        (textures, texture_views)
    }
}

/// `DeallocationErrorTextureNotFound` only be used in this file.
struct DeallocationErrorTextureNotFound;

#[derive(Error, Debug)]
pub enum TextureError {
    #[error("The texture's atlas has been dropped.")]
    AtlasGone,
    #[error("The texture was not found in the atlas.")]
    TextureNotFoundInAtlas,
    #[error("Data consistency error: {0}")]
    DataConsistencyError(String),
    #[error("Invalid format block copy size.")]
    InvalidFormatBlockCopySize,
}

#[derive(Error, Debug)]
pub enum TextureAtlasError {
    #[error("Allocation failed because there was not enough space in the atlas.")]
    AllocationFailedNotEnoughSpace,
    #[error("Resizing the atlas failed because there was not enough space for all the textures.")]
    ResizeFailedNotEnoughSpace,
    #[error("Allocation failed because the requested size is too large for the atlas.")]
    AllocationFailedTooLarge,
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
    impl TextureAtlas {
        fn allocation_count(&self) -> usize {
            self.state.texture_id_to_location.len()
        }
    }

    #[cfg(test)]
    impl Texture {
        fn location(&self) -> Option<TextureLocation> {
            let atlas = self.inner.atlas.upgrade()?;
            let atlas = atlas.lock();
            atlas.get_location(self.inner.texture_id)
        }
    }

    /// Tests if the `TextureAtlas` is initialized with the correct parameters.
    #[test]
    fn test_atlas_initialization() {
        pollster::block_on(async {
            let (device, _queue) = setup_wgpu().await;
            let size = wgpu::Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 4,
            };
            let formats = &[wgpu::TextureFormat::Rgba8UnormSrgb];
            let atlas = TextureAtlas::new(&device, size, formats);
            let atlas = atlas.lock();

            assert_eq!(atlas.size(), size);
            assert_eq!(atlas.formats(), formats);
            assert_eq!(atlas.capacity(), 256 * 256 * 4);
            assert_eq!(atlas.usage(), 0);
            assert_eq!(atlas.allocation_count(), 0);

            let textures = atlas.textures();
            assert_eq!(textures.len(), formats.len());

            for (texture, format) in textures.iter().zip(formats.iter()) {
                assert_eq!(texture.format(), *format);
            }
            let texture_views = atlas.texture_views();
            assert_eq!(texture_views.len(), formats.len());
        });
    }

    /// Tests the basic allocation and deallocation of textures.
    /// It verifies that allocation increases usage and deallocation (on drop) decreases it.
    #[test]
    fn test_texture_allocation_and_deallocation() {
        pollster::block_on(async {
            let (device, queue) = setup_wgpu().await;
            let size = wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            };
            let formats = &[wgpu::TextureFormat::Rgba8UnormSrgb];
            let atlas = TextureAtlas::new(&device, size, formats);

            // Allocate one texture
            let texture1 = atlas.lock().allocate(&device, &queue, [32, 32]).unwrap();
            assert_eq!(atlas.lock().allocation_count(), 1);
            assert_eq!(atlas.lock().usage(), 32 * 32);

            // Allocate another texture
            let texture2 = atlas.lock().allocate(&device, &queue, [16, 16]).unwrap();
            assert_eq!(atlas.lock().allocation_count(), 2);
            assert_eq!(atlas.lock().usage(), 32 * 32 + 16 * 16);

            // Deallocate one texture
            drop(texture1);
            assert_eq!(atlas.lock().allocation_count(), 1);
            assert_eq!(atlas.lock().usage(), 16 * 16);

            // Deallocate the other texture
            drop(texture2);
            assert_eq!(atlas.lock().allocation_count(), 0);
            assert_eq!(atlas.lock().usage(), 0);
        });
    }

    /*
    /// Tests that the atlas correctly returns an error when there is not enough space for a new allocation.
    #[test]
    fn test_allocation_failure() {
        pollster::block_on(async {
            let (device, queue) = setup_wgpu().await;
            let size = wgpu::Extent3d {
                width: 32,
                height: 32,
                depth_or_array_layers: 1,
            };
            let formats = &[wgpu::TextureFormat::Rgba8UnormSrgb];
            let atlas = TextureAtlas::new(&device, size, formats);

            // This should succeed
            let _texture1 = atlas.lock().allocate(&device, &queue, [32, 32]).unwrap();

            // This should fail
            let result = atlas.lock().allocate(&device, &queue, [1, 1]);

            assert!(matches!(
                result,
                Err(TextureAtlasError::AllocationFailedNotEnoughSpace)
            ));
        });
    }
    */

    /// Tests if the space freed by a deallocated texture can be reused by a new allocation.
    #[test]
    fn test_reuse_deallocated_space() {
        pollster::block_on(async {
            let (device, queue) = setup_wgpu().await;
            let size = wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            };
            let formats = &[wgpu::TextureFormat::Rgba8UnormSrgb];
            let atlas = TextureAtlas::new(&device, size, formats);

            let texture1 = atlas.lock().allocate(&device, &queue, [64, 64]).unwrap();
            assert_eq!(atlas.lock().allocation_count(), 1);

            drop(texture1);
            assert_eq!(atlas.lock().allocation_count(), 0);

            // Should be able to allocate again in the same space
            let _texture2 = atlas.lock().allocate(&device, &queue, [64, 64]).unwrap();
            assert_eq!(atlas.lock().allocation_count(), 1);
        });
    }

    /// Tests if the UV coordinates of an allocated texture are calculated correctly.
    #[test]
    fn test_texture_uv() {
        pollster::block_on(async {
            let (device, queue) = setup_wgpu().await;
            let size = wgpu::Extent3d {
                width: 128,
                height: 128,
                depth_or_array_layers: 1,
            };
            let formats = &[wgpu::TextureFormat::Rgba8UnormSrgb];
            let atlas = TextureAtlas::new(&device, size, formats);

            let texture = atlas.lock().allocate(&device, &queue, [32, 64]).unwrap();
            let uv = texture.uv().unwrap();

            assert!(uv.min.x >= 0.0 && uv.min.x < 1.0);
            assert!(uv.min.y >= 0.0 && uv.min.y < 1.0);
            assert!(uv.max.x > uv.min.x && uv.max.x <= 1.0);
            assert!(uv.max.y > uv.min.y && uv.max.y <= 1.0);

            let expected_uv_width = 32.0 / 128.0;
            let expected_uv_height = 64.0 / 128.0;
            assert!((uv.width() - expected_uv_width).abs() < f32::EPSILON);
            assert!((uv.height() - expected_uv_height).abs() < f32::EPSILON);
        });
    }

    /// Tests that texture methods return `TextureError::AtlasGone` after the atlas has been dropped.
    #[test]
    fn test_texture_error_when_atlas_gone() {
        pollster::block_on(async {
            let (device, queue) = setup_wgpu().await;
            let size = wgpu::Extent3d {
                width: 128,
                height: 128,
                depth_or_array_layers: 1,
            };
            let formats = &[wgpu::TextureFormat::Rgba8UnormSrgb];
            let atlas = TextureAtlas::new(&device, size, formats);

            let texture = atlas.lock().allocate(&device, &queue, [32, 32]).unwrap();

            drop(atlas);

            let result = texture.uv();
            assert!(matches!(result, Err(TextureError::AtlasGone)));
        });
    }

    /// Tests that `write_data` writes to the correct location in the atlas.
    #[test]
    fn test_texture_write_and_read_data() {
        pollster::block_on(async {
            let (device, queue) = setup_wgpu().await;
            let atlas_size = wgpu::Extent3d {
                width: 512,
                height: 512,
                depth_or_array_layers: 1,
            };
            let texture_format = wgpu::TextureFormat::R8Uint;
            let atlas = TextureAtlas::new(&device, atlas_size, &[texture_format]);

            // Allocate two textures to ensure the second one is not at the origin
            let _texture1 = atlas.lock().allocate(&device, &queue, [10, 10]).unwrap();
            let texture2 = atlas.lock().allocate(&device, &queue, [17, 17]).unwrap(); // Use non-aligned size

            let texture_size = texture2.size();
            let location = texture2.location().unwrap();

            // Create sample data to write
            let data: Vec<u8> = (0..texture_size[0] * texture_size[1])
                .map(|i| (i % 256) as u8)
                .collect();
            texture2.write_data(&queue, &[&data]).unwrap();

            // Create a buffer to read the data back, respecting alignment
            let bytes_per_pixel = texture_format.block_copy_size(None).unwrap();
            let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            let bytes_per_row_unaligned = texture_size[0] * bytes_per_pixel;
            let padded_bytes_per_row = bytes_per_row_unaligned.div_ceil(align) * align;
            let buffer_size = (padded_bytes_per_row * texture_size[1]) as u64;

            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Readback Buffer"),
                size: buffer_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });

            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            // Copy the written data from the atlas texture to the buffer
            let copy_size = wgpu::Extent3d {
                width: texture_size[0],
                height: texture_size[1],
                depth_or_array_layers: 1,
            };
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: &atlas.lock().textures()[0],
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: location.bounds.min.x as u32,
                        y: location.bounds.min.y as u32,
                        z: location.page_index,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(padded_bytes_per_row),
                        rows_per_image: Some(texture_size[1]),
                    },
                },
                copy_size,
            );

            queue.submit(Some(encoder.finish()));

            // Read the buffer and verify the data
            let buffer_slice = buffer.slice(..);
            let (tx, rx) = std::sync::mpsc::channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                tx.send(result).unwrap();
            });
            let _ = device.poll(wgpu::MaintainBase::Wait);
            rx.recv().unwrap().unwrap();

            let padded_data = buffer_slice.get_mapped_range();
            // Compare the original data with the (potentially padded) data from the buffer
            for y in 0..texture_size[1] {
                let start_padded = (y * padded_bytes_per_row) as usize;
                let end_padded = start_padded + bytes_per_row_unaligned as usize;
                let start_original = (y * bytes_per_row_unaligned) as usize;
                let end_original = start_original + bytes_per_row_unaligned as usize;
                assert_eq!(
                    &padded_data[start_padded..end_padded],
                    &data[start_original..end_original]
                );
            }
        });
    }
}
