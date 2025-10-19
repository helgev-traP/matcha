use std::sync::atomic::{AtomicBool, Ordering};

/// Runtime debug configuration used to selectively disable caches for profiling.
/// All fields are AtomicBool to allow low-cost runtime toggling.
pub(crate) struct DebugConfig {
    pub always_rebuild_widget: AtomicBool,
    pub disable_layout_measure_cache: AtomicBool,
    pub disable_layout_arrange_cache: AtomicBool,
    pub disable_render_node_cache: AtomicBool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            always_rebuild_widget: AtomicBool::new(false),
            disable_layout_measure_cache: AtomicBool::new(false),
            disable_layout_arrange_cache: AtomicBool::new(false),
            disable_render_node_cache: AtomicBool::new(false),
        }
    }
}

impl DebugConfig {
    pub fn always_rebuild_widget(&self) -> bool {
        self.always_rebuild_widget.load(Ordering::Relaxed)
    }

    pub fn disable_layout_measure_cache(&self) -> bool {
        self.disable_layout_measure_cache.load(Ordering::Relaxed)
    }

    pub fn disable_layout_arrange_cache(&self) -> bool {
        self.disable_layout_arrange_cache.load(Ordering::Relaxed)
    }

    pub fn disable_render_node_cache(&self) -> bool {
        self.disable_render_node_cache.load(Ordering::Relaxed)
    }
}
