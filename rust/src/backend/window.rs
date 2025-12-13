//! Mobile window backend
//!
//! Simplified compared to reference - removes features not applicable to mobile.

use notan_app::{CursorIcon, WindowBackend};

/// Mobile window state
pub struct MobileWindow {
    width: u32,
    height: u32,
    dpi: f64,
}

impl MobileWindow {
    pub fn new(width: u32, height: u32, dpi: f64) -> Self {
        Self { width, height, dpi }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
}

impl WindowBackend for MobileWindow {
    fn id(&self) -> u64 {
        0
    }
    fn dpi(&self) -> f64 {
        self.dpi
    }
    fn size(&self) -> (i32, i32) {
        (self.width as i32, self.height as i32)
    }
    fn screen_size(&self) -> (i32, i32) {
        self.size()
    }
    fn position(&self) -> (i32, i32) {
        (0, 0)
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::Default
    }
    fn visible(&self) -> bool {
        true
    }
    fn is_fullscreen(&self) -> bool {
        true
    } // Mobile is always fullscreen
    fn is_always_on_top(&self) -> bool {
        false
    }
    fn lazy_loop(&self) -> bool {
        false
    }
    fn capture_cursor(&self) -> bool {
        false
    }
    fn mouse_passthrough(&mut self) -> bool {
        false
    }
    fn title(&self) -> &str {
        "Game"
    }
    fn touch_as_mouse(&self) -> bool {
        true
    }

    // Setters - mostly no-ops on mobile
    fn set_size(&mut self, width: i32, height: i32) {
        if width > 0 && height > 0 {
            self.width = width as u32;
            self.height = height as u32;
        }
    }
    fn set_position(&mut self, _x: i32, _y: i32) {}
    fn set_cursor(&mut self, _cursor: CursorIcon) {}
    fn set_visible(&mut self, _visible: bool) {}
    fn set_fullscreen(&mut self, _enabled: bool) {}
    fn set_always_on_top(&mut self, _enabled: bool) {}
    fn set_lazy_loop(&mut self, _lazy: bool) {}
    fn set_capture_cursor(&mut self, _capture: bool) {}
    fn set_mouse_passthrough(&mut self, _clickable: bool) {}
    fn set_title(&mut self, _title: &str) {}
    fn set_touch_as_mouse(&mut self, _enable: bool) {}
    fn request_frame(&mut self) {}
}

// Required for notan Backend trait
unsafe impl Send for MobileWindow {}
unsafe impl Sync for MobileWindow {}
