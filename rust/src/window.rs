use crate::event_bus::{MobileEvent, MobileEventBus};
use notan_app::{EventIterator, WindowBackend, WindowConfig};
use std::cell::RefCell;
use std::rc::Rc;

pub struct MobileWindowBackend {
    width: u32,
    height: u32,
    scale: f64,
    #[allow(dead_code)]
    events: Rc<RefCell<EventIterator>>,
    event_bus: Rc<RefCell<MobileEventBus>>,
}

impl MobileWindowBackend {
    pub fn new(
        _config: WindowConfig,
        events: Rc<RefCell<EventIterator>>,
        event_bus: Rc<RefCell<MobileEventBus>>,
        window_scale_factor: f64,
    ) -> Result<Self, String> {
        // WindowConfig doesn't have size() method, use default or get from config fields
        // For now, use reasonable defaults - will be set via resize
        let width = 800;
        let height = 600;
        let scale = window_scale_factor;

        Ok(Self {
            width,
            height,
            scale,
            events,
            event_bus,
        })
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.event_bus.borrow().push(MobileEvent::Resized { width, height });
    }
}

impl WindowBackend for MobileWindowBackend {
    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn position(&self) -> (i32, i32) {
        (0, 0) // Mobile windows are always at origin
    }

    fn set_position(&mut self, _x: i32, _y: i32) {
        // Position cannot be changed on mobile
    }

    fn set_size(&mut self, width: u32, height: u32) {
        self.resize(width, height);
    }

    fn set_title(&mut self, _title: &str) {
        // Title cannot be changed on mobile
    }

    fn set_visible(&mut self, _visible: bool) {
        // Visibility is managed by the platform
    }

    fn set_fullscreen(&mut self, _fullscreen: bool) {
        // Fullscreen is managed by the platform
    }

    fn is_fullscreen(&self) -> bool {
        false
    }

    fn set_cursor(&mut self, _cursor: notan_app::CursorIcon) {
        // Cursor is not applicable on mobile
    }

    fn request_frame(&mut self) {
        self.event_bus.borrow().push(MobileEvent::Render);
    }

    fn dpi(&self) -> f64 {
        self.scale
    }

    fn id(&self) -> u64 {
        1 // Mobile has single window
    }

    fn title(&self) -> &str {
        ""
    }

    fn visible(&self) -> bool {
        true
    }

    fn is_focused(&self) -> bool {
        true
    }

    fn is_always_on_top(&self) -> bool {
        false
    }

    fn set_always_on_top(&mut self, _always_on_top: bool) {
        // Not applicable on mobile
    }

    fn cursor(&self) -> notan_app::CursorIcon {
        notan_app::CursorIcon::Default
    }

    fn set_capture_cursor(&mut self, _capture: bool) {
        // Not applicable on mobile
    }

    fn capture_cursor(&self) -> bool {
        false
    }

    fn set_cursor_position(&mut self, _x: f32, _y: f32) {
        // Not applicable on mobile
    }

    fn lazy_loop(&self) -> bool {
        false
    }

    fn set_lazy_loop(&mut self, _lazy: bool) {
        // Not applicable on mobile
    }

    fn mouse_passthrough(&mut self) -> bool {
        false
    }

    fn set_mouse_passthrough(&mut self, _passthrough: bool) {
        // Not applicable on mobile
    }

    fn screen_size(&self) -> (i32, i32) {
        let (w, h) = self.size();
        (w as i32, h as i32)
    }

    fn touch_as_mouse(&self) -> bool {
        true // On mobile, touch is treated as mouse
    }

    fn set_touch_as_mouse(&mut self, _enabled: bool) {
        // Always true on mobile
    }
}

