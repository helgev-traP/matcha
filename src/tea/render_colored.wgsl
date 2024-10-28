struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> affine: mat4x4<f32>;
@group(1) @binding(0)
var<uniform> color: vec4<f32>;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var model4 = vec4<f32>(
        model.position.x,
        model.position.y,
        model.position.z,
        1.0,
    );
    let out: VertexOutput = VertexOutput(affine * model4);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return color;
}