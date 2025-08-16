#[derive(Clone)]
pub struct RenderNode {
    pub texture_and_position: Option<(texture_atlas::AtlasRegion, nalgebra::Matrix4<f32>)>,
    pub stencil_and_position: Option<(texture_atlas::AtlasRegion, nalgebra::Matrix4<f32>)>,

    child_elements: Vec<(RenderNode, nalgebra::Matrix4<f32>)>,
}

impl Default for RenderNode {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderNode {
    pub fn new() -> Self {
        Self {
            texture_and_position: None,
            stencil_and_position: None,
            child_elements: Vec::new(),
        }
    }

    pub fn with_texture(
        mut self,
        texture: texture_atlas::AtlasRegion,
        texture_position: nalgebra::Matrix4<f32>,
    ) -> Self {
        self.texture_and_position = Some((texture, texture_position));
        self
    }

    pub fn with_stencil(
        mut self,
        stencil: texture_atlas::AtlasRegion,
        stencil_position: nalgebra::Matrix4<f32>,
    ) -> Self {
        self.stencil_and_position = Some((stencil, stencil_position));
        self
    }

    pub fn add_child(&mut self, child: RenderNode, transform: nalgebra::Matrix4<f32>) {
        self.child_elements.push((child, transform));
    }

    pub fn child_elements(&self) -> &[(RenderNode, nalgebra::Matrix4<f32>)] {
        &self.child_elements
    }
}
