use gpu_utils::texture_atlas;

/// Represents a render tree node that contains drawing information for the renderer.
///
/// Note: Coordinates used by the Dom/Widget/Style APIs are in pixels with the origin at the
/// top-left of the window and the Y axis pointing downwards. The renderer is responsible for
/// converting these coordinates (origin, Y direction, and scale) into normalized device
/// coordinates (NDC) required by the GPU/backend.
///
/// The RenderNode stores textures, stencil information, and child elements along with
/// per-node transform matrices. Transforms are applied by the renderer when generating GPU
/// draw calls.
#[derive(Debug)]
pub struct RenderNode {
    texture_and_position: Option<(texture_atlas::AtlasRegion, nalgebra::Matrix4<f32>)>,
    stencil_and_position: Option<(texture_atlas::AtlasRegion, nalgebra::Matrix4<f32>)>,

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

    pub(crate) fn texture(&self) -> Option<&(texture_atlas::AtlasRegion, nalgebra::Matrix4<f32>)> {
        self.texture_and_position.as_ref()
    }

    pub(crate) fn stencil(&self) -> Option<&(texture_atlas::AtlasRegion, nalgebra::Matrix4<f32>)> {
        self.stencil_and_position.as_ref()
    }

    pub(crate) fn child_elements(&self) -> &[(RenderNode, nalgebra::Matrix4<f32>)] {
        &self.child_elements
    }

    pub fn with_texture(
        mut self,
        texture: texture_atlas::AtlasRegion,
        texture_size: [f32; 2],
        texture_position: nalgebra::Matrix4<f32>,
    ) -> Self {
        let scale_matrix = nalgebra::Matrix4::new_nonuniform_scaling(&nalgebra::Vector3::new(
            texture_size[0],
            texture_size[1],
            1.0,
        ));
        self.texture_and_position = Some((texture, texture_position * scale_matrix));
        self
    }

    pub fn with_stencil(
        mut self,
        stencil: texture_atlas::AtlasRegion,
        stencil_size: [f32; 2],
        stencil_position: nalgebra::Matrix4<f32>,
    ) -> Self {
        let scale_matrix = nalgebra::Matrix4::new_nonuniform_scaling(&nalgebra::Vector3::new(
            stencil_size[0],
            stencil_size[1],
            1.0,
        ));
        self.stencil_and_position = Some((stencil, stencil_position * scale_matrix));
        self
    }

    pub fn push_child(&mut self, child: RenderNode, transform: nalgebra::Matrix4<f32>) {
        self.child_elements.push((child, transform));
    }

    pub fn add_child(mut self, child: RenderNode, transform: nalgebra::Matrix4<f32>) -> Self {
        self.child_elements.push((child, transform));
        self
    }
}

impl RenderNode {
    pub fn count(&self) -> usize {
        let mut count = 1; // Count this node
        for (child, _) in &self.child_elements {
            count += child.count();
        }
        count
    }
}
