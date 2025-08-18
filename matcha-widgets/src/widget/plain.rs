use std::any::Any;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::renderer::texture_color::TextureColor;
use matcha_core::{
    device_event::DeviceEvent,
    render_node::RenderNode,
    types::range::{CoverRange, Range2D},
    ui::{
        Background, Constraints, Dom, DomComPareResult, Style, UpdateWidgetError, Widget,
        WidgetContext,
    },
    update_flag::UpdateNotifier,
};
use texture_atlas::atlas_simple::atlas::AtlasRegion;
use utils::single_cache::SingleCache;

// todo: more documentation

// MARK: DOM

pub struct Plain<T> {
    label: Option<String>,
    style: Vec<Box<dyn Style>>,
    content: Option<Box<dyn Dom<T>>>,
}

impl<T> Plain<T> {
    pub fn new(label: Option<&str>) -> Box<Self> {
        Box::new(Self {
            label: label.map(|s| s.to_string()),
            style: Vec::new(),
            content: None,
        })
    }

    pub fn style(mut self, style: Box<dyn Style>) -> Self {
        self.style.push(style);
        self
    }

    pub fn content(mut self, content: Box<dyn Dom<T>>) -> Self {
        self.content = Some(content);
        self
    }
}

#[async_trait::async_trait]
impl<T: Send + 'static> Dom<T> for Plain<T> {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(PlainNode {
            label: self.label.clone(),
            style: self.style.clone(),
            content: self
                .content
                .as_ref()
                .map(|content| content.build_widget_tree()),
            update_notifier: None,
            size: [0.0, 0.0],
            style_cache: SingleCache::new(),
        })
    }

    async fn set_update_notifier(&self, notifier: &UpdateNotifier) {
        if let Some(content) = &self.content {
            content.set_update_notifier(notifier).await;
        }
    }
}

// MARK: Widget

pub struct PlainNode<T> {
    label: Option<String>,
    style: Vec<Box<dyn Style>>,
    content: Option<Box<dyn Widget<T>>>,
    update_notifier: Option<UpdateNotifier>,
    size: [f32; 2],
    style_cache: SingleCache<u64, AtlasRegion>,
}

// MARK: Widget trait

#[async_trait::async_trait]
impl<T: Send + 'static> Widget<T> for PlainNode<T> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    async fn update_widget_tree(
        &mut self,
        _component_updated: bool,
        dom: &dyn Dom<T>,
    ) -> Result<(), UpdateWidgetError> {
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Plain<T>>() {
            self.label = dom.label.clone();
            self.style = dom.style.clone();
            // Proper content update logic is needed here.
            // This might involve comparing and updating the child widget.
            Ok(())
        } else {
            Err(UpdateWidgetError::TypeMismatch)
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if (dom as &dyn Any).downcast_ref::<Plain<T>>().is_some() {
            DomComPareResult::Same // Simplified for now
        } else {
            DomComPareResult::Different
        }
    }

    fn device_event(&mut self, event: &DeviceEvent, context: &WidgetContext) -> Option<T> {
        if let Some(content) = &mut self.content {
            return content.device_event(event, context);
        }
        None
    }

    fn is_inside(&mut self, position: [f32; 2], context: &WidgetContext) -> bool {
        for style in &self.style {
            if style.is_inside(position, self.size, context) {
                return true;
            }
        }

        if let Some(content) = &mut self.content {
            if content.is_inside(position, context) {
                return true;
            }
        }

        false
    }

    fn preferred_size(&mut self, constraints: &Constraints, context: &WidgetContext) -> [f32; 2] {
        if let Some(content) = &mut self.content {
            content.preferred_size(constraints, context)
        } else {
            [constraints.min_width, constraints.min_height]
        }
    }

    fn arrange(&mut self, final_size: [f32; 2], context: &WidgetContext) {
        self.size = final_size;
        if let Some(content) = &mut self.content {
            content.arrange(final_size, context);
        }
    }

    fn cover_range(&mut self, context: &WidgetContext) -> CoverRange<f32> {
        // This needs a proper implementation based on styles and content.
        CoverRange::default()
    }

    fn need_rerendering(&self) -> bool {
        // A real widget would have state to track this.
        true // For now, always rerender to draw styles.
    }

    fn render(
        &mut self,
        background: Background,
        animation_update_flag_notifier: UpdateNotifier,
        ctx: &WidgetContext,
    ) -> RenderNode {
        self.update_notifier = Some(animation_update_flag_notifier.clone());

        let mut render_node = RenderNode::new();

        if !self.style.is_empty() {
            let mut hasher = DefaultHasher::new();
            self.size.iter().for_each(|f| f.to_bits().hash(&mut hasher));
            let hash = hasher.finish();

            let (_, style_texture) = self.style_cache.get_or_insert_with(hash, || {
                let mut x_min = f32::MAX;
                let mut x_max = f32::MIN;
                let mut y_min = f32::MAX;
                let mut y_max = f32::MIN;

                for style in &self.style {
                    let range = style.draw_range(self.size, ctx);
                    x_min = x_min.min(range.left());
                    x_max = x_max.max(range.right());
                    y_min = y_min.min(range.bottom());
                    y_max = y_max.max(range.top());
                }

                let texture_size = [(x_max - x_min).ceil() as u32, (y_max - y_min).ceil() as u32];

                let style_region = ctx
                    .texture_atlas()
                    .allocate_color(ctx.device(), ctx.queue(), texture_size)
                    .unwrap();

                let mut encoder =
                    ctx.device()
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Plain Style Render Encoder"),
                        });

                {
                    let mut render_pass = style_region.begin_render_pass(&mut encoder).unwrap();

                    for style in &self.style {
                        style.draw(
                            &mut render_pass,
                            texture_size,
                            style_region.formats()[0], // Assuming single format
                            self.size,
                            [x_min, y_min],
                            ctx,
                        );
                    }
                }
                ctx.queue().submit(Some(encoder.finish()));

                style_region
            });

            render_node.texture_and_position =
                Some((style_texture.clone(), nalgebra::Matrix4::identity()));
        }

        if let Some(content) = &mut self.content {
            let content_node = content.render(background, animation_update_flag_notifier, ctx);
            render_node.add_child(content_node, nalgebra::Matrix4::identity());
        }

        render_node
    }
}
