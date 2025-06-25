use std::{any::Any, sync::Arc};

use matcha_core::{
    context::WidgetContext,
    events::Event,
    observer::Observer,
    types::{cache::Cache, range::CoverRange},
    ui::{Background, Dom, DomComPareResult, Object, UpdateWidgetError, Widget},
};

// todo: more documentation

// MARK: DOM

type SizeFn =
    dyn for<'a> Fn([Option<f32>; 2], &'a WidgetContext) -> [f32; 2] + Send + Sync + 'static;

pub struct Space {
    label: Option<String>,

    size: Arc<SizeFn>,
}

impl Space {
    pub fn new(label: Option<&str>) -> Box<Self> {
        Box::new(Self {
            label: label.map(|s| s.to_string()),
            size: Arc::new(|_, _| [0.0, 0.0]),
        })
    }

    pub fn size<F>(mut self, size: F) -> Self
    where
        F: Fn([Option<f32>; 2], &WidgetContext) -> [f32; 2] + Send + Sync + 'static,
    {
        self.size = Arc::new(size);
        self
    }
}

#[async_trait::async_trait]
impl<T: Send + 'static> Dom<T> for Space {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(SpaceNode {
            label: self.label.clone(),
            size: Arc::clone(&self.size),
            size_cache: Cache::new(),
        })
    }

    async fn collect_observer(&self) -> Observer {
        // If your widget has any child widgets,
        // you should collect their observers for matcha ui system to catch child component updates.

        Observer::default()
    }
}

// MARK: Widget

pub struct SpaceNode {
    label: Option<String>,

    size: Arc<SizeFn>,

    size_cache: Cache<[Option<f32>; 2], [f32; 2]>,
}

// MARK: Widget trait

#[async_trait::async_trait]
impl<T: Send + 'static> Widget<T> for SpaceNode {
    // label
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    // for dom handling
    // keep in mind to change redraw flag to true if some change is made.
    async fn update_widget_tree(
        &mut self,
        _: bool,
        dom: &dyn Dom<T>,
    ) -> Result<(), UpdateWidgetError> {
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Space>() {
            self.label = dom.label.clone();
            self.size = Arc::clone(&dom.size);

            Ok(())
        } else {
            return Err(UpdateWidgetError::TypeMismatch);
        }
    }

    // comparing dom
    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Space>() {
            let _ = dom;
            todo!()
        } else {
            DomComPareResult::Different
        }
    }

    // widget event
    fn widget_event(&mut self, _: &Event, _: [Option<f32>; 2], _: &WidgetContext) -> Option<T> {
        None
    }

    // Actual size including its sub widgets with pixel value.
    fn px_size(&mut self, parent_size: [Option<f32>; 2], context: &WidgetContext) -> [f32; 2] {
        *self
            .size_cache
            .get_data_or_insert_with(&parent_size, || (self.size)(parent_size, context))
    }

    // The drawing range and the area that the widget always covers.
    fn cover_range(&mut self, _: [Option<f32>; 2], _: &WidgetContext) -> CoverRange<f32> {
        Default::default()
    }

    // if redraw is needed
    fn updated(&self) -> bool {
        false
    }

    // render
    fn render(
        &mut self,
        _: &mut wgpu::RenderPass<'_>,
        _: [u32; 2],
        _: wgpu::TextureFormat,
        _: [Option<f32>; 2],
        _: Background,
        _: &WidgetContext,
    ) -> Vec<Object> {
        vec![]
    }
}
