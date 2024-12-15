// textures
@group(0) @binding(0)
var source: texture_storage_2d<f32>;
@group(1) @binding(0)
var destination: texture_storage_2d<f32>;
@group(2) @binding(0)
var stencil: texture_storage_2d<f32>;

// offset
@group(3) @binding(0)
var offset: vec2<u32>;

// shader
@compute @workgroup_size(8,8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let is_within_bounds = all(global_id.xy < textureDimensions(destination).xy) && all(global_id.xy >= offset);
    textureStore(
        destination,
        vec2<i32>(global_id.xy) + vec2<i32>(offset),
        textureLoad(source, vec2<i32>(global_id.xy)) * textureLoad(stencil, vec2<i32>(global_id.xy).r)
    );
}