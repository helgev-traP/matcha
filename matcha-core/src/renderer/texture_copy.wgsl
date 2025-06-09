// resource
@group(0) @binding(0)
var copy_source: texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;

struct PushConstants {
    // [width, height] of the target texture
    size: vec2<f32>,
    // [bottom-left, top-right] corners of the target texture
    position: array<vec2<f32>, 2>,
    // color transform matrix (rgba)
    color_transform: mat4x4<f32>,
};
var<push_constant> pc: PushConstants;

// vertex shader
@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32
) -> VertexOutput {
    var positions: vec2<f32>;
    var tex_coords: vec2<f32>;

    // tex_coords are y-flipped

    if vertex_index == 0 {
        // top-left corner
        positions = vec2<f32>(pc.position[0].x, pc.position[1].y);
        tex_coords = vec2<f32>(0.0, 0.0);
    } else if vertex_index == 1 {
        // bottom-left corner
        positions = vec2<f32>(pc.position[0].x, pc.position[0].y);
        tex_coords = vec2<f32>(0.0, 1.0);
    } else if vertex_index == 2 {
        // bottom-right corner
        positions = vec2<f32>(pc.position[1].x, pc.position[0].y);
        tex_coords = vec2<f32>(1.0, 1.0);
    } else {
        // top-right corner
        positions = vec2<f32>(pc.position[1].x, pc.position[1].y);
        tex_coords = vec2<f32>(1.0, 0.0);
    }

    let position = (positions * 2.0) / pc.size + vec2<f32>(-1.0, 1.0);

    return VertexOutput(
        vec4<f32>(position, 0.0, 1.0),
        tex_coords
    );
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

// fragment shader
@fragment
fn fs_main(
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
) -> @location(0) vec4<f32> {
    let color = textureSample(copy_source, texture_sampler, tex_coords);
    // Apply color transform
    return pc.color_transform * color.rgba;
}