//! Game engine library with notan backend

mod backend;
mod game;

use backend::MobileBackend;
use game::GameState;
use notan::prelude::*;
use notan_draw::DrawConfig;
use std::panic;

/// Initialize logging for the platform
fn init_logging() {
    #[cfg(target_os = "android")]
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("RustGame"),
    );

    #[cfg(target_os = "ios")]
    {
        let _ = oslog::OsLogger::new("com.example.flutter_con")
            .level_filter(log::LevelFilter::Info)
            .init();
    }
}

/// Wrap FFI calls to catch panics
macro_rules! ffi_safe {
    ($default:expr, $body:expr) => {
        match panic::catch_unwind(panic::AssertUnwindSafe(|| $body)) {
            Ok(val) => val,
            Err(e) => {
                let msg = e
                    .downcast_ref::<&str>()
                    .map(|s| s.to_string())
                    .or_else(|| e.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "Unknown panic".into());
                log::error!("FFI panic: {}", msg);
                $default
            }
        }
    };
}

// ============================================================================
// FFI Exports
// ============================================================================

/// Initialize the game engine
#[no_mangle]
pub extern "C" fn game_init(width: u32, height: u32) -> *mut core::ffi::c_void {
    ffi_safe!(std::ptr::null_mut(), {
        init_logging();
        log::info!("game_init: {}x{}", width, height);

        // Initialize events before creating backend
        backend::init_events();

        let backend = MobileBackend::new(1.0);
        let window = WindowConfig::new()
            .size(width as i32, height as i32)
            .vsync(true);

        match notan::init_with_backend(
            move |gfx: &mut Graphics| {
                let state = GameState::new(gfx, width, height);
                // Store a copy in global for FFI access
                game::init_global(GameState::new(gfx, width, height));
                state
            },
            backend,
        )
        .add_config(window)
        .add_config(DrawConfig)
        .update(game::update)
        .draw(game::draw)
        .build()
        {
            Ok(_) => {
                log::info!("notan initialized successfully");
                // Return a non-null handle (we don't actually use it)
                1 as *mut core::ffi::c_void
            }
            Err(e) => {
                log::error!("Failed to init notan: {}", e);
                std::ptr::null_mut()
            }
        }
    })
}

/// Render frame - called from native render loop
#[no_mangle]
pub extern "C" fn game_render(_handle: *mut core::ffi::c_void) {
    ffi_safe!((), {
        log::info!("game_render here");
        backend::send_render();
    })
}

/// Update game logic - can be merged with render if needed
#[no_mangle]
pub extern "C" fn game_update(_handle: *mut core::ffi::c_void) {
    // Update is handled in render via notan's update callback
}

/// Handle window resize
#[no_mangle]
pub extern "C" fn game_resize(_handle: *mut core::ffi::c_void, width: u32, height: u32) {
    ffi_safe!((), {
        backend::send_resize(width, height);
        // Also update game state directly
        game::with_game(|g| g.resize(width, height));
    })
}

/// Handle touch events
#[no_mangle]
pub extern "C" fn game_touch(_handle: *mut core::ffi::c_void, x: f32, y: f32, action: i32) {
    ffi_safe!((), {
        backend::send_touch(x, y, action);
        // Also update game state directly for immediate response
        game::with_game(|g| g.handle_touch(x, y, action));
    })
}

/// Set direction from Flutter UI
#[no_mangle]
pub extern "C" fn game_set_direction(_handle: *mut core::ffi::c_void, direction: i32) {
    ffi_safe!((), {
        game::with_game(|g| g.set_direction(direction));
    })
}

/// Set game mode (0 = Manual, 1 = Auto)
#[no_mangle]
pub extern "C" fn game_set_mode(_handle: *mut core::ffi::c_void, mode: i32) {
    ffi_safe!((), {
        game::with_game(|g| g.set_mode(mode));
    })
}

/// Get player X position (for debugging/verification)
#[no_mangle]
pub extern "C" fn game_get_player_x(_handle: *mut core::ffi::c_void) -> f32 {
    ffi_safe!(0.0, {
        game::with_game(|g| g.player_x()).unwrap_or(0.0)
    })
}

/// Get player Y position (for debugging/verification)
#[no_mangle]
pub extern "C" fn game_get_player_y(_handle: *mut core::ffi::c_void) -> f32 {
    ffi_safe!(0.0, {
        game::with_game(|g| g.player_y()).unwrap_or(0.0)
    })
}

/// Cleanup the game engine
#[no_mangle]
pub extern "C" fn game_destroy(_handle: *mut core::ffi::c_void) {
    ffi_safe!((), {
        backend::send_exit();
        log::info!("game_destroy");
    })
}

// ============================================================================
// JNI Exports for Android
// ============================================================================
#[cfg(target_os = "android")]
mod android_jni {
    use super::*;
    use jni::objects::JClass;
    use jni::sys::{jfloat, jint, jlong};
    use jni::JNIEnv;

    #[no_mangle]
    pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameInit(
        _env: JNIEnv,
        _class: JClass,
        width: jint,
        height: jint,
    ) -> jlong {
        game_init(width as u32, height as u32) as jlong
    }

    #[no_mangle]
    pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameResize(
        _env: JNIEnv,
        _class: JClass,
        handle: jlong,
        width: jint,
        height: jint,
    ) {
        game_resize(handle as *mut core::ffi::c_void, width as u32, height as u32);
    }

    #[no_mangle]
    pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameUpdate(
        _env: JNIEnv,
        _class: JClass,
        handle: jlong,
    ) {
        game_update(handle as *mut core::ffi::c_void);
    }

    #[no_mangle]
    pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameRender(
        _env: JNIEnv,
        _class: JClass,
        handle: jlong,
    ) {
        game_render(handle as *mut core::ffi::c_void);
    }

    #[no_mangle]
    pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameSetDirection(
        _env: JNIEnv,
        _class: JClass,
        handle: jlong,
        direction: jint,
    ) {
        game_set_direction(handle as *mut core::ffi::c_void, direction);
    }

    #[no_mangle]
    pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameSetMode(
        _env: JNIEnv,
        _class: JClass,
        handle: jlong,
        mode: jint,
    ) {
        game_set_mode(handle as *mut core::ffi::c_void, mode);
    }

    #[no_mangle]
    pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameTouch(
        _env: JNIEnv,
        _class: JClass,
        handle: jlong,
        x: jfloat,
        y: jfloat,
        action: jint,
    ) {
        game_touch(handle as *mut core::ffi::c_void, x, y, action);
    }

    #[no_mangle]
    pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameDestroy(
        _env: JNIEnv,
        _class: JClass,
        handle: jlong,
    ) {
        game_destroy(handle as *mut core::ffi::c_void);
    }
}
