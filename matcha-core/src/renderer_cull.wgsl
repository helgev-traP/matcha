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

@group(0) @binding(0) var<storage, read> all_instances: array<InstanceData>;
@group(0) @binding(1) var<storage, read> all_stencils: array<StencilData>;
@group(0) @binding(2) var<storage, read_write> visible_instances: array<InstanceData>;
@group(0) @binding(3) var<storage, read_write> visible_instance_count: atomic<u32>;

var<push_constant> normalize_matrix: mat4x4<f32>;

const QUAD_VERTICES = array<vec2<f32>, 4>(
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 1.0),
);

@compute @workgroup_size(64)
fn culling_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let instance_index = global_id.x;
    let num_instances = arrayLength(&all_instances);

    if (instance_index >= num_instances) {
        return;
    }

    let instance = all_instances[instance_index];

    let p1 = normalize_matrix * instance.viewport_position * vec4<f32>(QUAD_VERTICES[0], 0.0, 1.0);
    let p2 = normalize_matrix * instance.viewport_position * vec4<f32>(QUAD_VERTICES[1], 0.0, 1.0);
    let p3 = normalize_matrix * instance.viewport_position * vec4<f32>(QUAD_VERTICES[2], 0.0, 1.0);
    let p4 = normalize_matrix * instance.viewport_position * vec4<f32>(QUAD_VERTICES[3], 0.0, 1.0);

    let min_x = min(min(p1.x, p2.x), min(p3.x, p4.x));
    let max_x = max(max(p1.x, p2.x), max(p3.x, p4.x));
    let min_y = min(min(p1.y, p2.y), min(p3.y, p4.y));
    let max_y = max(max(p1.y, p2.y), max(p3.y, p4.y));

    if (max_x < -1.0 || min_x > 1.0 || max_y < -1.0 || min_y > 1.0) {
        return;
    }

    let visible_index = atomicAdd(&visible_instance_count, 1u);
    visible_instances[visible_index] = instance;
}
