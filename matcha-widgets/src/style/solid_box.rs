use matcha_core::{
    render_node::RenderNode,
    types::range::Range2D,
    ui::{Style, WidgetContext},
};

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
        _render_pass: &mut wgpu::RenderPass<'_>,
        _target_size: [u32; 2],
        _target_format: wgpu::TextureFormat,
        _boundary_size: [f32; 2],
        _offset: [f32; 2],
        _ctx: &WidgetContext,
    ) {
        // This is where the actual drawing logic using wgpu would go.
        // For now, it's a placeholder. A real implementation would create a render pipeline
        // and draw a colored rectangle.
    }
}
