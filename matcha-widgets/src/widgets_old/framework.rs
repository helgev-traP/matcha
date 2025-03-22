// ! These is a experimental structs.

// ! **** !
// ! TODO !
// ! **** !

use std::hash::{Hash, Hasher};

use crate::context::SharedContext;
use crate::events::{UiEvent, UiEventResult};
use crate::renderer::Renderer;
use crate::types::{double_cache_set::DoubleSetCache, range::Range2D};
use crate::ui::{Dom, DomComPareResult, Object, Widget};

// MARK: Node

pub struct FrameworkNode<Properties, Contents, SettingsCache, ContextCache, T>
where
    Properties: 'static,
    Contents: 'static,
    SettingsCache: 'static,
    ContextCache: 'static,
    T: Send + 'static,
{
    // label
    label: Option<String>,
    id: u128,

    // properties
    properties: Properties,

    // contents
    contents: Contents,

    // settings cache / buffer
    settings_cache: Option<SettingsCache>,

    // context cache / buffer
    context_cache: DoubleSetCache<ContextKey, ContextCacheData<ContextCache>>,

    // redraw flag
    redraw: bool,

    // function pointers
    update_widget_tree: fn(
        &dyn Dom<T>,
        // fields
        &mut Option<String>,
        &mut Properties,
        &mut Contents,
        &mut Option<SettingsCache>,
    ) -> Result<bool, ()>,

    compare: fn(
        &dyn Dom<T>,
        // fields
        Option<&str>,
        &Properties,
        &Contents,
        Option<&SettingsCache>,
    ) -> DomComPareResult,

    widget_event: fn(
        &UiEvent,
        [Option<f32>; 2],
        &SharedContext,
        // fields
    ) -> UiEventResult<T>,

    size_calc: fn(
        [Option<f32>; 2],
        &SharedContext,
        // fields
        &Properties,
        &mut Contents,
        &mut Option<SettingsCache>,
    ) -> ([f32; 2], [Option<f32>; 2]),

    draw_range: fn(
        [f32; 2],
        [Option<f32>; 2],
        &mut Option<ContextCache>,
        // fields
        &Properties,
        &mut Contents,
        &mut Option<SettingsCache>,
    ) -> Option<Range2D<f32>>,

    cover_area: fn(
        [f32; 2],
        [Option<f32>; 2],
        &mut Option<ContextCache>,
        // fields
        &Properties,
        &mut Contents,
        &mut Option<SettingsCache>,
    ) -> Option<Range2D<f32>>,

    render: fn(
        // context
        [f32; 2],
        [Option<f32>; 2],
        &mut Option<ContextCache>,
        // fields
        &Properties,
        &mut Contents,
        &mut Option<SettingsCache>,
        // ui environment
        &wgpu::TextureView,
        Range2D<f32>,
        &SharedContext,
        &Renderer,
    ) -> Vec<Object>,
}

impl<Properties, Contents, SettingsCache, ContextCache, T>
    FrameworkNode<Properties, Contents, SettingsCache, ContextCache, T>
where
    Properties: 'static,
    Contents: 'static,
    SettingsCache: 'static,
    ContextCache: 'static,
    T: Send + 'static,
{
    pub fn new(
        label: Option<&str>,
        properties: Properties,
        contents: Contents,
        // function pointers
        update_widget_tree: fn(
            &dyn Dom<T>,
            &mut Option<String>,
            &mut Properties,
            &mut Contents,
            &mut Option<SettingsCache>,
        ) -> Result<bool, ()>,
        compare: fn(
            &dyn Dom<T>,
            Option<&str>,
            &Properties,
            &Contents,
            Option<&SettingsCache>,
        ) -> DomComPareResult,
        widget_event: fn(&UiEvent, [Option<f32>; 2], &SharedContext) -> UiEventResult<T>,
        size_calc: fn(
            [Option<f32>; 2],
            &SharedContext,
            &Properties,
            &mut Contents,
            &mut Option<SettingsCache>,
        ) -> ([f32; 2], [Option<f32>; 2]),
        draw_range: fn(
            [f32; 2],
            [Option<f32>; 2],
            &mut Option<ContextCache>,
            &Properties,
            &mut Contents,
            &mut Option<SettingsCache>,
        ) -> Option<Range2D<f32>>,
        cover_area: fn(
            [f32; 2],
            [Option<f32>; 2],
            &mut Option<ContextCache>,
            &Properties,
            &mut Contents,
            &mut Option<SettingsCache>,
        ) -> Option<Range2D<f32>>,
        render: fn(
            [f32; 2],
            [Option<f32>; 2],
            &mut Option<ContextCache>,
            &Properties,
            &mut Contents,
            &mut Option<SettingsCache>,
            &wgpu::TextureView,
            Range2D<f32>,
            &SharedContext,
            &Renderer,
        ) -> Vec<Object>,
    ) -> Self {
        let id = uuid::Uuid::new_v4().as_u128();

        Self {
            label: label.map(|s| s.to_string()),
            id,
            properties,
            contents,
            settings_cache: None,
            context_cache: DoubleSetCache::new(),
            redraw: true,
            update_widget_tree,
            compare,
            widget_event,
            size_calc,
            draw_range,
            cover_area,
            render,
        }
    }
}

// MARK: ContextCache

// stores tenfold width and height with integer.
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
struct ContextKey {
    x: Option<u32>,
    y: Option<u32>,
    tag: u64,
}

impl ContextKey {
    fn new(size: [Option<f32>; 2], tag: u64) -> Self {
        Self {
            x: size[0].map(|f| (f * 10.0) as u32),
            y: size[1].map(|f| (f * 10.0) as u32),
            tag,
        }
    }
}

struct ContextCacheData<T> {
    // actual size equals to texture size
    actual_size: [f32; 2],
    content_option_size: [Option<f32>; 2],
    // any cache
    any_cache: Option<T>,
    // rendering result
    objects: Option<Vec<Object>>,
}

impl<T> ContextCacheData<T> {
    fn new(actual_size: [f32; 2], content_option_size: [Option<f32>; 2]) -> Self {
        Self {
            actual_size,
            content_option_size,
            any_cache: None,
            objects: None,
        }
    }

    fn get_object_or_create_with<F>(&mut self, f: F) -> &mut Vec<Object>
    where
        F: FnOnce([f32; 2], [Option<f32>; 2], &mut Option<T>) -> Vec<Object>,
    {
        self.objects.get_or_insert_with(|| {
            f(
                self.actual_size,
                self.content_option_size,
                &mut self.any_cache,
            )
        })
    }
}

// MARK: Widget impl

impl<Properties, Contents, SettingsCache, ContextCache, T> Widget<T>
    for FrameworkNode<Properties, Contents, SettingsCache, ContextCache, T>
where
    Properties: 'static,
    Contents: 'static,
    SettingsCache: 'static,
    ContextCache: 'static,
    T: Send + 'static,
{
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn update_widget_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), ()> {
        if let Ok(redraw) = (self.update_widget_tree)(
            dom,
            &mut self.label,
            &mut self.properties,
            &mut self.contents,
            &mut self.settings_cache,
        ) {
            self.redraw = redraw;
            Ok(())
        } else {
            Err(())
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        (self.compare)(
            dom,
            self.label.as_deref(),
            &self.properties,
            &self.contents,
            self.settings_cache.as_ref(),
        )
    }

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> UiEventResult<T> {
        (self.widget_event)(event, parent_size, context)
    }

    fn px_size(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> [f32; 2] {
        let context_cache =
            self.context_cache
                .get_or_insert_with(ContextKey::new(parent_size, tag), frame, || {
                    let (actual_size, content_size) = (self.size_calc)(
                        parent_size,
                        context,
                        &self.properties,
                        &mut self.contents,
                        &mut self.settings_cache,
                    );

                    ContextCacheData::new(actual_size, content_size)
                });

        context_cache.actual_size
    }

    fn draw_range(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> Option<Range2D<f32>> {
        let context_cache =
            self.context_cache
                .get_or_insert_with(ContextKey::new(parent_size, tag), frame, || {
                    let (actual_size, content_size) = (self.size_calc)(
                        parent_size,
                        context,
                        &self.properties,
                        &mut self.contents,
                        &mut self.settings_cache,
                    );

                    ContextCacheData::new(actual_size, content_size)
                });

        let new_tag = hash_tag(tag, self.id);

        (self.draw_range)(
            context_cache.actual_size,
            context_cache.content_option_size,
            &mut context_cache.any_cache,
            &self.properties,
            &mut self.contents,
            &mut self.settings_cache,
        )
    }

    fn cover_area(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> Option<Range2D<f32>> {
        let context_cache =
            self.context_cache
                .get_or_insert_with(ContextKey::new(parent_size, tag), frame, || {
                    let (actual_size, content_size) = (self.size_calc)(
                        parent_size,
                        context,
                        &self.properties,
                        &mut self.contents,
                        &mut self.settings_cache,
                    );

                    ContextCacheData::new(actual_size, content_size)
                });

        (self.cover_area)(
            context_cache.actual_size,
            context_cache.content_option_size,
            &mut context_cache.any_cache,
            &self.properties,
            &mut self.contents,
            &mut self.settings_cache,
        )
    }

    fn redraw(&self) -> bool {
        self.redraw
    }

    fn render(
        &mut self,
        // ui environment
        parent_size: [Option<f32>; 2],
        background_view: &wgpu::TextureView,
        background_range: Range2D<f32>,
        // context
        shared_context: &SharedContext,
        renderer: &Renderer,
        tag: u64,
        frame: u64,
    ) -> Vec<Object> {
        let context_cache =
            self.context_cache
                .get_or_insert_with(ContextKey::new(parent_size, tag), frame, || {
                    let (actual_size, content_size) = (self.size_calc)(
                        parent_size,
                        shared_context,
                        &self.properties,
                        &mut self.contents,
                        &mut self.settings_cache,
                    );

                    ContextCacheData::new(actual_size, content_size)
                });

        let objects =
            context_cache.get_object_or_create_with(|actual_size, content_size, any_cache| {
                (self.render)(
                    // context
                    actual_size,
                    content_size,
                    any_cache,
                    // fields
                    &self.properties,
                    &mut self.contents,
                    &mut self.settings_cache,
                    // ui environment
                    background_view,
                    background_range,
                    shared_context,
                    renderer,
                )
            });

        objects.clone()
    }
}

// MARK: util

fn hash_tag(tag: u64, id: u128) -> u64 {
    let mut hasher = ahash::AHasher::default();
    tag.hash(&mut hasher);
    id.hash(&mut hasher);
    hasher.finish()
}
