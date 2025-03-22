// paint the widget with a texture.

// --- bindings ---

// widget texture
@group(0) @binding(0)
var widget_texture : texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;

// viewport size
@group(1) @binding(0)
var<uniform> viewport_info: ViewportInfo;

struct ViewportInfo {
    size: vec2<f32>,
    phantom_data_1: vec2<f32>,
    phantom_data_2: vec2<f32>,
};

// image settings
@group(2) @binding(0)
var<uniform> settings: Settings;

struct Settings {
    alpha: f32,
    phantom_data_1: f32,
    phantom_data_2: i32,
}

// --- Vertex Input ---

struct Vertex {
    @location(0) position: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

// --- Vertex Output ---

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// --- Vertex Shader ---

@vertex
fn vs_main(input: Vertex) -> VertexOutput {
    var output: VertexOutput;
    output.position = input.position * 2.0 / vec4<f32>(viewport_info.size, 1.0, 1.0) + vec4<f32>(-1.0, 1.0, 0.0, 0.0);
    output.uv = input.uv;
    return output;
}

// --- Fragment Shader ---

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(widget_texture, texture_sampler, input.uv) * vec4<f32>(1.0, 1.0, 1.0, settings.alpha);
}

