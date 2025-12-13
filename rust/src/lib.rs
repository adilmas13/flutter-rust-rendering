// JNI module only for Android
#[cfg(target_os = "android")]
mod jni;

mod backend;
mod event_bus;
mod touch;
mod window;

// Stub android_main function to satisfy android-activity linker requirement
// We use a custom backend, so this function will never be called
#[cfg(target_os = "android")]
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn android_main(_app: android_activity::AndroidApp) {
    // This is a stub - we use custom backend, so this entry point is never used
    // The app is managed by Flutter, not by android-activity
    log::warn!("android_main called but we use custom backend - this should not happen");
}

use std::panic;
use std::sync::{Arc, Mutex};

use egui::{Color32, Pos2, Rect, Rounding, Stroke, Vec2};
use glow::HasContext;
use notan::prelude::*;

pub use backend::MobileBackend;
pub use event_bus::{MobileEvent, MobileEventBus};

/// Wrap FFI calls with panic catching to prevent crashes across FFI boundary
macro_rules! catch_panic {
    ($default:expr, $body:expr) => {
        match panic::catch_unwind(panic::AssertUnwindSafe(|| $body)) {
            Ok(result) => result,
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic".to_string()
                };
                log::error!("Panic caught in FFI: {}", msg);
                $default
            }
        }
    };
}

// Platform-specific GL loader
#[cfg(target_os = "android")]
#[allow(dead_code)]
#[link(name = "EGL")]
extern "C" {
    fn eglGetProcAddress(procname: *const i8) -> *const std::ffi::c_void;
}

// iOS uses EAGL - GL functions are resolved at link time
// No runtime loader needed

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

/// Game mode enum
#[derive(Default, Clone, Copy, Debug, PartialEq)]
#[repr(i32)]
pub enum GameMode {
    #[default]
    Manual = 0,
    Auto = 1,
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
    app_state: Arc<Mutex<GameAppState>>,
    backend: Arc<Mutex<MobileBackend>>,
    egui_ctx: Option<Arc<Mutex<egui::Context>>>,
    egui_painter: Option<Arc<Mutex<egui_glow::Painter>>>,
    gl: Arc<glow::Context>, // Store glow context for clearing/viewport
    width: u32,
    height: u32,
}

/// Game app state for notan
#[derive(AppState)]
pub struct GameAppState {
    // Player state
    player_x: f32,
    player_y: f32,
    player_size: f32,
    current_direction: Direction,

    // Touch state
    is_player_touched: bool,
    drag_offset_x: f32,
    drag_offset_y: f32,

    // Game mode
    game_mode: GameMode,
    velocity_x: f32,
    velocity_y: f32,

    // Player texture (keep TextureHandle alive to prevent texture from being freed)
    player_texture: Option<egui::TextureHandle>,
    player_texture_size: (f32, f32), // (width, height) of the original image

    // Player tint color (changes on bounce)
    player_tint: Color32,

    // Time tracking
    last_frame_time: std::time::Instant,

    // Window dimensions
    width: u32,
    height: u32,
}

/// Opaque handle for FFI
pub type GameHandle = *mut GameState;

impl GameAppState {
    fn new(width: u32, height: u32) -> Self {
        let player_size = 200.0;

        Self {
            player_x: width as f32 / 2.0,
            player_y: height as f32 / 2.0,
            player_size,
            current_direction: Direction::None,
            is_player_touched: false,
            drag_offset_x: 0.0,
            drag_offset_y: 0.0,
            game_mode: GameMode::Manual,
            velocity_x: 0.0,
            velocity_y: 0.0,
            player_texture: None,
            player_texture_size: (player_size, player_size),
            player_tint: Color32::WHITE,
            last_frame_time: std::time::Instant::now(),
            width,
            height,
        }
    }

    fn load_texture(&mut self, egui_ctx: &egui::Context) {
        if self.player_texture.is_some() {
            return; // Already loaded
        }

        match image::load_from_memory(PLAYER_IMAGE_BYTES) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let img_width = rgba.width() as f32;
                let img_height = rgba.height() as f32;
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.into_raw();

                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                let texture =
                    egui_ctx.load_texture("player", color_image, egui::TextureOptions::LINEAR);
                log::info!("Player texture loaded: {}x{}", img_width, img_height);
                self.player_texture = Some(texture);
                self.player_texture_size = (img_width, img_height);
            }
            Err(e) => {
                log::error!("Failed to load player image: {}", e);
            }
        }
    }

    fn update(&mut self) {
        // Calculate delta time with frame cap to prevent huge jumps
        let now = std::time::Instant::now();
        let delta = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        // Cap delta time to prevent physics explosions after pause
        let delta = delta.min(0.1); // Max 100ms per frame

        let half = self.player_size / 2.0;

        match self.game_mode {
            GameMode::Manual => {
                // Move player based on direction
                let speed = 300.0 * delta;
                match self.current_direction {
                    Direction::Up => self.player_y -= speed,
                    Direction::Down => self.player_y += speed,
                    Direction::Left => self.player_x -= speed,
                    Direction::Right => self.player_x += speed,
                    Direction::None => {}
                }

                // Clamp to bounds
                self.player_x = self.player_x.clamp(half, self.width as f32 - half);
                self.player_y = self.player_y.clamp(half, self.height as f32 - half);
            }
            GameMode::Auto => {
                // Velocity-based movement
                self.player_x += self.velocity_x * delta;
                self.player_y += self.velocity_y * delta;

                // Bounce off walls and change color on each bounce
                if self.player_x <= half || self.player_x >= self.width as f32 - half {
                    self.velocity_x = -self.velocity_x;
                    self.player_x = self.player_x.clamp(half, self.width as f32 - half);
                    self.player_tint = random_color();
                }
                if self.player_y <= half || self.player_y >= self.height as f32 - half {
                    self.velocity_y = -self.velocity_y;
                    self.player_y = self.player_y.clamp(half, self.height as f32 - half);
                    self.player_tint = random_color();
                }
            }
        }
    }

    fn draw(&mut self, egui_ctx: &egui::Context) {
        // Skip render if dimensions are zero
        if self.width == 0 || self.height == 0 {
            log::warn!("draw called with zero dimensions");
            return;
        }

        // Load texture if needed
        self.load_texture(egui_ctx);

        // Pre-compute values
        let player_x = self.player_x;
        let player_y = self.player_y;
        let player_size = self.player_size;
        let is_touched = self.is_player_touched;
        let player_texture_id = self.player_texture.as_ref().map(|t| t.id());
        let player_texture_size = self.player_texture_size;
        let player_tint = self.player_tint;

        let painter = egui_ctx.layer_painter(egui::LayerId::background());
        let center = Pos2::new(player_x, player_y);

        // Calculate render size maintaining aspect ratio
        let (tex_w, tex_h) = player_texture_size;
        let aspect = tex_w / tex_h;
        let (render_w, render_h) = if aspect >= 1.0 {
            (player_size, player_size / aspect)
        } else {
            (player_size * aspect, player_size)
        };
        let rect = Rect::from_center_size(center, Vec2::new(render_w, render_h));

        // Draw player image or fallback to box
        if let Some(tex_id) = player_texture_id {
            let tint = if is_touched {
                Color32::from_rgb(255, 150, 50) // Orange when dragging
            } else {
                player_tint // Current color (changes on bounce)
            };

            painter.image(
                tex_id,
                rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), // UV coords
                tint,
            );
        } else {
            // Fallback: draw colored box if texture failed to load
            let fill_color = if is_touched {
                Color32::from_rgb(255, 150, 50)
            } else {
                player_tint
            };

            painter.rect(
                rect,
                Rounding::same(8.0),
                fill_color,
                Stroke::new(2.0, Color32::WHITE),
            );
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        // Center player on first resize (when dimensions were 0)
        if self.width == 0 || self.height == 0 {
            self.player_x = width as f32 / 2.0;
            self.player_y = height as f32 / 2.0;
        }

        self.width = width;
        self.height = height;
    }
}

/// Generate a random bright color based on current time
fn random_color() -> Color32 {
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let r = ((time >> 0) & 0xFF) as u8;
    let g = ((time >> 8) & 0xFF) as u8;
    let b = ((time >> 16) & 0xFF) as u8;

    // Ensure colors are bright (minimum 128)
    Color32::from_rgb(128 + (r / 2), 128 + (g / 2), 128 + (b / 2))
}

/// Embed player image at compile time
const PLAYER_IMAGE_BYTES: &[u8] = include_bytes!("../assets/player.png");

/// Initialize the game engine
/// Called from GLSurfaceView.onSurfaceCreated() on Android
/// Called from GLKView.setup() on iOS
/// Returns null on failure
#[no_mangle]
pub extern "C" fn game_init(width: u32, height: u32) -> GameHandle {
    catch_panic!(std::ptr::null_mut(), {
        // Initialize platform-specific logging (only once)
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

        log::info!("game_init: {}x{}", width, height);

        // Validate dimensions
        if width == 0 || height == 0 {
            log::warn!("game_init called with zero dimensions, will resize later");
        }

        // Determine window scale factor (typically 1.0 on mobile, but can vary)
        let window_scale_factor = 1.0;

        // Create mobile backend
        let backend = match MobileBackend::new(window_scale_factor) {
            Ok(b) => Arc::new(Mutex::new(b)),
            Err(e) => {
                log::error!("Failed to create mobile backend: {}", e);
                return std::ptr::null_mut();
            }
        };

        // Create glow context for rendering (using same proc address as backend)
        #[cfg(target_os = "android")]
        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                let c_str = match std::ffi::CString::new(s) {
                    Ok(c) => c,
                    Err(_) => return std::ptr::null(),
                };
                eglGetProcAddress(c_str.as_ptr() as *const i8)
            })
        };

        #[cfg(target_os = "ios")]
        let gl = unsafe {
            extern "C" {
                fn dlsym(handle: *mut std::ffi::c_void, symbol: *const i8)
                    -> *mut std::ffi::c_void;
            }
            const RTLD_DEFAULT: *mut std::ffi::c_void = -2isize as *mut std::ffi::c_void;

            glow::Context::from_loader_function(|s| {
                let c_str = match std::ffi::CString::new(s) {
                    Ok(c) => c,
                    Err(_) => return std::ptr::null_mut(),
                };
                dlsym(RTLD_DEFAULT, c_str.as_ptr())
            })
        };

        let gl = Arc::new(gl);

        // Set initial viewport
        unsafe {
            gl.viewport(0, 0, width as i32, height as i32);
        }

        // Create egui context
        let egui_ctx = Arc::new(Mutex::new(egui::Context::default()));

        // Create egui_glow painter for rendering
        let egui_painter = match egui_glow::Painter::new(gl.clone(), "", None, false) {
            Ok(painter) => Arc::new(Mutex::new(painter)),
            Err(e) => {
                log::error!("Failed to create egui painter: {}", e);
                return std::ptr::null_mut();
            }
        };

        // Create app state
        let app_state = Arc::new(Mutex::new(GameAppState::new(width, height)));

        let state = Box::new(GameState {
            app_state,
            backend,
            egui_ctx: Some(egui_ctx),
            egui_painter: Some(egui_painter),
            gl,
            width,
            height,
        });

        log::info!("Game initialized successfully");
        Box::into_raw(state)
    })
}

/// Handle surface size changes
/// Called from GLSurfaceView.onSurfaceChanged()
#[no_mangle]
pub extern "C" fn game_resize(handle: GameHandle, width: u32, height: u32) {
    catch_panic!((), {
        if handle.is_null() {
            return;
        }
        let state = unsafe { &mut *handle };

        // Update app state
        let mut app_state = state.app_state.lock().unwrap();
        app_state.resize(width, height);

        // Update viewport
        unsafe {
            use glow::HasContext;
            state.gl.viewport(0, 0, width as i32, height as i32);
        }

        // Notify backend of resize
        {
            let mut backend = state.backend.lock().unwrap();
            backend.push_event(MobileEvent::Resized { width, height });
        }

        state.width = width;
        state.height = height;

        log::info!("game_resize: {}x{}", width, height);
    })
}

/// Update game state
/// Called each frame before render
/// Optimized: minimal allocations, no logging in hot path
#[no_mangle]
pub extern "C" fn game_update(handle: GameHandle) {
    catch_panic!((), {
        if handle.is_null() {
            return;
        }
        let state = unsafe { &mut *handle };
        let mut app_state = state.app_state.lock().unwrap();
        app_state.update();
    })
}

/// Render the game using egui
/// Called from GLSurfaceView.onDrawFrame()
/// Optimized: pre-computed colors, minimal allocations
#[no_mangle]
pub extern "C" fn game_render(handle: GameHandle) {
    catch_panic!((), {
        if handle.is_null() {
            return;
        }
        let state = unsafe { &mut *handle };

        // Skip render if dimensions are zero
        if state.width == 0 || state.height == 0 {
            return;
        }

        // Get egui context
        let egui_ctx = match &state.egui_ctx {
            Some(e) => e.clone(),
            None => return,
        };

        let ctx = egui_ctx.lock().unwrap();

        let screen_rect = Rect::from_min_size(
            Pos2::ZERO,
            Vec2::new(state.width as f32, state.height as f32),
        );

        let raw_input = egui::RawInput {
            screen_rect: Some(screen_rect),
            ..Default::default()
        };

        // Set viewport and clear background first (before egui context run)
        unsafe {
            use glow::HasContext;
            state
                .gl
                .viewport(0, 0, state.width as i32, state.height as i32);
            state.gl.clear_color(0.1, 0.1, 0.15, 1.0);
            state.gl.clear(glow::COLOR_BUFFER_BIT);
        }

        // Draw game content into egui context
        let mut app_state = state.app_state.lock().unwrap();
        let full_output = ctx.run(raw_input, |ui_ctx| {
            app_state.draw(ui_ctx);
        });

        // Render egui output
        if let Some(egui_painter) = &state.egui_painter {
            let mut painter = egui_painter.lock().unwrap();

            // Tessellate egui shapes into renderable primitives
            let clipped_primitives = ctx.tessellate(full_output.shapes, 1.0);

            // Render the primitives and update textures
            painter.paint_and_update_textures(
                [state.width, state.height],
                1.0,
                &clipped_primitives,
                &full_output.textures_delta,
            );

            // Ensure OpenGL commands are executed
            unsafe {
                use glow::HasContext;
                state.gl.finish();

                // Check for OpenGL errors
                let error = state.gl.get_error();
                if error != glow::NO_ERROR {
                    log::error!(
                        "OpenGL error after paint_and_update_textures: 0x{:x}",
                        error
                    );
                }
            }
        } else {
            log::error!("egui_painter is None - cannot render!");
        }
    })
}

/// Handle direction input from Flutter
/// No logging in hot path for performance
#[no_mangle]
pub extern "C" fn game_set_direction(handle: GameHandle, direction: i32) {
    catch_panic!((), {
        if handle.is_null() {
            return;
        }
        let state = unsafe { &mut *handle };
        let mut app_state = state.app_state.lock().unwrap();
        app_state.current_direction = Direction::from(direction);
    })
}

/// Set game mode (Manual=0, Auto=1)
#[no_mangle]
pub extern "C" fn game_set_mode(handle: GameHandle, mode: i32) {
    catch_panic!((), {
        if handle.is_null() {
            return;
        }
        let state = unsafe { &mut *handle };
        let mut app_state = state.app_state.lock().unwrap();

        let new_mode = match mode {
            1 => GameMode::Auto,
            _ => GameMode::Manual,
        };

        // Initialize velocity when switching to auto mode
        if new_mode == GameMode::Auto && app_state.game_mode != GameMode::Auto {
            app_state.velocity_x = 250.0;
            app_state.velocity_y = 200.0;
        }

        app_state.game_mode = new_mode;
        log::info!("Game mode set to {:?}", new_mode);
    })
}

/// Handle touch events
/// Optimized: no logging in hot path, minimal branching
#[no_mangle]
pub extern "C" fn game_touch(handle: GameHandle, x: f32, y: f32, action: i32) {
    catch_panic!((), {
        if handle.is_null() {
            return;
        }
        let state = unsafe { &mut *handle };
        let mut app_state = state.app_state.lock().unwrap();
        let touch_action = TouchAction::from(action);

        // Push touch event to backend for processing
        let mut backend = state.backend.lock().unwrap();
        backend.push_event(MobileEvent::Touch { x, y, action });

        // Also handle directly for immediate response
        let half = app_state.player_size / 2.0;
        let is_on_player = x >= app_state.player_x - half
            && x <= app_state.player_x + half
            && y >= app_state.player_y - half
            && y <= app_state.player_y + half;

        match touch_action {
            TouchAction::Down => {
                if is_on_player {
                    app_state.is_player_touched = true;
                    app_state.drag_offset_x = app_state.player_x - x;
                    app_state.drag_offset_y = app_state.player_y - y;
                }
            }
            TouchAction::Up => {
                app_state.is_player_touched = false;
            }
            TouchAction::Move => {
                if app_state.is_player_touched {
                    app_state.player_x = x + app_state.drag_offset_x;
                    app_state.player_y = y + app_state.drag_offset_y;

                    // Clamp to screen bounds
                    app_state.player_x = app_state
                        .player_x
                        .clamp(half, app_state.width as f32 - half);
                    app_state.player_y = app_state
                        .player_y
                        .clamp(half, app_state.height as f32 - half);
                }
            }
        }
    })
}

/// Get player X position (for debugging/verification)
#[no_mangle]
pub extern "C" fn game_get_player_x(handle: GameHandle) -> f32 {
    catch_panic!(0.0, {
        if handle.is_null() {
            return 0.0;
        }
        let state = unsafe { &*handle };
        let app_state = state.app_state.lock().unwrap();
        app_state.player_x
    })
}

/// Get player Y position (for debugging/verification)
#[no_mangle]
pub extern "C" fn game_get_player_y(handle: GameHandle) -> f32 {
    catch_panic!(0.0, {
        if handle.is_null() {
            return 0.0;
        }
        let state = unsafe { &*handle };
        let app_state = state.app_state.lock().unwrap();
        app_state.player_y
    })
}

/// Clean up resources
/// Safe to call multiple times (idempotent)
#[no_mangle]
pub extern "C" fn game_destroy(handle: GameHandle) {
    catch_panic!((), {
        if handle.is_null() {
            return;
        }
        let state = unsafe { Box::from_raw(handle) };

        // Notify backend of exit
        {
            let mut backend_guard = state.backend.lock().unwrap();
            backend_guard.push_event(MobileEvent::Exit);
        }

        log::info!("game_destroy: cleaned up");
        // state is dropped here, freeing all resources
    })
}
