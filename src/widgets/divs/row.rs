use nalgebra as na;
use std::{cell::Cell, sync::Arc};

use crate::{
    context::SharedContext,
    events::UiEvent,
    renderer::Renderer,
    types::size::{Size, StdSize},
    ui::{Dom, DomComPareResult, Widget},
    vertex::uv_vertex::UvVertex,
};

pub struct RowDescriptor<R> {
    // label
    pub label: Option<String>,
    // style
    pub size: [Size; 2],
    pub margin: Margin,
    pub padding: Padding,
    pub border: Border,
    pub box_sizing: BoxSizing,
    pub visibility: Visibility,
    // todo: layout
    // items
    pub items: Vec<Box<dyn Dom<R>>>,
}

impl<R> Default for RowDescriptor<R> {
    fn default() -> Self {
        Self {
            label: None,
            size: [Size::Content(1.0), Size::Content(1.0)],
            margin: Margin {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            },
            padding: Padding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            },
            border: Border {
                px: 0.0,
                color: [0, 0, 0, 0],
                top_left_radius: 0.0,
                top_right_radius: 0.0,
                bottom_left_radius: 0.0,
                bottom_right_radius: 0.0,
            },
            box_sizing: BoxSizing::BorderBox,
            visibility: Visibility::Visible,
            items: vec![],
        }
    }
}

pub struct Row<R: 'static> {
    label: Option<String>,

    size: [Size; 2],
    margin: Margin,
    padding: Padding,
    border: Border,
    box_sizing: BoxSizing,
    visibility: Visibility,

    children: Vec<Box<dyn Dom<R>>>,
}

impl<R> Row<R> {
    pub fn new(disc: RowDescriptor<R>) -> Box<Self> {
        Box::new(Self {
            label: disc.label,
            size: disc.size,
            margin: disc.margin,
            padding: disc.padding,
            border: disc.border,
            box_sizing: disc.box_sizing,
            visibility: disc.visibility,
            children: disc.items,
        })
    }

    pub fn push(&mut self, child: Box<dyn Dom<R>>) {
        self.children.push(child);
    }
}

impl<R: Send + 'static> Dom<R> for Row<R> {
    fn build_widget_tree(&self) -> Box<dyn Widget<R>> {
        Box::new(RowRenderNode {
            label: self.label.clone(),
            size: self.size,
            margin: self.margin,
            padding: self.padding,
            border: self.border,
            box_sizing: self.box_sizing,
            visibility: self.visibility,
            redraw: true,
            children: self
                .children
                .iter()
                .map(|child| Child {
                    item: child.build_widget_tree(),
                    position: None,
                    size: None,
                })
                .collect(),
            cache_self_size: Cell::new(None),
            mouse_hovering_index: None,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct RowRenderNode<T: 'static> {
    label: Option<String>,

    size: [Size; 2],
    margin: Margin,
    padding: Padding,
    border: Border,
    box_sizing: BoxSizing,
    visibility: Visibility,

    redraw: bool,
    children: Vec<Child<T>>,
    cache_self_size: Cell<Option<[f32; 2]>>,
    mouse_hovering_index: Option<usize>,
}

struct Child<T> {
    item: Box<dyn Widget<T>>,
    // cache
    position: Option<[f32; 2]>,
    size: Option<[f32; 2]>,
}

impl<T> Widget<T> for RowRenderNode<T> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> crate::events::UiEventResult<T> {
        todo!()
    }

    fn is_inside(&self, position: [f32; 2], parent_size: [StdSize; 2], context: &SharedContext) -> bool {
        todo!()
    }

    fn update_widget_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<Row<T>>() {
            Err(())
        } else {
            let dom = dom.as_any().downcast_ref::<Row<T>>().unwrap();
            // todo: differential update
            let mut i = 0;
            loop {
                match (self.children.get_mut(i), dom.children.get(i)) {
                    (Some(child), Some(new_child)) => {
                        child.item.update_widget_tree(&**new_child)?;
                        i += 1;
                    }
                    (Some(_), None) => {
                        self.children.pop();
                    }
                    (None, Some(new_child)) => {
                        self.children.push(Child {
                            item: new_child.build_widget_tree(),
                            position: None,
                            size: None,
                        });
                        i += 1;
                    }
                    (None, None) => break,
                }
            }
            Ok(())
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(_) = dom.as_any().downcast_ref::<Row<T>>() {
            // todo: calculate difference

            DomComPareResult::Different
        } else {
            DomComPareResult::Different
        }
    }

    fn size(&self) -> [Size; 2] {
        self.size
    }

    fn px_size(&self, parent_size: [StdSize; 2], context: &SharedContext) -> [f32; 2] {
        todo!()
    }

    fn render(
        &mut self,
        // ui environment
        parent_size: [StdSize; 2],
        // context
        context: &SharedContext,
        renderer: &Renderer,
        frame: u64,
    ) -> Vec<(
        Arc<wgpu::Texture>,
        Arc<Vec<UvVertex>>,
        Arc<Vec<u16>>,
        nalgebra::Matrix4<f32>,
    )> {
        todo!()
    }
}

// style

#[derive(Debug, Clone, Copy)]
pub struct Margin {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Border {
    pub px: f32,
    pub color: [u8; 4],
    pub top_left_radius: f32,
    pub top_right_radius: f32,
    pub bottom_left_radius: f32,
    pub bottom_right_radius: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum BoxSizing {
    ContentBox,
    BorderBox,
}

#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    Visible,
    Hidden,
    None,
}