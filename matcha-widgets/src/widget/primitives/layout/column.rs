use std::sync::Arc;

use matcha_core::{
    context::WidgetContext,
    events::Event,
    observer::Observer,
    renderer::Renderer,
    types::range::{CoverRange, Range2D},
    ui::{Background, Dom, DomComPareResult, Object, UpdateWidgetError, Widget},
};

use crate::widget::primitives::property::flex::{AlignItems, JustifyContent};

// MARK: DOM

pub struct Column<T> {
    pub label: Option<String>,

    pub justify_content: JustifyContent,
    pub align_items: AlignItems,

    pub items: Vec<Box<dyn Dom<T>>>,
}

impl<T> Column<T> {
    pub fn new(label: Option<&str>) -> Box<Self> {
        Box::new(Self {
            label: label.map(String::from),
            justify_content: JustifyContent::FlexStart {
                gap: Arc::new(|_, _| 0.0),
            },
            align_items: AlignItems::Start,
            items: Vec::new(),
        })
    }
}

#[async_trait::async_trait]
impl<T: Send + 'static> Dom<T> for Column<T> {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(ColumnNode {
            label: self.label.clone(),
            justify_content: self.justify_content.clone(),
            align_items: self.align_items,
            items: self
                .items
                .iter()
                .map(|item| item.build_widget_tree())
                .collect(),
            cache: None,
        })
    }

    async fn collect_observer(&self) -> Observer {
        use futures::future::join_all;

        let observers = join_all(self.items.iter().map(|dom| dom.collect_observer())).await;
        observers
            .into_iter()
            .fold(Observer::default(), |obs, o| obs.join(o))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        // Do not change this.
        self
    }
}

// MARK: Widget

pub struct ColumnNode<T> {
    label: Option<String>,

    justify_content: JustifyContent,
    align_items: AlignItems,

    items: Vec<Box<dyn Widget<T>>>,

    cache: Option<(CacheKey, CacheData)>,
}

// MARK: Cache

/// stores tenfold width and height with integer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    size: [Option<u32>; 2],
}

impl CacheKey {
    fn new(size: [Option<f32>; 2]) -> Self {
        Self {
            size: [
                size[0].map(|f| (f * 10.0) as u32),
                size[1].map(|f| (f * 10.0) as u32),
            ],
        }
    }
}

// MARK: CacheData

struct CacheData {
    column_size: [f32; 2],
    content_position: Vec<[f32; 2]>,
}

// MARK: Cache preparation

impl<T> ColumnNode<T> {
    fn invalidate_cache(&mut self, parent_size: [Option<f32>; 2]) {
        let current_tag = CacheKey::new(parent_size);

        if let Some((tag, _)) = self.cache.as_ref() {
            if *tag == current_tag {
                // Cache is valid, no need to invalidate
                return;
            }
        }

        // Invalidate cache
        self.cache = None;
    }

    fn prepare_cache(&mut self, parent_size: [Option<f32>; 2], context: &WidgetContext) {
        self.invalidate_cache(parent_size);

        if self.cache.is_some() {
            // Cache is valid, no need to recalculate
            return;
        }
        let current_tag = CacheKey::new(parent_size);

        // check tag

        if let Some((tag, _)) = self.cache.as_ref() {
            if *tag == current_tag {
                // hit -> do nothing
                return;
            }
        }

        // calculate actual size

        let mut max_width = 0.0f32; // use later
        let mut acc_height = 0.0f32;

        let actual_size = match parent_size {
            // when actual size not need calculate
            [Some(width), Some(height)] => [width, height],
            _ => {
                for item in self.items.iter_mut() {
                    let size = item.px_size(parent_size, context);
                    max_width = max_width.max(size[0]);
                    acc_height += size[1];
                }

                [
                    parent_size[0].unwrap_or(max_width),
                    parent_size[1].unwrap_or(acc_height),
                ]
            }
        };

        // calculate position of items

        let items_len = self.items.len() as f32;
        let margin_all = (actual_size[1] - acc_height).max(0.0);
        let mut positions = vec![[0.0f32, 0.0f32]; self.items.len()];

        // main axis positioning (vertical for column)
        let mut horizontal_fold_fn: Box<dyn StateClosureTrait> = justify_content_horizontal_fold_fn(
            margin_all,
            items_len,
            &self.justify_content,
            parent_size,
            context,
        );

        // cross axis positioning (horizontal for column)
        let mut vertical_fold_fn: Box<dyn StateClosureTrait> = match self.align_items {
            AlignItems::Start => Box::new(StateClosure {
                acc: 0.0,
                func: |_, _| 0.0,
            }),
            AlignItems::Center => Box::new(StateClosure {
                acc: max_width / 2.0,
                func: |_, w: f32| (max_width - w) / 2.0,
            }),
            AlignItems::End => Box::new(StateClosure {
                acc: max_width,
                func: |_, w: f32| max_width - w,
            }),
        };

        // fill positions
        for (item, position) in self.items.iter_mut().zip(positions.iter_mut()) {
            let size = item.px_size(parent_size, context);
            position[1] = horizontal_fold_fn.next(size[1]);
            position[0] = vertical_fold_fn.next(size[0]);
        }

        // cache data
        let cache_data = CacheData {
            column_size: actual_size,
            content_position: positions,
        };

        self.cache = Some((CacheKey::new(parent_size), cache_data));

        // end
    }
}

fn justify_content_horizontal_fold_fn(
    margin_all: f32,
    items_len: f32,
    justify_content: &JustifyContent,
    parent_size: [Option<f32>; 2],
    context: &WidgetContext,
) -> Box<dyn StateClosureTrait> {
    match justify_content {
        JustifyContent::FlexStart { gap } => {
            let gap = gap(parent_size[1], context);
            Box::new(StateClosure {
                acc: 0.0,
                func: move |acc: &mut f32, next: f32| {
                    let r = *acc;
                    *acc += next + gap;
                    r
                },
            })
        }
        JustifyContent::FlexEnd { gap } => {
            let gap = gap(parent_size[1], context);
            Box::new(StateClosure {
                acc: margin_all - gap * (items_len - 1.0),
                func: move |acc: &mut f32, next: f32| {
                    let r = *acc;
                    *acc += next + gap;
                    r
                },
            })
        }
        JustifyContent::Center { gap } => {
            let gap = gap(parent_size[1], context);
            Box::new(StateClosure {
                acc: (margin_all - gap * (items_len - 1.0)) / 2.0,
                func: move |acc: &mut f32, next: f32| {
                    let r = *acc;
                    *acc += next + gap;
                    r
                },
            })
        }
        JustifyContent::SpaceBetween => {
            let gap = margin_all / (items_len - 1.0);
            Box::new(StateClosure {
                acc: 0.0,
                func: move |acc: &mut f32, next: f32| {
                    let r = *acc;
                    *acc += next + gap;
                    r
                },
            })
        }
        JustifyContent::SpaceAround => {
            let gap = margin_all / items_len;
            Box::new(StateClosure {
                acc: gap / 2.0,
                func: move |acc: &mut f32, next: f32| {
                    let r = *acc;
                    *acc += next + gap;
                    r
                },
            })
        }
        JustifyContent::SpaceEvenly => {
            let gap = margin_all / (items_len + 1.0);
            Box::new(StateClosure {
                acc: gap,
                func: move |acc: &mut f32, next: f32| {
                    let r = *acc;
                    *acc += next + gap;
                    r
                },
            })
        }
    }
}

// MARK: Widget trait

#[async_trait::async_trait]
impl<T: Send + 'static> Widget<T> for ColumnNode<T> {
    // label
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    // for dom handling
    // keep in mind to change redraw flag to true if some change is made.
    async fn update_widget_tree(
        &mut self,
        component_updated: bool,
        dom: &dyn Dom<T>,
    ) -> Result<(), UpdateWidgetError> {
        if let Some(dom) = dom.as_any().downcast_ref::<Column<T>>() {
            todo!()
        } else {
            return Err(UpdateWidgetError::TypeMismatch);
        }
    }

    // comparing dom
    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(dom) = dom.as_any().downcast_ref::<Column<T>>() {
            todo!()
        } else {
            DomComPareResult::Different
        }
    }

    // widget event
    fn widget_event(
        &mut self,
        event: &Event,
        parent_size: [Option<f32>; 2],
        context: &WidgetContext,
    ) -> Option<T> {
        let _ = (event, parent_size, context);
        todo!()
    }

    // inside / outside check
    // implement this if your widget has a non rectangular shape.
    /*
    fn is_inside(
        &mut self,
        position: [f32; 2],
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
    ) -> bool {
        let px_size = Widget::<T>::px_size(self, parent_size, context);

        !(position[0] < 0.0
            || position[0] > px_size[0]
            || position[1] < 0.0
            || position[1] > px_size[1])
    }
    */

    // Actual size including its sub widgets with pixel value.
    fn px_size(&mut self, parent_size: [Option<f32>; 2], context: &WidgetContext) -> [f32; 2] {
        // prepare cache
        self.prepare_cache(parent_size, context);
        // get cache
        self.cache
            .as_ref()
            .map(|(_, cache)| cache.column_size)
            .expect("unreachable!")
    }

    // The drawing range and the area that the widget always covers.
    fn cover_range(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &WidgetContext,
    ) -> CoverRange<f32> {
        todo!()
    }

    // if redraw is needed
    fn redraw(&self) -> bool {
        todo!()
    }

    // render
    fn render(
        &mut self,
        // ui environment
        parent_size: [Option<f32>; 2],
        background: Background,
        // context
        context: &WidgetContext,
        renderer: &Renderer,
    ) -> Vec<Object> {
        // prepare cache
        self.prepare_cache(parent_size, context);
        // get cache
        let (_, cache) = self.cache.as_ref().expect("unreachable!");

        // render children
        let mut objects = Vec::new();
        for (item, position) in self.items.iter_mut().zip(cache.content_position.iter()) {
            let item_objects = item.render(
                parent_size,
                background.transition(*position),
                context,
                renderer,
            );
            for mut object in item_objects {
                object.transform(nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(
                    position[0],
                    position[1],
                    0.0,
                )));
                objects.push(object);
            }
        }

        objects
    }
}

// MARK: utility

struct StateClosure<F>
where
    F: Fn(&mut f32, f32) -> f32,
{
    acc: f32,
    func: F,
}

trait StateClosureTrait {
    fn next(&mut self, next: f32) -> f32;
}

impl<F> StateClosureTrait for StateClosure<F>
where
    F: Fn(&mut f32, f32) -> f32,
{
    fn next(&mut self, next: f32) -> f32 {
        (self.func)(&mut self.acc, next)
    }
}
