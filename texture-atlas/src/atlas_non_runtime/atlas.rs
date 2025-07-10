use std::sync::{Arc, Weak};

use dashmap::DashMap;
use euclid::Box2D;
use guillotiere::{AllocId, AtlasAllocator, Size, euclid};
use parking_lot::{Mutex, RwLock};
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
    atlas: Weak<RwLock<TextureAtlas>>,
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
        let atlas = atlas.read();

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
        let atlas = atlas.read();
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
        let atlas = atlas.read();
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
        let atlas = atlas.read();
        let Some(location) = atlas.get_location(self.inner.texture_id) else {
            return Err(TextureError::TextureNotFoundInAtlas);
        };

        Ok(location.uv)
    }
}

// Ensure the texture area will be deallocated when the texture is dropped.
impl Drop for TextureInner {
    fn drop(&mut self) {
        if let Some(atlas) = self.atlas.upgrade() {
            match atlas.read().deallocate(self.texture_id) {
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

    weak_self: Weak<RwLock<Self>>,
}

struct TextureAtlasState {
    allocators: Vec<Mutex<AtlasAllocator>>,
    texture_id_to_location: DashMap<TextureId, TextureLocation>,
    texture_id_to_alloc_id: DashMap<TextureId, AllocId>,
    usage: std::sync::atomic::AtomicUsize,
}

/// Constructor and information methods.
impl TextureAtlas {
    pub fn new(
        device: &wgpu::Device,
        size: wgpu::Extent3d,
        formats: &[wgpu::TextureFormat],
    ) -> Arc<RwLock<Self>> {
        let (textures, texture_views) = Self::create_texture_and_view(device, formats, size);

        // Initialize the state with an empty allocator and allocation map.
        let state = TextureAtlasState {
            allocators: (0..size.depth_or_array_layers)
                .map(|_| Size::new(size.width as i32, size.height as i32))
                .map(AtlasAllocator::new)
                .map(Mutex::new)
                .collect(),
            texture_id_to_location: DashMap::new(),
            texture_id_to_alloc_id: DashMap::new(),
            usage: std::sync::atomic::AtomicUsize::new(0),
        };

        Arc::new_cyclic(|weak_self| {
            RwLock::new(Self {
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
        self.state.usage.load(std::sync::atomic::Ordering::SeqCst)
    }

    // todo: we can optimize this performance.
    pub fn max_allocation_size(&self) -> [u32; 2] {
        let mut max_size = [0; 2];

        for entry in self.state.texture_id_to_location.iter() {
            let size = entry.value().size();
            max_size[0] = max_size[0].max(size[0]);
            max_size[1] = max_size[1].max(size[1]);
        }
        max_size
    }
}

/// TextureAtlas allocation and deallocation
impl TextureAtlas {
    /// Allocate a texture in the atlas.
    pub fn allocate(&self, size: [u32; 2]) -> Result<Texture, TextureAtlasError> {
        let size = Size::new(size[0] as i32, size[1] as i32);

        for (page_index, allocator) in self.state.allocators.iter().enumerate() {
            if let Some(alloc) = allocator.lock().allocate(size) {
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
                // use `SeqCst` for safety, just for now. we can optimize this later.
                self.state.usage.fetch_add(
                    location.bounds.area() as usize,
                    std::sync::atomic::Ordering::SeqCst,
                );

                // Return the allocated texture
                return Ok(texture);
            }
        }

        Err(TextureAtlasError::AllocationFailedNotEnoughSpace)
    }

    /// Deallocate a texture from the atlas.
    /// This will be called automatically when the `TextureInner` is dropped.
    fn deallocate(&self, id: TextureId) -> Result<(), DeallocationErrorTextureNotFound> {
        // Find the texture location and remove it from the id-to-location map.
        let (_, location) = self
            .state
            .texture_id_to_location
            .remove(&id)
            .ok_or(DeallocationErrorTextureNotFound)?;

        // Find the allocation id and remove it from the id-to-alloc-id map.
        let (_, alloc_id) = self
            .state
            .texture_id_to_alloc_id
            .remove(&id)
            .ok_or(DeallocationErrorTextureNotFound)?;

        // Deallocate the texture from the allocator.
        self.state.allocators[location.page_index as usize]
            .lock()
            .deallocate(alloc_id);

        // Update usage
        // use `SeqCst` for safety, just for now. we can optimize this later.
        self.state.usage.fetch_sub(
            location.area() as usize,
            std::sync::atomic::Ordering::SeqCst,
        );

        Ok(())
    }
}

/// Resize the atlas to a new size.
impl TextureAtlas {
    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        new_size: wgpu::Extent3d,
        allow_data_loss: bool,
        new_allocation: Option<[u32; 2]>,
    ) -> Result<Option<Texture>, TextureAtlasError> {
        // mutable reference ensures we can modify the atlas state without threading issues.

        // new allocator and allocation map
        let mut new_allocators = (0..new_size.depth_or_array_layers)
            .map(|_| Size::new(new_size.width as i32, new_size.height as i32))
            .map(AtlasAllocator::new)
            .map(Mutex::new)
            .collect::<Vec<_>>();

        // Re-allocate existing textures
        let (new_texture_id_to_location, new_texture_id_to_alloc_id, mut new_usage) =
            self.reallocate_existing(&mut new_allocators, new_size, allow_data_loss)?;

        // Allocate for the new texture if requested
        let return_texture = if let Some(size) = new_allocation {
            match self.allocate_new(&mut new_allocators, new_size, size)? {
                Some(new_texture_data) => Some(new_texture_data),
                None => {
                    if !allow_data_loss {
                        return Err(TextureAtlasError::ResizeFailedNotEnoughSpace);
                    }
                    None
                }
            }
        } else {
            None
        };

        // Copy data from old textures to new textures
        let (new_textures, new_texture_views) =
            Self::create_texture_and_view(device, &self.formats, new_size);

        let location_map = new_texture_id_to_location.iter().map(|entry| {
            let id = entry.key();
            let new_location = entry.value();
            let old_location = self
                .state
                .texture_id_to_location
                .get(id)
                .expect("New allocation map was constructed from old allocation map, so it should contain all ids");
            (*old_location, *new_location)
        });

        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("TextureAtlas Resize Command Encoder"),
        });

        Self::copy_texture_data(
            &mut command_encoder,
            &self.textures,
            &self.texture_views,
            &new_textures,
            &new_texture_views,
            location_map,
        );

        // Submit the command encoder to copy data
        queue.submit(Some(command_encoder.finish()));

        // Update the atlas state
        self.textures = new_textures;
        self.texture_views = new_texture_views;
        self.size = new_size;

        if let Some((ref texture, location, alloc_id)) = return_texture {
            new_texture_id_to_location.insert(texture.inner.texture_id, location);
            new_texture_id_to_alloc_id.insert(texture.inner.texture_id, alloc_id);
            new_usage += location.bounds.area() as usize;
        }

        self.state.allocators = new_allocators;
        self.state.texture_id_to_location = new_texture_id_to_location;
        self.state.texture_id_to_alloc_id = new_texture_id_to_alloc_id;
        self.state.usage = std::sync::atomic::AtomicUsize::new(new_usage);

        Ok(return_texture.map(|(texture, _, _)| texture))
    }

    #[allow(clippy::type_complexity)] // This function is for internal use only, so I think it's not a problem.
    fn reallocate_existing(
        &self,
        new_allocators: &mut [Mutex<AtlasAllocator>],
        new_atlas_size: wgpu::Extent3d,
        allow_data_loss: bool,
    ) -> Result<
        (
            DashMap<TextureId, TextureLocation>,
            DashMap<TextureId, AllocId>,
            usize,
        ),
        TextureAtlasError,
    > {
        let new_texture_id_to_location = DashMap::new();
        let new_texture_id_to_alloc_id = DashMap::new();
        let mut new_usage = 0;

        'for_each_textures: for (texture_id, size) in self
            .state
            .texture_id_to_location
            .iter()
            .map(|entry| (*entry.key(), entry.value().size()))
        {
            let allocate_request = Size::new(size[0] as i32, size[1] as i32);

            for (page_index, allocator) in new_allocators.iter_mut().enumerate() {
                if let Some(alloc) = allocator.lock().allocate(allocate_request) {
                    let bounds = alloc.rectangle;
                    let uvs = euclid::Box2D::new(
                        euclid::Point2D::new(
                            (bounds.min.x as f32) / (new_atlas_size.width as f32),
                            (bounds.min.y as f32) / (new_atlas_size.height as f32),
                        ),
                        euclid::Point2D::new(
                            (bounds.max.x as f32) / (new_atlas_size.width as f32),
                            (bounds.max.y as f32) / (new_atlas_size.height as f32),
                        ),
                    );

                    let location = TextureLocation {
                        page_index: page_index as u32,
                        bounds,
                        uv: uvs,
                    };

                    new_texture_id_to_location.insert(texture_id, location);
                    new_texture_id_to_alloc_id.insert(texture_id, alloc.id);
                    new_usage += location.bounds.area() as usize;
                    continue 'for_each_textures;
                }
            }

            // If we reach here, it means we couldn't allocate the texture in any page.
            if !allow_data_loss {
                return Err(TextureAtlasError::ResizeFailedNotEnoughSpace);
            }
        }

        Ok((
            new_texture_id_to_location,
            new_texture_id_to_alloc_id,
            new_usage,
        ))
    }

    fn allocate_new(
        &self,
        new_allocators: &mut [Mutex<AtlasAllocator>],
        new_atlas_size: wgpu::Extent3d,
        size: [u32; 2],
    ) -> Result<Option<(Texture, TextureLocation, AllocId)>, TextureAtlasError> {
        let allocate_request = Size::new(size[0] as i32, size[1] as i32);

        for (page_index, allocator) in new_allocators.iter_mut().enumerate() {
            if let Some(alloc) = allocator.lock().allocate(allocate_request) {
                let bounds = alloc.rectangle;
                let uvs = euclid::Box2D::new(
                    euclid::Point2D::new(
                        (bounds.min.x as f32) / (new_atlas_size.width as f32),
                        (bounds.min.y as f32) / (new_atlas_size.height as f32),
                    ),
                    euclid::Point2D::new(
                        (bounds.max.x as f32) / (new_atlas_size.width as f32),
                        (bounds.max.y as f32) / (new_atlas_size.height as f32),
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
                let texture = Texture {
                    inner: Arc::new(TextureInner {
                        texture_id,
                        atlas: self.weak_self.clone(),
                        size,
                        formats: self.formats.clone(),
                    }),
                };

                return Ok(Some((texture, location, alloc.id)));
            }
        }

        Ok(None)
    }
}

// for internal use only
impl TextureAtlas {
    fn get_location(&self, id: TextureId) -> Option<TextureLocation> {
        self.state
            .texture_id_to_location
            .get(&id)
            .map(|entry| *entry.value())
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
                ..wgpu::TextureViewDescriptor::default()
            });
            textures.push(texture);
            texture_views.push(texture_view);
        }

        (textures, texture_views)
    }

    // Leave unused args to make refactoring easier.
    fn copy_texture_data(
        encoder: &mut wgpu::CommandEncoder,
        old_textures: &[wgpu::Texture],
        _old_texture_views: &[wgpu::TextureView],
        new_textures: &[wgpu::Texture],
        _new_texture_views: &[wgpu::TextureView],
        location_map: impl Iterator<Item = (TextureLocation, TextureLocation)>,
    ) {
        for (old_location, new_location) in location_map {
            for (old_texture, new_texture) in old_textures.iter().zip(new_textures.iter()) {
                let old_origin = wgpu::Origin3d {
                    x: old_location.bounds.min.x as u32,
                    y: old_location.bounds.min.y as u32,
                    z: old_location.page_index,
                };

                let new_origin = wgpu::Origin3d {
                    x: new_location.bounds.min.x as u32,
                    y: new_location.bounds.min.y as u32,
                    z: new_location.page_index,
                };

                let size = old_location.size();

                encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: old_texture,
                        mip_level: 0,
                        origin: old_origin,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyTextureInfo {
                        texture: new_texture,
                        mip_level: 0,
                        origin: new_origin,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width: size[0],
                        height: size[1],
                        depth_or_array_layers: 1,
                    },
                );
            }
        }
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
            let atlas = atlas.read();
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
            let atlas = atlas.read();

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
            let (device, _queue) = setup_wgpu().await;
            let size = wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            };
            let formats = &[wgpu::TextureFormat::Rgba8UnormSrgb];
            let atlas = TextureAtlas::new(&device, size, formats);

            // Allocate one texture
            let texture1 = atlas.read().allocate([32, 32]).unwrap();
            assert_eq!(atlas.read().allocation_count(), 1);
            assert_eq!(atlas.read().usage(), 32 * 32);

            // Allocate another texture
            let texture2 = atlas.read().allocate([16, 16]).unwrap();
            assert_eq!(atlas.read().allocation_count(), 2);
            assert_eq!(atlas.read().usage(), 32 * 32 + 16 * 16);

            // Deallocate one texture
            drop(texture1);
            assert_eq!(atlas.read().allocation_count(), 1);
            assert_eq!(atlas.read().usage(), 16 * 16);

            // Deallocate the other texture
            drop(texture2);
            assert_eq!(atlas.read().allocation_count(), 0);
            assert_eq!(atlas.read().usage(), 0);
        });
    }

    /// Tests that the atlas correctly returns an error when there is not enough space for a new allocation.
    #[test]
    fn test_allocation_failure() {
        pollster::block_on(async {
            let (device, _queue) = setup_wgpu().await;
            let size = wgpu::Extent3d {
                width: 32,
                height: 32,
                depth_or_array_layers: 1,
            };
            let formats = &[wgpu::TextureFormat::Rgba8UnormSrgb];
            let atlas = TextureAtlas::new(&device, size, formats);

            // This should succeed
            let _texture1 = atlas.read().allocate([32, 32]).unwrap();

            // This should fail
            let result = atlas.read().allocate([1, 1]);
            assert!(matches!(
                result,
                Err(TextureAtlasError::AllocationFailedNotEnoughSpace)
            ));
        });
    }

    /// Tests if the space freed by a deallocated texture can be reused by a new allocation.
    #[test]
    fn test_reuse_deallocated_space() {
        pollster::block_on(async {
            let (device, _queue) = setup_wgpu().await;
            let size = wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            };
            let formats = &[wgpu::TextureFormat::Rgba8UnormSrgb];
            let atlas = TextureAtlas::new(&device, size, formats);

            let texture1 = atlas.read().allocate([64, 64]).unwrap();
            assert_eq!(atlas.read().allocation_count(), 1);

            drop(texture1);
            assert_eq!(atlas.read().allocation_count(), 0);

            // Should be able to allocate again in the same space
            let _texture2 = atlas.read().allocate([64, 64]).unwrap();
            assert_eq!(atlas.read().allocation_count(), 1);
        });
    }

    /// Tests if the atlas can be resized to a larger size, preserving existing allocations.
    #[test]
    fn test_atlas_resize_grow() {
        pollster::block_on(async {
            let (device, queue) = setup_wgpu().await;
            let initial_size = wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            };
            let formats = &[wgpu::TextureFormat::Rgba8UnormSrgb];
            let atlas = TextureAtlas::new(&device, initial_size, formats);

            let _texture1 = atlas.read().allocate([32, 32]).unwrap();
            assert_eq!(atlas.read().allocation_count(), 1);

            let new_size = wgpu::Extent3d {
                width: 128,
                height: 128,
                depth_or_array_layers: 1,
            };
            atlas
                .write()
                .resize(&device, &queue, new_size, false, None)
                .unwrap();

            assert_eq!(atlas.read().size(), new_size);
            assert_eq!(atlas.read().allocation_count(), 1);
        });
    }

    /// Tests that resizing the atlas to a smaller size fails if existing textures do not fit
    /// and `allow_data_loss` is false.
    #[test]
    fn test_atlas_resize_shrink_fail() {
        pollster::block_on(async {
            let (device, queue) = setup_wgpu().await;
            let initial_size = wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            };
            let formats = &[wgpu::TextureFormat::Rgba8UnormSrgb];
            let atlas = TextureAtlas::new(&device, initial_size, formats);

            let _texture1 = atlas.read().allocate([64, 64]).unwrap();

            let new_size = wgpu::Extent3d {
                width: 32,
                height: 32,
                depth_or_array_layers: 1,
            };
            let result = atlas.write().resize(&device, &queue, new_size, false, None);

            assert!(matches!(
                result,
                Err(TextureAtlasError::ResizeFailedNotEnoughSpace)
            ));
            // Ensure state is unchanged
            assert_eq!(atlas.read().size(), initial_size);
            assert_eq!(atlas.read().allocation_count(), 1);
        });
    }

    /// Tests that resizing the atlas to a smaller size succeeds with data loss
    /// if `allow_data_loss` is true, discarding textures that no longer fit.
    #[test]
    fn test_atlas_resize_shrink_with_data_loss() {
        pollster::block_on(async {
            let (device, queue) = setup_wgpu().await;
            let initial_size = wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            };
            let formats = &[wgpu::TextureFormat::Rgba8UnormSrgb];
            let atlas = TextureAtlas::new(&device, initial_size, formats);

            let _texture1 = atlas.read().allocate([40, 40]).unwrap();

            let new_size = wgpu::Extent3d {
                width: 32,
                height: 32,
                depth_or_array_layers: 1,
            };
            atlas
                .write()
                .resize(&device, &queue, new_size, true, None)
                .unwrap();

            assert_eq!(atlas.read().size(), new_size);
            assert_eq!(atlas.read().allocation_count(), 0); // texture should be gone
        });
    }

    /// Tests if the UV coordinates of an allocated texture are calculated correctly.
    #[test]
    fn test_texture_uv() {
        pollster::block_on(async {
            let (device, _queue) = setup_wgpu().await;
            let size = wgpu::Extent3d {
                width: 128,
                height: 128,
                depth_or_array_layers: 1,
            };
            let formats = &[wgpu::TextureFormat::Rgba8UnormSrgb];
            let atlas = TextureAtlas::new(&device, size, formats);

            let texture = atlas.read().allocate([32, 64]).unwrap();
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
            let (device, _queue) = setup_wgpu().await;
            let size = wgpu::Extent3d {
                width: 128,
                height: 128,
                depth_or_array_layers: 1,
            };
            let formats = &[wgpu::TextureFormat::Rgba8UnormSrgb];
            let atlas = TextureAtlas::new(&device, size, formats);

            let texture = atlas.read().allocate([32, 32]).unwrap();

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
            let _texture1 = atlas.read().allocate([10, 10]).unwrap();
            let texture2 = atlas.read().allocate([17, 17]).unwrap(); // Use non-aligned size

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
                    texture: &atlas.read().textures()[0],
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
