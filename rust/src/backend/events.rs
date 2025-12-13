//! Event bus for mobile backend - callback registration pattern
//!
//! Based on iv-notan/crates/notan_mobile/src/event_bus.rs
//!
//! Key insight: `run_event_loop()` doesn't run a loop - it registers a callback
//! in a global static handler. `publish_event()` is called from FFI and
//! immediately invokes the registered callback.

use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::fmt::{self, Debug};
use std::rc::{Rc, Weak};
use std::sync::Mutex;

/// Events dispatched from native code
#[derive(Debug, Clone)]
pub enum MobileEvent {
    Render,
    Touch { x: f32, y: f32, action: TouchAction },
    Resize { width: u32, height: u32 },
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchAction {
    Down,
    Move,
    Up,
}

impl From<i32> for TouchAction {
    fn from(value: i32) -> Self {
        match value {
            0 => TouchAction::Down,
            2 => TouchAction::Move,
            _ => TouchAction::Up,
        }
    }
}

// ============================================================================
// Event Handler Trait & Global Handler
// ============================================================================

trait EventHandler: Debug + Send {
    fn handle_event(&mut self, event: MobileEvent);
}

#[derive(Default)]
struct GlobalEventHandler {
    callback: Mutex<Option<Box<dyn EventHandler>>>,
}

unsafe impl Send for GlobalEventHandler {}
unsafe impl Sync for GlobalEventHandler {}

impl GlobalEventHandler {
    fn handle_event(&self, event: MobileEvent) {
        if let Ok(mut guard) = self.callback.lock() {
            if let Some(ref mut handler) = *guard {
                handler.handle_event(event);
            }
        }
    }

    fn set_handler(&self, handler: Box<dyn EventHandler>) {
        if let Ok(mut guard) = self.callback.lock() {
            *guard = Some(handler);
        }
    }

    fn clear(&self) {
        if let Ok(mut guard) = self.callback.lock() {
            *guard = None;
        }
    }
}

static HANDLER: Lazy<GlobalEventHandler> = Lazy::new(GlobalEventHandler::default);

// ============================================================================
// Event Loop Handler - wraps the callback
// ============================================================================

pub(crate) type MobileCallback = RefCell<dyn FnMut(MobileEvent)>;

struct EventLoopHandler {
    callback: Weak<MobileCallback>,
}

impl Debug for EventLoopHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventLoopHandler").finish()
    }
}

unsafe impl Send for EventLoopHandler {}

impl EventHandler for EventLoopHandler {
    fn handle_event(&mut self, event: MobileEvent) {
        if let Some(callback) = self.callback.upgrade() {
            if let Ok(mut cb) = callback.try_borrow_mut() {
                (*cb)(event);
            }
        }
    }
}

// ============================================================================
// Mobile Event Bus
// ============================================================================

pub struct MobileEventBus {
    _callback: Option<Rc<MobileCallback>>,
}

impl MobileEventBus {
    pub fn new() -> Self {
        Self { _callback: None }
    }

    /// Register a callback to be invoked when events are published.
    /// This doesn't run a loop - it stores the callback for later invocation.
    pub fn run_event_loop<F>(&mut self, callback: F)
    where
        F: FnMut(MobileEvent) + 'static,
    {
        let callback: Rc<MobileCallback> = Rc::new(RefCell::new(callback));
        self._callback = Some(Rc::clone(&callback));

        let weak_cb = Rc::downgrade(&callback);
        HANDLER.set_handler(Box::new(EventLoopHandler { callback: weak_cb }));
    }

    /// Publish an event - immediately invokes the registered callback
    pub fn publish_event(event: MobileEvent) {
        HANDLER.handle_event(event);
    }

    pub fn cleanup(&mut self) {
        self._callback = None;
        HANDLER.clear();
    }
}

impl Default for MobileEventBus {
    fn default() -> Self {
        Self::new()
    }
}

// Legacy init function (for compatibility with lib.rs)
pub fn init() {
    // No-op - event bus handles its own initialization
}

pub fn cleanup() {
    HANDLER.clear();
}
