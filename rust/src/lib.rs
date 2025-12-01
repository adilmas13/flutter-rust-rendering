use std::ffi::c_void;
use std::sync::Arc;

use glow::HasContext;

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

/// Type for GL proc address function
pub type GlGetProcAddress = extern "C" fn(*const i8) -> *const c_void;

/// Initialize the game engine with an external GL context
/// Called from GLSurfaceView.onSurfaceCreated()
#[no_mangle]
pub extern "C" fn game_init(
    get_proc_addr: GlGetProcAddress,
    width: u32,
    height: u32,
) -> GameHandle {
    // Initialize Android logging
    #[cfg(target_os = "android")]
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("RustGame"),
    );

    log::info!("game_init: {}x{}", width, height);

    // Create glow context from Android's EGL proc address
    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            let c_str = std::ffi::CString::new(s).unwrap();
            get_proc_addr(c_str.as_ptr())
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

    unsafe {
        // Clear with dark background
        state.gl.clear_color(0.1, 0.1, 0.15, 1.0);
        state.gl.clear(glow::COLOR_BUFFER_BIT);
    }

    // For Phase 4, we'll use simple color-based rendering
    // In Phase 6/7, we'll add proper shape rendering with egui_glow

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
