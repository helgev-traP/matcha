// paint the widget with a blur of the background.

// --- bindings ---

// background texture
@group(0) @binding(0)
var bg_texture : texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;

// viewport size and background texture position
@group(1) @binding(0)
var<uniform> viewport_info: ViewportInfo;

struct ViewportInfo {
    size: vec2<f32>,
    bg_x_range: vec2<f32>,
    bg_y_range: vec2<f32>,
};

// blur settings
@group(2) @binding(0)
var<uniform> settings: BlurSettings;

struct BlurSettings {
    alpha: f32,
    gauss_sigma: f32,
    kernel_size: i32,
}

// --- Vertex Input ---

struct Vertex {
    @location(0) position: vec4<f32>
};

// --- Vertex Shader ---

@vertex
fn vs_main(input: Vertex) -> @builtin(position) vec4<f32> {
    // transform the vertex position to clip space
    return input.position * 2.0 / vec4<f32>(viewport_info.size, 1.0, 1.0) + vec4<f32>(-1.0, 1.0, 0.0, 0.0);
}

// --- Fragment Shader ---

// gaussian weight
fn gaussian_weight(x: f32, y: f32, sigma: f32) -> f32 {
    let sigma2 = sigma * sigma;
    return exp(- (x * x + y * y) / (2.0 * sigma2)) / (2.0 * 3.14159265359 * sigma2);
}

@fragment
fn fs_main(@builtin(position) px_position: vec4<f32>) -> @location(0) vec4<f32> {
    var sum: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var weight_sum: f32 = 0.0;

    // calculate the sampling position of the background texture
    var bg_tex_center = vec2<f32>(
        viewport_info.bg_x_range[0] * (viewport_info.size.x - px_position.x) / viewport_info.size.x + viewport_info.bg_x_range[1] * px_position.x / viewport_info.size.x,
        viewport_info.bg_y_range[0] * (viewport_info.size.y - px_position.y) / viewport_info.size.y + viewport_info.bg_y_range[1] * px_position.y / viewport_info.size.y,
    );

    // todo: this may be better to be calculated at cpu side...?
    var kernel_increment_size = vec2<f32>(
        (viewport_info.bg_x_range[1] - viewport_info.bg_x_range[0]) / viewport_info.size.x,
        (viewport_info.bg_y_range[1] - viewport_info.bg_y_range[0]) / viewport_info.size.y
    );

    // kernel
    for (var x: i32 = -settings.kernel_size; x <= settings.kernel_size; x = x + 1) {
        for (var y: i32 = -settings.kernel_size; y <= settings.kernel_size; y = y + 1) {
            var weight: f32 = gaussian_weight(f32(x), f32(y), settings.gauss_sigma);

            sum = sum + textureSample(
                bg_texture,
                texture_sampler,
                vec2<f32>(
                    bg_tex_center.x + f32(x) * kernel_increment_size.x,
                    bg_tex_center.y + f32(y) * kernel_increment_size.y,
                )
            ) * weight;
            weight_sum += weight;
        }
    }

    return (sum / weight_sum) * vec4<f32>(1.0, 1.0, 1.0, settings.alpha);
}
