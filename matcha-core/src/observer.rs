use std::sync::{Arc, atomic::AtomicBool};

pub struct Observer {
    // default is false
    // when the flag is set to true, then dom update is triggered
    flag: Arc<AtomicBool>,
}

impl Observer {
    pub(crate) fn new() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Creates a new Observer that is used to trigger rendering.
    pub(crate) fn new_render_trigger() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn sender(&self) -> ObserverSender {
        ObserverSender {
            flag: self.flag.clone(),
        }
    }
}

impl Observer {
    pub(crate) fn is_updated(&self) -> bool {
        self.flag.load(std::sync::atomic::Ordering::Acquire)
    }

    pub(crate) async fn wait_for_update(&self) {
        while !self.is_updated() {
            tokio::task::yield_now().await;
        }
    }
}

pub struct ObserverSender {
    flag: Arc<AtomicBool>,
}

impl ObserverSender {
    pub fn send_update(&mut self) {
        self.flag.store(true, std::sync::atomic::Ordering::Release);
    }
}
