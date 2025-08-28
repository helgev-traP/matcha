use std::any::Any;

use parking_lot::Mutex;
use renderer::render_node::RenderNode;
use utils::{back_prop_dirty::BackPropDirty, cache::Cache};

use crate::{
    Background, Constraints, DeviceEvent, UpdateNotifier, WidgetContext,
    ui::{Arrangement, metrics::LayoutSizeKey},
};

/// Lightweight handle passed into widget update / event handlers allowing them
/// to request layout or visual invalidation without touching internal caches.
///
/// Semantics:
/// - `rearrange_next_frame()`: marks both layout(measure/arrange) + redraw dirty.
///   Use when child structure, ordering, or any layout-affecting setting changes.
/// - `redraw_next_frame()`: marks only visual redraw (layout cache remains). Use for pure
///   style / animation / time-based visual changes that do not affect geometry.
/// - Requests are frame-scoped: flags are consumed internally during `measure/arrange`
///   (layout) or after a successful `render` (redraw).
/// - Do NOT store this handle beyond the synchronous call; it borrows internal flags.
///
/// Future:
/// - Could evolve into an enum-based invalidation or generation counter if finer
///   granularity / statistics are needed.
/// - Async widgets: if future async tasks need to trigger invalidation, they will
///   use a cloneable channel-based handle instead of this borrowed form.
pub struct RebuildRequest<'a> {
    need_rearrange: &'a BackPropDirty,
    need_redraw: &'a BackPropDirty,
}

impl<'a> RebuildRequest<'a> {
    fn new(need_rearrange: &'a BackPropDirty, need_redraw: &'a BackPropDirty) -> Self {
        Self {
            need_rearrange,
            need_redraw,
        }
    }

    pub fn rearrange_next_frame(&self) {
        self.need_rearrange.mark_dirty();
        self.need_redraw.mark_dirty();
    }

    pub fn redraw_next_frame(&self) {
        self.need_redraw.mark_dirty();
    }
}

#[async_trait::async_trait]
pub trait Dom<E>: Sync + Any {
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

pub trait Widget<D: Dom<E>, E: 'static = (), ChildSetting: PartialEq + 'static = ()> {
    /// Returns the children of this `Dom` node.
    /// vector values are tuples of (child, setting, id).
    fn update_widget(
        &mut self,
        dom: &D,
        cache_invalidator: RebuildRequest,
    ) -> Vec<(&dyn Dom<E>, ChildSetting, u128)>;

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

    fn measure(
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
///
/// Publish this trait to `super` to allow components can implement it.
pub(super) trait AnyWidgetFramePrivate {
    fn arrange(&self, final_size: [f32; 2], ctx: &WidgetContext);
}

/// Methods that Widget implementor should not use.
// AnyWidgetFramePrivate is intended for use only within this module.
// Making this trait private prevents external code from implementing
// AnyWidgetFrame for arbitrary types.
#[allow(private_bounds)]
#[async_trait::async_trait]
pub trait AnyWidgetFrame<E: 'static>:
    AnyWidget<E> + AnyWidgetFramePrivate + std::any::Any + Send
{
    fn label(&self) -> Option<&str>;

    fn need_redraw(&self) -> bool;

    async fn update_widget_tree(&mut self, dom: &dyn Dom<E>) -> Result<(), UpdateWidgetError>;

    fn update_dirty_flags(&mut self, rearrange_flags: BackPropDirty, redraw_flags: BackPropDirty);

    fn update_gpu_device(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);
}

/// Represents an error that can occur when updating a `Widget` tree.
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateWidgetError {
    /// Occurs when the type of the new `Dom` node does not match the existing `Widget`.
    TypeMismatch,
}

pub struct WidgetFrame<D, W, E, ChildSetting = ()>
where
    D: Dom<E> + Send + Sync + 'static,
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
    need_redraw: BackPropDirty,

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
}

impl<D, W, T, ChildSetting> AnyWidget<T> for WidgetFrame<D, W, T, ChildSetting>
where
    D: Dom<T> + Send + Sync + 'static,
    W: Widget<D, T, ChildSetting> + Send + Sync + 'static,
    T: 'static,
    ChildSetting: Send + Sync + PartialEq + Clone + 'static,
{
    fn device_event(&mut self, event: &DeviceEvent, ctx: &WidgetContext) -> Option<T> {
        let cache = self.cache.lock();

        let Some((&actual_bounds, arrangement)) = cache.layout.get() else {
            // not rendered yet, cannot process event.
            return None;
        };

        let actual_bounds: [f32; 2] = actual_bounds.into();

        // PERF NOTE (hot path):
        // We allocate a Vec each event to pair children with their Arrangement.
        // For deep or high-frequency event streams (mouse move, drag), this can add pressure.
        // Future optimization options (measured via profiler before changing):
        //   1. smallvec::SmallVec<[(&mut dyn AnyWidget<T>, &mut ChildSetting, &Arrangement); 8]>
        //      to stack-allocate typical small child counts.
        //   2. Precompute a stable temporary scratch buffer reused per frame (unsafe careful).
        //   3. Pass iterator of zipped triplets into widget_impl.device_event to let caller
        //      iterate without intermediate collection.
        //   4. Maintain arrangement indices and let widget_impl pull lazily.
        // Start with SmallVec if profiling shows >5% frame time here.
        // PERF NOTE (hit-test path):
        // Similar allocation pattern as in device_event. See that block for optimization
        // strategies (SmallVec / iterator API / scratch buffer reuse).
        // Not optimized yet to keep code clarity until profiling justifies change.
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

        self.widget_impl.device_event(
            actual_bounds,
            event,
            &children_with_arrangement,
            RebuildRequest {
                need_rearrange: &self.need_rearrange,
                need_redraw: &self.need_redraw,
            },
            ctx,
        )
    }

    fn is_inside(&self, position: [f32; 2], ctx: &WidgetContext) -> bool {
        let cache = self.cache.lock();

        let Some((&actual_bounds, arrangement)) = cache.layout.get() else {
            // not rendered yet, cannot process event.
            return false;
        };

        let actual_bounds: [f32; 2] = actual_bounds.into();

        // TODO: Potential performance tuning.
        // In high-frequency events like mouse movements, the overhead of allocating this Vec on the heap
        // for each event could become a bottleneck.
        // Future optimizations might involve using an iterator-based API or a library like `smallvec`
        // to avoid heap allocations.
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

        let cache = self.cache.lock();
        let (_, arrangement) = cache.layout.get().expect("infallible: arrange just called");

        let children = self
            .children
            .iter()
            .map(|(child, setting)| (&**child as &dyn AnyWidget<T>, setting))
            .collect::<Vec<_>>();

        let node = self.widget_impl.render(
            background,
            &children
                .iter()
                .zip(arrangement)
                .map(|((c, s), a)| (*c, *s, a))
                .collect::<Vec<_>>(),
            ctx,
        );

        // BUGFIX / SPEC:
        // Redraw flag consumption happens AFTER a successful render pass.
        // Contract:
        //   - Multiple redraw requests within the same frame coalesce to a single render.
        //   - Layout (rearrange) dirtiness is consumed earlier by measure()/arrange() paths.
        //   - If render aborts (panic/early return in future), flag may remain set -> next frame still draws.
        // Future optimization: switch to a generation counter (u32) if we need to record
        // how many redraws were requested in a frame (statistics / throttling).
        // NOTE: This intentionally _does not_ clear need_rearrange: that path is already
        // cleared when caches are invalidated at layout time.
        let _ = self.need_redraw.take_dirty();

        node
    }
}

impl<D, W, T, ChildSetting> AnyWidgetFramePrivate for WidgetFrame<D, W, T, ChildSetting>
where
    D: Dom<T> + Send + Sync + 'static,
    W: Widget<D, T, ChildSetting> + Send + Sync + 'static,
    T: 'static,
    ChildSetting: Send + Sync + PartialEq + Clone + 'static,
{
    fn arrange(&self, final_size: [f32; 2], ctx: &WidgetContext) {
        let mut cache = self.cache.lock();

        if self.need_rearrange.take_dirty() {
            // arrangement changed, need to redraw
            cache.measure.clear();
            cache.layout.clear();
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
                for ((child, _), arrangement) in self.children.iter().zip(arrangement.iter()) {
                    child.arrange(arrangement.size, ctx);
                }
                arrangement
            });
    }
}

#[async_trait::async_trait]
impl<D, W, T, ChildSetting> AnyWidgetFrame<T> for WidgetFrame<D, W, T, ChildSetting>
where
    D: Dom<T> + Send + Sync + 'static,
    W: Widget<D, T, ChildSetting> + Send + Sync + 'static,
    T: 'static,
    ChildSetting: Send + Sync + PartialEq + Clone + 'static,
{
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn need_redraw(&self) -> bool {
        self.need_redraw.is_dirty()
    }

    async fn update_widget_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), UpdateWidgetError> {
        // downcast dom
        let dom = (dom as &dyn Any)
            .downcast_ref::<D>()
            .ok_or(UpdateWidgetError::TypeMismatch)?;

        let children = self.widget_impl.update_widget(
            dom,
            RebuildRequest::new(&self.need_rearrange, &self.need_redraw),
        );

        // update children

        let mut need_rearrange = false;

        // collect old children and its ids

        let old_children = std::mem::take(&mut self.children);
        let old_children_id = std::mem::take(&mut self.children_id);

        let mut old_children_map = old_children
            .into_iter()
            .zip(old_children_id.iter())
            .map(|((child, setting), id)| (*id, (child, setting)))
            .collect::<fxhash::FxHashMap<_, _>>();

        // update

        // Potential future use:
        // Collect new child id sequence for diff algorithms that may want
        // an O(n) LCS / move-detection optimization. Currently unused.
        // Prefixed with underscore to silence warnings.
        let _new_children_id = children.iter().map(|(_, _, id)| *id).collect::<Vec<_>>();

        for (child_dom, setting, id) in children {
            let mut old_pair = old_children_map.remove(&id);

            // check child identity
            if let Some((old_child, _)) = &mut old_pair {
                if old_child.update_widget_tree(child_dom).await.is_err() {
                    old_pair = None;
                }
            }

            // check setting identity
            if let Some((_, old_setting)) = &old_pair {
                if *old_setting != setting {
                    // Setting changed.
                    // CURRENT STRATEGY: treat ANY setting difference as layout-affecting,
                    // thus trigger full rearrange + redraw.
                    //
                    // FUTURE OPTIMIZATION (design note):
                    // Introduce a SettingImpact classification (layout / redraw-only / none)
                    // so purely visual changes (e.g. colors) set only redraw, avoiding
                    // measure/arrange cache invalidation.
                    // See design memo: "Setting の再配置要否判定 API 抽象".
                    // Keep simple conservative behavior until profiling justifies refinement.
                    need_rearrange = true;
                }
            }

            // push to self.children
            if let Some((old_child, _)) = old_pair {
                self.children.push((old_child, setting));
                self.children_id.push(id);
            } else {
                let new_child = child_dom.build_widget_tree();
                self.children.push((new_child, setting));
                self.children_id.push(id);
                need_rearrange = true;
            }
        }

        if !old_children_map.is_empty() {
            // children removed
            need_rearrange = true;
        }

        if self.children_id != old_children_id {
            // children reordered
            need_rearrange = true;
        }

        if need_rearrange {
            self.need_rearrange.mark_dirty();
            self.need_redraw.mark_dirty();
        }

        Ok(())
    }

    fn update_dirty_flags(&mut self, rearrange_flags: BackPropDirty, redraw_flags: BackPropDirty) {
        self.need_rearrange = rearrange_flags;
        self.need_redraw = redraw_flags;

        for (child, _) in &mut self.children {
            // NOTE:
            // Originally used `self.need_rearrange.make_child()` but build reported method not found.
            // Fallback to explicit constructor to preserve parent linking semantics.
            // If `make_child` becomes available (it exists in utils crate), you may revert for brevity.
            child.update_dirty_flags(
                BackPropDirty::with_parent(&self.need_rearrange),
                BackPropDirty::with_parent(&self.need_redraw),
            );
        }
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::{Constraints, DeviceEvent, UpdateNotifier};
    use utils::back_prop_dirty::BackPropDirty;

    #[derive(Debug, Clone, PartialEq, Default)]
    struct MockSetting {
        value: i32,
    }

    #[derive(Clone)]
    struct MockDom {
        id: u128,
        children: Vec<(MockDom, MockSetting)>,
    }

    #[async_trait::async_trait]
    impl Dom<String> for MockDom {
        fn build_widget_tree(&self) -> Box<dyn AnyWidgetFrame<String>> {
            Box::new(WidgetFrame {
                label: None,
                children: self
                    .children
                    .iter()
                    .map(|(child, setting)| (child.build_widget_tree(), setting.clone()))
                    .collect(),
                children_id: self.children.iter().map(|(c, _)| c.id).collect(),
                need_rearrange: BackPropDirty::new(),
                need_redraw: BackPropDirty::new(),
                cache: Mutex::new(WidgetFrameCache {
                    measure: Cache::new(),
                    layout: Cache::new(),
                }),
                widget_impl: MockWidget,
                _dom_type: std::marker::PhantomData,
            })
        }

        async fn set_update_notifier(&self, _notifier: &UpdateNotifier) {}
    }

    struct MockWidget;

    impl Widget<MockDom, String, MockSetting> for MockWidget {
        fn update_widget(
            &mut self,
            dom: &MockDom,
            _cache_invalidator: RebuildRequest,
        ) -> Vec<(&dyn Dom<String>, MockSetting, u128)> {
            // This is not a good example, but it works for testing.
            // A real widget would probably have a more complex logic to handle children.
            let children_with_ids: Vec<_> = dom
                .children
                .iter()
                .map(|(child, setting)| (child.clone(), setting.clone(), child.id))
                .collect();

            // This is a memory leak, but it's the easiest way to get a &'static dyn Dom<String>
            // for testing purposes. In a real application, you would have a proper arena
            // or some other way to manage the lifetime of the Dom nodes.
            let leaked_children: Vec<_> = children_with_ids
                .into_iter()
                .map(|(child, setting, id)| {
                    let boxed_child: Box<dyn Dom<String>> = Box::new(child);
                    let static_child: &'static dyn Dom<String> = Box::leak(boxed_child);
                    (static_child, setting, id)
                })
                .collect();

            leaked_children
        }

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

        widget_frame.update_widget_tree(&updated_dom).await.unwrap();

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

        widget_frame.update_widget_tree(&updated_dom).await.unwrap();

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

        widget_frame.update_widget_tree(&updated_dom).await.unwrap();

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

        widget_frame.update_widget_tree(&updated_dom).await.unwrap();

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

        widget_frame.update_widget_tree(&updated_dom).await.unwrap();

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
