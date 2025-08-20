use std::any::Any;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use image::{DynamicImage, GenericImageView};
use matcha_core::Style;
use matcha_core::{
    device_event::DeviceEvent,
    render_node::RenderNode,
    types::range::{CoverRange, Range2D},
    ui::{
        Background, Constraints, Dom, DomComPareResult, UpdateWidgetError, Widget, WidgetContext,
    },
    update_flag::UpdateNotifier,
};
use texture_atlas::atlas_simple::atlas::AtlasRegion;
use utils::single_cache::SingleCache;

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
            size: [Size::default(), Size::default()],
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
