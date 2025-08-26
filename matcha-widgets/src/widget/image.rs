use std::any::Any;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use image::{DynamicImage, GenericImageView};
use matcha_core::Style;
use matcha_core::{
    device_event::DeviceEvent,
    types::range::{CoverRange, Range2D},
    ui::{
        Background, Constraints, Dom, DomCompareResult, UpdateWidgetError, Widget, WidgetContext,
    },
    update_flag::UpdateNotifier,
};
use renderer::render_node::RenderNode;
use texture_atlas::atlas_simple::atlas::AtlasRegion;
use utils::cache::Cache;

use crate::style;
use crate::types::size::Size;

// MARK: DOM

pub struct Image {
    label: Option<String>,
    image_style: style::image::Image,
    size: [Size; 2],
}

impl Image {
    pub fn new(image: impl Into<style::image::ImageSource>) -> Box<Self> {
        Box::new(Self {
            label: None,
            image_style: style::image::Image::new(image),
            size: [Size::child_w(1.0), Size::child_h(1.0)],
        })
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn size<F>(mut self, size: [Size; 2]) -> Self {
        self.size = size;
        self
    }
}
