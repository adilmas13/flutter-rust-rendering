use crate::event_bus::{MobileEvent, MobileEventBus};
use crate::touch;
use crate::window::MobileWindowBackend;
use notan_app::{App, Backend, BackendSystem, EventIterator, InitializeFn, WindowBackend};
use notan_app::{FrameState, WindowConfig};
use notan_core::events::Event;
use notan_graphics::DeviceBackend;
use std::cell::RefCell;
use std::rc::Rc;

pub struct MobileBackend {
    window: Option<MobileWindowBackend>,
    events: Rc<RefCell<EventIterator>>,
    event_bus: Option<Rc<RefCell<MobileEventBus>>>,
    exit_requested: bool,
    window_scale_factor: f64,
}

impl Drop for MobileBackend {
    fn drop(&mut self) {
        // Cleanup handled automatically
    }
}

impl MobileBackend {
    pub fn new(window_scale_factor: f64) -> Result<Self, String> {
        let events = Rc::new(RefCell::new(EventIterator::new()));

        Ok(Self {
            window: None,
            events,
            event_bus: None,
            exit_requested: false,
            window_scale_factor,
        })
    }

    pub fn has_windows(&mut self) -> bool {
        self.window.is_some()
    }

    pub fn window_scale(&mut self) -> f64 {
        self.window.as_mut().unwrap().scale()
    }

    pub fn push_event(&mut self, event: MobileEvent) {
        if let Some(event_bus) = &self.event_bus {
            event_bus.borrow().push(event);
        }
    }
}

impl Backend for MobileBackend {
    fn events_iter(&mut self) -> EventIterator {
        self.events.borrow_mut().take_events()
    }

    fn set_clipboard_text(&mut self, text: &str) {
        log::warn!("Clipboard not implemented on mobile: {}", text);
    }

    fn window(&mut self) -> &mut dyn WindowBackend {
        self.window.as_mut().unwrap()
    }

    fn exit(&mut self) {
        self.exit_requested = true;
    }

    fn system_timestamp(&self) -> u64 {
        1
    }

    fn open_link(&self, _url: &str, _new_tab: bool) {
        // Link opening not implemented on mobile
    }
}

impl BackendSystem for MobileBackend {
    fn initialize<S, R>(&mut self, window: WindowConfig) -> Result<Box<InitializeFn<S, R>>, String>
    where
        S: 'static,
        R: FnMut(&mut App, &mut S) -> Result<FrameState, String> + 'static,
    {
        let event_bus = Rc::new(RefCell::new(MobileEventBus::new()));
        let win = MobileWindowBackend::new(
            window,
            self.events.clone(),
            event_bus.clone(),
            self.window_scale_factor,
        )?;
        let scale = win.scale();
        self.window = Some(win);
        self.event_bus = Some(event_bus.clone());

        let events_ref = self.events.clone();
        let event_bus_ref = event_bus.clone();

        let event_bus_for_cleanup = event_bus.clone();
        Ok(Box::new(move |mut app: App, mut state: S, mut cb: R| {
            event_bus_ref.borrow_mut().run_event_loop(|event| {
                let backend = backend(&mut app);

                match event {
                    MobileEvent::Render => {
                        if !backend.exit_requested {
                            if let Err(e) = cb(&mut app, &mut state) {
                                log::error!("Error from EventBus iv_notan {}", e);
                            }
                        }
                    }
                    MobileEvent::Touch {
                        x: _,
                        y: _,
                        action: _,
                    } => {
                        if let Some(evt) = touch::process_events(&event, scale) {
                            events_ref.borrow_mut().push(evt);
                        }
                    }
                    MobileEvent::Resized { width, height } => {
                        if let Some(win) = &mut backend.window {
                            win.resize(width, height);
                        }
                        events_ref
                            .borrow_mut()
                            .push(Event::WindowResize { width, height });
                    }
                    MobileEvent::Exit => {
                        backend.exit();
                        event_bus_for_cleanup.borrow_mut().cleanup();
                    }
                }
            });
            Ok(())
        }))
    }

    fn get_graphics_backend(&self) -> Box<dyn DeviceBackend> {
        let backend = notan_glow::GlowBackend::new(|s| get_proc_address(s) as *const _).unwrap();
        Box::new(backend)
    }
}

#[cfg(target_os = "ios")]
fn get_proc_address(addr: &str) -> *const core::ffi::c_void {
    use core_foundation::base::TCFType;
    use core_foundation::bundle::{
        CFBundleGetBundleWithIdentifier, CFBundleGetFunctionPointerForName,
    };
    use core_foundation::string::CFString;
    use std::str::FromStr;

    let symbol_name: CFString = FromStr::from_str(addr).unwrap();
    let framework_name: CFString = FromStr::from_str("com.apple.opengles").unwrap();
    let framework =
        unsafe { CFBundleGetBundleWithIdentifier(framework_name.as_concrete_TypeRef()) };
    let symbol =
        unsafe { CFBundleGetFunctionPointerForName(framework, symbol_name.as_concrete_TypeRef()) };
    symbol as *const _
}

#[cfg(target_os = "android")]
#[link(name = "GLESv3")]
extern "C" {
    fn dlopen(filename: *const core::ffi::c_char, flag: i32) -> *mut core::ffi::c_void;
    fn dlsym(
        handle: *mut core::ffi::c_void,
        symbol: *const core::ffi::c_char,
    ) -> *mut core::ffi::c_void;
    fn dlclose(handle: *mut core::ffi::c_void) -> i32;
}

#[cfg(target_os = "android")]
fn get_proc_address(addr: &str) -> *const core::ffi::c_void {
    use std::ffi::{c_void, CString};
    use std::ptr;

    let c_addr = CString::new(addr).unwrap();
    let lib_name = CString::new("libGLESv3.so").unwrap();

    let rtld_lazy: i32 = 0x00001;

    unsafe {
        let handle = dlopen(lib_name.as_ptr(), rtld_lazy);
        if handle.is_null() {
            return ptr::null();
        }

        let symbol = dlsym(handle, c_addr.as_ptr());

        dlclose(handle);
        symbol as *const c_void
    }
}

fn backend(app: &mut App) -> &mut MobileBackend {
    app.backend.downcast_mut::<MobileBackend>().unwrap()
}
