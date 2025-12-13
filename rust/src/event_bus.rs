use std::cell::RefCell;

/// Mobile-specific events
#[derive(Debug, Clone)]
pub enum MobileEvent {
    Render,
    Touch { x: f32, y: f32, action: i32 },
    Resized { width: u32, height: u32 },
    Exit,
}

/// Event bus for mobile platforms
pub struct MobileEventBus {
    events: RefCell<Vec<MobileEvent>>,
    render_callback: RefCell<Option<Box<dyn FnMut()>>>,
}

impl MobileEventBus {
    pub fn new() -> Self {
        Self {
            events: RefCell::new(Vec::new()),
            render_callback: RefCell::new(None),
        }
    }

    pub fn push(&self, event: MobileEvent) {
        self.events.borrow_mut().push(event);
    }

    pub fn run_event_loop<F>(&self, mut callback: F)
    where
        F: FnMut(MobileEvent),
    {
        let mut events = self.events.borrow_mut();
        for event in events.drain(..) {
            callback(event);
        }
    }

    pub fn cleanup(&self) {
        self.events.borrow_mut().clear();
        *self.render_callback.borrow_mut() = None;
    }
}

impl Default for MobileEventBus {
    fn default() -> Self {
        Self::new()
    }
}

