// simple implementation of a texture atlas.
pub mod atlas_simple;
// atlas with runtime witch automatically resizes the atlas(did not complete yet).
pub mod atlas_with_runtime;

pub use atlas_simple::{
    AtlasManager, AtlasManagerError, MemoryAllocateStrategy, Texture, TextureAtlas,
    TextureAtlasError, TextureError,
};

// re-exports
pub use guillotiere::euclid;
