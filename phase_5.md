# Phase 5: Android-Rust JNI Bridge

Connect Android native layer to Rust library.

---

## File: `rust/src/lib.rs` (updated)

Update to use eglGetProcAddress directly instead of receiving function pointer:

```rust
use std::sync::Arc;

use glow::HasContext;

// Link to EGL for eglGetProcAddress
#[link(name = "EGL")]
extern "C" {
    fn eglGetProcAddress(procname: *const i8) -> *const std::ffi::c_void;
}

/// Direction enum for player movement
#[derive(Default, Clone, Copy, Debug)]
#[repr(i32)]
pub enum Direction {
    #[default]
    None = 0,
    Up = 1,
    Down = 2,
    Left = 3,
    Right = 4,
}

impl From<i32> for Direction {
    fn from(value: i32) -> Self {
        match value {
            1 => Direction::Up,
            2 => Direction::Down,
            3 => Direction::Left,
            4 => Direction::Right,
            _ => Direction::None,
        }
    }
}

/// Touch action enum
#[derive(Clone, Copy, Debug)]
#[repr(i32)]
pub enum TouchAction {
    Down = 0,
    Up = 1,
    Move = 2,
}

impl From<i32> for TouchAction {
    fn from(value: i32) -> Self {
        match value {
            0 => TouchAction::Down,
            1 => TouchAction::Up,
            2 => TouchAction::Move,
            _ => TouchAction::Down,
        }
    }
}

/// Game state held across FFI boundary
pub struct GameState {
    gl: Arc<glow::Context>,
    width: u32,
    height: u32,

    // Player state
    player_x: f32,
    player_y: f32,
    player_radius: f32,
    current_direction: Direction,

    // Touch state
    is_player_touched: bool,

    // Time tracking
    last_frame_time: std::time::Instant,
}

/// Opaque handle for FFI
pub type GameHandle = *mut GameState;

/// Initialize the game engine
/// Called from GLSurfaceView.onSurfaceCreated()
#[no_mangle]
pub extern "C" fn game_init(width: u32, height: u32) -> GameHandle {
    // Initialize Android logging
    #[cfg(target_os = "android")]
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("RustGame"),
    );

    log::info!("game_init: {}x{}", width, height);

    // Create glow context using eglGetProcAddress
    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            let c_str = std::ffi::CString::new(s).unwrap();
            eglGetProcAddress(c_str.as_ptr() as *const i8)
        })
    };
    let gl = Arc::new(gl);

    // Set initial viewport
    unsafe {
        gl.viewport(0, 0, width as i32, height as i32);
    }

    let state = Box::new(GameState {
        gl,
        width,
        height,
        player_x: width as f32 / 2.0,
        player_y: height as f32 / 2.0,
        player_radius: 40.0,
        current_direction: Direction::None,
        is_player_touched: false,
        last_frame_time: std::time::Instant::now(),
    });

    Box::into_raw(state)
}

/// Handle surface size changes
/// Called from GLSurfaceView.onSurfaceChanged()
#[no_mangle]
pub extern "C" fn game_resize(handle: GameHandle, width: u32, height: u32) {
    if handle.is_null() {
        return;
    }
    let state = unsafe { &mut *handle };
    state.width = width;
    state.height = height;

    unsafe {
        state.gl.viewport(0, 0, width as i32, height as i32);
    }

    log::info!("game_resize: {}x{}", width, height);
}

/// Update game state
/// Called each frame before render
#[no_mangle]
pub extern "C" fn game_update(handle: GameHandle) {
    if handle.is_null() {
        return;
    }
    let state = unsafe { &mut *handle };

    // Calculate delta time
    let now = std::time::Instant::now();
    let delta = now.duration_since(state.last_frame_time).as_secs_f32();
    state.last_frame_time = now;

    // Move player based on direction
    let speed = 300.0 * delta;
    match state.current_direction {
        Direction::Up => state.player_y -= speed,
        Direction::Down => state.player_y += speed,
        Direction::Left => state.player_x -= speed,
        Direction::Right => state.player_x += speed,
        Direction::None => {}
    }

    // Clamp to bounds
    let r = state.player_radius;
    state.player_x = state.player_x.clamp(r, state.width as f32 - r);
    state.player_y = state.player_y.clamp(r, state.height as f32 - r);
}

/// Render the game
/// Called from GLSurfaceView.onDrawFrame()
#[no_mangle]
pub extern "C" fn game_render(handle: GameHandle) {
    if handle.is_null() {
        return;
    }
    let state = unsafe { &mut *handle };

    // Change clear color based on player state for visual feedback
    let (r, g, b) = if state.is_player_touched {
        (0.8, 0.4, 0.1) // Orange when touched
    } else {
        match state.current_direction {
            Direction::Up => (0.1, 0.5, 0.1),    // Green
            Direction::Down => (0.5, 0.1, 0.1),  // Red
            Direction::Left => (0.1, 0.1, 0.5),  // Blue
            Direction::Right => (0.5, 0.5, 0.1), // Yellow
            Direction::None => (0.1, 0.1, 0.15), // Dark gray
        }
    };

    unsafe {
        state.gl.clear_color(r, g, b, 1.0);
        state.gl.clear(glow::COLOR_BUFFER_BIT);
    }
}

/// Handle direction input from Flutter
#[no_mangle]
pub extern "C" fn game_set_direction(handle: GameHandle, direction: i32) {
    if handle.is_null() {
        return;
    }
    let state = unsafe { &mut *handle };
    state.current_direction = Direction::from(direction);
    log::debug!("game_set_direction: {:?}", state.current_direction);
}

/// Handle touch events from Android
#[no_mangle]
pub extern "C" fn game_touch(handle: GameHandle, x: f32, y: f32, action: i32) {
    if handle.is_null() {
        return;
    }
    let state = unsafe { &mut *handle };
    let touch_action = TouchAction::from(action);

    // Check if touch is within player circle
    let dx = x - state.player_x;
    let dy = y - state.player_y;
    let distance = (dx * dx + dy * dy).sqrt();
    let is_on_player = distance <= state.player_radius;

    match touch_action {
        TouchAction::Down => {
            if is_on_player {
                state.is_player_touched = true;
                log::info!("Player touched at ({}, {})", x, y);
            }
        }
        TouchAction::Up => {
            if state.is_player_touched {
                log::info!("Player released");
            }
            state.is_player_touched = false;
        }
        TouchAction::Move => {
            // Could be used for drag behavior later
        }
    }
}

/// Get player X position (for debugging/verification)
#[no_mangle]
pub extern "C" fn game_get_player_x(handle: GameHandle) -> f32 {
    if handle.is_null() {
        return 0.0;
    }
    let state = unsafe { &*handle };
    state.player_x
}

/// Get player Y position (for debugging/verification)
#[no_mangle]
pub extern "C" fn game_get_player_y(handle: GameHandle) -> f32 {
    if handle.is_null() {
        return 0.0;
    }
    let state = unsafe { &*handle };
    state.player_y
}

/// Clean up resources
#[no_mangle]
pub extern "C" fn game_destroy(handle: GameHandle) {
    if handle.is_null() {
        return;
    }
    // Reconstruct the Box to drop it properly
    let _ = unsafe { Box::from_raw(handle) };
    log::info!("game_destroy: cleaned up");
}
```

---

## File: `android/app/src/main/kotlin/com/example/flutter_con/GameNative.kt`

JNI native method declarations:

```kotlin
package com.example.flutter_con

object GameNative {
    init {
        System.loadLibrary("game_engine")
    }

    // Native methods - these call into Rust FFI
    external fun gameInit(width: Int, height: Int): Long
    external fun gameResize(handle: Long, width: Int, height: Int)
    external fun gameUpdate(handle: Long)
    external fun gameRender(handle: Long)
    external fun gameSetDirection(handle: Long, direction: Int)
    external fun gameTouch(handle: Long, x: Float, y: Float, action: Int)
    external fun gameDestroy(handle: Long)

    // Direction constants matching Rust enum
    const val DIRECTION_NONE = 0
    const val DIRECTION_UP = 1
    const val DIRECTION_DOWN = 2
    const val DIRECTION_LEFT = 3
    const val DIRECTION_RIGHT = 4

    // Touch action constants matching Rust enum
    const val TOUCH_DOWN = 0
    const val TOUCH_UP = 1
    const val TOUCH_MOVE = 2
}
```

---

## File: `rust/src/jni.rs` (new file)

JNI wrappers that call the C FFI functions:

```rust
#![allow(non_snake_case)]

use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::{jlong, jint, jfloat};

use crate::{game_init, game_resize, game_update, game_render, game_set_direction, game_touch, game_destroy, GameHandle};

#[no_mangle]
pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameInit(
    _env: JNIEnv,
    _class: JClass,
    width: jint,
    height: jint,
) -> jlong {
    let handle = game_init(width as u32, height as u32);
    handle as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameResize(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    width: jint,
    height: jint,
) {
    game_resize(handle as GameHandle, width as u32, height as u32);
}

#[no_mangle]
pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameUpdate(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    game_update(handle as GameHandle);
}

#[no_mangle]
pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameRender(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    game_render(handle as GameHandle);
}

#[no_mangle]
pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameSetDirection(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    direction: jint,
) {
    game_set_direction(handle as GameHandle, direction);
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
    game_touch(handle as GameHandle, x, y, action);
}

#[no_mangle]
pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameDestroy(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    game_destroy(handle as GameHandle);
}
```

---

## File: `rust/src/lib.rs` (add module)

Add at the top of lib.rs:

```rust
mod jni;
```

---

## File: `rust/Cargo.toml` (updated)

Add jni dependency:

```toml
[package]
name = "game_engine"
version = "0.1.0"
edition = "2021"

[lib]
name = "game_engine"
crate-type = ["cdylib"]

[dependencies]
# OpenGL bindings
glow = "0.14"

# egui core (no winit dependency)
egui = { version = "0.29", default-features = false }

# egui OpenGL renderer
egui_glow = { version = "0.29", default-features = false }

# Logging
log = "0.4"
android_logger = "0.14"

# JNI bindings
jni = { version = "0.21", default-features = false }

[profile.release]
lto = true
opt-level = "z"
strip = true
```

---

## File: `android/app/src/main/kotlin/com/example/flutter_con/GameGLRenderer.kt` (updated)

Replace with Rust-backed renderer:

```kotlin
package com.example.flutter_con

import android.opengl.GLSurfaceView
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10

class GameGLRenderer : GLSurfaceView.Renderer {

    private var gameHandle: Long = 0
    private var width: Int = 0
    private var height: Int = 0

    @Volatile
    private var pendingDirection: Int = GameNative.DIRECTION_NONE

    @Volatile
    private var pendingTouch: TouchEvent? = null

    data class TouchEvent(val x: Float, val y: Float, val action: Int)

    override fun onSurfaceCreated(gl: GL10?, config: EGLConfig?) {
        // Initialize Rust game engine
        gameHandle = GameNative.gameInit(width, height)
    }

    override fun onSurfaceChanged(gl: GL10?, width: Int, height: Int) {
        this.width = width
        this.height = height
        if (gameHandle != 0L) {
            GameNative.gameResize(gameHandle, width, height)
        }
    }

    override fun onDrawFrame(gl: GL10?) {
        if (gameHandle == 0L) return

        // Process pending direction
        GameNative.gameSetDirection(gameHandle, pendingDirection)

        // Process pending touch
        pendingTouch?.let { touch ->
            GameNative.gameTouch(gameHandle, touch.x, touch.y, touch.action)
            pendingTouch = null
        }

        // Update and render
        GameNative.gameUpdate(gameHandle)
        GameNative.gameRender(gameHandle)
    }

    fun setDirection(direction: String) {
        pendingDirection = when (direction) {
            "up" -> GameNative.DIRECTION_UP
            "down" -> GameNative.DIRECTION_DOWN
            "left" -> GameNative.DIRECTION_LEFT
            "right" -> GameNative.DIRECTION_RIGHT
            else -> GameNative.DIRECTION_NONE
        }
    }

    fun onTouch(x: Float, y: Float, action: Int) {
        pendingTouch = TouchEvent(x, y, action)
    }

    fun destroy() {
        if (gameHandle != 0L) {
            GameNative.gameDestroy(gameHandle)
            gameHandle = 0
        }
    }
}
```

---

## File: `android/app/src/main/kotlin/com/example/flutter_con/GameGLPlatformView.kt` (updated)

Add touch event handling:

```kotlin
package com.example.flutter_con

import android.annotation.SuppressLint
import android.content.Context
import android.opengl.GLSurfaceView
import android.view.MotionEvent
import android.view.View
import io.flutter.plugin.common.BinaryMessenger
import io.flutter.plugin.platform.PlatformView

class GameGLPlatformView(
    context: Context,
    private val viewId: Int,
    private val messenger: BinaryMessenger
) : PlatformView {

    private val glSurfaceView: GameGLSurfaceView
    private val renderer: GameGLRenderer

    init {
        renderer = GameGLRenderer()
        glSurfaceView = GameGLSurfaceView(context, renderer)
    }

    override fun getView(): View = glSurfaceView

    override fun dispose() {
        glSurfaceView.queueEvent {
            renderer.destroy()
        }
    }

    fun setDirection(direction: String) {
        renderer.setDirection(direction)
    }
}

@SuppressLint("ViewConstructor")
class GameGLSurfaceView(
    context: Context,
    private val renderer: GameGLRenderer
) : GLSurfaceView(context) {

    init {
        setEGLContextClientVersion(2)
        setRenderer(renderer)
        renderMode = RENDERMODE_CONTINUOUSLY
    }

    @SuppressLint("ClickableViewAccessibility")
    override fun onTouchEvent(event: MotionEvent): Boolean {
        val action = when (event.action) {
            MotionEvent.ACTION_DOWN -> GameNative.TOUCH_DOWN
            MotionEvent.ACTION_UP -> GameNative.TOUCH_UP
            MotionEvent.ACTION_MOVE -> GameNative.TOUCH_MOVE
            else -> return super.onTouchEvent(event)
        }

        // Queue touch event to be processed on GL thread
        queueEvent {
            renderer.onTouch(event.x, event.y, action)
        }

        return true
    }
}
```

---

## Summary

This phase connects Android to Rust via JNI:

1. **rust/src/lib.rs** - Updated to use `eglGetProcAddress` directly (no function pointer needed)
2. **rust/src/jni.rs** - JNI wrappers that map Kotlin calls to Rust FFI
3. **rust/Cargo.toml** - Added `jni` dependency
4. **GameNative.kt** - Loads `libgame_engine.so` and declares native methods
5. **GameGLRenderer.kt** - Calls Rust for init/update/render instead of doing GL directly
6. **GameGLPlatformView.kt** - Added touch event handling via custom GLSurfaceView

**Data flow:**
```
Flutter Button Press
       ↓
MethodChannel → MainActivity → GameGLSurfaceFactory → GameGLPlatformView
       ↓
GameGLRenderer.setDirection()
       ↓
onDrawFrame() → GameNative.gameSetDirection() → Rust game_set_direction()
       ↓
Rust updates game state and renders via glow
```

**Touch flow:**
```
User touches GLSurfaceView
       ↓
GameGLSurfaceView.onTouchEvent()
       ↓
queueEvent → GameGLRenderer.onTouch()
       ↓
onDrawFrame() → GameNative.gameTouch() → Rust game_touch()
       ↓
Rust checks hit detection, updates state
```

---

## Rebuild Required

After implementing, rebuild the Rust library:

```bash
cd rust
./build_android.sh
```

---

## Checklist

- [x] Add JNI native method declarations in Kotlin (GameNative.kt)
- [x] Load Rust .so library via System.loadLibrary()
- [x] Call Rust init function from GLRenderer.onSurfaceCreated()
- [x] Call Rust render function from GLRenderer.onDrawFrame()
- [x] Forward input events to Rust via JNI

---

Awaiting your review and approval to implement this code.
