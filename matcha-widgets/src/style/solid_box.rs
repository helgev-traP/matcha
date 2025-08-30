use matcha_core::{
    types::{color::Color, range::Range2D},
    ui::{Style, WidgetContext},
};
use renderer::{
    vertex::colored_vertex::ColorVertex,
    widgets_renderer::vertex_color::{RenderData, TargetData, VertexColor},
};
use gpu_utils::texture_atlas::atlas_simple::atlas::AtlasRegion;

// todo: more documentation

// MARK: Style

pub struct SolidBox {
    pub color: [f32; 4],
}

impl Style for SolidBox {
    fn clone_boxed(&self) -> Box<dyn Style> {
        Box::new(Self { color: self.color })
    }

    fn is_inside(&self, position: [f32; 2], boundary_size: [f32; 2], _ctx: &WidgetContext) -> bool {
        position[0] >= 0.0
            && position[0] <= boundary_size[0]
            && position[1] >= 0.0
            && position[1] <= boundary_size[1]
    }

    fn draw_range(&self, boundary_size: [f32; 2], _ctx: &WidgetContext) -> Range2D<f32> {
        Range2D::new([0.0, boundary_size[0]], [0.0, boundary_size[1]])
    }

    fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &AtlasRegion,
        target_size: [u32; 2],
        target_format: wgpu::TextureFormat,
        boundary_size: [f32; 2],
        offset: [f32; 2],
        ctx: &WidgetContext,
    ) {
        let renderer = ctx.any_resource().get_or_insert_default::<VertexColor>();

        // create a render pass targeting the atlas region so implementations can use multiple passes if needed
        let mut render_pass = match target.begin_render_pass(encoder) {
            Ok(rp) => rp,
            Err(_) => return,
        };

        let vertices = [
            ColorVertex {
                position: nalgebra::Point3::new(0.0, 0.0, 0.0),
                color: self.color,
            },
            ColorVertex {
                position: nalgebra::Point3::new(boundary_size[0], 0.0, 0.0),
                color: self.color,
            },
            ColorVertex {
                position: nalgebra::Point3::new(boundary_size[0], boundary_size[1], 0.0),
                color: self.color,
            },
            ColorVertex {
                position: nalgebra::Point3::new(0.0, boundary_size[1], 0.0),
                color: self.color,
            },
        ];

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let screen_to_clip =
            nalgebra::Matrix4::new_nonuniform_scaling(&nalgebra::Vector3::new(
                2.0 / target_size[0] as f32,
                -2.0 / target_size[1] as f32,
                1.0,
            )) * nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(-1.0, 1.0, 0.0));

        let local_to_screen =
            nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(offset[0], offset[1], 0.0));

        let transform_matrix = screen_to_clip * local_to_screen;

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
