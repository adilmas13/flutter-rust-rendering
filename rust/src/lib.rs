mod jni;

use std::sync::Arc;

use egui::{Color32, Pos2, Rect, Rounding, Stroke, Vec2};
use glow::HasContext;

// Link to EGL for eglGetProcAddress
#[link(name = "EGL")]
extern "C" {
    fn eglGetProcAddress(procname: *const i8) -> *const std::ffi::c_void;
}

/// Direction enum for player movement
#[derive(Default, Clone, Copy, Debug, PartialEq)]
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

    // egui
    egui_ctx: egui::Context,
    egui_painter: egui_glow::Painter,

    // Player state
    player_x: f32,
    player_y: f32,
    player_size: f32,
    current_direction: Direction,

    // Touch state
    is_player_touched: bool,
    touch_pos: Option<Pos2>,

    // Game stats
    score: u32,
    moves: u32,

    // UI state
    show_debug: bool,

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

    // Create egui context
    let egui_ctx = egui::Context::default();

    // Create egui_glow painter for OpenGL ES 2.0
    let egui_painter = match egui_glow::Painter::new(gl.clone(), "", None, false) {
        Ok(painter) => painter,
        Err(e) => {
            log::error!("Failed to create egui painter: {}", e);
            return std::ptr::null_mut();
        }
    };

    let player_size = 200.0;

    let state = Box::new(GameState {
        gl,
        width,
        height,
        egui_ctx,
        egui_painter,
        player_x: width as f32 / 2.0,
        player_y: height as f32 / 2.0,
        player_size,
        current_direction: Direction::None,
        is_player_touched: false,
        touch_pos: None,
        score: 0,
        moves: 0,
        show_debug: true,
        last_frame_time: std::time::Instant::now(),
    });

    log::info!("Game initialized with egui");
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

    // Center player on first resize (when dimensions were 0)
    if state.width == 0 || state.height == 0 {
        state.player_x = width as f32 / 2.0;
        state.player_y = height as f32 / 2.0;
    }

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

    // Clamp to bounds (account for status bar at top ~40px)
    let half = state.player_size / 2.0;
    let top_margin = 50.0;
    state.player_x = state.player_x.clamp(half, state.width as f32 - half);
    state.player_y = state.player_y.clamp(half + top_margin, state.height as f32 - half);
}

/// Render the game using egui
/// Called from GLSurfaceView.onDrawFrame()
#[no_mangle]
pub extern "C" fn game_render(handle: GameHandle) {
    if handle.is_null() {
        return;
    }
    let state = unsafe { &mut *handle };

    // Clear background
    unsafe {
        state.gl.clear_color(0.1, 0.1, 0.15, 1.0);
        state.gl.clear(glow::COLOR_BUFFER_BIT);
    }

    let screen_rect = Rect::from_min_size(
        Pos2::ZERO,
        Vec2::new(state.width as f32, state.height as f32),
    );

    // Build egui input with touch
    let mut raw_input = egui::RawInput {
        screen_rect: Some(screen_rect),
        ..Default::default()
    };

    // Add touch/pointer events for egui interaction
    if let Some(pos) = state.touch_pos {
        raw_input.events.push(egui::Event::PointerMoved(pos));
    }

    // Capture state for closure
    let player_x = state.player_x;
    let player_y = state.player_y;
    let player_size = state.player_size;
    let current_direction = state.current_direction;
    let is_player_touched = state.is_player_touched;
    let score = state.score;
    let moves = state.moves;
    let show_debug = state.show_debug;
    let width = state.width;
    let height = state.height;

    let mut new_show_debug = show_debug;
    let mut reset_requested = false;

    let full_output = state.egui_ctx.run(raw_input, |ctx| {
        // === Status bar at top ===
        egui::TopBottomPanel::top("status_bar")
            .frame(egui::Frame::none().fill(Color32::from_rgb(30, 30, 40)).inner_margin(8.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("Score: {}", score))
                            .size(22.0)
                            .color(Color32::from_rgb(255, 220, 100)),
                    );
                    ui.add_space(20.0);
                    ui.label(
                        egui::RichText::new(format!("Moves: {}", moves))
                            .size(18.0)
                            .color(Color32::LIGHT_GRAY),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(if show_debug { "Hide Debug" } else { "Show Debug" }).clicked() {
                            new_show_debug = !show_debug;
                        }
                    });
                });
            });

        // === Debug panel ===
        if show_debug {
            egui::Window::new("Debug")
                .anchor(egui::Align2::RIGHT_BOTTOM, [-10.0, -10.0])
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label(format!("Position: ({:.0}, {:.0})", player_x, player_y));
                    ui.label(format!("Direction: {:?}", current_direction));
                    ui.label(format!("Touched: {}", is_player_touched));
                    ui.label(format!("Screen: {}x{}", width, height));
                    ui.separator();
                    if ui.button("Reset Position").clicked() {
                        reset_requested = true;
                    }
                });
        }

        // === Draw player shape on background layer ===
        let painter = ctx.layer_painter(egui::LayerId::background());

        let fill_color = if is_player_touched {
            Color32::from_rgb(255, 150, 50)
        } else {
            match current_direction {
                Direction::Up => Color32::from_rgb(50, 200, 50),
                Direction::Down => Color32::from_rgb(200, 50, 50),
                Direction::Left => Color32::from_rgb(50, 50, 200),
                Direction::Right => Color32::from_rgb(200, 200, 50),
                Direction::None => Color32::from_rgb(150, 150, 150),
            }
        };

        let stroke_color = Color32::WHITE;
        let center = Pos2::new(player_x, player_y);
        let half = player_size / 2.0;

        // Draw rounded rectangle
        let rect = Rect::from_center_size(center, Vec2::splat(player_size));
        painter.rect(rect, Rounding::same(12.0), fill_color, Stroke::new(3.0, stroke_color));

        // Draw circle inside
        painter.circle(center, half * 0.5, fill_color.gamma_multiply(0.6), Stroke::new(2.0, stroke_color));
    });

    // Apply UI state changes
    state.show_debug = new_show_debug;
    if reset_requested {
        state.player_x = state.width as f32 / 2.0;
        state.player_y = state.height as f32 / 2.0;
        state.score = 0;
        state.moves = 0;
    }

    // Tessellate and paint
    let pixels_per_point = 1.0;
    let clipped_primitives = state.egui_ctx.tessellate(full_output.shapes, pixels_per_point);

    state.egui_painter.paint_and_update_textures(
        [state.width, state.height],
        pixels_per_point,
        &clipped_primitives,
        &full_output.textures_delta,
    );
}

/// Handle direction input from Flutter
#[no_mangle]
pub extern "C" fn game_set_direction(handle: GameHandle, direction: i32) {
    if handle.is_null() {
        return;
    }
    let state = unsafe { &mut *handle };
    let new_direction = Direction::from(direction);

    // Count direction changes as moves
    if new_direction != Direction::None && new_direction != state.current_direction {
        state.moves += 1;
    }

    state.current_direction = new_direction;
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

    let pos = Pos2::new(x, y);

    match touch_action {
        TouchAction::Down => {
            state.touch_pos = Some(pos);
            // Check if touch is within player box
            let half = state.player_size / 2.0;
            let is_on_player = x >= state.player_x - half
                && x <= state.player_x + half
                && y >= state.player_y - half
                && y <= state.player_y + half;
            if is_on_player {
                state.is_player_touched = true;
                state.score += 10;
                log::info!("Player touched! Score: {}", state.score);
            }
        }
        TouchAction::Up => {
            state.touch_pos = None;
            state.is_player_touched = false;
        }
        TouchAction::Move => {
            state.touch_pos = Some(pos);
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
    let mut state = unsafe { Box::from_raw(handle) };

    // egui_painter cleanup
    state.egui_painter.destroy();

    log::info!("game_destroy: cleaned up");
}
