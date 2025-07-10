// not be used
pub mod single_thread;
// simple implementation of a texture atlas.
pub mod atlas_non_runtime;
// atlas with runtime witch automatically resizes the atlas(did not complete yet).
pub mod atlas_with_runtime;

pub use atlas_non_runtime::{
    AtlasManager, AtlasManagerError, MemoryAllocateStrategy, Texture, TextureAtlas,
    TextureAtlasError, TextureError,
};

// re-exports
pub use guillotiere::euclid;
