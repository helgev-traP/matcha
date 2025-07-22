pub mod atlas;
pub use atlas::{Texture, TextureAtlas, TextureAtlasError, TextureAtlasId, TextureError};
pub mod manager;
pub use manager::{AtlasManager, AtlasManagerError, MemoryAllocateStrategy};
