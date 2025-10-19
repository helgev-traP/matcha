use std::{future::Future, pin::Pin};

use async_trait::async_trait;

#[async_trait]
pub trait Backend<Event: Send> {
    async fn send_event(&self, event: Event);
}

// --- Stub Backend ---

#[async_trait]
impl<Event: Send + 'static> Backend<Event> for () {
    async fn send_event(&self, _event: Event) {
        // This backend does nothing.
    }
}

// --- Sync Channel Backend ---

#[async_trait]
impl<Event: Send> Backend<Event> for std::sync::mpsc::Sender<Event> {
    async fn send_event(&self, event: Event) {
        if let Err(e) = self.send(event) {
            eprintln!("Failed to send event to backend: {e}");
        }
    }
}

// ---　Async Channel Backend ---

#[async_trait]
impl<Event: Send> Backend<Event> for tokio::sync::mpsc::Sender<Event> {
    async fn send_event(&self, event: Event) {
        if let Err(e) = self.send(event).await {
            eprintln!("Failed to send event to backend: {e}");
        }
    }
}

// --- Future Backend ---

#[async_trait]
impl<Event: Send> Backend<Event>
    for Box<dyn Fn(Event) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>
{
    async fn send_event(&self, event: Event) {
        (self)(event).await;
    }
}
