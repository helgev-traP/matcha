struct InstanceData {
    viewport_position: mat4x4<f32>,
    atlas_page: u32,
    _padding1: u32,
    in_atlas_offset: vec2<f32>,
    in_atlas_size: vec2<f32>,
    stencil_index: u32,
    _padding2: u32,
};

struct StencilData {
    viewport_position: mat4x4<f32>,
    viewport_position_inverse_exists: u32,
    _padding1: array<u32, 3>,
    viewport_position_inverse: mat4x4<f32>,
    atlas_page: u32,
    _padding2: u32,
    in_atlas_offset: vec2<f32>,
    in_atlas_size: vec2<f32>,
    _padding3: array<u32, 2>,
};

@group(0) @binding(0) var<storage, read> all_instances: array<InstanceData>;
@group(0) @binding(1) var<storage, read> all_stencils: array<StencilData>;
@group(0) @binding(2) var<storage, read_write> visible_instances: array<u32>;
@group(0) @binding(3) var<storage, read_write> visible_instance_count: atomic<u32>;

var<push_constant> normalize_matrix: mat4x4<f32>;

// vertices:
// 0 - 3
// |   |
// 1 - 2
const QUAD_VERTICES = array<vec4<f32>, 4>(
    vec4<f32>(0.0, 0.0, 0.0, 1.0),
    vec4<f32>(0.0,-1.0, 0.0, 1.0),
    vec4<f32>(1.0,-1.0, 0.0, 1.0),
    vec4<f32>(1.0, 0.0, 0.0, 1.0),
);

const CLIP_VERTICES = array<vec4<f32>, 4>(
    vec4<f32>(-1.0,  1.0, 0.0, 1.0),
    vec4<f32>(-1.0, -1.0, 0.0, 1.0),
    vec4<f32>( 1.0, -1.0, 0.0, 1.0),
    vec4<f32>( 1.0,  1.0, 0.0, 1.0),
);

@compute @workgroup_size(64)
fn culling_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let instance_index = global_id.x;
    let instance = all_instances[instance_index];

    let stencil_index_add_1 = instance.stencil_index;
    let use_stencil = stencil_index_add_1 > 0u;
    let stencil_index = max(stencil_index_add_1 - 1u, 0u);
    let stencil = all_stencils[stencil_index];

    // Visible conditions:
    // 1. instance is within the viewport
    // 2. (no stencil) or (stencil is within the viewport)
    // 3. instance's polygon and stencil's polygon have overlap

    var texture_position: array<vec4<f32>, 4>;
    for (var i = 0u; i < 4u; i++) {
        texture_position[i] = normalize_matrix * instance.viewport_position * QUAD_VERTICES[i];
    }

    var stencil_position: array<vec4<f32>, 4>;
    for (var i = 0u; i < 4u; i++) {
        stencil_position[i] = normalize_matrix * stencil.viewport_position * QUAD_VERTICES[i];
    }

    let texture_is_in_viewport = is_overlapping(texture_position, CLIP_VERTICES);
    let stencil_is_in_viewport = use_stencil && is_overlapping(stencil_position, CLIP_VERTICES);
    let polygons_overlap = use_stencil && is_overlapping(texture_position, stencil_position);

    let is_visible = texture_is_in_viewport && (!use_stencil || stencil_is_in_viewport) && polygons_overlap;

    if (is_visible) {
        let visible_count = atomicAdd(&visible_instance_count, 1u);
        visible_instances[visible_count] = instance_index;
    }
}

fn is_overlapping(
    a: array<vec4<f32>, 4>,
    b: array<vec4<f32>, 4>
) -> bool {
    var flag = false;
    for (var i = 0u; i < 4u; i++) {
        flag = flag || point_in_polygon(a[i], b);
    }
    for (var i = 0u; i < 4u; i++) {
        flag = flag || point_in_polygon(b[i], a);
    }
    return flag;
}

fn cross_2d(a: vec2<f32>, b: vec2<f32>) -> f32 {
    return a.x * b.y - a.y * b.x;
}

fn point_in_polygon(
    point: vec4<f32>,
    polygon: array<vec4<f32>, 4>
) -> bool {
    // use cross product to determine if the point is inside the polygon
    let points = array<vec2<f32>, 4>(
        polygon[0].xy - point.xy,
        polygon[1].xy - point.xy,
        polygon[2].xy - point.xy,
        polygon[3].xy - point.xy,
    );
    let lines = array<vec2<f32>, 4>(
        polygon[1].xy - polygon[0].xy,
        polygon[2].xy - polygon[1].xy,
        polygon[3].xy - polygon[2].xy,
        polygon[0].xy - polygon[3].xy,
    );

    let signs = array<bool, 4>(
        cross_2d(points[0], lines[0]) > 0.0,
        cross_2d(points[1], lines[1]) > 0.0,
        cross_2d(points[2], lines[2]) > 0.0,
        cross_2d(points[3], lines[3]) > 0.0,
    );

    return signs[0] == signs[1] && signs[1] == signs[2] && signs[2] == signs[3];
}
