use nalgebra as na;

pub fn init_2d() -> na::Matrix3<f32> {
    na::Matrix3::new(1.0, 0.0, 0.0,
                    0.0, 1.0, 0.0,
                    0.0, 0.0, 1.0,)
}

pub fn translate_2d(x: f32, y: f32) -> na::Matrix3<f32> {
    na::Matrix3::new(1.0, 0.0, x,
                    0.0, 1.0, y,
                    0.0, 0.0, 1.0,)
}

pub fn scale_2d(x: f32, y: f32) -> na::Matrix3<f32> {
    na::Matrix3::new(x, 0.0, 0.0,
                    0.0, y, 0.0,
                    0.0, 0.0, 1.0,)
}

pub fn rotate_2d(angle: f32) -> na::Matrix3<f32> {
    let (s, c) = angle.sin_cos();
    na::Matrix3::new(c, -s, 0.0,
                    s, c, 0.0,
                    0.0, 0.0, 1.0,)
}

pub fn shear_2d(x: f32, y: f32) -> na::Matrix3<f32> {
    na::Matrix3::new(1.0, x, 0.0,
                    y, 1.0, 0.0,
                    0.0, 0.0, 1.0,)
}

pub fn reflect_by_x() -> na::Matrix3<f32> {
    na::Matrix3::new(1.0, 0.0, 0.0,
                    0.0, -1.0, 0.0,
                    0.0, 0.0, 1.0,)
}

pub fn reflect_by_y() -> na::Matrix3<f32> {
    na::Matrix3::new(-1.0, 0.0, 0.0,
                    0.0, 1.0, 0.0,
                    0.0, 0.0, 1.0,)
}