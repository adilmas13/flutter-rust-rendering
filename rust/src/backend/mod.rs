//! Mobile backend for notan
//!
//! Provides a custom notan backend that:
//! - Receives render events from native (Android/iOS) render loops
//! - Handles touch input from native layer
//! - Loads OpenGL functions at runtime
//!
//! Uses callback registration pattern:
//! - run_event_loop() registers callback during init
//! - publish_event() invokes callback from FFI calls

pub mod events;
pub mod gl_loader;
pub mod system;
pub mod window;

pub use events::{MobileEvent, MobileEventBus, TouchAction};
pub use system::MobileBackend;

/// Initialize the backend event system (legacy, now no-op)
pub fn init_events() {
    events::init();
}

/// Send a render event - immediately invokes the frame callback
pub fn send_render() {
    MobileEventBus::publish_event(MobileEvent::Render);
}

/// Send a touch event
pub fn send_touch(x: f32, y: f32, action: i32) {
    MobileEventBus::publish_event(MobileEvent::Touch {
        x,
        y,
        action: TouchAction::from(action),
    });
}

/// Send a resize event
pub fn send_resize(width: u32, height: u32) {
    MobileEventBus::publish_event(MobileEvent::Resize { width, height });
}

/// Send exit event
pub fn send_exit() {
    MobileEventBus::publish_event(MobileEvent::Exit);
}
