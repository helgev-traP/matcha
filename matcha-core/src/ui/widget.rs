use std::any::Any;

use parking_lot::Mutex;
use utils::{back_prop_dirty::BackPropDirty, cache::Cache};

use crate::{
    Background, Constraints, DeviceEvent, RenderNode, UpdateNotifier, WidgetContext,
    ui::{Arrangement, metrics::LayoutSizeKey},
};

pub struct RebuildRequest<'a> {
    need_rearrange: &'a BackPropDirty,
    need_rerender: &'a BackPropDirty,
}

impl RebuildRequest<'_> {
    pub fn rearrange_next_frame(&self) {
        self.need_rearrange.mark_dirty();
        self.need_rerender.mark_dirty();
    }

    pub fn rerender_next_frame(&self) {
        self.need_rerender.mark_dirty();
    }
}

#[async_trait::async_trait]
pub trait Dom<E, ChildSetting = ()>: Sync + Any {
    fn id(&self) -> u128;

    fn children(&self) -> Vec<(&dyn Dom<E, ChildSetting>, ChildSetting)>;

    /// Builds the corresponding stateful `Widget` tree from this `Dom` node.
    fn build_widget_tree(&self) -> Box<dyn AnyWidgetFrame<E>>;

    /// Sets an `UpdateNotifier` for the `Dom` tree to listen for model updates.
    ///
    /// This method is crucial for the `Component` system to detect changes in the `Model`.
    /// `ComponentDom` uses this to receive the notifier.
    ///
    /// Custom `Dom` implementations that contain children (e.g., layout widgets)
    /// have the responsibility to recursively propagate this notifier to all their children.
    /// Failure to do so will prevent descendant `Component`s from detecting model updates.
    async fn set_update_notifier(&self, notifier: &UpdateNotifier);
}

pub trait Widget<D: Dom<E, ChildSetting>, E: 'static, ChildSetting: 'static = ()> {
    fn update_widget(&mut self, dom: &D, cache_invalidator: RebuildRequest);

    fn device_event(
        &mut self,
        bounds: [f32; 2],
        event: &DeviceEvent,
        children: &[(&mut dyn AnyWidget<E>, &mut ChildSetting, &Arrangement)],
        cache_invalidator: RebuildRequest,
        ctx: &WidgetContext,
    ) -> Option<E>;

    fn is_inside(
        &self,
        bounds: [f32; 2],
        position: [f32; 2],
        children: &[(&dyn AnyWidget<E>, &ChildSetting, &Arrangement)],
        ctx: &WidgetContext,
    ) -> bool;

    fn measure<'a>(
        &self,
        constraints: &Constraints,
        children: &[(&dyn AnyWidget<E>, &ChildSetting)],
        ctx: &WidgetContext,
    ) -> [f32; 2];

    fn arrange(
        &self,
        size: [f32; 2],
        children: &[(&dyn AnyWidget<E>, &ChildSetting)],
        ctx: &WidgetContext,
    ) -> Vec<Arrangement>;

    fn render(
        &self,
        background: Background,
        children: &[(&dyn AnyWidget<E>, &ChildSetting, &Arrangement)],
        ctx: &WidgetContext,
    ) -> RenderNode;
}

/// Make trait object that can be used from widget implement.
pub trait AnyWidget<E: 'static> {
    fn device_event(&mut self, event: &DeviceEvent, ctx: &WidgetContext) -> Option<E>;

    fn is_inside(&self, position: [f32; 2], ctx: &WidgetContext) -> bool;

    fn measure(&self, constraints: &Constraints, ctx: &WidgetContext) -> [f32; 2];

    fn render(
        &self,
        final_size: [f32; 2],
        background: Background,
        ctx: &WidgetContext,
    ) -> RenderNode;
}

/// Make it impossible to call arrange from outside the widget frame.
/// Ensure render() will be called after arrange() in this module.
pub(super) trait AnyWidgetFramePrivate {
    fn arrange(&self, final_size: [f32; 2], ctx: &WidgetContext);
}

/// Methods that Widget implementor should not use.
#[async_trait::async_trait]
pub(crate) trait AnyWidgetFrame<E: 'static>:
    AnyWidget<E> + AnyWidgetFramePrivate + std::any::Any + Send
{
    fn label(&self) -> Option<&str>;

    fn need_rerender(&self) -> bool;

    async fn update_widget_tree(&mut self, dom: &dyn Any) -> Result<(), UpdateWidgetError>;

    fn update_gpu_device(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);
}

/// Represents an error that can occur when updating a `Widget` tree.
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateWidgetError {
    /// Occurs when the type of the new `Dom` node does not match the existing `Widget`.
    TypeMismatch,
}

pub(crate) struct WidgetFrame<D, W, E, ChildSetting = ()>
where
    D: Dom<E, ChildSetting> + Send + Sync + 'static,
    W: Widget<D, E, ChildSetting> + Send + Sync + 'static,
    E: 'static,
    ChildSetting: Send + Sync + PartialEq + Clone + 'static,
{
    label: Option<String>,
    /// children it self and its settings and arrangement (if cache valid).
    children: Vec<(Box<dyn AnyWidgetFrame<E>>, ChildSetting)>,

    /// child ids. keep same order as children.
    // we separate child ids from their settings and arrangement because they are used independently.
    children_id: Vec<u128>, // hash

    // dirty flags
    need_rearrange: BackPropDirty,
    need_rerender: BackPropDirty,

    /// cache
    cache: Mutex<WidgetFrameCache>,

    /// impl the widget process.
    widget_impl: W,
    _dom_type: std::marker::PhantomData<D>,
}

struct WidgetFrameCache {
    /// cache the output of measure method.
    measure: Cache<Constraints, [f32; 2]>,
    /// cache the output of layout method.
    layout: Cache<LayoutSizeKey, Vec<Arrangement>>,
    /// cache the output of render method.
    /// `render_cache` existing ensures that this widget already rendered at least once.
    /// this means we can use the cached `Arrangement` to process device_event.
    render: Option<RenderNode>,
}

impl<D, W, T, ChildSetting> AnyWidget<T> for WidgetFrame<D, W, T, ChildSetting>
where
    D: Dom<T, ChildSetting> + Send + Sync + 'static,
    W: Widget<D, T, ChildSetting> + Send + Sync + 'static,
    T: 'static,
    ChildSetting: Send + Sync + PartialEq + Clone + 'static,
{
    fn device_event(&mut self, event: &DeviceEvent, ctx: &WidgetContext) -> Option<T> {
        let cache = self.cache.lock();

        let Some(_render_node) = cache.render.as_ref() else {
            // not rendered yet, cannot process event.
            return None;
        };
        let Some((&actual_bounds, arrangement)) = cache.layout.get() else {
            // this should not happen
            debug_assert!(false, "render_cache exists but layout_cache does not");
            return None;
        };

        let actual_bounds: [f32; 2] = actual_bounds.into();

        let children_with_arrangement: Vec<(
            &mut dyn AnyWidget<T>,
            &mut ChildSetting,
            &Arrangement,
        )> = self
            .children
            .iter_mut()
            .zip(arrangement.iter())
            .map(|((child, setting), arr)| (&mut **child as &mut dyn AnyWidget<T>, setting, arr))
            .collect();

        let event = self.widget_impl.device_event(
            actual_bounds,
            event,
            &children_with_arrangement,
            RebuildRequest {
                need_rearrange: &self.need_rearrange,
                need_rerender: &self.need_rerender,
            },
            ctx,
        );

        event
    }

    fn is_inside(&self, position: [f32; 2], ctx: &WidgetContext) -> bool {
        let cache = self.cache.lock();

        let Some(render_node) = cache.render.as_ref() else {
            // not rendered yet, cannot process event.
            return false;
        };
        let Some((&actual_bounds, arrangement)) = cache.layout.get() else {
            // this should not happen
            debug_assert!(false, "render_cache exists but layout_cache does not");
            return false;
        };

        let actual_bounds: [f32; 2] = actual_bounds.into();

        let children = self
            .children
            .iter()
            .map(|(child, setting)| (&**child as &dyn AnyWidget<T>, setting))
            .collect::<Vec<_>>();

        self.widget_impl.is_inside(
            actual_bounds,
            position,
            &children
                .iter()
                .zip(arrangement)
                .map(|((c, s), a)| (*c, *s, a))
                .collect::<Vec<_>>(),
            ctx,
        )
    }

    fn measure(&self, constraints: &Constraints, ctx: &WidgetContext) -> [f32; 2] {
        let mut cache = self.cache.lock();

        // clear measure cache if rearrange is needed
        if self.need_rearrange.take_dirty() {
            cache.measure.clear();
            // we cannot partially ensure both arrange() and measure() to be called so we need to clear both caches.
            cache.layout.clear();
        }

        let children = self
            .children
            .iter()
            .map(|(child, setting)| (&**child as &dyn AnyWidget<T>, setting))
            .collect::<Vec<_>>();

        let (_, size) = cache.measure.get_or_insert_with(*constraints, || {
            self.widget_impl.measure(constraints, &children, ctx)
        });

        *size
    }

    fn render(
        &self,
        final_size: [f32; 2],
        background: Background,
        ctx: &WidgetContext,
    ) -> RenderNode {
        // arrange must be called before locking the cache to avoid deadlocks.
        self.arrange(final_size, ctx);

        let mut cache = self.cache.lock();
        let (_, arrangement) = cache.layout.get().expect("infallible: arrange just called");

        // check render cache
        if self.need_rerender.take_dirty() {
            cache.render = None;
        }

        let render_node = cache.render.get_or_insert_with(|| {
            let children = self
                .children
                .iter()
                .map(|(child, setting)| (&**child as &dyn AnyWidget<T>, setting))
                .collect::<Vec<_>>();

            self.widget_impl.render(
                background,
                &children
                    .iter()
                    .zip(arrangement)
                    .map(|((c, s), a)| (*c, *s, a))
                    .collect::<Vec<_>>(),
                ctx,
            )
        });

        render_node.clone()
    }
}

impl<D, W, T, ChildSetting> AnyWidgetFramePrivate for WidgetFrame<D, W, T, ChildSetting>
where
    D: Dom<T, ChildSetting> + Send + Sync + 'static,
    W: Widget<D, T, ChildSetting> + Send + Sync + 'static,
    T: 'static,
    ChildSetting: Send + Sync + PartialEq + Clone + 'static,
{
    fn arrange(&self, final_size: [f32; 2], ctx: &WidgetContext) {
        let mut cache = self.cache.lock();

        if self.need_rearrange.take_dirty() {
            // arrangement changed, need to re-render
            cache.measure.clear();
            cache.layout.clear();
            cache.render = None;
        }
        cache
            .layout
            .get_or_insert_with(LayoutSizeKey::from(final_size), || {
                // calc arrangement
                let children = self
                    .children
                    .iter()
                    .map(|(child, setting)| (&**child as &dyn AnyWidget<T>, setting))
                    .collect::<Vec<_>>();
                let arrangement = self.widget_impl.arrange(final_size, &children, ctx);
                // update child arrangements
                for ((child, _), arrangement) in self.children.iter_mut().zip(arrangement.iter()) {
                    child.arrange(arrangement.size, ctx);
                }
                arrangement
            });
    }
}

#[async_trait::async_trait]
impl<D, W, T, ChildSetting> AnyWidgetFrame<T> for WidgetFrame<D, W, T, ChildSetting>
where
    D: Dom<T, ChildSetting> + Send + Sync + 'static,
    W: Widget<D, T, ChildSetting> + Send + Sync + 'static,
    T: 'static,
    ChildSetting: Send + Sync + PartialEq + Clone + 'static,
{
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn need_rerender(&self) -> bool {
        self.need_rerender.is_dirty()
    }

    async fn update_widget_tree(&mut self, dom: &dyn Any) -> Result<(), UpdateWidgetError> {
        // check type
        let Some(dom) = (dom as &dyn Any).downcast_ref::<D>() else {
            return Err(UpdateWidgetError::TypeMismatch);
        };

        // update widget
        self.widget_impl.update_widget(
            dom,
            RebuildRequest {
                need_rearrange: &self.need_rearrange,
                need_rerender: &self.need_rerender,
            },
        );

        let new_children_dom = dom.children();
        let mut arrangement_changed = false;

        // Create a map of old children for efficient lookup
        let mut old_children_map: std::collections::HashMap<
            u128,
            (Box<dyn AnyWidgetFrame<T>>, ChildSetting),
        > = self
            .children_id
            .iter()
            .cloned()
            .zip(std::mem::take(&mut self.children))
            .collect();

        let mut new_children_vec: Vec<(Box<dyn AnyWidgetFrame<T>>, ChildSetting)> =
            Vec::with_capacity(new_children_dom.len());
        let mut new_children_id_vec: Vec<u128> = Vec::with_capacity(new_children_dom.len());

        for (i, (child_dom, new_setting)) in new_children_dom.iter().enumerate() {
            let child_id = child_dom.id();
            new_children_id_vec.push(child_id);

            if let Some((mut child, old_setting)) = old_children_map.remove(&child_id) {
                // Reuse existing child

                // Check if order changed
                if self.children_id.get(i) != Some(&child_id) {
                    arrangement_changed = true;
                }

                // Check if setting changed
                if &old_setting != new_setting {
                    arrangement_changed = true;
                }

                child.update_widget_tree(*child_dom).await?;
                new_children_vec.push((child, new_setting.clone()));
            } else {
                // Create new child
                arrangement_changed = true;
                let mut child = child_dom.build_widget_tree();
                child.update_widget_tree(*child_dom).await?;
                new_children_vec.push((child, new_setting.clone()));
            }
        }

        // If there are remaining children in the map, they were removed.
        if !old_children_map.is_empty() {
            arrangement_changed = true;
        }

        // If the number of children changed.
        if self.children_id.len() != new_children_id_vec.len() {
            arrangement_changed = true;
        }

        if arrangement_changed {
            self.need_rearrange.mark_dirty();
        }

        self.children = new_children_vec;
        self.children_id = new_children_id_vec;

        Ok(())
    }

    fn update_gpu_device(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        // 何らかの理由によりGPU論理デバイスが変更になったときのためのリソース再確保用のメソッド
        // todo
        for (child, _) in &mut self.children {
            child.update_gpu_device(device, queue);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Constraints, DeviceEvent, RenderNode, UpdateNotifier};
    use utils::back_prop_dirty::BackPropDirty;

    #[derive(Debug, Clone, PartialEq)]
    struct MockSetting {
        value: i32,
    }

    impl Default for MockSetting {
        fn default() -> Self {
            Self { value: 0 }
        }
    }

    struct MockDom {
        id: u128,
        children: Vec<(MockDom, MockSetting)>,
    }

    #[async_trait::async_trait]
    impl Dom<String, MockSetting> for MockDom {
        fn id(&self) -> u128 {
            self.id
        }

        fn children(&self) -> Vec<(&dyn Dom<String, MockSetting>, MockSetting)> {
            self.children
                .iter()
                .map(|(dom, setting)| (dom as &dyn Dom<String, MockSetting>, setting.clone()))
                .collect()
        }

        fn build_widget_tree(&self) -> Box<dyn AnyWidgetFrame<String>> {
            Box::new(WidgetFrame {
                label: None,
                children: self
                    .children
                    .iter()
                    .map(|(child, setting)| (child.build_widget_tree(), setting.clone()))
                    .collect(),
                children_id: self.children.iter().map(|(c, _)| c.id()).collect(),
                need_rearrange: BackPropDirty::new(),
                need_rerender: BackPropDirty::new(),
                cache: Mutex::new(WidgetFrameCache {
                    measure: Cache::new(),
                    layout: Cache::new(),
                    render: None,
                }),
                widget_impl: MockWidget,
                _dom_type: std::marker::PhantomData,
            })
        }

        async fn set_update_notifier(&self, _notifier: &UpdateNotifier) {}
    }

    struct MockWidget;

    impl Widget<MockDom, String, MockSetting> for MockWidget {
        fn update_widget(&mut self, _dom: &MockDom, _cache_invalidator: RebuildRequest) {}

        fn device_event(
            &mut self,
            _bounds: [f32; 2],
            _event: &DeviceEvent,
            _children: &[(&mut dyn AnyWidget<String>, &mut MockSetting, &Arrangement)],
            _cache_invalidator: RebuildRequest,
            _ctx: &WidgetContext,
        ) -> Option<String> {
            None
        }

        fn is_inside(
            &self,
            _bounds: [f32; 2],
            _position: [f32; 2],
            _children: &[(&dyn AnyWidget<String>, &MockSetting, &Arrangement)],
            _ctx: &WidgetContext,
        ) -> bool {
            true
        }

        fn measure(
            &self,
            _constraints: &Constraints,
            _children: &[(&dyn AnyWidget<String>, &MockSetting)],
            _ctx: &WidgetContext,
        ) -> [f32; 2] {
            [100.0, 100.0]
        }

        fn arrange(
            &self,
            _size: [f32; 2],
            _children: &[(&dyn AnyWidget<String>, &MockSetting)],
            _ctx: &WidgetContext,
        ) -> Vec<Arrangement> {
            vec![]
        }

        fn render(
            &self,
            _background: Background,
            _children: &[(&dyn AnyWidget<String>, &MockSetting, &Arrangement)],
            _ctx: &WidgetContext,
        ) -> RenderNode {
            RenderNode::default()
        }
    }

    type MockWidgetFrame = WidgetFrame<MockDom, MockWidget, String, MockSetting>;

    #[tokio::test]
    async fn test_update_no_change() {
        let initial_dom = MockDom {
            id: 0,
            children: vec![
                (
                    MockDom {
                        id: 1,
                        children: vec![],
                    },
                    MockSetting { value: 10 },
                ),
                (
                    MockDom {
                        id: 2,
                        children: vec![],
                    },
                    MockSetting { value: 20 },
                ),
            ],
        };

        let mut widget_frame: Box<dyn AnyWidgetFrame<String>> = initial_dom.build_widget_tree();

        let widget_frame_concrete = (&mut *widget_frame as &mut dyn Any)
            .downcast_mut::<MockWidgetFrame>()
            .unwrap();
        assert_eq!(widget_frame_concrete.children.len(), 2);
        assert_eq!(widget_frame_concrete.children_id, vec![1, 2]);
        assert!(!widget_frame_concrete.need_rearrange.is_dirty());

        // Update with the same DOM
        let updated_dom = MockDom {
            id: 0,
            children: vec![
                (
                    MockDom {
                        id: 1,
                        children: vec![],
                    },
                    MockSetting { value: 10 },
                ),
                (
                    MockDom {
                        id: 2,
                        children: vec![],
                    },
                    MockSetting { value: 20 },
                ),
            ],
        };

        widget_frame
            .update_widget_tree(&updated_dom as &dyn Any)
            .await
            .unwrap();

        let widget_frame_concrete = (&mut *widget_frame as &mut dyn Any)
            .downcast_mut::<MockWidgetFrame>()
            .unwrap();
        assert_eq!(widget_frame_concrete.children.len(), 2);
        assert_eq!(widget_frame_concrete.children_id, vec![1, 2]);
        // No change, so rearrange should not be needed.
        assert!(!widget_frame_concrete.need_rearrange.is_dirty());
    }

    #[tokio::test]
    async fn test_update_add_child() {
        let initial_dom = MockDom {
            id: 0,
            children: vec![(
                MockDom {
                    id: 1,
                    children: vec![],
                },
                MockSetting { value: 10 },
            )],
        };

        let mut widget_frame: Box<dyn AnyWidgetFrame<String>> = initial_dom.build_widget_tree();

        // Update with a new child added
        let updated_dom = MockDom {
            id: 0,
            children: vec![
                (
                    MockDom {
                        id: 1,
                        children: vec![],
                    },
                    MockSetting { value: 10 },
                ),
                (
                    MockDom {
                        id: 2,
                        children: vec![],
                    },
                    MockSetting { value: 20 },
                ),
            ],
        };

        widget_frame
            .update_widget_tree(&updated_dom as &dyn Any)
            .await
            .unwrap();

        let widget_frame_concrete = (&mut *widget_frame as &mut dyn Any)
            .downcast_mut::<MockWidgetFrame>()
            .unwrap();
        assert_eq!(widget_frame_concrete.children.len(), 2);
        assert_eq!(widget_frame_concrete.children_id, vec![1, 2]);
        assert!(widget_frame_concrete.need_rearrange.is_dirty());
    }

    #[tokio::test]
    async fn test_update_remove_child() {
        let initial_dom = MockDom {
            id: 0,
            children: vec![
                (
                    MockDom {
                        id: 1,
                        children: vec![],
                    },
                    MockSetting { value: 10 },
                ),
                (
                    MockDom {
                        id: 2,
                        children: vec![],
                    },
                    MockSetting { value: 20 },
                ),
            ],
        };

        let mut widget_frame: Box<dyn AnyWidgetFrame<String>> = initial_dom.build_widget_tree();

        // Update with a child removed
        let updated_dom = MockDom {
            id: 0,
            children: vec![(
                MockDom {
                    id: 1,
                    children: vec![],
                },
                MockSetting { value: 10 },
            )],
        };

        widget_frame
            .update_widget_tree(&updated_dom as &dyn Any)
            .await
            .unwrap();

        let widget_frame_concrete = (&mut *widget_frame as &mut dyn Any)
            .downcast_mut::<MockWidgetFrame>()
            .unwrap();
        assert_eq!(widget_frame_concrete.children.len(), 1);
        assert_eq!(widget_frame_concrete.children_id, vec![1]);
        assert!(widget_frame_concrete.need_rearrange.is_dirty());
    }

    #[tokio::test]
    async fn test_update_reorder_children() {
        let initial_dom = MockDom {
            id: 0,
            children: vec![
                (
                    MockDom {
                        id: 1,
                        children: vec![],
                    },
                    MockSetting { value: 10 },
                ),
                (
                    MockDom {
                        id: 2,
                        children: vec![],
                    },
                    MockSetting { value: 20 },
                ),
            ],
        };

        let mut widget_frame: Box<dyn AnyWidgetFrame<String>> = initial_dom.build_widget_tree();

        // Update with children reordered
        let updated_dom = MockDom {
            id: 0,
            children: vec![
                (
                    MockDom {
                        id: 2,
                        children: vec![],
                    },
                    MockSetting { value: 20 },
                ),
                (
                    MockDom {
                        id: 1,
                        children: vec![],
                    },
                    MockSetting { value: 10 },
                ),
            ],
        };

        widget_frame
            .update_widget_tree(&updated_dom as &dyn Any)
            .await
            .unwrap();

        let widget_frame_concrete = (&mut *widget_frame as &mut dyn Any)
            .downcast_mut::<MockWidgetFrame>()
            .unwrap();
        assert_eq!(widget_frame_concrete.children.len(), 2);
        assert_eq!(widget_frame_concrete.children_id, vec![2, 1]);
        assert!(widget_frame_concrete.need_rearrange.is_dirty());
    }

    #[tokio::test]
    async fn test_update_change_setting() {
        let initial_dom = MockDom {
            id: 0,
            children: vec![(
                MockDom {
                    id: 1,
                    children: vec![],
                },
                MockSetting { value: 10 },
            )],
        };

        let mut widget_frame: Box<dyn AnyWidgetFrame<String>> = initial_dom.build_widget_tree();
        let widget_frame_concrete = (&mut *widget_frame as &mut dyn Any)
            .downcast_mut::<MockWidgetFrame>()
            .unwrap();
        assert!(!widget_frame_concrete.need_rearrange.is_dirty());

        // Update with setting changed
        let updated_dom = MockDom {
            id: 0,
            children: vec![(
                MockDom {
                    id: 1,
                    children: vec![],
                },
                MockSetting { value: 99 }, // Changed value
            )],
        };

        widget_frame
            .update_widget_tree(&updated_dom as &dyn Any)
            .await
            .unwrap();

        let widget_frame_concrete = (&mut *widget_frame as &mut dyn Any)
            .downcast_mut::<MockWidgetFrame>()
            .unwrap();
        assert_eq!(widget_frame_concrete.children.len(), 1);
        assert_eq!(widget_frame_concrete.children_id, vec![1]);
        let (_, setting) = &widget_frame_concrete.children[0];
        assert_eq!(setting.value, 99);
        assert!(widget_frame_concrete.need_rearrange.is_dirty());
    }
}
