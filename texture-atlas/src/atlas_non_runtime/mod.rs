pub mod atlas;
pub use atlas::{Texture, TextureAtlas, TextureAtlasError, TextureError};
pub mod manager;
pub use manager::{AtlasManager, AtlasManagerError, MemoryAllocateStrategy};