struct InstanceData {
    viewport_position: mat4x4<f32>,
    atlas_page: u32,
    atlas_position: mat2x2<f32>,
    stencil_index: u32,
};

struct StencilData {
    viewport_position: mat4x4<f32>,
    atlas_page: u32,
    atlas_position: mat2x2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) @interpolate(flat) atlas_page: u32,
    @location(2) @interpolate(flat) stencil_index: u32,
};

@group(0) @binding(0) var texture_sampler: sampler;
@group(0) @binding(1) var texture_atlas: texture_2d_array<f32>;
@group(0) @binding(2) var stencil_atlas: texture_2d_array<f32>;

@group(1) @binding(1) var<storage, read> all_stencils: array<StencilData>;
@group(1) @binding(2) var<storage, read> visible_instances: array<InstanceData>;

var<push_constant> normalize_matrix: mat4x4<f32>;

const VERTICES = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 1.0),
);

@vertex
fn vertex_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32
) -> VertexOutput {
    let instance = visible_instances[instance_index];
    let vertex = VERTICES[vertex_index];

    var out: VertexOutput;
    out.clip_position = normalize_matrix * instance.viewport_position * vec4<f32>(vertex, 0.0, 1.0);
    out.tex_coords = mix(
        instance.atlas_position[0],
        instance.atlas_position[1],
        vertex
    );
    out.atlas_page = instance.atlas_page;
    out.stencil_index = instance.stencil_index;
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.stencil_index > 0u) {
        let stencil = all_stencils[in.stencil_index - 1u];
        
        // calculate stencil texture coordinates
        let p = inverse(normalize_matrix * stencil.viewport_position) * in.clip_position;
        let stencil_coords = mix(
            stencil.atlas_position[0],
            stencil.atlas_position[1],
            p.xy
        );

        let stencil_alpha = textureSample(stencil_atlas, texture_sampler, stencil_coords, stencil.atlas_page).a;
        if (stencil_alpha < 0.5) {
            discard;
        }
    }

    return textureSample(texture_atlas, texture_sampler, in.tex_coords, in.atlas_page);
}
