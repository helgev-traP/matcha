use matcha_core::{
    context::WidgetContext,
    renderer::{RendererMap, RendererSetup},
    vertex::{self, BoxDescriptor, BoxMesh, ColorVertex, box_mesh},
};

// todo: more documentation

// MARK: DOM

pub struct SolidBox {
    label: Option<String>,

    // box settings
    radius: f32,

    // background settings
    background_color: [f32; 4],
    background_blur: f32,

    // border settings
    border_width: f32,
    border_color: [f32; 4],
    border_blur: f32,

    // render context
    resources: Option<RenderResource>,
}

struct RenderResource {
    vertices: Option<Vec<ColorVertex>>,
    rect_indices: Option<Vec<u16>>,
    border_indices: Option<Vec<u16>>,
}

impl SolidBox {
    pub fn new(label: Option<&str>) -> Box<Self> {
        Box::new(Self {
            label: label.map(|s| s.to_string()),
            radius: 0.0,
            background_color: [0.0, 0.0, 0.0, 0.0],
            background_blur: 0.0,
            border_width: 0.0,
            border_color: [0.0, 0.0, 0.0, 0.0],
            border_blur: 0.0,
            resources: None,
        })
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn background_color(mut self, color: [f32; 4]) -> Self {
        self.background_color = color;
        self
    }

    pub fn background_blur(mut self, blur: f32) -> Self {
        self.background_blur = blur;
        self
    }

    pub fn border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    pub fn border_color(mut self, color: [f32; 4]) -> Self {
        self.border_color = color;
        self
    }

    pub fn border_blur(mut self, blur: f32) -> Self {
        self.border_blur = blur;
        self
    }
}

impl SolidBox {
    pub fn render(&mut self, size: [f32; 2], target: wgpu::TextureView, renderer: &RendererMap) {
        let resource = self.resources.get_or_insert_with(|| {
            // make vertices and indices
            let box_desc = BoxDescriptor::new(size[0], size[1], self.border_width).unwrap();

            match box_mesh(&box_desc) {
                Some(BoxMesh {
                    vertices,
                    rect_indices,
                    border_indices,
                }) => {
                    // let vertices = vertices
                    //     .into_iter()
                    //     .map(|v| ColorVertex {
                    //         position: v.position,
                    //         color: self.background_color,
                    //     })
                    //     .collect::<Vec<_>>();

                    // RenderResource {
                    //     vertices: Some(vertices),
                    //     rect_indices: Some(rect_indices),
                    //     border_indices: Some(border_indices),
                    // }

                    todo!()
                }
                None => todo!(),
            }
        });
    }
}
