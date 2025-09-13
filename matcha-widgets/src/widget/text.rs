use crate::style::Style;
use cosmic_text::{Attrs, Color, Metrics};
use matcha_core::{
    device_input::DeviceInput,
    metrics::{Arrangement, Constraints},
    ui::{
        AnyWidgetFrame, ApplicationHandler, Background, Dom, Widget, WidgetContext, WidgetFrame,
        widget::{AnyWidget, InvalidationHandle},
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
impl<'a: 'static, T: Send + Sync + 'static> Dom<T> for Text<'a> {
    fn build_widget_tree(&self) -> Box<dyn AnyWidgetFrame<T>> {
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

        Box::new(WidgetFrame::new(
            self.label.clone(),
            vec![],
            vec![],
            TextNode {
                content: self.content.clone(),
                attrs: self.attrs.clone(),
                metrics: self.metrics,
                style,
            },
        ))
    }

    async fn set_update_notifier(&self, _notifier: &UpdateNotifier) {}
}

// MARK: Widget

pub struct TextNode<'a> {
    content: String,
    attrs: Attrs<'a>,
    metrics: Metrics,
    style: TextCosmic<'a>,
}

impl<'a: 'static, T: Send + Sync + 'static> Widget<Text<'a>, T, ()> for TextNode<'a> {
    fn update_widget<'b>(
        &mut self,
        dom: &'b Text<'a>,
        cache_invalidator: Option<InvalidationHandle>,
    ) -> Vec<(&'b dyn Dom<T>, (), u128)> {
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
            if let Some(handle) = cache_invalidator {
                handle.relayout_next_frame();
            }
            let text_element = TextElement {
                text: self.content.clone(),
                attrs: self.attrs.clone(),
            };
            self.style.texts = vec![text_element];
            self.style.metrics = self.metrics;
        }
        vec![]
    }

    fn measure(
        &self,
        constraints: &Constraints,
        _: &[(&dyn AnyWidget<T>, &())],
        ctx: &WidgetContext,
    ) -> [f32; 2] {
        let rect = self.style.required_region(constraints, ctx);
        if let Some(rect) = rect {
            [rect.width(), rect.height()]
        } else {
            [0.0, 0.0]
        }
    }

    fn arrange(
        &self,
        _bounds: [f32; 2],
        _children: &[(&dyn AnyWidget<T>, &())],
        _ctx: &WidgetContext,
    ) -> Vec<Arrangement> {
        vec![]
    }

    fn device_input(
        &mut self,
        _bounds: [f32; 2],
        _event: &DeviceInput,
        _children: &mut [(&mut dyn AnyWidget<T>, &mut (), &Arrangement)],
        _cache_invalidator: InvalidationHandle,
        _ctx: &WidgetContext,
        _app_handler: &ApplicationHandler,
    ) -> Option<T> {
        None
    }

    fn is_inside(
        &self,
        bounds: [f32; 2],
        position: [f32; 2],
        _children: &[(&dyn AnyWidget<T>, &(), &Arrangement)],
        ctx: &WidgetContext,
    ) -> bool {
        self.style.is_inside(position, bounds, ctx)
    }

    fn render(
        &self,
        _bounds: [f32; 2],
        _children: &[(&dyn AnyWidget<T>, &(), &Arrangement)],
        _background: Background,
        ctx: &WidgetContext,
    ) -> RenderNode {
        let mut render_node = RenderNode::new();
        let size = <Self as Widget<Text, T, ()>>::measure(
            self,
            &Constraints::new([0.0f32, f32::INFINITY], [0.0f32, f32::INFINITY]),
            &[],
            ctx,
        );

        if size[0] > 0.0 && size[1] > 0.0 {
            let texture_size = [size[0].ceil() as u32, size[1].ceil() as u32];
            if let Ok(style_region) =
                ctx.texture_atlas()
                    .allocate_color(ctx.device(), ctx.queue(), texture_size)
            {
                let mut encoder =
                    ctx.device()
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Text Render Encoder"),
                        });

                self.style
                    .draw(&mut encoder, &style_region, size, [0.0, 0.0], ctx);

                ctx.queue().submit(Some(encoder.finish()));
                render_node =
                    render_node.with_texture(style_region, size, nalgebra::Matrix4::identity());
            }
        }

        render_node
    }
}
