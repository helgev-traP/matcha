pub mod panel;
pub use panel::Panel;

pub mod teacup;
pub use teacup::Teacup;

use std::sync::Arc;

use wgpu::{core::device, util::DeviceExt};

use winit::{self, event::Event, platform::run_on_demand::EventLoopExtRunOnDemand};

use cgmath::prelude::*;

use super::types::Size;
use super::window::DeviceQueue;

pub trait Widget {
    fn set_device_queue(&mut self, device_queue: DeviceQueue);
    fn size(&self) -> &Size;
    fn render(&self) -> Option<&wgpu::Texture>;
    // ...
}
