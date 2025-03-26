use std::any::Any;

use matcha_core::{
    context::SharedContext,
    events::{UiEvent, UiEventResult},
    types::range::Range2D,
    ui::{Dom, DomComPareResult, UiBackground, UiContext, UpdateWidgetError, Widget},
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

impl<T> Widget<T> for VisibilityNode<T>
where
    T: Send + 'static,
{
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn update_widget_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), UpdateWidgetError> {
        if let Some(dom) = dom.as_any().downcast_ref::<Visibility<T>>() {
            // update properties
            self.label = dom.label.clone();

            // update content
            if let Some(dom_content) = &dom.content {
                if let Some(self_content) = self.content.as_mut() {
                    self_content.update_widget_tree(dom_content.as_ref())?;
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
            todo!()
        } else {
            DomComPareResult::Different
        }
    }

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> UiEventResult<T> {
        match self.visible {
            VisibilityState::Visible => self
                .content
                .as_mut()
                .map(|content| content.widget_event(event, parent_size, context, tag, frame))
                .unwrap_or_default(),
            VisibilityState::Hidden | VisibilityState::None => UiEventResult::default(),
        }
    }

    fn px_size(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> [f32; 2] {
        match self.visible {
            VisibilityState::Visible | VisibilityState::Hidden => self
                .content
                .as_mut()
                .map(|content| content.px_size(parent_size, context, tag, frame))
                .unwrap_or([0.0, 0.0]),
            VisibilityState::None => [0.0, 0.0],
        }
    }

    fn draw_range(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> Option<Range2D<f32>> {
        match self.visible {
            VisibilityState::Visible => self
                .content
                .as_mut()
                .map(|content| content.draw_range(parent_size, context, tag, frame))
                .unwrap_or(None),
            VisibilityState::Hidden | VisibilityState::None => None,
        }
    }

    fn cover_area(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> Option<Range2D<f32>> {
        match self.visible {
            VisibilityState::Visible => self
                .content
                .as_mut()
                .map(|content| content.cover_area(parent_size, context, tag, frame))
                .unwrap_or(None),
            VisibilityState::Hidden | VisibilityState::None => None,
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
        ui_background: UiBackground,
        ui_context: UiContext,
    ) -> Vec<matcha_core::ui::Object> {
        match self.visible {
            VisibilityState::Visible => self
                .content
                .as_mut()
                .map(|content| content.render(ui_background, ui_context))
                .unwrap_or_default(),
            VisibilityState::Hidden | VisibilityState::None => vec![],
        }
    }
}
