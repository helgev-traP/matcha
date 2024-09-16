struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> position: vec2<f32>;
@group(0) @binding(1)
var<uniform> panel_size: vec2<f32>;


@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var position = vec4<f32>(
        model.position.x + position.x,
        model.position.y + position.y,
        model.position.z,
        1.0,
    );
    position.x = (position.x * 2.0 / panel_size[0]) - 1.0;
    position.y = (position.y * 2.0 / panel_size[1]) + 1.0;

    let out: VertexOutput = VertexOutput(position, model.tex_coords);
    return out;
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}