use std::sync::Arc;

use matcha_core::{
    context::WidgetContext,
    types::{color::Color, range::Range2D},
    ui::Style,
};
use parking_lot::Mutex;

type PolygonFn = dyn for<'a> Fn([f32; 2], &'a WidgetContext) -> Mesh + Send + Sync + 'static;
type AdaptFn =
    dyn for<'a> Fn([f32; 2], &'a WidgetContext) -> nalgebra::Matrix4<f32> + Send + Sync + 'static;

pub struct Polygon {
    polygon: Arc<PolygonFn>,
    adaptive_affine: Arc<AdaptFn>,
    cache_the_mesh: bool,
    caches: Mutex<utils::cache::Cache<[f32; 2], Caches>>,
}

pub enum Mesh {
    TriangleStrip {
        vertices: Vec<Vertex>,
    },
    TriangleList {
        vertices: Vec<Vertex>,
    },
    TriangleFan {
        vertices: Vec<Vertex>,
    },
    TriangleIndexed {
        indices: Vec<u16>,
        vertices: Vec<Vertex>,
    },
}

pub struct Vertex {
    position: [f32; 2],
    color: Color,
}

struct Caches {
    mesh: Mesh,
    draw_range: Option<Range2D<f32>>,
}

// constructor

impl Polygon {
    pub fn new<F>(polygon: F) -> Box<Self>
    where
        F: Fn([f32; 2], &WidgetContext) -> Mesh + Send + Sync + 'static,
    {
        Box::new(Self {
            polygon: Arc::new(polygon),
            adaptive_affine: Arc::new(|_, _| nalgebra::Matrix4::identity()),
            cache_the_mesh: true,
            caches: Mutex::new(utils::cache::Cache::default()),
        })
    }

    pub fn adaptive_affine<F>(mut self, affine: F) -> Self
    where
        F: Fn([f32; 2], &WidgetContext) -> nalgebra::Matrix4<f32> + Send + Sync + 'static,
    {
        self.adaptive_affine = Arc::new(affine);
        self
    }

    pub fn do_not_cache_mesh(mut self) -> Self {
        self.cache_the_mesh = false;
        self
    }
}

// MARK: Style

impl Style for Polygon {
    fn is_inside(&self, position: [f32; 2], boundary_size: [f32; 2], ctx: &WidgetContext) -> bool {
        let mut cache = self.caches.lock();

        let mesh = if self.cache_the_mesh {
            &cache
                .get_or_insert_with(boundary_size, || Caches {
                    mesh: (self.polygon)(boundary_size, ctx),
                    draw_range: None,
                })
                .1
                .mesh
        } else {
            &(self.polygon)(boundary_size, ctx)
        };

        match mesh {
            Mesh::TriangleStrip { vertices } => vertices.windows(3).any(|window| {
                let triangle = [window[0].position, window[1].position, window[2].position];
                is_inside_of_triangle(position, triangle)
            }),
            Mesh::TriangleList { vertices } => vertices.chunks(3).any(|chunk| {
                if chunk.len() == 3 {
                    let triangle = [chunk[0].position, chunk[1].position, chunk[2].position];
                    is_inside_of_triangle(position, triangle)
                } else {
                    false
                }
            }),
            Mesh::TriangleFan { vertices } => {
                if vertices.len() >= 3 {
                    let center = vertices[0].position;
                    vertices[1..].windows(2).any(|window| {
                        let triangle = [center, window[0].position, window[1].position];
                        is_inside_of_triangle(position, triangle)
                    })
                } else {
                    false
                }
            }
            Mesh::TriangleIndexed { indices, vertices } => indices.chunks(3).any(|chunk| {
                if chunk.len() == 3 {
                    let triangle = [
                        vertices[chunk[0] as usize].position,
                        vertices[chunk[1] as usize].position,
                        vertices[chunk[2] as usize].position,
                    ];
                    is_inside_of_triangle(position, triangle)
                } else {
                    false
                }
            }),
        }
    }

    fn draw_range(&self, boundary_size: [f32; 2], ctx: &WidgetContext) -> Range2D<f32> {
        let mut caches = self.caches.lock();

        let caches = caches
            .get_or_insert_with(boundary_size, || Caches {
                mesh: (self.polygon)(boundary_size, ctx),
                draw_range: None,
            })
            .1;

        *caches.draw_range.get_or_insert_with(|| {
            let mesh = &caches.mesh;
            match mesh {
                Mesh::TriangleStrip { vertices }
                | Mesh::TriangleList { vertices }
                | Mesh::TriangleFan { vertices }
                | Mesh::TriangleIndexed { vertices, .. } => {
                    let x_min_max = vertices
                        .iter()
                        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), v| {
                            (min.min(v.position[0]), max.max(v.position[0]))
                        });
                    let y_min_max = vertices
                        .iter()
                        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), v| {
                            (min.min(v.position[1]), max.max(v.position[1]))
                        });

                    Range2D::new([x_min_max.0, y_min_max.0], [x_min_max.1, y_min_max.1])
                }
            }
        })
    }

    fn draw(
        &self,
        render_pass: &mut wgpu::RenderPass<'_>,
        texture_size: [u32; 2],
        texture_format: wgpu::TextureFormat,
        boundary_size: [f32; 2],
        offset: [f32; 2],
        ctx: &WidgetContext,
    ) {
        todo!()
    }
}

fn is_inside_of_triangle(position: [f32; 2], triangle: [[f32; 2]; 3]) -> bool {
    let [a, b, c] = triangle;

    // use cross product to determine if the point is inside the triangle

    let pa = [position[0] - a[0], position[1] - a[1]];
    let pb = [position[0] - b[0], position[1] - b[1]];
    let pc = [position[0] - c[0], position[1] - c[1]];

    let cross_ab_positive = cross(pa, pb) >= 0.0;
    let cross_bc_positive = cross(pb, pc) >= 0.0;
    let cross_ca_positive = cross(pc, pa) >= 0.0;

    (cross_ab_positive && cross_bc_positive && cross_ca_positive)
        || (!cross_ab_positive && !cross_bc_positive && !cross_ca_positive)
}

fn cross(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[1] - a[1] * b[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_inside_of_triangle() {
        let triangle = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        assert!(is_inside_of_triangle([0.5, 0.5], triangle));
        assert!(is_inside_of_triangle([0.0, 0.0], triangle));
        assert!(!is_inside_of_triangle([1.5, 0.5], triangle));
        assert!(!is_inside_of_triangle([-0.5, -0.5], triangle));
    }
}
