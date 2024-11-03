use nalgebra as na;
use std::{any::Any, cell::Cell};

use crate::{
    application_context::ApplicationContext,
    events::{UiEvent, UiEventResult},
    renderer::RendererCommandEncoder,
    types::size::{PxSize, Size, SizeUnit, StdSize, StdSizeUnit},
    ui::{Dom, DomComPareResult, RenderingTrait, Widget, WidgetTrait},
};

pub struct ColumnDescriptor<R> {
    pub label: Option<String>,
    pub vec: Vec<Box<dyn Dom<R>>>,
}

impl<R> Default for ColumnDescriptor<R> {
    fn default() -> Self {
        Self {
            label: None,
            vec: Vec::new(),
        }
    }
}

pub struct Column<R: 'static> {
    label: Option<String>,
    children: Vec<Box<dyn Dom<R>>>,
}

impl<R: 'static> Column<R> {
    pub fn new(disc: ColumnDescriptor<R>) -> Self {
        Self {
            label: disc.label,
            children: disc.vec,
        }
    }

    pub fn push(&mut self, child: Box<dyn Dom<R>>) {
        self.children.push(child);
    }
}

impl<R: 'static> Dom<R> for Column<R> {
    fn build_render_tree(&self) -> Box<dyn Widget<R>> {
        Box::new(ColumnRenderNode {
            label: self.label.clone(),
            redraw: true,
            children: self
                .children
                .iter()
                .map(|child| child.build_render_tree())
                .collect(),
            cache_self_size: Cell::new(None),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct ColumnRenderNode<R: 'static> {
    label: Option<String>,
    redraw: bool,
    children: Vec<Box<dyn Widget<R>>>,
    cache_self_size: Cell<Option<PxSize>>,
}

impl<R: 'static> WidgetTrait<R> for ColumnRenderNode<R> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> UiEventResult<R> {
        // todo: event handling
        UiEventResult::default()
    }

    fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> bool {
        // todo
        true
    }

    fn update_render_tree(&mut self, dom: &dyn Dom<R>) -> Result<(), ()> {
        if (*dom).type_id() != (*self).type_id() {
            Err(())
        } else {
            let dom = dom.as_any().downcast_ref::<Column<R>>().unwrap();
            // todo: differential update
            self.children.clear();
            for child in dom.children.iter() {
                self.children.push(child.build_render_tree());
            }
            Ok(())
        }
    }

    fn compare(&self, dom: &dyn Dom<R>) -> DomComPareResult {
        if let Some(_) = dom.as_any().downcast_ref::<Column<R>>() {
            // todo: calculate difference

            DomComPareResult::Different
        } else {
            DomComPareResult::Different
        }
    }
}

impl<R> RenderingTrait for ColumnRenderNode<R> {
    fn size(&self) -> Size {
        Size {
            width: SizeUnit::Content(1.0),
            height: SizeUnit::Content(1.0),
        }
    }

    fn px_size(&self, _: PxSize, context: &ApplicationContext) -> PxSize {
        let mut width: f32 = 0.0;
        let mut height_px: f32 = 0.0;
        let mut height_percent: f32 = 0.0;

        for child in &self.children {
            let child_std_size = StdSize::from_size(child.size(), context);

            match child_std_size.width {
                StdSizeUnit::Pixel(px) => width = width.max(px),
                StdSizeUnit::Percent(_) => (),
                StdSizeUnit::None => width = width.max(child.default_size().width),
            }

            match child_std_size.height {
                StdSizeUnit::Pixel(px) => height_px += px,
                StdSizeUnit::Percent(percent) => height_percent += percent,
                StdSizeUnit::None => height_px += child.default_size().height,
            }
        }

        let height = height_px / (1.0 - height_percent);

        self.cache_self_size.set(Some(PxSize { width, height }));
        PxSize { width, height }
    }

    fn default_size(&self) -> PxSize {
        PxSize {
            width: 0.0,
            height: 0.0,
        }
    }

    fn render(
        &mut self,
        parent_size: PxSize,
        affine: na::Matrix4<f32>,
        encoder: RendererCommandEncoder,
    ) {
        let current_size = self.px_size(parent_size, encoder.get_context());

        let mut accumulated_height: f32 = 0.0;
        for child in &mut self.children {
            let child_px_size = child.px_size(current_size, encoder.get_context());
            let child_affine =
                na::Matrix4::new_translation(&na::Vector3::new(0.0, -accumulated_height, 0.0))
                    * affine;
            let encoder = encoder.clone();

            child.render(child_px_size, child_affine, encoder);

            accumulated_height += child_px_size.height;
        }
    }
}
