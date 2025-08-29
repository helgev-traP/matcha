use std::any::Any;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use cosmic_text::{Attrs, Color, Metrics};
use matcha_core::{
    device_input::DeviceInput,
    types::range::CoverRange,
    ui::{
        Background, Constraints, Dom, DomCompareResult, Style, UpdateWidgetError, Widget,
        WidgetContext,
    },
    update_flag::UpdateNotifier,
};
use renderer::render_node::RenderNode;

use crate::style::text_cosmic::{TextCosmic, TextElement};

// MARK: DOM

pub struct Text<'a> {
    label: Option<String>,
    content: String,
    attrs: Attrs<'a>,
    metrics: Metrics,
}

impl<'a> Text<'a> {
    pub fn new(content: &str) -> Box<Self> {
        Box::new(Self {
            label: None,
            content: content.to_string(),
            attrs: Attrs::new(),
            metrics: Metrics::new(14.0, 20.0), // Default metrics
        })
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn attrs(mut self, attrs: Attrs<'a>) -> Self {
        self.attrs = attrs;
        self
    }

    pub fn metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = metrics;
        self
    }
}

#[async_trait::async_trait]
impl<'a: 'static, T: Send + 'static> Dom<T> for Text<'a> {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        let text_element = TextElement {
            text: self.content.clone(),
            attrs: self.attrs.clone(),
        };

        let style = TextCosmic {
            texts: vec![text_element],
            color: Color::rgb(0, 0, 0), // Default to black
            metrics: self.metrics,
            max_size: [None, None],
            buffer: Default::default(),
            cache_in_memory: Default::default(),
            cache_in_texture: Default::default(),
        };

        Box::new(TextNode {
            label: self.label.clone(),
            content: self.content.clone(),
            attrs: self.attrs.clone(),
            metrics: self.metrics,
            style,
            size: [0.0, 0.0],
        })
    }

    async fn set_update_notifier(&self, _notifier: &UpdateNotifier) {}
}

// MARK: Widget

pub struct TextNode<'a> {
    label: Option<String>,
    content: String,
    attrs: Attrs<'a>,
    metrics: Metrics,
    style: TextCosmic<'a>,
    size: [f32; 2],
}

#[async_trait::async_trait]
impl<'a: 'static, T: Send + 'static> Widget<T> for TextNode<'a> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    async fn update_widget_tree(
        &mut self,
        _component_updated: bool,
        dom: &dyn Dom<T>,
    ) -> Result<(), UpdateWidgetError> {
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Text<'a>>() {
            let mut changed = false;
            if self.content != dom.content {
                self.content = dom.content.clone();
                changed = true;
            }
            if self.attrs != dom.attrs {
                self.attrs = dom.attrs.clone();
                changed = true;
            }
            if self.metrics != dom.metrics {
                self.metrics = dom.metrics;
                changed = true;
            }

            if changed {
                let text_element = TextElement {
                    text: self.content.clone(),
                    attrs: self.attrs.clone(),
                };
                self.style.texts = vec![text_element];
                self.style.metrics = self.metrics;
            }

            self.label = dom.label.clone();
            Ok(())
        } else {
            Err(UpdateWidgetError::TypeMismatch)
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomCompareResult {
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Text<'a>>() {
            if self.content == dom.content && self.attrs == dom.attrs && self.metrics == dom.metrics
            {
                DomCompareResult::Same
            } else {
                let mut hasher = DefaultHasher::new();
                dom.content.hash(&mut hasher);
                dom.attrs.hash(&mut hasher);
                // dom.metrics.hash(&mut hasher); // Metrics does not implement Hash
                DomCompareResult::Changed(hasher.finish() as usize)
            }
        } else {
            DomCompareResult::Different
        }
    }

    fn device_input(&mut self, _event: &DeviceInput, _context: &WidgetContext) -> Option<T> {
        None
    }

    fn is_inside(&mut self, position: [f32; 2], context: &WidgetContext) -> bool {
        self.style.is_inside(position, self.size, context)
    }

    fn preferred_size(&mut self, _constraints: &Constraints, context: &WidgetContext) -> [f32; 2] {
        let range = self.style.draw_range(self.size, context);
        [range.width(), range.height()]
    }

    fn arrange(&mut self, final_size: [f32; 2], _context: &WidgetContext) {
        self.size = final_size;
    }

    fn need_rerendering(&self) -> bool {
        true
    }

    fn render(&mut self, _background: Background, ctx: &WidgetContext) -> RenderNode {
        // The actual drawing logic is in TextCosmic's draw method,
        // which is not fully implemented yet.
        // For now, we'll just return an empty node.
        // When TextCosmic::draw is implemented, this should be updated
        // to render the text to a texture and return a RenderNode with it.
        RenderNode::new()
    }
}
