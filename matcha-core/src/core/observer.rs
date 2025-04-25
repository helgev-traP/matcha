use std::sync::{atomic::AtomicBool, Arc};

pub fn create_observer_ch() -> (ObserverSender, ObserverReceiver) {
    let flag = Arc::new(AtomicBool::new(false));
    let sender = ObserverSender::new(&flag);
    let receiver = ObserverReceiver::new(&flag);
    (sender, receiver)
}

pub struct ObserverSender {
    flag: Arc<AtomicBool>,
}

impl ObserverSender {
    fn new(flag: &Arc<AtomicBool>) -> Self {
        Self { flag: flag.clone() }
    }

    pub fn send_update(&mut self) {
        self.flag.store(true, std::sync::atomic::Ordering::Release);
    }
}

pub struct ObserverReceiver {
    flag: Arc<AtomicBool>,
}

impl ObserverReceiver {
    fn new(flag: &Arc<AtomicBool>) -> Self {
        Self { flag: flag.clone() }
    }

    pub fn is_updated(&mut self) -> bool {
        self.flag.load(std::sync::atomic::Ordering::Acquire)
    }
}

impl Future for ObserverReceiver {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self.is_updated() {
            std::task::Poll::Ready(())
        } else {
            std::task::Poll::Pending
        }
    }
}

// MARK: observer

pub struct Observer {
    receivers: Vec<ObserverReceiver>,
}

impl Default for Observer {
    fn default() -> Self {
        Self::new()
    }
}

impl Observer {
    fn new() -> Self {
        Self {
            receivers: Vec::new(),
        }
    }

    pub fn new_render_trigger() -> Self {
        let (mut sender, receiver) = create_observer_ch();
        sender.send_update();
        Self {
            receivers: vec![receiver],
        }
    }

    pub fn add_receiver(&mut self, receiver: ObserverReceiver) {
        self.receivers.push(receiver);
    }

    pub fn extend(&mut self, other: Observer) {
        self.receivers.extend(other.receivers);
    }

    pub fn join(mut self, other: Observer) -> Self {
        self.extend(other);
        self
    }

    pub fn is_updated(&mut self) -> bool {
        self.receivers.iter_mut().any(|receiver| {
            receiver.is_updated()
        })
    }
}

/// wait for the observer to receive an component update.
impl Future for Observer {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self.is_updated() {
            std::task::Poll::Ready(())
        } else {
            std::task::Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {}
