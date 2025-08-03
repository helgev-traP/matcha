use std::{future::Future, pin::Pin, sync::Arc};

use async_trait::async_trait;

#[async_trait]
pub trait Backend<Event: Send + 'static>: Send + Sync + Clone {
    async fn send_event(&self, event: Event);
}

// --- Stub Backend ---

#[derive(Clone, Copy, Debug, Default)]
pub struct StubBackend;

#[async_trait]
impl<Event: Send + 'static> Backend<Event> for StubBackend {
    async fn send_event(&self, _event: Event) {
        // This backend does nothing.
    }
}

// --- Sync Channel Backend ---

pub struct SyncChannelBackend<Event: Send> {
    sender: std::sync::mpsc::Sender<Event>,
}

impl<Event: Send> Clone for SyncChannelBackend<Event> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<Event: Send> SyncChannelBackend<Event> {
    pub fn new(sender: std::sync::mpsc::Sender<Event>) -> Self {
        Self { sender }
    }
}

#[async_trait]
impl<Event: Send + 'static> Backend<Event> for SyncChannelBackend<Event> {
    async fn send_event(&self, event: Event) {
        if let Err(e) = self.sender.send(event) {
            eprintln!("Failed to send event to backend: {e}");
        }
    }
}

// ---　Async Channel Backend ---

pub struct ChannelBackend<Event: Send> {
    sender: tokio::sync::mpsc::Sender<Event>,
}

impl<Event: Send> Clone for ChannelBackend<Event> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<Event: Send> ChannelBackend<Event> {
    pub fn new(sender: tokio::sync::mpsc::Sender<Event>) -> Self {
        Self { sender }
    }
}

#[async_trait]
impl<Event: Send + 'static> Backend<Event> for ChannelBackend<Event> {
    async fn send_event(&self, event: Event) {
        if let Err(e) = self.sender.send(event).await {
            eprintln!("Failed to send event to backend: {e}");
        }
    }
}

// --- Future Backend ---

pub struct FutureBackend<Event: Send> {
    handler: Arc<dyn Fn(Event) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>,
}

impl<Event: Send> Clone for FutureBackend<Event> {
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
        }
    }
}

impl<Event: Send> FutureBackend<Event> {
    pub fn new<F, Fut>(handler: F) -> Self
    where
        F: Fn(Event) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self {
            handler: Arc::new(move |event| Box::pin(handler(event))),
        }
    }
}

#[async_trait]
impl<Event: Send + 'static> Backend<Event> for FutureBackend<Event> {
    async fn send_event(&self, event: Event) {
        (self.handler)(event).await;
    }
}
