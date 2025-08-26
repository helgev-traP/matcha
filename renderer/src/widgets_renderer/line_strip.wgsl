struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

var<push_constant> normalize_affine: mat4x4<f32>;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    // let out: VertexOutput = VertexOutput(normalize_affine * model.position, model.color);
    let out: VertexOutput = VertexOutput(normalize_affine * vec4(model.position, 1.0), model.color);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}