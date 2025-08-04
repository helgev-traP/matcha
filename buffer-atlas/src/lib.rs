use std::{
    collections::VecDeque,
    sync::{Arc, Weak},
};

use parking_lot::Mutex;

pub struct Buffer<const N: usize> {
    data: Arc<BufferData<N>>,
}

pub struct BufferData<const N: usize> {
    atlas_id: BufferAtlasId,
    /// data.is_some() means the buffer needs to be uploaded to the GPU.
    /// Only change this to `None` when BufferAtlas::flash() is called.
    data: Mutex<Option<[u8; N]>>,
}

impl<const N: usize> BufferData<N> {
    fn new(atlas_id: BufferAtlasId, data: [u8; N]) -> Arc<Self> {
        Arc::new(Self {
            atlas_id,
            data: Mutex::new(Some(data)),
        })
    }

    fn is_updated(&self) -> bool {
        self.data.lock().is_some()
    }
}

static ATLAS_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferAtlasId {
    id: usize,
}

impl Default for BufferAtlasId {
    fn default() -> Self {
        Self::new()
    }
}

impl BufferAtlasId {
    pub fn new() -> Self {
        let id = ATLAS_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        BufferAtlasId { id }
    }
}

pub struct BufferAtlas<const N: usize> {
    id: BufferAtlasId,

    // N * allocations.len() == {size of the buffer_atlas in bytes}

    // the actual GPU buffer that holds all data
    atlas: Option<wgpu::Buffer>,
    // allocations capacity always matches the slots in the buffer_atlas
    // vec_item.upgrade().is_none() means the slot is empty
    allocations: Vec<Weak<BufferData<N>>>,

    // buffers that are to be allocated in the next flash() call
    // vec_item.upgrade().is_none() means the buffer dropped before flash and not needed
    to_be_allocated: Vec<Weak<BufferData<N>>>,
}

impl<const N: usize> Default for BufferAtlas<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> BufferAtlas<N> {
    pub fn new() -> Self {
        Self {
            id: BufferAtlasId::new(),
            atlas: None,
            allocations: Vec::new(),
            to_be_allocated: Vec::new(),
        }
    }

    pub fn allocate(&mut self, data: [u8; N]) -> Buffer<N> {
        let buffer = BufferData::new(self.id, data);
        self.to_be_allocated.push(Arc::downgrade(&buffer));
        Buffer { data: buffer }
    }

    pub fn flash(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        // find empty slots from dropped buffers
        let mut empty_slots: VecDeque<usize> = self
            .allocations
            .iter()
            .enumerate()
            .filter_map(|(i, weak)| {
                if weak.upgrade().is_none() {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        // filter `self.to_be_allocated` to only include buffers that are still valid
        self.to_be_allocated.retain(|weak| weak.upgrade().is_some());

        // calculate if we need to resize the atlas
        let empty_slots_count = empty_slots.len();
        let needed_slots = self.to_be_allocated.len();

        if needed_slots > empty_slots_count {
            let additional_slots = needed_slots - empty_slots_count;
            let current_capacity = self.allocations.len();
            let needed_capacity = current_capacity + additional_slots;
            let new_capacity = needed_capacity.next_power_of_two();
            Self::resize(
                device,
                queue,
                &mut self.atlas,
                &mut self.allocations,
                &mut empty_slots,
                new_capacity,
            );
        }

        // move out all buffers which are to be allocated and store in `self.allocations`
        for new_item in std::mem::take(&mut self.to_be_allocated)
            .into_iter()
            .filter_map(|weak| weak.upgrade())
        {
            // get the index of the first empty slot
            let index = empty_slots
                .pop_front()
                .expect("We checked there is enough space in the atlas");

            // store the new item in the allocations vector
            // this will never panic because index is guaranteed to be in the range of allocations
            self.allocations[index] = Arc::downgrade(&new_item);
        }

        // At this point, self.to_be_allocated is empty.
        // Buffers which BufferData::is_updated() need to be uploaded to the GPU.
        // If some buffers which are need to be uploaded lines sequentially, we can connect its data and upload them in one go.
        // Search `self.allocations` from the beginning.
        let mut i = 0;
        while i < self.allocations.len() {
            let Some(buffer_data) = self.allocations[i].upgrade() else {
                i += 1;
                continue;
            };

            if !buffer_data.is_updated() {
                i += 1;
                continue;
            }

            let start_index = i;
            let mut end_index = i;

            // Find end of the dirty chunk
            while end_index + 1 < self.allocations.len() {
                let Some(next_buffer_data) = self.allocations[end_index + 1].upgrade() else {
                    break;
                };
                if !next_buffer_data.is_updated() {
                    break;
                }
                end_index += 1;
            }

            // We have a dirty chunk from start_index to end_index.
            // Collect data and upload.
            let chunk_len = end_index - start_index + 1;
            let mut combined_data = Vec::with_capacity(chunk_len * N);
            for j in start_index..=end_index {
                if let Some(b_data) = self.allocations[j].upgrade() {
                    let mut data_opt = b_data.data.lock();
                    if let Some(data) = data_opt.take() {
                        combined_data.extend_from_slice(&data);
                    } else {
                        // This case should ideally not be hit if is_updated was true,
                        // but as a fallback, we must push data to not mess up alignment.
                        combined_data.extend_from_slice(&[0u8; N]);
                    }
                }
            }

            // Upload
            if let Some(atlas_buffer) = &self.atlas {
                queue.write_buffer(
                    atlas_buffer,
                    (start_index * N) as wgpu::BufferAddress,
                    &combined_data,
                );
            }

            i = end_index + 1;
        }
    }
}

// helper methods
impl<const N: usize> BufferAtlas<N> {
    fn resize(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &mut Option<wgpu::Buffer>,
        allocations: &mut Vec<Weak<BufferData<N>>>,
        empty_slots: &mut VecDeque<usize>,
        new_size: usize,
    ) {
        let old_size = allocations.len();
        if new_size <= old_size {
            return;
        }

        let new_buffer_size = (N * new_size) as wgpu::BufferAddress;

        let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("buffer-atlas buffer"),
            size: new_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        if let Some(old_buffer) = atlas.take() {
            let old_buffer_size = (N * old_size) as wgpu::BufferAddress;
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("buffer-atlas resize encoder"),
            });
            encoder.copy_buffer_to_buffer(&old_buffer, 0, &new_buffer, 0, old_buffer_size);
            queue.submit(std::iter::once(encoder.finish()));
        }

        *atlas = Some(new_buffer);
        allocations.resize_with(new_size, Weak::new);
        empty_slots.extend(old_size..new_size);
    }
}
