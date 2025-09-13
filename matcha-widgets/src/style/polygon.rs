use std::sync::Arc;

use crate::style::Style;
use gpu_utils::texture_atlas::atlas_simple::atlas::AtlasRegion;
use matcha_core::{color::Color, metrics::QRect, ui::WidgetContext};
use parking_lot::Mutex;
use renderer::{
    vertex::colored_vertex::ColorVertex,
    widgets_renderer::vertex_color::{RenderData, TargetData, VertexColor},
};

type PolygonFn = dyn for<'a> Fn([f32; 2], &'a WidgetContext) -> Mesh + Send + Sync + 'static;
type AdaptFn =
    dyn for<'a> Fn([f32; 2], &'a WidgetContext) -> nalgebra::Matrix4<f32> + Send + Sync + 'static;

pub struct Polygon {
    polygon: Arc<PolygonFn>,
    adaptive_affine: Arc<AdaptFn>,
    cache_the_mesh: bool,
    caches: Mutex<utils::cache::Cache<[f32; 2], Caches>>,
}

#[derive(Clone)]
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

#[derive(Clone)]
pub struct Vertex {
    position: [f32; 2],
    color: Color,
}

struct Caches {
    mesh: Mesh,
    rect: Option<QRect>,
}

// constructor

impl Clone for Polygon {
    fn clone(&self) -> Self {
        Self {
            polygon: self.polygon.clone(),
            adaptive_affine: self.adaptive_affine.clone(),
            cache_the_mesh: self.cache_the_mesh,
            caches: Mutex::new(utils::cache::Cache::default()),
        }
    }
}

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
    fn required_region(
        &self,
        constraints: &matcha_core::metrics::Constraints,
        ctx: &WidgetContext,
    ) -> Option<matcha_core::metrics::QRect> {
        let mut cache = self.caches.lock();
        let key = constraints.max_size();

        if self.cache_the_mesh {
            let (_k, v) = cache.get_or_insert_with(key, || Caches {
                mesh: (self.polygon)(key, ctx),
                rect: None,
            });

            if let Some(rect) = v.rect {
                if rect.area() > 0.0 {
                    return Some(rect);
                } else {
                    return None;
                }
            }

            // compute bounding rect from mesh
            let (min_x, min_y, max_x, max_y) = {
                let mut min_x = f32::INFINITY;
                let mut min_y = f32::INFINITY;
                let mut max_x = f32::NEG_INFINITY;
                let mut max_y = f32::NEG_INFINITY;

                match &v.mesh {
                    Mesh::TriangleStrip { vertices }
                    | Mesh::TriangleList { vertices }
                    | Mesh::TriangleFan { vertices } => {
                        for vert in vertices {
                            min_x = min_x.min(vert.position[0]);
                            min_y = min_y.min(vert.position[1]);
                            max_x = max_x.max(vert.position[0]);
                            max_y = max_y.max(vert.position[1]);
                        }
                    }
                    Mesh::TriangleIndexed { indices, vertices } => {
                        for &i in indices {
                            let vert = &vertices[i as usize];
                            min_x = min_x.min(vert.position[0]);
                            min_y = min_y.min(vert.position[1]);
                            max_x = max_x.max(vert.position[0]);
                            max_y = max_y.max(vert.position[1]);
                        }
                    }
                }
                (min_x, min_y, max_x, max_y)
            };

            let rect = if min_x.is_finite() && min_y.is_finite() && max_x > min_x && max_y > min_y {
                matcha_core::metrics::QRect::new([min_x, min_y], [max_x - min_x, max_y - min_y])
            } else {
                matcha_core::metrics::QRect::zero()
            };

            v.rect = Some(rect);
            if v.rect.unwrap().area() > 0.0 {
                Some(v.rect.unwrap())
            } else {
                None
            }
        } else {
            let mesh = (self.polygon)(key, ctx);

            let (min_x, min_y, max_x, max_y) = {
                let mut min_x = f32::INFINITY;
                let mut min_y = f32::INFINITY;
                let mut max_x = f32::NEG_INFINITY;
                let mut max_y = f32::NEG_INFINITY;

                match &mesh {
                    Mesh::TriangleStrip { vertices }
                    | Mesh::TriangleList { vertices }
                    | Mesh::TriangleFan { vertices } => {
                        for vert in vertices {
                            min_x = min_x.min(vert.position[0]);
                            min_y = min_y.min(vert.position[1]);
                            max_x = max_x.max(vert.position[0]);
                            max_y = max_y.max(vert.position[1]);
                        }
                    }
                    Mesh::TriangleIndexed { indices, vertices } => {
                        for &i in indices {
                            let vert = &vertices[i as usize];
                            min_x = min_x.min(vert.position[0]);
                            min_y = min_y.min(vert.position[1]);
                            max_x = max_x.max(vert.position[0]);
                            max_y = max_y.max(vert.position[1]);
                        }
                    }
                }
                (min_x, min_y, max_x, max_y)
            };

            let rect = if min_x.is_finite() && min_y.is_finite() && max_x > min_x && max_y > min_y {
                matcha_core::metrics::QRect::new([min_x, min_y], [max_x - min_x, max_y - min_y])
            } else {
                matcha_core::metrics::QRect::zero()
            };

            if rect.area() > 0.0 { Some(rect) } else { None }
        }
    }

    fn is_inside(&self, position: [f32; 2], boundary_size: [f32; 2], ctx: &WidgetContext) -> bool {
        let mut cache = self.caches.lock();

        let mesh = if self.cache_the_mesh {
            &cache
                .get_or_insert_with(boundary_size, || Caches {
                    mesh: (self.polygon)(boundary_size, ctx),
                    rect: None,
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

    fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &AtlasRegion,
        boundary_size: [f32; 2],
        offset: [f32; 2],
        ctx: &WidgetContext,
    ) {
        let target_size = target.size();
        let target_format = target.format();
        let mut render_pass = match target.begin_render_pass(encoder) {
            Ok(rp) => rp,
            Err(_) => return,
        };
        let mut cache = self.caches.lock();
        let mesh = if self.cache_the_mesh {
            &cache
                .get_or_insert_with(boundary_size, || Caches {
                    mesh: (self.polygon)(boundary_size, ctx),
                    rect: None,
                })
                .1
                .mesh
        } else {
            &(self.polygon)(boundary_size, ctx)
        };

        let renderer = ctx.any_resource().get_or_insert_default::<VertexColor>();

        let (vertices, indices): (Vec<ColorVertex>, Vec<u16>) = match mesh {
            Mesh::TriangleStrip { vertices } => {
                let color_vertices = vertices
                    .iter()
                    .map(|v| ColorVertex {
                        position: nalgebra::Point3::new(v.position[0], v.position[1], 0.0),
                        color: v.color.to_rgba_f32(),
                    })
                    .collect();
                let indices = (0..vertices.len() as u16 - 2)
                    .flat_map(|i| [i, i + 1, i + 2])
                    .collect();
                (color_vertices, indices)
            }
            Mesh::TriangleList { vertices } => {
                let color_vertices = vertices
                    .iter()
                    .map(|v| ColorVertex {
                        position: nalgebra::Point3::new(v.position[0], v.position[1], 0.0),
                        color: v.color.to_rgba_f32(),
                    })
                    .collect();
                let indices = (0..vertices.len() as u16).collect();
                (color_vertices, indices)
            }
            Mesh::TriangleFan { vertices } => {
                let color_vertices = vertices
                    .iter()
                    .map(|v| ColorVertex {
                        position: nalgebra::Point3::new(v.position[0], v.position[1], 0.0),
                        color: v.color.to_rgba_f32(),
                    })
                    .collect();
                let indices = (1..vertices.len() as u16 - 1)
                    .flat_map(|i| [0, i, i + 1])
                    .collect();
                (color_vertices, indices)
            }
            Mesh::TriangleIndexed { indices, vertices } => {
                let color_vertices = vertices
                    .iter()
                    .map(|v| ColorVertex {
                        position: nalgebra::Point3::new(v.position[0], v.position[1], 0.0),
                        color: v.color.to_rgba_f32(),
                    })
                    .collect();
                (color_vertices, indices.clone())
            }
        };

        if vertices.is_empty() {
            return;
        }

        let screen_to_clip =
            nalgebra::Matrix4::new_nonuniform_scaling(&nalgebra::Vector3::new(
                2.0 / target_size[0] as f32,
                -2.0 / target_size[1] as f32,
                1.0,
            )) * nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(-1.0, 1.0, 0.0));

        let local_to_screen =
            nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(offset[0], offset[1], 0.0));

        let adaptive_affine = (self.adaptive_affine)(boundary_size, ctx);

        let transform_matrix = screen_to_clip * local_to_screen * adaptive_affine;

        renderer.render(
            &mut render_pass,
            TargetData {
                target_size,
                target_format,
            },
            RenderData {
                position: offset,
                vertices: &vertices,
                indices: &indices,
            },
            ctx.device(),
        );
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
