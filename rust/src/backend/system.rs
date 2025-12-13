//! Mobile backend system for notan
//!
//! Uses callback registration pattern like iv-notan:
//! - run_event_loop() registers a callback in global handler
//! - publish_event() immediately invokes the callback from FFI

use crate::backend::{events, gl_loader, window::MobileWindow};
use notan_app::{
    App, Backend, BackendSystem, EventIterator, FrameState, InitializeFn, WindowBackend,
    WindowConfig,
};
use notan_core::events::Event;
use notan_graphics::DeviceBackend;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Mobile backend for notan
pub struct MobileBackend {
    window: Option<MobileWindow>,
    events: EventIterator,
    exit_requested: bool,
    dpi: f64,
}

impl MobileBackend {
    pub fn new(dpi: f64) -> Self {
        Self {
            window: None,
            events: EventIterator::new(),
            exit_requested: false,
            dpi,
        }
    }
}

impl Backend for MobileBackend {
    fn events_iter(&mut self) -> EventIterator {
        self.events.take_events()
    }

    fn set_clipboard_text(&mut self, _text: &str) {
        // Not supported on mobile
    }

    fn window(&mut self) -> &mut dyn WindowBackend {
        self.window.as_mut().expect("Window not initialized")
    }

    fn exit(&mut self) {
        self.exit_requested = true;
    }

    fn system_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    fn open_link(&self, _url: &str, _new_tab: bool) {
        // Could implement via native bridge if needed
    }
}

impl BackendSystem for MobileBackend {
    fn initialize<S, R>(
        &mut self,
        config: WindowConfig,
    ) -> Result<Box<InitializeFn<S, R>>, String>
    where
        S: 'static,
        R: FnMut(&mut App, &mut S) -> Result<FrameState, String> + 'static,
    {
        // Create window
        self.window = Some(MobileWindow::new(
            config.width as u32,
            config.height as u32,
            self.dpi,
        ));

        // Create event bus (will be moved into closure)
        let event_bus = Rc::new(RefCell::new(events::MobileEventBus::new()));
        let event_bus_ref = event_bus.clone();

        Ok(Box::new(
            move |mut app: App, mut state: S, mut frame_cb: R| {
                // Register callback that processes events
                // This doesn't run a loop - it stores the callback for later invocation
                event_bus.borrow_mut().run_event_loop(move |event| {
                    let backend = app.backend.downcast_mut::<MobileBackend>().unwrap();

                    match event {
                        events::MobileEvent::Render => {
                            
                            if !backend.exit_requested {
                                log::info!("MobileBackend::initialize() events: {:?}", event);
                                if let Err(e) = frame_cb(&mut app, &mut state) {
                                    log::error!("Frame error: {}", e);
                                }
                            }
                        }
                        events::MobileEvent::Touch { x, y, action } => {
                            let notan_event = match action {
                                events::TouchAction::Down => Event::TouchStart { id: 0, x, y },
                                events::TouchAction::Move => Event::TouchMove { id: 0, x, y },
                                events::TouchAction::Up => Event::TouchEnd { id: 0, x, y },
                            };
                            backend.events.push(notan_event);
                        }
                        events::MobileEvent::Resize { width, height } => {
                            if let Some(win) = &mut backend.window {
                                win.resize(width, height);
                            }
                            backend.events.push(Event::WindowResize {
                                width: width as i32,
                                height: height as i32,
                            });
                        }
                        events::MobileEvent::Exit => {
                            backend.exit_requested = true;
                            event_bus_ref.borrow_mut().cleanup();
                        }
                    }
                });

                Ok(())
            },
        ))
    }

    fn get_graphics_backend(&self) -> Box<dyn DeviceBackend> {
        let loader = |name: &str| gl_loader::get_proc_address(name) as *const _;
        let backend =
            notan_glow::GlowBackend::new(loader).expect("Failed to create GlowBackend");
        Box::new(backend)
    }
}
