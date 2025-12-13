//! Platform-specific OpenGL function loader
//!
//! This module provides `get_proc_address` for loading GL function pointers
//! at runtime. Each platform uses its native mechanism.

use core::ffi::c_void;

/// Load an OpenGL function pointer by name.
/// Returns null if the function is not found.
pub fn get_proc_address(name: &str) -> *const c_void {
    #[cfg(target_os = "android")]
    {
        android::get_proc_address(name)
    }
    #[cfg(target_os = "ios")]
    {
        ios::get_proc_address(name)
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let _ = name;
        std::ptr::null()
    }
}

#[cfg(target_os = "android")]
mod android {
    use core::ffi::c_void;
    use std::ffi::CString;
    use std::os::raw::c_char;

    // Link against EGL for eglGetProcAddress
    #[link(name = "EGL")]
    extern "C" {
        fn eglGetProcAddress(name: *const c_char) -> *const c_void;
    }

    pub fn get_proc_address(name: &str) -> *const c_void {
        let c_name = match CString::new(name) {
            Ok(s) => s,
            Err(_) => return std::ptr::null(),
        };
        unsafe { eglGetProcAddress(c_name.as_ptr()) }
    }
}

#[cfg(target_os = "ios")]
mod ios {
    use core::ffi::c_void;
    use core_foundation::base::TCFType;
    use core_foundation::bundle::{
        CFBundleGetBundleWithIdentifier, CFBundleGetFunctionPointerForName,
    };
    use core_foundation::string::CFString;
    use std::str::FromStr;

    pub fn get_proc_address(name: &str) -> *const c_void {
        let symbol_name: CFString = match FromStr::from_str(name) {
            Ok(s) => s,
            Err(_) => return std::ptr::null(),
        };
        let framework_name: CFString = match FromStr::from_str("com.apple.opengles") {
            Ok(s) => s,
            Err(_) => return std::ptr::null(),
        };

        unsafe {
            let framework = CFBundleGetBundleWithIdentifier(framework_name.as_concrete_TypeRef());
            if framework.is_null() {
                return std::ptr::null();
            }
            CFBundleGetFunctionPointerForName(framework, symbol_name.as_concrete_TypeRef())
                as *const c_void
        }
    }
}
