use std::any::Any;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use image::{DynamicImage, GenericImageView};
use matcha_core::{
    device_event::DeviceEvent,
    render_node::RenderNode,
    types::range::CoverRange,
    ui::{
        Background, Constraints, Dom, DomComPareResult, UpdateWidgetError, Widget, WidgetContext,
    },
    update_flag::UpdateNotifier,
};
use texture_atlas::atlas_simple::atlas::AtlasRegion;
use utils::single_cache::SingleCache;

// MARK: DOM

pub struct Image {
    label: Option<String>,
    image: DynamicImage,
}

impl Image {
    pub fn new(image: DynamicImage) -> Box<Self> {
        Box::new(Self { label: None, image })
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }
}

#[async_trait::async_trait]
impl<T: Send + 'static> Dom<T> for Image {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(ImageNode {
            label: self.label.clone(),
            image: self.image.clone(),
            texture_cache: SingleCache::new(),
            update_notifier: None,
        })
    }

    async fn set_update_notifier(&self, _notifier: &UpdateNotifier) {}
}

// MARK: Widget

pub struct ImageNode {
    label: Option<String>,
    image: DynamicImage,
    texture_cache: SingleCache<u64, AtlasRegion>,
    update_notifier: Option<UpdateNotifier>,
}

#[async_trait::async_trait]
impl<T: Send + 'static> Widget<T> for ImageNode {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    async fn update_widget_tree(
        &mut self,
        _component_updated: bool,
        dom: &dyn Dom<T>,
    ) -> Result<(), UpdateWidgetError> {
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Image>() {
            // For simplicity, we'll just replace the image.
            // A more sophisticated implementation might compare image data.
            self.image = dom.image.clone();
            self.texture_cache = SingleCache::new(); // Invalidate cache
            self.label = dom.label.clone();
            Ok(())
        } else {
            Err(UpdateWidgetError::TypeMismatch)
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if (dom as &dyn Any).downcast_ref::<Image>().is_some() {
            // Simplified comparison. A real implementation should hash the image data.
            DomComPareResult::Same
        } else {
            DomComPareResult::Different
        }
    }

    fn device_event(&mut self, _event: &DeviceEvent, _context: &WidgetContext) -> Option<T> {
        None
    }

    fn is_inside(&mut self, _position: [f32; 2], _context: &WidgetContext) -> bool {
        true
    }

    fn preferred_size(&mut self, _constraints: &Constraints, _context: &WidgetContext) -> [f32; 2] {
        let (width, height) = self.image.dimensions();
        [width as f32, height as f32]
    }

    fn arrange(&mut self, _final_size: [f32; 2], _context: &WidgetContext) {
        // The image widget takes the size determined by preferred_size.
    }

    fn cover_range(&mut self, _context: &WidgetContext) -> CoverRange<f32> {
        CoverRange::default()
    }

    fn need_rerendering(&self) -> bool {
        true
    }

    fn render(
        &mut self,
        _background: Background,
        animation_update_flag_notifier: UpdateNotifier,
        ctx: &WidgetContext,
    ) -> RenderNode {
        self.update_notifier = Some(animation_update_flag_notifier);

        let mut hasher = DefaultHasher::new();
        self.image.as_bytes().hash(&mut hasher);
        let hash = hasher.finish();

        let (_, texture_region) = self.texture_cache.get_or_insert_with(hash, || {
            let (width, height) = self.image.dimensions();
            let rgba_image = self.image.to_rgba8();

            let texture_region = ctx
                .texture_atlas()
                .allocate_color(ctx.device(), ctx.queue(), [width, height])
                .unwrap();

            texture_region
                .write_data(ctx.queue(), &[&rgba_image])
                .unwrap();

            texture_region
        });

        let mut render_node = RenderNode::new();
        render_node.texture_and_position =
            Some((texture_region.clone(), nalgebra::Matrix4::identity()));

        render_node
    }
}
