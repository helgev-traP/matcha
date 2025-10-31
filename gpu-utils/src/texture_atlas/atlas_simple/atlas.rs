use std::collections::HashMap;
use std::sync::{Arc, Weak};

use guillotiere::euclid::Box2D;
use guillotiere::{AllocId, AtlasAllocator, Size, euclid};
use log::{trace, warn};
use parking_lot::{Mutex, RwLock};
use thiserror::Error;
use uuid::Uuid;

use crate::device_loss_recoverable::DeviceLossRecoverable;

mod viewport_clear;
use viewport_clear::ViewportClear;

#[derive(Debug, Clone)]
pub struct AtlasRegion {
    inner: Arc<RegionData>,
}

// We only store the texture id and reference to the atlas,
// to make `Texture` remain valid after `TextureAtlas` resizes or changes,
// except for data loss when the atlas shrinks.
struct RegionData {
    // allocation info
    region_id: RegionId,
    atlas_id: TextureAtlasId,
    // interaction with the atlas
    atlas: Weak<TextureAtlas>,
    // It may be useful to store some information about the texture that will not change during atlas resizing
    allocation_size: [u32; 2], // size of the allocated area including margins
    usable_size: [u32; 2],     // size of the usable texture area excluding margins
    atlas_size: [u32; 2],      // size of the atlas when the texture was allocated
    format: wgpu::TextureFormat, // format of the texture
}

impl std::fmt::Debug for RegionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegionData")
            .field("region_id", &self.region_id)
            .field("atlas_id", &self.atlas_id)
            .field("texture_size", &self.usable_size)
            .field("atlas_size", &self.atlas_size)
            .field("format", &self.format)
            .finish()
    }
}

/// Public API to interact with a texture.
/// User code should not need to know about its id, location, or atlas.
impl AtlasRegion {
    pub fn atlas_id(&self) -> TextureAtlasId {
        trace!(
            "AtlasRegion::atlas_id called for region={:?}",
            self.inner.region_id
        );
        self.inner.atlas_id
    }

    pub fn position_in_atlas(&self) -> Result<(u32, Box2D<f32, euclid::UnknownUnit>), RegionError> {
        trace!(
            "AtlasRegion::position_in_atlas: querying region={:?}",
            self.inner.region_id
        );
        // Get the texture location in the atlas
        let Some(atlas) = self.inner.atlas.upgrade() else {
            warn!("AtlasRegion::position_in_atlas: atlas dropped");
            return Err(RegionError::AtlasGone);
        };
        let Some(location) = atlas.get_location(self.inner.region_id) else {
            warn!("AtlasRegion::position_in_atlas: region not found in atlas");
            return Err(RegionError::TextureNotFoundInAtlas);
        };

        Ok((location.page_index, location.usable_uv_bounds))
    }

    pub fn area(&self) -> u32 {
        self.inner.usable_size[0] * self.inner.usable_size[1]
    }

    pub fn allocation_size(&self) -> [u32; 2] {
        self.inner.allocation_size
    }

    pub fn texture_size(&self) -> [u32; 2] {
        self.inner.usable_size
    }

    pub fn atlas_size(&self) -> [u32; 2] {
        self.inner.atlas_size
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.inner.format
    }

    pub fn atlas_pointer(&self) -> Option<usize> {
        self.inner
            .atlas
            .upgrade()
            .map(|arc| Arc::as_ptr(&arc) as usize)
    }

    pub fn translate_uv(&self, uvs: &[[f32; 2]]) -> Result<Vec<[f32; 2]>, RegionError> {
        trace!(
            "AtlasRegion::translate_uv: translating {} vertices for region={:?}",
            uvs.len(),
            self.inner.region_id
        );
        // Get the texture location in the atlas
        let Some(atlas) = self.inner.atlas.upgrade() else {
            warn!("AtlasRegion::translate_uv: atlas dropped");
            return Err(RegionError::AtlasGone);
        };
        let Some(location) = atlas.get_location(self.inner.region_id) else {
            warn!("AtlasRegion::translate_uv: region not found in atlas");
            return Err(RegionError::TextureNotFoundInAtlas);
        };
        let x_max = location.usable_uv_bounds.max.x;
        let y_max = location.usable_uv_bounds.max.y;
        let x_min = location.usable_uv_bounds.min.x;
        let y_min = location.usable_uv_bounds.min.y;

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

    pub fn write_data(&self, queue: &wgpu::Queue, data: &[u8]) -> Result<(), RegionError> {
        trace!(
            "AtlasRegion::write_data: uploading {} bytes to region={:?}",
            data.len(),
            self.inner.region_id
        );
        // Check data consistency
        // todo: `block_copy_size()` may return deferent size from the actual size.
        let bytes_per_pixel = self
            .inner
            .format
            .block_copy_size(None)
            .ok_or(RegionError::InvalidFormatBlockCopySize)?;
        let expected_size = self.inner.usable_size[0] * self.inner.usable_size[1] * bytes_per_pixel;
        if data.len() as u32 != expected_size {
            warn!(
                "AtlasRegion::write_data: data size mismatch (expected {} bytes, got {})",
                expected_size,
                data.len()
            );
            return Err(RegionError::DataConsistencyError(format!(
                "Data size({}byte) does not match expected size({}byte)",
                data.len(),
                expected_size
            )));
        }

        // Get the texture in the atlas and location
        let Some(atlas) = self.inner.atlas.upgrade() else {
            warn!("AtlasRegion::write_data: atlas dropped");
            return Err(RegionError::AtlasGone);
        };
        let texture = atlas.texture();
        let Some(location) = atlas.get_location(self.inner.region_id) else {
            warn!("AtlasRegion::write_data: region not found in atlas");
            return Err(RegionError::TextureNotFoundInAtlas);
        };

        let bytes_per_row = self.inner.usable_size[0] * bytes_per_pixel;

        let origin = wgpu::Origin3d {
            x: location.usable_bounds.min.x as u32,
            y: location.usable_bounds.min.y as u32,
            z: location.page_index,
        };

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
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
                width: self.inner.usable_size[0],
                height: self.inner.usable_size[1],
                depth_or_array_layers: 1,
            },
        );

        trace!("AtlasRegion::write_data: upload completed");

        Ok(())
    }

    pub fn read_data(&self) -> Result<(), RegionError> {
        todo!()
    }

    pub fn copy_from_texture(&self) -> Result<(), RegionError> {
        todo!()
    }

    pub fn copy_to_texture(&self) -> Result<(), RegionError> {
        todo!()
    }

    pub fn copy_from_buffer(&self) -> Result<(), RegionError> {
        todo!()
    }

    pub fn copy_to_buffer(&self) -> Result<(), RegionError> {
        todo!()
    }

    pub fn set_viewport(&self, render_pass: &mut wgpu::RenderPass<'_>) -> Result<(), RegionError> {
        // Get the texture location in the atlas
        let Some(atlas) = self.inner.atlas.upgrade() else {
            return Err(RegionError::AtlasGone);
        };
        let Some(location) = atlas.get_location(self.inner.region_id) else {
            return Err(RegionError::TextureNotFoundInAtlas);
        };

        // Set the viewport to the texture area
        render_pass.set_viewport(
            location.usable_bounds.min.x as f32,
            location.usable_bounds.min.y as f32,
            location.usable_bounds.width() as f32,
            location.usable_bounds.height() as f32,
            0.0,
            1.0,
        );

        Ok(())
    }

    pub fn begin_render_pass<'a>(
        &'a self,
        encoder: &'a mut wgpu::CommandEncoder,
    ) -> Result<wgpu::RenderPass<'a>, RegionError> {
        // Get the texture location in the atlas
        let Some(atlas) = self.inner.atlas.upgrade() else {
            return Err(RegionError::AtlasGone);
        };
        let Some(location) = atlas.get_location(self.inner.region_id) else {
            return Err(RegionError::TextureNotFoundInAtlas);
        };

        // Clear the allocated region (including the margin) before exposing the render pass to users.
        let view = atlas.layer_texture_view(location.page_index as usize);
        let allocation_bounds = location.allocation_bounds();
        let allocation_width = (allocation_bounds.max.x - allocation_bounds.min.x) as u32;
        let allocation_height = (allocation_bounds.max.y - allocation_bounds.min.y) as u32;
        debug_assert!(allocation_width > 0 && allocation_height > 0);
        debug_assert!(allocation_bounds.min.x >= 0);
        debug_assert!(allocation_bounds.min.y >= 0);

        {
            let mut clear_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Texture Atlas Margin Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            clear_pass.set_viewport(
                allocation_bounds.min.x as f32,
                allocation_bounds.min.y as f32,
                allocation_width as f32,
                allocation_height as f32,
                0.0,
                1.0,
            );
            clear_pass.set_scissor_rect(
                allocation_bounds.min.x as u32,
                allocation_bounds.min.y as u32,
                allocation_width,
                allocation_height,
            );
            atlas.viewport_clear.render(
                &atlas.device(),
                &mut clear_pass,
                atlas.format(),
                [0.0, 0.0, 0.0, 0.0],
            );
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Texture Atlas Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Set the viewport to the usable texture area (excluding margins)
        render_pass.set_viewport(
            location.usable_bounds.min.x as f32,
            location.usable_bounds.min.y as f32,
            location.usable_bounds.width() as f32,
            location.usable_bounds.height() as f32,
            0.0,
            1.0,
        );

        Ok(render_pass)
    }

    pub fn uv(&self) -> Result<Box2D<f32, euclid::UnknownUnit>, RegionError> {
        // Get the texture location in the atlas
        let Some(atlas) = self.inner.atlas.upgrade() else {
            return Err(RegionError::AtlasGone);
        };
        let Some(location) = atlas.get_location(self.inner.region_id) else {
            return Err(RegionError::TextureNotFoundInAtlas);
        };

        Ok(location.usable_uv_bounds)
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
impl Drop for RegionData {
    fn drop(&mut self) {
        if let Some(atlas) = self.atlas.upgrade() {
            match atlas.deallocate(self.region_id) {
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
struct RegionId {
    texture_uuid: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RegionLocation {
    page_index: u32,
    margin: u32,
    allocation_bounds: euclid::Box2D<i32, euclid::UnknownUnit>,
    usable_bounds: euclid::Box2D<i32, euclid::UnknownUnit>,
    usable_uv_bounds: euclid::Box2D<f32, euclid::UnknownUnit>,
}

impl RegionLocation {
    fn new(
        allocation_bounds: Box2D<i32, euclid::UnknownUnit>,
        atlas_size: [u32; 2],
        page_index: usize,
        margin: u32,
    ) -> Self {
        let bounds = if margin == 0 {
            allocation_bounds
        } else {
            euclid::Box2D::new(
                euclid::Point2D::new(
                    allocation_bounds.min.x + margin as i32,
                    allocation_bounds.min.y + margin as i32,
                ),
                euclid::Point2D::new(
                    allocation_bounds.max.x - margin as i32,
                    allocation_bounds.max.y - margin as i32,
                ),
            )
        };

        debug_assert!(bounds.min.x >= allocation_bounds.min.x);
        debug_assert!(bounds.min.y >= allocation_bounds.min.y);
        debug_assert!(bounds.max.x <= allocation_bounds.max.x);
        debug_assert!(bounds.max.y <= allocation_bounds.max.y);
        debug_assert!(bounds.max.x > bounds.min.x);
        debug_assert!(bounds.max.y > bounds.min.y);

        // Normalize the usable bounds to UV space.
        let uv = euclid::Box2D::new(
            euclid::Point2D::new(
                bounds.min.x as f32 / atlas_size[0] as f32,
                bounds.min.y as f32 / atlas_size[1] as f32,
            ),
            euclid::Point2D::new(
                bounds.max.x as f32 / atlas_size[0] as f32,
                bounds.max.y as f32 / atlas_size[1] as f32,
            ),
        );
        Self {
            page_index: page_index as u32,
            margin,
            allocation_bounds,
            usable_bounds: bounds,
            usable_uv_bounds: uv,
        }
    }

    fn area(&self) -> u32 {
        self.usable_bounds.area() as u32
    }

    fn allocation_area(&self) -> u32 {
        self.allocation_bounds.area() as u32
    }

    fn allocation_bounds(&self) -> euclid::Box2D<i32, euclid::UnknownUnit> {
        self.allocation_bounds
    }

    fn size(&self) -> [u32; 2] {
        [
            (self.usable_bounds.max.x - self.usable_bounds.min.x) as u32,
            (self.usable_bounds.max.y - self.usable_bounds.min.y) as u32,
        ]
    }
}

static ATLAS_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureAtlasId {
    id: usize,
}

impl TextureAtlasId {
    fn new() -> Self {
        let id = ATLAS_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self { id }
    }
}

pub struct TextureAtlas {
    id: TextureAtlasId,
    format: wgpu::TextureFormat,
    state: Mutex<TextureAtlasState>,
    resources: RwLock<TextureAtlasResources>,
    device: RwLock<wgpu::Device>,
    viewport_clear: ViewportClear,
    margin: u32,
    weak_self: Weak<Self>,
}

struct TextureAtlasResources {
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    layer_texture_views: Vec<wgpu::TextureView>,
    size: wgpu::Extent3d,
}

struct TextureAtlasState {
    allocators: Vec<AtlasAllocator>,
    texture_id_to_location: HashMap<RegionId, RegionLocation>,
    texture_id_to_alloc_id: HashMap<RegionId, AllocId>,
    usage: usize,
}

/// Constructor and information methods.
impl TextureAtlas {
    pub const DEFAULT_MARGIN_PX: u32 = 1;

    pub fn new(
        device: &wgpu::Device,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        margin: u32,
    ) -> Arc<Self> {
        let (texture, texture_view, layer_texture_views) =
            Self::create_texture_and_view(device, format, size);

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

        let resources = TextureAtlasResources {
            texture,
            texture_view,
            layer_texture_views,
            size,
        };

        Arc::new_cyclic(|weak_self| Self {
            id: TextureAtlasId::new(),
            format,
            state: Mutex::new(state),
            resources: RwLock::new(resources),
            device: RwLock::new(device.clone()),
            viewport_clear: ViewportClear::default(),
            margin,
            weak_self: weak_self.clone(),
        })
    }
}

impl DeviceLossRecoverable for TextureAtlas {
    fn recover(&self, device: &wgpu::Device, _: &wgpu::Queue) {
        let format = self.format;
        let size = self.size();
        let id = self.id;

        let (texture, texture_view, layer_texture_views) =
            Self::create_texture_and_view(device, format, size);

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

        let resources = TextureAtlasResources {
            texture,
            texture_view,
            layer_texture_views,
            size,
        };

        let mut state_lock = self.state.lock();
        *state_lock = state;

        let mut resources_lock = self.resources.write();
        *resources_lock = resources;

        *self.device.write() = device.clone();
        self.viewport_clear.reset();

        trace!(
            "TextureAtlas::recover: recovered atlas id={id:?} with size={size:?} and format={format:?}"
        );
    }
}

impl TextureAtlas {
    pub fn size(&self) -> wgpu::Extent3d {
        self.resources.read().size
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    fn device(&self) -> wgpu::Device {
        self.device.read().clone()
    }

    pub fn margin(&self) -> u32 {
        self.margin
    }

    pub fn capacity(&self) -> usize {
        let resources = self.resources.read();
        resources.size.width as usize
            * resources.size.height as usize
            * resources.size.depth_or_array_layers as usize
    }

    pub fn usage(&self) -> usize {
        self.state.lock().usage
    }

    // todo: we can optimize this performance.
    pub fn max_allocation_size(&self) -> [u32; 2] {
        let mut max_size = [0; 2];

        let state = self.state.lock();
        for location in state.texture_id_to_location.values() {
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
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        requested_size: [u32; 2],
    ) -> Result<AtlasRegion, TextureAtlasError> {
        // Check if size is smaller than the atlas size
        if requested_size[0] == 0 || requested_size[1] == 0 {
            return Err(TextureAtlasError::AllocationFailedInvalidSize {
                requested: requested_size,
            });
        }
        let atlas_size = self.size();
        if requested_size[0] + self.margin * 2 > atlas_size.width
            || requested_size[1] + self.margin * 2 > atlas_size.height
        {
            return Err(TextureAtlasError::AllocationFailedTooLarge {
                requested: requested_size,
                available: [atlas_size.width, atlas_size.height],
                margin_needed: self.margin * 2,
            });
        }

        let doubled_margin =
            self.margin
                .checked_mul(2)
                .ok_or(TextureAtlasError::AllocationFailedInvalidSize {
                    requested: requested_size,
                })?;

        let allocation_width = requested_size[0].checked_add(doubled_margin).ok_or(
            TextureAtlasError::AllocationFailedInvalidSize {
                requested: requested_size,
            },
        )?;
        let allocation_height = requested_size[1].checked_add(doubled_margin).ok_or(
            TextureAtlasError::AllocationFailedInvalidSize {
                requested: requested_size,
            },
        )?;

        if allocation_width > i32::MAX as u32 || allocation_height > i32::MAX as u32 {
            return Err(TextureAtlasError::AllocationFailedInvalidSize {
                requested: requested_size,
            });
        }

        let allocation_size = Size::new(allocation_width as i32, allocation_height as i32);

        if let Some(region) =
            self.try_allocate(allocation_size, [atlas_size.width, atlas_size.height])
        {
            return Ok(region);
        }

        self.add_one_page(device, queue);

        let updated_size = self.size();
        self.try_allocate(allocation_size, [updated_size.width, updated_size.height])
            .ok_or(TextureAtlasError::AllocationFailedNotEnoughSpace)
    }

    /// Deallocate a texture from the atlas.
    /// This will be called automatically when the `TextureInner` is dropped.
    fn deallocate(&self, id: RegionId) -> Result<(), DeallocationErrorTextureNotFound> {
        let mut state = self.state.lock();

        // Find the texture location and remove it from the id-to-location map.
        let location = state
            .texture_id_to_location
            .remove(&id)
            .ok_or(DeallocationErrorTextureNotFound)?;

        // Find the allocation id and remove it from the id-to-alloc-id map.
        let alloc_id = state
            .texture_id_to_alloc_id
            .remove(&id)
            .ok_or(DeallocationErrorTextureNotFound)?;

        // Deallocate the texture from the allocator.
        state.allocators[location.page_index as usize].deallocate(alloc_id);

        // Update usage
        state.usage -= location.allocation_area() as usize;

        Ok(())
    }

    fn try_allocate(&self, allocation_size: Size, atlas_size: [u32; 2]) -> Option<AtlasRegion> {
        let mut state = self.state.lock();

        for (page_index, allocator) in state.allocators.iter_mut().enumerate() {
            if let Some(alloc) = allocator.allocate(allocation_size) {
                let location =
                    RegionLocation::new(alloc.rectangle, atlas_size, page_index, self.margin);

                let texture_id = RegionId {
                    texture_uuid: Uuid::new_v4(),
                };
                let texture_inner = RegionData {
                    region_id: texture_id,
                    atlas_id: self.id,
                    atlas: self.weak_self.clone(),
                    allocation_size: [
                        location.allocation_bounds.width() as u32,
                        location.allocation_bounds.height() as u32,
                    ],
                    usable_size: [
                        location.usable_bounds.width() as u32,
                        location.usable_bounds.height() as u32,
                    ],
                    atlas_size,
                    format: self.format,
                };
                let texture = AtlasRegion {
                    inner: Arc::new(texture_inner),
                };

                state.texture_id_to_location.insert(texture_id, location);
                state.texture_id_to_alloc_id.insert(texture_id, alloc.id);
                state.usage += location.allocation_area() as usize;

                return Some(texture);
            }
        }

        None
    }
}

/// Resize the atlas to a new size.
impl TextureAtlas {
    fn add_one_page(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut resources = self.resources.write();
        let previous_size = resources.size;
        let new_size = wgpu::Extent3d {
            width: previous_size.width,
            height: previous_size.height,
            depth_or_array_layers: previous_size.depth_or_array_layers + 1,
        };

        let (new_texture, new_texture_view, new_layer_texture_views) =
            Self::create_texture_and_view(device, self.format, new_size);

        {
            let mut state = self.state.lock();
            state.allocators.push(AtlasAllocator::new(Size::new(
                new_size.width as i32,
                new_size.height as i32,
            )));
        }

        let old_texture = resources.texture.clone();

        // Copy existing texture data to the new textures.
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("TextureAtlas Resize Encoder"),
        });

        // Copy existing pages into the new texture
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &old_texture,
                mip_level: 0,
                aspect: wgpu::TextureAspect::All,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
            },
            wgpu::TexelCopyTextureInfo {
                texture: &new_texture,
                mip_level: 0,
                aspect: wgpu::TextureAspect::All,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
            },
            wgpu::Extent3d {
                width: previous_size.width,
                height: previous_size.height,
                depth_or_array_layers: previous_size.depth_or_array_layers,
            },
        );

        // Clear only the newly added layer to ensure it is initialized and transparent.
        // This prevents uninitialized memory in the new layer and keeps existing pages intact.
        let new_layer_index = new_size.depth_or_array_layers - 1;
        if let Some(view) = new_layer_texture_views.get(new_layer_index as usize) {
            let _clear_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("TextureAtlas Init New Layer Clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        queue.submit(Some(encoder.finish()));

        resources.texture = new_texture;
        resources.texture_view = new_texture_view;
        resources.layer_texture_views = new_layer_texture_views;
        resources.size = new_size;
    }
}

impl TextureAtlas {
    fn get_location(&self, id: RegionId) -> Option<RegionLocation> {
        self.state.lock().texture_id_to_location.get(&id).copied()
    }

    pub fn texture(&self) -> wgpu::Texture {
        self.resources.read().texture.clone()
    }

    pub fn texture_view(&self) -> wgpu::TextureView {
        self.resources.read().texture_view.clone()
    }

    fn layer_texture_view(&self, index: usize) -> wgpu::TextureView {
        self.resources.read().layer_texture_views[index].clone()
    }
}

// helper functions
impl TextureAtlas {
    fn create_texture_and_view(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        page_size: wgpu::Extent3d,
    ) -> (wgpu::Texture, wgpu::TextureView, Vec<wgpu::TextureView>) {
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

        // D2Array view for sampling all layers
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&texture_view_label),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            aspect: wgpu::TextureAspect::All,
            ..Default::default()
        });

        // Per-layer D2 views for render attachments (one per array layer)
        let mut per_layer_views = Vec::with_capacity(page_size.depth_or_array_layers as usize);
        for layer in 0..page_size.depth_or_array_layers {
            let layer_view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(&format!("texture_atlas_layer_view_{format:?}_{layer}")),
                dimension: Some(wgpu::TextureViewDimension::D2),
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: layer,
                array_layer_count: Some(1),
                aspect: wgpu::TextureAspect::All,
                ..Default::default()
            });
            per_layer_views.push(layer_view);
        }

        (texture, texture_view, per_layer_views)
    }
}

/// `DeallocationErrorTextureNotFound` only be used in this file.
struct DeallocationErrorTextureNotFound;

#[derive(Error, Debug)]
pub enum RegionError {
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
    #[error(
        "Allocation failed because the requested size is too large for the atlas. requested: {requested:?} available: {available:?}"
    )]
    AllocationFailedTooLarge {
        requested: [u32; 2],
        available: [u32; 2],
        margin_needed: u32,
    },
    #[error("Allocation failed because the requested size is invalid. requested: {requested:?}")]
    AllocationFailedInvalidSize { requested: [u32; 2] },
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    /// Verifies tokio test harness can await to fetch a WGPU device/queue in this crate.
    /// This is a sanity test for the async runtime integration.
    #[tokio::test]
    async fn use_tokio_test_macro_to_await_to_get_wgpu_device() {
        // Arrange & Act
        // let (_instance, _adapter, _device, _queue) = crate::wgpu_utils::noop_wgpu().await;

        // Assert
        // not implemented: this is only a harness check.
    }

    // -------------------------------
    // TextureAtlas::new / basic info
    // -------------------------------

    /// TextureAtlas::new initializes GPU resources and metadata consistently.
    /// - Precondition: Provide valid device, extent, format, margin.
    /// - Verify:
    ///   - size() equals the requested Extent3d.
    ///   - format() equals requested format.
    ///   - margin() equals requested margin.
    ///   - capacity() == width * height * layers.
    ///   - usage() == 0 on creation.
    #[tokio::test]
    async fn atlas_new_initializes_resources_and_metadata() {
        // not implemented
    }

    /// create_texture_and_view (indirectly via new) produces:
    /// - an array texture with 1 mip, COPY_SRC/COPY_DST/TEXTURE_BINDING/RENDER_ATTACHMENT usages,
    /// - a D2Array view (texture_view()),
    /// - per-layer D2 views (layer_texture_view(i)) whose count equals depth_or_array_layers.
    #[tokio::test]
    async fn atlas_texture_and_views_have_expected_dimensions_and_usages() {
        // not implemented
    }

    // ------------
    // allocation
    // ------------

    /// allocate fails with AllocationFailedInvalidSize when requested size has 0 in either dimension.
    #[tokio::test]
    async fn allocate_rejects_zero_dimension() {
        // not implemented
    }

    /// allocate fails with AllocationFailedTooLarge when requested_size + 2*margin exceeds a page.
    /// - Verify returned error fields: requested, available and margin_needed == margin*2.
    #[tokio::test]
    async fn allocate_rejects_too_large_including_margins() {
        // not implemented
    }

    /// allocate succeeds for a valid size and returns an AtlasRegion whose:
    /// - atlas_id() matches the creator atlas,
    /// - texture_size() == requested size,
    /// - allocation_size() == requested + 2*margin,
    /// - atlas_size() equals atlas.size() at allocation time,
    /// - format() matches atlas.format(),
    /// - area() == prod(texture_size),
    /// - position_in_atlas() returns (page_index, usable_uv) with 0<=uv<=1,
    /// - uv() equals the usable_uv from position_in_atlas(),
    /// - atlas_pointer() is Some(ptr).
    #[tokio::test]
    async fn allocate_success_and_region_exposes_expected_properties() {
        // not implemented
    }

    /// When a page has no free space for a new allocation, allocate triggers add_one_page:
    /// - Verify atlas.size().depth_or_array_layers increases by 1,
    /// - capacity() increases accordingly,
    /// - new allocations can land on the new page (page_index of position_in_atlas()).
    #[tokio::test]
    async fn allocate_triggers_growth_by_adding_one_page() {
        // not implemented
    }

    /// usage() tracks allocated area including margins:
    /// - After N allocations, usage() == sum(allocation_area),
    /// - After dropping regions (thus deallocation via Drop), usage() decreases accordingly.
    #[tokio::test]
    async fn usage_tracks_allocation_and_drop_deallocation() {
        // not implemented
    }

    /// max_allocation_size() reflects the largest usable (margin-excluded) size among live regions.
    /// - After mixed allocations, verify it returns the maximum width/height of usable bounds.
    #[tokio::test]
    async fn max_allocation_size_reflects_largest_live_region() {
        // not implemented
    }

    // ----------------
    // Region queries
    // ----------------

    /// position_in_atlas() returns TextureNotFoundInAtlas after device-loss recovery,
    /// because alloc maps are reset. Existing regions still hold metadata but have no mapping.
    #[tokio::test]
    async fn position_in_atlas_returns_not_found_after_recover() {
        // not implemented
    }

    /// position_in_atlas() returns AtlasGone when the backing atlas (Arc) is dropped and only the region remains.
    #[tokio::test]
    async fn position_in_atlas_returns_atlas_gone_when_atlas_dropped() {
        // not implemented
    }

    /// atlas_pointer() mirrors position_in_atlas()'s ownership semantics:
    /// - When the originating TextureAtlas is alive, atlas_pointer() yields Some(raw_ptr).
    /// - After dropping the Arc<TextureAtlas> (keeping only AtlasRegion), the Weak upgrade fails and atlas_pointer() must return None.
    #[tokio::test]
    async fn atlas_pointer_returns_none_after_atlas_drop() {
        // not implemented
    }

    /// uv() returns the same Box2D as position_in_atlas().1 (usable_uv_bounds).
    #[tokio::test]
    async fn uv_matches_position_in_atlas_usable_uv_bounds() {
        // not implemented
    }

    // -----------------
    // UV translation
    // -----------------

    /// translate_uv() maps [0,0],[1,0],[0,1],[1,1] into the usable uv rectangle,
    /// and clamps intermediate values into [0,1].
    /// - Also verify AtlasGone and TextureNotFoundInAtlas error paths are surfaced when applicable.
    #[tokio::test]
    async fn translate_uv_maps_corners_and_clamps() {
        // not implemented
    }

    // --------------
    // Data I/O
    // --------------

    /// write_data() succeeds when data length matches usable_size * bytes_per_pixel
    /// (bytes_per_pixel = format.block_copy_size(None)).
    /// Use a format with defined block size (e.g. Rgba8Unorm).
    #[tokio::test]
    async fn write_data_succeeds_on_consistent_size() {
        // not implemented
    }

    /// write_data() returns DataConsistencyError when provided data length does not match expected size.
    #[tokio::test]
    async fn write_data_fails_on_data_size_mismatch() {
        // not implemented
    }

    /// write_data() returns InvalidFormatBlockCopySize when format.block_copy_size(None) is None
    /// (e.g. unsupported/compressed formats where applicable).
    #[tokio::test]
    async fn write_data_fails_on_invalid_format_block_size() {
        // not implemented
    }

    /// write_data() propagates AtlasGone when the backing atlas has been dropped but an AtlasRegion handle remains.
    /// - Allocate a region, drop the TextureAtlas Arc, and attempt to write_data(); expect Err(AtlasGone).
    #[tokio::test]
    async fn write_data_fails_with_atlas_gone_when_atlas_dropped() {
        // not implemented
    }

    /// write_data() surfaces TextureNotFoundInAtlas after recover() resets allocator state.
    /// - Allocate + recover(), then calling write_data() on the stale region should return Err(TextureNotFoundInAtlas).
    #[tokio::test]
    async fn write_data_fails_with_texture_not_found_after_recover() {
        // not implemented
    }

    /// read_data()/copy_* are currently todo!():
    /// - Keep placeholders documenting expected future behavior (buffer/texture copy paths).
    /// - Enable later when implemented.
    #[test]
    fn read_and_copy_operations_placeholders() {
        // not implemented
    }

    // -------------------------
    // Rendering entry points
    // -------------------------

    /// set_viewport() sets the viewport to the region usable bounds on the provided render pass.
    /// - Build a transient render pass, call set_viewport(), and (optionally later) verify via debug markers or replay.
    #[tokio::test]
    async fn set_viewport_sets_usable_bounds() {
        // not implemented
    }

    /// set_viewport() returns AtlasGone when called after the TextureAtlas is dropped, and TextureNotFoundInAtlas after recover().
    /// - Exercise both error branches to ensure defensive error propagation for rendering paths.
    #[tokio::test]
    async fn set_viewport_errors_when_atlas_missing_or_region_unmapped() {
        // not implemented
    }

    /// begin_render_pass():
    /// - Clears the allocation rectangle (including margins) on the target layer,
    /// - Returns a render pass preconfigured with viewport==usable bounds.
    /// The test should record commands and (optionally later) validate via readback or debug renderer.
    #[tokio::test]
    async fn begin_render_pass_clears_allocation_and_sets_viewport() {
        // not implemented
    }

    /// begin_render_pass() mirrors set_viewport() error semantics for stale regions.
    /// - After dropping the atlas Arc, calling begin_render_pass() should return Err(AtlasGone).
    /// - After recover(), calling begin_render_pass() with the pre-recover region should return Err(TextureNotFoundInAtlas).
    #[tokio::test]
    async fn begin_render_pass_errors_when_atlas_missing_or_region_unmapped() {
        // not implemented
    }

    // ---------------------------
    // Device loss recoverability
    // ---------------------------

    /// DeviceLossRecoverable::recover():
    /// - Rebuilds texture and views with same size/format,
    /// - Resets allocators/maps (usage==0, no locations),
    /// - Resets viewport_clear,
    /// - Pre-existing AtlasRegion handles observe TextureNotFoundInAtlas on location queries.
    #[tokio::test]
    async fn recover_reinitializes_resources_and_resets_state() {
        // not implemented
    }

    /// After recover(), GPU resources are freshly recreated.
    /// - Capture Arc::as_ptr() for texture()/texture_view()/layer_texture_view(0) before recover().
    /// - After recover(), ensure new pointers differ, max_allocation_size() resets to [0,0],
    ///   and viewport_clear.reset() observable state (e.g., pending clears) is cleared.
    #[tokio::test]
    async fn recover_recreates_gpu_resources_and_resets_caches() {
        // not implemented
    }

    // ---------------------------
    // Layer views consistency
    // ---------------------------

    /// layer_texture_view(i) provides a per-layer D2 view for each page (array layer).
    /// - After growth, verify the indexable range matches depth_or_array_layers.
    #[tokio::test]
    async fn layer_texture_view_index_range_matches_page_count() {
        // not implemented
    }

    /// allocate() should guard against arithmetic overflow when applying margins.
    /// - Request a size close to u32::MAX or large margins so that requested + 2*margin exceeds u32,
    ///   expecting AllocationFailedInvalidSize instead of panicking or wrapping.
    #[tokio::test]
    async fn allocate_rejects_overflow_when_applying_margins() {
        // not implemented
    }
}
