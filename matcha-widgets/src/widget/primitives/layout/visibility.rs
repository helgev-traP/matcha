use std::any::Any;

use matcha_core::{
    context::SharedContext,
    events::Event,
    observer::Observer,
    renderer::Renderer,
    types::range::CoverRange,
    ui::{Background, Dom, DomComPareResult, Object, UpdateWidgetError, Widget},
};

#[derive(Debug, Clone, Copy)]
enum VisibilityState {
    Visible,
    Hidden,
    None,
}

pub struct Visibility<T>
where
    T: Send + 'static,
{
    // label
    label: Option<String>,

    // properties
    visible: VisibilityState,

    // content
    content: Option<Box<dyn Dom<T>>>,
}

impl<T> Default for Visibility<T>
where
    T: Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

// constructor
impl<T> Visibility<T>
where
    T: Send + 'static,
{
    pub fn new() -> Self {
        Self {
            label: None,
            visible: VisibilityState::Visible,
            content: None,
        }
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn visible(mut self) -> Self {
        self.visible = VisibilityState::Visible;
        self
    }

    pub fn hidden(mut self) -> Self {
        self.visible = VisibilityState::Hidden;
        self
    }

    pub fn none(mut self) -> Self {
        self.visible = VisibilityState::None;
        self
    }

    pub fn content(mut self, content: Box<dyn Dom<T>>) -> Self {
        self.content = Some(content);
        self
    }
}

#[async_trait::async_trait]
impl<T> Dom<T> for Visibility<T>
where
    T: Send + 'static,
{
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(VisibilityNode {
            label: self.label.clone(),
            visible: self.visible,
            content: self
                .content
                .as_ref()
                .map(|content| content.build_widget_tree()),
        })
    }

    async fn collect_observer(&self) -> Observer {
        match self.visible {
            VisibilityState::Visible => {
                if let Some(content) = &self.content {
                    content.collect_observer().await
                } else {
                    Observer::default()
                }
            }
            VisibilityState::Hidden | VisibilityState::None => Observer::default(),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct VisibilityNode<T>
where
    T: Send + 'static,
{
    // label
    label: Option<String>,

    // properties
    visible: VisibilityState,

    // content
    content: Option<Box<dyn Widget<T>>>,
}

#[async_trait::async_trait]
impl<T> Widget<T> for VisibilityNode<T>
where
    T: Send + 'static,
{
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    async fn update_widget_tree(
        &mut self,
        component_updated: bool,
        dom: &dyn Dom<T>,
    ) -> Result<(), UpdateWidgetError> {
        if let Some(dom) = dom.as_any().downcast_ref::<Visibility<T>>() {
            // update properties
            self.label = dom.label.clone();

            // update content
            if let Some(dom_content) = &dom.content {
                if let Some(self_content) = self.content.as_mut() {
                    self_content
                        .update_widget_tree(component_updated, dom_content.as_ref())
                        .await?;
                } else {
                    self.content = Some(dom_content.build_widget_tree());
                }
            } else {
                self.content = None;
            }

            Ok(())
        } else {
            Err(UpdateWidgetError::TypeMismatch)
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(dom) = dom.as_any().downcast_ref::<Visibility<T>>() {
            let _ = dom;
            todo!()
        } else {
            DomComPareResult::Different
        }
    }

    fn widget_event(
        &mut self,
        event: &Event,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
    ) -> Option<T> {
        match self.visible {
            VisibilityState::Visible => self
                .content
                .as_mut()
                .map(|content| content.widget_event(event, parent_size, context))
                .unwrap_or_default(),
            VisibilityState::Hidden | VisibilityState::None => None,
        }
    }

    fn px_size(&mut self, parent_size: [Option<f32>; 2], context: &SharedContext) -> [f32; 2] {
        match self.visible {
            VisibilityState::Visible | VisibilityState::Hidden => self
                .content
                .as_mut()
                .map(|content| content.px_size(parent_size, context))
                .unwrap_or([0.0, 0.0]),
            VisibilityState::None => [0.0, 0.0],
        }
    }

    fn cover_range(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
    ) -> CoverRange<f32> {
        match self.visible {
            VisibilityState::Visible => self
                .content
                .as_mut()
                .map(|content| content.cover_range(parent_size, context))
                .unwrap_or_default(),
            VisibilityState::Hidden | VisibilityState::None => CoverRange::default(),
        }
    }

    fn redraw(&self) -> bool {
        self.content
            .as_ref()
            .map(|content| content.redraw())
            .unwrap_or(false)
    }

    fn render(
        &mut self,
        parent_size: [Option<f32>; 2],
        background: Background,
        context: &SharedContext,
        renderer: &Renderer,
    ) -> Vec<Object> {
        match self.visible {
            VisibilityState::Visible => self
                .content
                .as_mut()
                .map(|content| content.render(parent_size, background, context, renderer))
                .unwrap_or_default(),
            VisibilityState::Hidden | VisibilityState::None => vec![],
        }
    }
}
