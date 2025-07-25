struct InstanceData {
    viewport_position: mat4x4<f32>,
    atlas_page: u32,
    in_atlas_offset: vec2<f32>,
    in_atlas_size: vec2<f32>,
    stencil_index: u32,
};

struct StencilData {
    viewport_position: mat4x4<f32>,
    viewport_position_inverse_exists: u32,
    viewport_position_inverse: mat4x4<f32>,
    atlas_page: u32,
    in_atlas_offset: vec2<f32>,
    in_atlas_size: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    // texture
    @location(0) texture_uv: vec2<f32>,
    @location(1) texture_atlas_page: u32,
    @location(2) texture_atlas_bounds_x: vec2<f32>,
    @location(3) texture_atlas_bounds_y: vec2<f32>,
    // stencil
    @location(4) use_stencil: u32,
    @location(5) stencil_uv: vec2<f32>,
    @location(6) stencil_atlas_page: u32,
    @location(7) stencil_atlas_bounds_x: vec2<f32>,
    @location(8) stencil_atlas_bounds_y: vec2<f32>,
};

@group(0) @binding(0) var texture_sampler: sampler;
@group(0) @binding(1) var texture_atlas: texture_2d_array<f32>;
@group(0) @binding(2) var stencil_atlas: texture_2d_array<f32>; // R channel only be used.

@group(1) @binding(0) var<storage, read> all_instances: array<InstanceData>;
@group(1) @binding(1) var<storage, read> all_stencils: array<StencilData>;
@group(1) @binding(2) var<storage, read> visible_instances: array<u32>;

var<push_constant> normalize_matrix: mat4x4<f32>;

// vertices (y-axis is up):
// 0 - 2
// | / |
// 1 - 3
const VERTICES = array<vec4<f32>, 4>(
    vec4<f32>(0.0, 0.0, 0.0, 1.0),
    vec4<f32>(0.0,-1.0, 0.0, 1.0),
    vec4<f32>(1.0, 0.0, 0.0, 1.0),
    vec4<f32>(1.0,-1.0, 0.0, 1.0),
);
// vertices (y-axis is down):
// 0 - 3
// |   |
// 1 - 2
const UVS = array<vec2<f32>, 4>(
    vec2<f32>(0.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(1.0, 1.0),
);

@vertex
fn vertex_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32
) -> VertexOutput {
    // preparation
    let all_instance_index = visible_instances[instance_index];
    let instance = all_instances[all_instance_index];
    let stencil_index_add_1 = instance.stencil_index;
    let use_stencil = stencil_index_add_1 > 0u;
    let stencil_index = max(stencil_index_add_1 - 1u, 0u);
    let stencil = all_stencils[stencil_index];

    // vertex position
    let vertex_position = normalize_matrix * instance.viewport_position * VERTICES[vertex_index];
    let texture_uv = instance.in_atlas_offset + instance.in_atlas_size * UVS[vertex_index];

    // stencil uv
    // space that stencil position becomes {(0, 0), (0, -1), (1, -1), (1, 0)}
    let stencil_space_vertex_position_y_up = stencil.viewport_position_inverse * vertex_position;
    let stencil_space_vertex_position_y_down = stencil_space_vertex_position_y_up * vec4<f32>(1.0, -1.0, 1.0, 1.0);
    let stencil_uv = stencil_space_vertex_position_y_down.xy / stencil_space_vertex_position_y_down.w;

    // output
    var output: VertexOutput;
    output.position = vertex_position;
    output.texture_uv = texture_uv;
    output.texture_atlas_page = instance.atlas_page;
    output.texture_atlas_bounds_x = vec2<f32>(instance.in_atlas_offset.x, instance.in_atlas_offset.x + instance.in_atlas_size.x);
    output.texture_atlas_bounds_y = vec2<f32>(instance.in_atlas_offset.y, instance.in_atlas_offset.y + instance.in_atlas_size.y);
    output.use_stencil = select(
        /*false*/0u,
        /*true*/ 1u,
        use_stencil && (stencil.viewport_position_inverse_exists != 0u)
    );
    output.stencil_uv = stencil_uv;
    output.stencil_atlas_page = stencil.atlas_page;
    output.stencil_atlas_bounds_x = vec2<f32>(stencil.in_atlas_offset.x, stencil.in_atlas_offset.x + stencil.in_atlas_size.x);
    output.stencil_atlas_bounds_y = vec2<f32>(stencil.in_atlas_offset.y, stencil.in_atlas_offset.y + stencil.in_atlas_size.y);
    return output;
}

@fragment
fn fragment_main(
    @location(0) texture_uv: vec2<f32>,
    @location(1) texture_atlas_page: u32,
    @location(2) texture_atlas_bounds_x: vec2<f32>,
    @location(3) texture_atlas_bounds_y: vec2<f32>,
    @location(4) use_stencil_num: u32,
    @location(5) stencil_uv: vec2<f32>,
    @location(6) stencil_atlas_page: u32,
    @location(7) stencil_atlas_bounds_x: vec2<f32>,
    @location(8) stencil_atlas_bounds_y: vec2<f32>
) -> @location(0) vec4<f32> {
    let use_stencil = use_stencil_num != 0u;

    // clump texture_uv and stencil_uv to the texture atlas bounds
    let clamped_texture_uv = vec2<f32>(
        clamp(texture_uv.x, texture_atlas_bounds_x[0], texture_atlas_bounds_x[1]),
        clamp(texture_uv.y, texture_atlas_bounds_y[0], texture_atlas_bounds_y[1])
    );

    let clamped_stencil_uv = vec2<f32>(
        clamp(stencil_uv.x, stencil_atlas_bounds_x[0], stencil_atlas_bounds_x[1]),
        clamp(stencil_uv.y, stencil_atlas_bounds_y[0], stencil_atlas_bounds_y[1])
    );

    let texture_color = textureSample(
        texture_atlas,
        texture_sampler,
        clamped_texture_uv,
        texture_atlas_page,
    );

    let stencil_color = textureSample(
        stencil_atlas,
        texture_sampler,
        clamped_stencil_uv,
        stencil_atlas_page,
    );

    let stencil = select(
        /*false*/ 0.0,
        /*true*/  stencil_color.r,
        use_stencil,
    );

    let final_color = texture_color * stencil;

    return final_color;
}
