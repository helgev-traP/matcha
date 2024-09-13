struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};
@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    let position = vec4<f32>(model.position, 1.0);
    return VertexOutput(position, model.color);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}