use std::{cell::LazyCell, collections::HashMap, hash::Hash, sync::Arc};

use crate::{
    context::SharedContext,
    events::UiEvent,
    renderer::Renderer,
    types::{
        double_cache_set::DoubleSetCache, range::Range2D, size::{Size, StdSize}
    },
    ui::{Dom, DomComPareResult, Object, TextureObject, Widget},
    widgets::paint::Paint,
};

use crate::widgets::style::{Border, BoxSizing, Padding, Visibility};

mod renderer;

use rayon::iter::Map;
use renderer::blur_renderer;
use renderer::texture_renderer;
use wgpu::{core::device, util::DeviceExt};

// MARK: DOM

pub struct StyleBox<T: Send + 'static> {
    // label
    label: Option<String>,

    // layout
    size: [Size; 2],
    padding: Padding,
    box_sizing: BoxSizing,
    visibility: Visibility,

    // border painting
    border_shape: Border,
    border_paint: Vec<Paint>,

    // background painting
    background_paint: Vec<Paint>,

    // content
    content: Option<Box<dyn Dom<T>>>,
}

/// build chain
impl<T: Send + 'static> StyleBox<T> {
    pub fn new(label: Option<&str>) -> Box<Self> {
        Box::new(Self {
            label: label.map(|s| s.to_string()),
            size: [Size::Content(1.0), Size::Content(1.0)],
            padding: Padding::default(),
            box_sizing: BoxSizing::default(),
            visibility: Visibility::Visible,
            border_shape: Border::default(),
            border_paint: Vec::new(),
            background_paint: Vec::new(),
            content: None,
        })
    }

    pub fn size(mut self, width: Size, height: Size) -> Self {
        self.size = [width, height];
        self
    }

    pub fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    pub fn box_sizing(mut self, box_sizing: BoxSizing) -> Self {
        self.box_sizing = box_sizing;
        self
    }

    pub fn visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn border_shape(mut self, border_shape: Border) -> Self {
        self.border_shape = border_shape;
        self
    }

    pub fn border_paint(mut self, border_paint: Vec<Paint>) -> Self {
        self.border_paint = border_paint;
        self
    }

    pub fn push_border_paint(mut self, paint: Paint) -> Self {
        self.border_paint.push(paint);
        self
    }

    pub fn background_paint(mut self, paint: Vec<Paint>) -> Self {
        self.background_paint = paint;
        self
    }

    pub fn push_background_paint(mut self, paint: Paint) -> Self {
        self.background_paint.push(paint);
        self
    }

    pub fn content(mut self, content: Box<dyn Dom<T>>) -> Self {
        self.content = Some(content);
        self
    }
}

impl<T: Send + 'static> Dom<T> for StyleBox<T> {
    fn build_widget_tree(&self) -> (Box<dyn Widget<T>>, bool) {
        let mut has_dynamic = false;

        let content = self.content.as_ref().map(|content| {
            let (c, d) = content.build_widget_tree();
            has_dynamic |= d;
            c
        });

        (
            Box::new(StyleBoxNode {
                label: self.label.clone(),
                size: self.size,
                padding: self.padding,
                box_sizing: self.box_sizing,
                visibility: self.visibility,
                border_shape: self.border_shape,
                border_paint: self.border_paint.clone(),
                background_paint: self.background_paint.clone(),
                content,
                blur_renderer: None,
                texture_renderer: None,
                border_settings: todo!(),
                background_settings: todo!(),
                context: None,
            }),
            has_dynamic,
        )
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// MARK: Widget

pub struct StyleBoxNode<T: Send + 'static> {
    // dom data

    // label
    label: Option<String>,
    // layout
    size: [Size; 2],
    padding: Padding,
    box_sizing: BoxSizing,
    visibility: Visibility,
    // border
    border_shape: Border,
    border_paint: Vec<Paint>,
    // background
    background_paint: Vec<Paint>,

    // content
    content: Option<Box<dyn Widget<T>>>,

    // renderer

    blur_renderer: Option<blur_renderer::BlurRenderer>,
    texture_renderer: Option<texture_renderer::TextureRenderer>,

    // settings buffer
    border_settings: Vec<BufferData<renderer::Settings>>,
    background_settings: Vec<BufferData<renderer::Settings>>,

    // context buffer

    // use [viewport_size.width * 10, viewport_size.height * 10] as key.
    // context_buffer: DoubleSetCache<[u32; 2], BufferData<renderer::ViewportInfo>>,

    // currently just store one context cache
    context: Option<Context>,
}

struct Context {
    // id
    size: [StdSize; 2],
    tag: u64,
    // data
    px_size: [f32; 2],
    content_size: [StdSize; 2],
    // caches
}

// MARK: Widget trait

impl<T: Send + 'static> Widget<T> for StyleBoxNode<T> {
    fn label(&self) -> Option<&str> {
        todo!()
    }

    fn update_widget_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), ()> {
        todo!()
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        todo!()
    }

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> crate::events::UiEventResult<T> {
        todo!()
    }

    fn size(&self) -> [Size; 2] {
        todo!()
    }

    fn px_size(&mut self, parent_size: [StdSize; 2], context: &SharedContext) -> [f32; 2] {
        todo!()
    }

    fn draw_range(
        &mut self,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> Option<Range2D<f32>> {
        todo!()
    }

    fn cover_area(
        &mut self,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> Option<Range2D<f32>> {
        todo!()
    }

    fn has_dynamic(&self) -> bool {
        todo!()
    }

    fn redraw(&self) -> bool {
        todo!()
    }

    fn render(
        &mut self,
        // ui environment
        parent_size: [StdSize; 2],
        background_view: &wgpu::TextureView,
        background_range: Range2D<f32>,
        // context
        context: &SharedContext,
        renderer: &Renderer,
        frame: u64,
    ) -> Vec<Object> {
        todo!()
    }
}

// MARK: Utils ?

// MARK: BufferData

struct BufferData<T: bytemuck::Pod + PartialEq> {
    raw: T,
    buffer: Option<wgpu::Buffer>,
    buffer_usage: Option<wgpu::BufferUsages>,
    binding_group: Option<wgpu::BindGroup>,
    binding_type: Option<wgpu::BindingType>,
}

impl<T: bytemuck::Pod + PartialEq> BufferData<T> {
    fn new(raw: T) -> Self {
        Self {
            raw,
            buffer: None,
            buffer_usage: None,
            binding_group: None,
            binding_type: None,
        }
    }

    fn get_binding_group(
        &mut self,
        device: &wgpu::Device,
        usage: wgpu::BufferUsages,
        binding_type: wgpu::BindingType,
    ) -> &wgpu::BindGroup {
        self.buffer_usage = Some(usage);
        self.binding_type = Some(binding_type);

        let buffer = self.buffer.get_or_insert_with(|| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[self.raw]),
                usage,
            })
        });

        self.binding_group.get_or_insert_with(|| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::all(),
                        ty: binding_type,
                        count: None,
                    }],
                }),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer,
                        offset: 0,
                        size: None,
                    }),
                }],
            })
        })
    }

    fn update(
        &mut self,
        new: T,
    ) {
        if self.raw != new {
            self.raw = new;
            self.buffer = None;
            self.binding_group = None;
            self.binding_group = None;
            self.binding_type = None;
        }
    }
}
