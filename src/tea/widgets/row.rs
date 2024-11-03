use nalgebra as na;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    application_context::ApplicationContext,
    events::{UiEvent, UiEventResult},
    renderer::RendererCommandEncoder,
    types::size::{PxSize, Size, SizeUnit, StdSize},
    ui::{Dom, DomComPareResult, RenderingTrait, Widget, WidgetTrait},
};

pub struct RowDescriptor<R> {
    pub label: Option<String>,
    pub vec: Vec<Box<dyn Dom<R>>>,
}

impl<R> Default for RowDescriptor<R> {
    fn default() -> Self {
        Self {
            label: None,
            vec: Vec::new(),
        }
    }
}

pub struct Row<R: 'static> {
    label: Option<String>,
    children: Vec<Box<dyn Dom<R>>>,
}

impl<R> Row<R> {
    pub fn new(disc: RowDescriptor<R>) -> Self {
        Self {
            label: disc.label,
            children: disc.vec,
        }
    }

    pub fn push(&mut self, child: Box<dyn Dom<R>>) {
        self.children.push(child);
    }
}

impl<R: Send + 'static> Dom<R> for Row<R> {
    fn build_render_tree(&self) -> Box<dyn Widget<R>> {
        let mut render_tree = Vec::new();

        for child in &self.children {
            render_tree.push(Arc::new(Mutex::new(child.build_render_tree())));
        }

        Box::new(RowRenderNode {
            label: self.label.clone(),
            redraw: true,
            children: render_tree,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct RowRenderNode<R: Send + 'static> {
    label: Option<String>,
    redraw: bool,
    children: Vec<Arc<Mutex<Box<dyn Widget<R>>>>>,
}

#[async_trait::async_trait]
impl<R: Send + 'static> WidgetTrait<R> for RowRenderNode<R> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    async fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> crate::events::UiEventResult<R> {
        // todo: event handling
        UiEventResult::default()
    }

    async fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> bool {
        let size = self.px_size(parent_size, context).await;

        println!("{:?}", size);
        println!("{:?}", position);

        0.0 <= position[0]
            && position[0] <= size.width
            && 0.0 >= -position[1]
            && -position[1] >= -size.height
    }

    fn update_render_tree(&mut self, dom: &dyn Dom<R>) -> Result<(), ()> {
        if (*dom).type_id() != std::any::TypeId::of::<Row<R>>() {
            Err(())
        } else {
            let dom = dom.as_any().downcast_ref::<Row<R>>().unwrap();
            // todo: differential update
            self.children.clear();
            for child in dom.children.iter() {
                self.children
                    .push(Arc::new(Mutex::new(child.build_render_tree())));
            }
            Ok(())
        }
    }

    fn compare(&self, dom: &dyn Dom<R>) -> DomComPareResult {
        if let Some(_) = dom.as_any().downcast_ref::<Row<R>>() {
            // todo: calculate difference

            DomComPareResult::Different
        } else {
            DomComPareResult::Different
        }
    }
}

#[async_trait::async_trait]
impl<R: Send + 'static> RenderingTrait for RowRenderNode<R> {
    async fn size(&self) -> crate::types::size::Size {
        Size {
            width: SizeUnit::Content(1.0),
            height: SizeUnit::Content(1.0),
        }
    }

    async fn px_size(&self, _: PxSize, context: &ApplicationContext) -> PxSize {
        let mut width_px: f32 = 0.0;
        let mut width_percent: f32 = 0.0;
        let mut height: f32 = 0.0;

        for child in &self.children {
            let child_std_size = StdSize::from_size(child.lock().await.size().await, context);

            match child_std_size.width {
                crate::types::size::StdSizeUnit::None => {
                    width_px += child.lock().await.default_size().await.width
                }
                crate::types::size::StdSizeUnit::Pixel(px) => width_px += px,
                crate::types::size::StdSizeUnit::Percent(percent) => width_percent += percent,
            }

            match child_std_size.height {
                crate::types::size::StdSizeUnit::None => {
                    height = height.max(child.lock().await.default_size().await.height)
                }
                crate::types::size::StdSizeUnit::Pixel(px) => height = height.max(px),
                crate::types::size::StdSizeUnit::Percent(_) => (),
            }
        }

        let width = width_px / (1.0 - width_percent);

        PxSize { width, height }
    }

    async fn default_size(&self) -> PxSize {
        PxSize {
            width: 0.0,
            height: 0.0,
        }
    }

    async fn render(
        &mut self,
        parent_size: PxSize,
        affine: nalgebra::Matrix4<f32>,
        encoder: RendererCommandEncoder,
    ) {
        let current_size = self.px_size(parent_size, encoder.get_context()).await;

        let mut join_handles = Vec::new();

        let mut accumulated_width: f32 = 0.0;
        for child in &mut self.children {
            let child_px_size = child
                .lock()
                .await
                .px_size(current_size, encoder.get_context())
                .await;
            let child_affine =
                na::Matrix4::new_translation(&na::Vector3::new(accumulated_width, 0.0, 0.0))
                    * affine;

            let encoder = encoder.clone();
            let child = child.clone();
            join_handles.push(tokio::spawn(async move {
                child
                    .lock()
                    .await
                    .render(child_px_size, child_affine, encoder)
                    .await;
            }));
            accumulated_width += child_px_size.width;
        }

        for join_handle in join_handles {
            join_handle.await.unwrap();
        }
    }
}
