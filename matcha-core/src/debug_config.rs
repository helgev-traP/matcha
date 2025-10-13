use std::sync::atomic::{AtomicBool, Ordering};

/// Runtime debug configuration used to selectively disable caches for profiling.
/// All fields are AtomicBool to allow low-cost runtime toggling.
pub struct DebugConfig {
    pub always_rebuild_widget: AtomicBool,
    pub disable_layout_measure_cache: AtomicBool,
    pub disable_layout_arrange_cache: AtomicBool,
    pub disable_rendernode_cache: AtomicBool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugConfig {
    /// Create a new DebugConfig with all flags disabled (default).
    pub fn new() -> Self {
        Self {
            always_rebuild_widget: AtomicBool::new(false),
            disable_layout_measure_cache: AtomicBool::new(false),
            disable_layout_arrange_cache: AtomicBool::new(false),
            disable_rendernode_cache: AtomicBool::new(false),
        }
    }

    /// Convenience setters/getters that hide Ordering details.
    pub fn set_always_rebuild_widget(&self, v: bool) {
        self.always_rebuild_widget.store(v, Ordering::Relaxed)
    }
    pub fn always_rebuild_widget(&self) -> bool {
        self.always_rebuild_widget.load(Ordering::Relaxed)
    }

    pub fn set_disable_layout_measure_cache(&self, v: bool) {
        self.disable_layout_measure_cache
            .store(v, Ordering::Relaxed)
    }
    pub fn disable_layout_measure_cache(&self) -> bool {
        self.disable_layout_measure_cache.load(Ordering::Relaxed)
    }

    pub fn set_disable_layout_arrange_cache(&self, v: bool) {
        self.disable_layout_arrange_cache
            .store(v, Ordering::Relaxed)
    }
    pub fn disable_layout_arrange_cache(&self) -> bool {
        self.disable_layout_arrange_cache.load(Ordering::Relaxed)
    }

    pub fn set_disable_rendernode_cache(&self, v: bool) {
        self.disable_rendernode_cache.store(v, Ordering::Relaxed)
    }
    pub fn disable_render_node_cache(&self) -> bool {
        self.disable_rendernode_cache.load(Ordering::Relaxed)
    }
}

impl Clone for DebugConfig {
    fn clone(&self) -> Self {
        Self {
            always_rebuild_widget: AtomicBool::new(
                self.always_rebuild_widget.load(Ordering::Relaxed),
            ),
            disable_layout_measure_cache: AtomicBool::new(
                self.disable_layout_measure_cache.load(Ordering::Relaxed),
            ),
            disable_layout_arrange_cache: AtomicBool::new(
                self.disable_layout_arrange_cache.load(Ordering::Relaxed),
            ),
            disable_rendernode_cache: AtomicBool::new(
                self.disable_rendernode_cache.load(Ordering::Relaxed),
            ),
        }
    }
}
