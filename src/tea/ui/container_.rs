use super::{EventHandle, RenderObject, Style, UiRendering};

pub struct ContainerItem {
    pub element: Box<dyn UiRendering>,
    pub position: [f32; 2],
}

pub enum Layout {
    Row,
    Column,
    Grid,
}

pub struct Container {
    pub items: Vec<ContainerItem>,
    pub layout: Layout,
}

impl Container {
}

impl UiRendering for Container {
    fn render_object(&self) -> Result<RenderObject, ()> {
        todo!()
    }
}

impl EventHandle for Container {}

impl Style for Container {}
