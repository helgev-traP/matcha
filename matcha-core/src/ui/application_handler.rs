use std::mem;
use std::sync::Arc;

use parking_lot::Mutex;

/// ApplicationHandler is owned by the window / WinitInstance and holds the
/// shared command buffer. Components receive `ApplicationHandle` clones to
/// enqueue commands.
#[derive(Clone)]
pub struct ApplicationHandler {
    inner: Arc<Mutex<ApplicationHandlerInner>>,
}

pub(crate) struct ApplicationHandlerInner {
    commands: Vec<ApplicationHandlerCommand>,
}

/// Commands that can be enqueued from components / handlers.
/// Extend this enum when new application-level commands are needed.
pub(crate) enum ApplicationHandlerCommand {
    // Define events that the application handler will process
    Quit,
    // future: Custom(Box<dyn FnOnce(&mut AppState) + Send>), etc.
}

impl ApplicationHandler {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ApplicationHandlerInner {
                commands: Vec::new(),
            })),
        }
    }

    /// Create a cloneable handle to give to Components / ModelAccessor.
    pub fn handle(&self) -> ApplicationHandler {
        ApplicationHandler {
            inner: Arc::clone(&self.inner),
        }
    }

    /// Drain and take all pending commands atomically.
    /// Intended to be called by WinitInstance (main/UI thread) at frame end.
    pub(crate) fn drain_commands(&self) -> Vec<ApplicationHandlerCommand> {
        let mut guard = self.inner.lock();
        mem::take(&mut guard.commands)
    }
}

impl ApplicationHandler {
    /// Enqueue a Quit command.
    pub fn quit(&self) {
        let mut guard = self.inner.lock();
        guard.commands.push(ApplicationHandlerCommand::Quit);
    }

    // future: push_custom, query_with_oneshot, etc.
}

impl ApplicationHandlerInner {
    pub fn commands(&self) -> &[ApplicationHandlerCommand] {
        &self.commands
    }
}
