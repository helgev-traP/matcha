pub mod blur_renderer;
pub mod texture_renderer;

// MARK: BufferData

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewportInfo {
    // common for both renderer
    pub size: [f32; 2],
    // only blur renderer uses these
    pub bg_x_range: [f32; 2],
    pub bg_y_range: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Settings {
    // common for both renderer
    pub alpha: f32,
    // only blur renderer uses these
    pub gauss_sigma: f32,
    pub kernel_size: i32,
}
