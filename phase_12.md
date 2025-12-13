# Phase 12: Flutter Rust Bridge Integration

Replace manual FFI/JNI bindings with flutter_rust_bridge for type-safe, auto-generated Dart ↔ Rust communication.

---

## Architecture Change

```
BEFORE (Current):
Flutter → MethodChannel → Kotlin/Swift → JNI/C FFI → Rust
                         (manual bindings)

AFTER (Target):
Flutter → flutter_rust_bridge (auto-generated) → Rust
                    (type-safe, async)
```

---

## Step 1: Update `rust/Cargo.toml`

```toml
[package]
name = "game_engine"
version = "0.1.0"
edition = "2021"

[lib]
name = "game_engine"
crate-type = ["cdylib", "staticlib"]

[dependencies]
# Flutter Rust Bridge
flutter_rust_bridge = "2.0"

# OpenGL bindings
glow = "0.14"

# egui core (no winit dependency)
egui = { version = "0.29", default-features = false }

# egui OpenGL renderer
egui_glow = { version = "0.29", default-features = false }

# Logging
log = "0.4"

# Image loading for textures
image = { version = "0.25", default-features = false, features = ["png"] }

# Android-specific dependencies
[target.'cfg(target_os = "android")'.dependencies]
android_logger = "0.14"

# iOS-specific dependencies
[target.'cfg(target_os = "ios")'.dependencies]
oslog = "0.2"

[profile.release]
lto = true
opt-level = "z"
strip = true
```

---

## Step 2: Create `rust/src/api.rs`

This is the main API file that flutter_rust_bridge will parse:

```rust
//! Flutter Rust Bridge API
//!
//! This module defines the public API exposed to Flutter/Dart.

use std::sync::Arc;
use flutter_rust_bridge::frb;

use egui::{Color32, Pos2, Rect, Rounding, Stroke, Vec2};
use glow::HasContext;

// =============================================================================
// Enums (auto-generated as Dart enums)
// =============================================================================

/// Direction enum for player movement
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    #[default]
    None,
    Up,
    Down,
    Left,
    Right,
}

/// Game mode enum
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum GameMode {
    #[default]
    Manual,
    Auto,
}

/// Touch action enum
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TouchAction {
    Down,
    Up,
    Move,
}

// =============================================================================
// Data Structures (auto-generated as Dart classes)
// =============================================================================

/// Game state snapshot for Flutter
#[derive(Clone, Debug)]
pub struct GameStateSnapshot {
    pub player_x: f32,
    pub player_y: f32,
    pub player_size: f32,
    pub mode: GameMode,
    pub direction: Direction,
    pub is_touched: bool,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub width: u32,
    pub height: u32,
}

// =============================================================================
// GameEngine (Opaque Type - lives in Rust memory)
// =============================================================================

/// Link to EGL for eglGetProcAddress (Android)
#[cfg(target_os = "android")]
#[link(name = "EGL")]
extern "C" {
    fn eglGetProcAddress(procname: *const i8) -> *const std::ffi::c_void;
}

/// iOS GL loader
#[cfg(target_os = "ios")]
extern "C" {
    fn CFBundleGetBundleWithIdentifier(bundleID: *const std::ffi::c_void) -> *const std::ffi::c_void;
    fn CFBundleGetFunctionPointerForName(bundle: *const std::ffi::c_void, functionName: *const std::ffi::c_void) -> *const std::ffi::c_void;
}

/// The main game engine - opaque to Dart, managed by Rust
#[frb(opaque)]
pub struct GameEngine {
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
    drag_offset_x: f32,
    drag_offset_y: f32,

    // Game mode and velocity (Auto mode)
    game_mode: GameMode,
    velocity_x: f32,
    velocity_y: f32,

    // Player texture
    player_texture: Option<egui::TextureHandle>,
    player_texture_size: (f32, f32),

    // Player tint color (changes on bounce in Auto mode)
    player_tint: Color32,

    // Time tracking
    last_frame_time: std::time::Instant,
}

impl GameEngine {
    /// Create a new game engine instance
    /// Called once when the GL surface is created
    #[frb(sync)]
    pub fn new(width: u32, height: u32) -> Result<GameEngine, String> {
        // Initialize platform-specific logging
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

        log::info!("GameEngine::new: {}x{}", width, height);

        // Create glow context
        let gl = unsafe {
            #[cfg(target_os = "android")]
            let gl = glow::Context::from_loader_function(|s| {
                let c_str = std::ffi::CString::new(s).unwrap();
                eglGetProcAddress(c_str.as_ptr() as *const i8)
            });

            #[cfg(target_os = "ios")]
            let gl = glow::Context::from_loader_function(|s| {
                // iOS GL loader implementation
                std::ptr::null() // Simplified - actual impl uses CFBundle
            });

            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            let gl = glow::Context::from_loader_function(|_| std::ptr::null());

            gl
        };
        let gl = Arc::new(gl);

        // Set initial viewport
        unsafe {
            gl.viewport(0, 0, width as i32, height as i32);
        }

        // Create egui context
        let egui_ctx = egui::Context::default();

        // Create egui_glow painter
        let egui_painter = egui_glow::Painter::new(gl.clone(), "", None, false)
            .map_err(|e| format!("Failed to create egui painter: {}", e))?;

        let player_size = 200.0;

        // Load player texture
        let (player_texture, player_texture_size) = Self::load_player_texture(&egui_ctx);

        let engine = GameEngine {
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
            drag_offset_x: 0.0,
            drag_offset_y: 0.0,
            game_mode: GameMode::Manual,
            velocity_x: 0.0,
            velocity_y: 0.0,
            player_texture,
            player_texture_size,
            player_tint: Color32::WHITE,
            last_frame_time: std::time::Instant::now(),
        };

        log::info!("GameEngine initialized successfully");
        Ok(engine)
    }

    /// Load player texture from embedded bytes
    fn load_player_texture(ctx: &egui::Context) -> (Option<egui::TextureHandle>, (f32, f32)) {
        const PLAYER_IMAGE_BYTES: &[u8] = include_bytes!("../assets/player.png");

        match image::load_from_memory(PLAYER_IMAGE_BYTES) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.into_raw();

                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                let texture = ctx.load_texture("player", color_image, egui::TextureOptions::LINEAR);

                log::info!("Player texture loaded: {}x{}", size[0], size[1]);
                (Some(texture), (size[0] as f32, size[1] as f32))
            }
            Err(e) => {
                log::error!("Failed to load player image: {}", e);
                (None, (200.0, 200.0))
            }
        }
    }

    /// Handle surface resize
    #[frb(sync)]
    pub fn resize(&mut self, width: u32, height: u32) {
        // Center player on first resize (when dimensions were 0)
        if self.width == 0 || self.height == 0 {
            self.player_x = width as f32 / 2.0;
            self.player_y = height as f32 / 2.0;
        }

        self.width = width;
        self.height = height;

        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
        }

        log::info!("GameEngine::resize: {}x{}", width, height);
    }

    /// Update game state - call before render each frame
    #[frb(sync)]
    pub fn update(&mut self) {
        // Calculate delta time with frame cap
        let now = std::time::Instant::now();
        let delta = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        // Cap delta time to prevent physics explosions after pause
        let delta = delta.min(0.1);

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
                    self.player_tint = Self::random_color();
                }
                if self.player_y <= half || self.player_y >= self.height as f32 - half {
                    self.velocity_y = -self.velocity_y;
                    self.player_y = self.player_y.clamp(half, self.height as f32 - half);
                    self.player_tint = Self::random_color();
                }
            }
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
        Color32::from_rgb(128 + r / 2, 128 + g / 2, 128 + b / 2)
    }

    /// Render the game - call from GL draw frame
    #[frb(sync)]
    pub fn render(&mut self) {
        // Skip render if dimensions are zero
        if self.width == 0 || self.height == 0 {
            return;
        }

        // Clear background
        unsafe {
            self.gl.clear_color(0.1, 0.1, 0.15, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }

        let screen_rect = Rect::from_min_size(
            Pos2::ZERO,
            Vec2::new(self.width as f32, self.height as f32),
        );

        // Run egui frame
        let raw_input = egui::RawInput {
            screen_rect: Some(screen_rect),
            ..Default::default()
        };

        // Calculate aspect-ratio-preserving render size
        let (tex_w, tex_h) = self.player_texture_size;
        let aspect = tex_w / tex_h;
        let (render_w, render_h) = if aspect >= 1.0 {
            (self.player_size, self.player_size / aspect)
        } else {
            (self.player_size * aspect, self.player_size)
        };

        let player_x = self.player_x;
        let player_y = self.player_y;
        let is_touched = self.is_player_touched;
        let tint = self.player_tint;
        let player_texture = self.player_texture.clone();

        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            let painter = ctx.layer_painter(egui::LayerId::background());

            // Determine tint color
            let fill_color = if is_touched {
                Color32::from_rgb(255, 150, 50) // Orange when dragging
            } else {
                tint
            };

            // Calculate top-left position (center at player_x, player_y)
            let x = player_x - render_w / 2.0;
            let y = player_y - render_h / 2.0;

            // Draw player
            if let Some(tex) = &player_texture {
                let rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(render_w, render_h));
                let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
                painter.image(tex.id(), rect, uv, fill_color);
            } else {
                // Fallback: colored rectangle if texture failed to load
                let rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(render_w, render_h));
                painter.rect(
                    rect,
                    Rounding::same(8.0),
                    fill_color,
                    Stroke::new(2.0, Color32::WHITE),
                );
            }
        });

        // Tessellate and paint
        let pixels_per_point = 1.0;
        let clipped_primitives = self.egui_ctx.tessellate(full_output.shapes, pixels_per_point);

        self.egui_painter.paint_and_update_textures(
            [self.width, self.height],
            pixels_per_point,
            &clipped_primitives,
            &full_output.textures_delta,
        );
    }

    /// Set player movement direction
    #[frb(sync)]
    pub fn set_direction(&mut self, direction: Direction) {
        self.current_direction = direction;
    }

    /// Set game mode (Manual or Auto)
    #[frb(sync)]
    pub fn set_mode(&mut self, mode: GameMode) {
        // Initialize velocity when switching to auto mode
        if mode == GameMode::Auto && self.game_mode != GameMode::Auto {
            self.velocity_x = 250.0;
            self.velocity_y = 200.0;
        }

        self.game_mode = mode;
        log::info!("Game mode set to {:?}", mode);
    }

    /// Handle touch event
    #[frb(sync)]
    pub fn touch(&mut self, x: f32, y: f32, action: TouchAction) {
        // Check if touch is within player box
        let half = self.player_size / 2.0;
        let is_on_player = x >= self.player_x - half
            && x <= self.player_x + half
            && y >= self.player_y - half
            && y <= self.player_y + half;

        match action {
            TouchAction::Down => {
                if is_on_player {
                    self.is_player_touched = true;
                    self.drag_offset_x = self.player_x - x;
                    self.drag_offset_y = self.player_y - y;
                }
            }
            TouchAction::Up => {
                self.is_player_touched = false;
            }
            TouchAction::Move => {
                if self.is_player_touched {
                    self.player_x = x + self.drag_offset_x;
                    self.player_y = y + self.drag_offset_y;

                    // Clamp to screen bounds
                    self.player_x = self.player_x.clamp(half, self.width as f32 - half);
                    self.player_y = self.player_y.clamp(half, self.height as f32 - half);
                }
            }
        }
    }

    /// Get current game state snapshot
    #[frb(sync)]
    pub fn get_state(&self) -> GameStateSnapshot {
        GameStateSnapshot {
            player_x: self.player_x,
            player_y: self.player_y,
            player_size: self.player_size,
            mode: self.game_mode,
            direction: self.current_direction,
            is_touched: self.is_player_touched,
            velocity_x: self.velocity_x,
            velocity_y: self.velocity_y,
            width: self.width,
            height: self.height,
        }
    }

    /// Get player X position
    #[frb(sync)]
    pub fn get_player_x(&self) -> f32 {
        self.player_x
    }

    /// Get player Y position
    #[frb(sync)]
    pub fn get_player_y(&self) -> f32 {
        self.player_y
    }
}

impl Drop for GameEngine {
    fn drop(&mut self) {
        self.egui_painter.destroy();
        log::info!("GameEngine destroyed");
    }
}
```

---

## Step 3: Update `rust/src/lib.rs`

```rust
//! Flutter/Rust game engine using flutter_rust_bridge
//!
//! This library provides a game engine that renders via egui
//! while being controlled from Flutter through flutter_rust_bridge.

mod api;

// Re-export API types for flutter_rust_bridge
pub use api::*;

// Include generated bridge code
mod frb_generated;
```

---

## Step 4: Create `flutter_rust_bridge.yaml` (in project root)

```yaml
rust_input: rust/src/api.rs
dart_output: lib/src/rust/
```

---

## Step 5: Update `pubspec.yaml`

```yaml
name: flutter_con
description: Flutter/Rust game demo with flutter_rust_bridge

publish_to: 'none'

version: 1.0.0+1

environment:
  sdk: '>=3.0.0 <4.0.0'

dependencies:
  flutter:
    sdk: flutter
  flutter_rust_bridge: ^2.0.0
  cupertino_icons: ^1.0.2

dev_dependencies:
  flutter_test:
    sdk: flutter
  flutter_lints: ^2.0.0
  ffigen: ^8.0.0

flutter:
  uses-material-design: true
```

---

## Step 6: Update `lib/main.dart`

```dart
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

// Import generated flutter_rust_bridge code
import 'src/rust/api.dart';
import 'src/rust/frb_generated.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Rust Game',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      home: const GamePage(),
    );
  }
}

class GamePage extends StatefulWidget {
  const GamePage({super.key});

  @override
  State<GamePage> createState() => _GamePageState();
}

class _GamePageState extends State<GamePage> {
  GameEngine? _engine;
  GameMode _currentMode = GameMode.Manual;
  bool _isAutoMode = false;

  @override
  void initState() {
    super.initState();
  }

  void _onEngineCreated(GameEngine engine) {
    setState(() {
      _engine = engine;
    });
  }

  void _setDirection(Direction direction) {
    _engine?.setDirection(direction: direction);
  }

  void _clearDirection() {
    _engine?.setDirection(direction: Direction.None);
  }

  void _toggleMode() {
    setState(() {
      _isAutoMode = !_isAutoMode;
      _currentMode = _isAutoMode ? GameMode.Auto : GameMode.Manual;
      _engine?.setMode(mode: _currentMode);
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Flutter + Rust Game'),
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
      ),
      body: Column(
        children: [
          // Game view
          Expanded(
            child: GameView(
              onEngineCreated: _onEngineCreated,
            ),
          ),
          // Mode toggle
          Padding(
            padding: const EdgeInsets.all(8.0),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                ChoiceChip(
                  label: const Text('Manual'),
                  selected: !_isAutoMode,
                  onSelected: (_) => _toggleMode(),
                ),
                const SizedBox(width: 16),
                ChoiceChip(
                  label: const Text('Auto'),
                  selected: _isAutoMode,
                  onSelected: (_) => _toggleMode(),
                ),
              ],
            ),
          ),
          // Direction pad
          Opacity(
            opacity: _isAutoMode ? 0.3 : 1.0,
            child: IgnorePointer(
              ignoring: _isAutoMode,
              child: DirectionPad(
                onDirectionStart: _setDirection,
                onDirectionEnd: _clearDirection,
              ),
            ),
          ),
          const SizedBox(height: 20),
        ],
      ),
    );
  }
}

/// Direction pad widget
class DirectionPad extends StatelessWidget {
  final void Function(Direction) onDirectionStart;
  final void Function() onDirectionEnd;

  const DirectionPad({
    super.key,
    required this.onDirectionStart,
    required this.onDirectionEnd,
  });

  Widget _buildButton(Direction direction, IconData icon) {
    return GestureDetector(
      onTapDown: (_) => onDirectionStart(direction),
      onTapUp: (_) => onDirectionEnd(),
      onTapCancel: onDirectionEnd,
      child: Container(
        width: 60,
        height: 60,
        decoration: BoxDecoration(
          color: Colors.grey[300],
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: Colors.grey[600]!),
        ),
        child: Icon(icon, size: 30),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        _buildButton(Direction.Up, Icons.arrow_upward),
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            _buildButton(Direction.Left, Icons.arrow_back),
            const SizedBox(width: 60, height: 60),
            _buildButton(Direction.Right, Icons.arrow_forward),
          ],
        ),
        _buildButton(Direction.Down, Icons.arrow_downward),
      ],
    );
  }
}

/// Game rendering view - uses PlatformView for GL surface
class GameView extends StatefulWidget {
  final void Function(GameEngine) onEngineCreated;

  const GameView({super.key, required this.onEngineCreated});

  @override
  State<GameView> createState() => _GameViewState();
}

class _GameViewState extends State<GameView> {
  static const String viewType = 'game_gl_view';

  @override
  Widget build(BuildContext context) {
    if (Platform.isAndroid) {
      return AndroidView(
        viewType: viewType,
        onPlatformViewCreated: _onPlatformViewCreated,
        creationParamsCodec: const StandardMessageCodec(),
      );
    } else if (Platform.isIOS) {
      return UiKitView(
        viewType: viewType,
        onPlatformViewCreated: _onPlatformViewCreated,
        creationParamsCodec: const StandardMessageCodec(),
      );
    } else {
      return const Center(child: Text('Platform not supported'));
    }
  }

  void _onPlatformViewCreated(int id) {
    // The PlatformView will create the GameEngine internally
    // and communicate via flutter_rust_bridge
    // For now, we create it here with default size
    // Actual size comes from resize callback
    try {
      final engine = GameEngine.newInstance(width: 100, height: 100);
      widget.onEngineCreated(engine);
    } catch (e) {
      debugPrint('Failed to create GameEngine: $e');
    }
  }
}
```

---

## Step 7: Update Android Native Code

### `android/app/src/main/kotlin/.../GameGLRenderer.kt`

The renderer now uses flutter_rust_bridge instead of JNI:

```kotlin
package com.example.flutter_con

import android.opengl.GLES30
import android.opengl.GLSurfaceView
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10

class GameGLRenderer : GLSurfaceView.Renderer {

    // GameEngine is now managed via flutter_rust_bridge
    // The Dart side holds the reference

    private var width: Int = 0
    private var height: Int = 0

    // Callback to notify Dart side
    var onSurfaceCreatedCallback: ((Int, Int) -> Unit)? = null
    var onSurfaceChangedCallback: ((Int, Int) -> Unit)? = null
    var onDrawFrameCallback: (() -> Unit)? = null

    override fun onSurfaceCreated(gl: GL10?, config: EGLConfig?) {
        GLES30.glClearColor(0.1f, 0.1f, 0.15f, 1.0f)
        onSurfaceCreatedCallback?.invoke(width, height)
    }

    override fun onSurfaceChanged(gl: GL10?, width: Int, height: Int) {
        this.width = width
        this.height = height
        GLES30.glViewport(0, 0, width, height)
        onSurfaceChangedCallback?.invoke(width, height)
    }

    override fun onDrawFrame(gl: GL10?) {
        GLES30.glClear(GLES30.GL_COLOR_BUFFER_BIT)
        onDrawFrameCallback?.invoke()
    }
}
```

### Remove `GameNative.kt` (JNI declarations no longer needed)

Delete or comment out the file as JNI is replaced by flutter_rust_bridge.

---

## Step 8: Build Commands

```bash
# 1. Install flutter_rust_bridge codegen
cargo install flutter_rust_bridge_codegen

# 2. Generate Dart/Rust bindings
flutter_rust_bridge_codegen generate

# 3. Build Rust library for Android
cd rust
cargo ndk -t arm64-v8a build --release

# 4. Copy to jniLibs (if not using Gradle integration)
cp target/aarch64-linux-android/release/libgame_engine.so \
   ../android/app/src/main/jniLibs/arm64-v8a/

# 5. Run Flutter app
cd ..
flutter run
```

---

## Generated Files (by flutter_rust_bridge_codegen)

After running `flutter_rust_bridge_codegen generate`, these files are auto-created:

### `rust/src/frb_generated.rs`
Auto-generated Rust bridge code.

### `lib/src/rust/api.dart`
```dart
// Auto-generated - DO NOT EDIT

enum Direction {
  none,
  up,
  down,
  left,
  right,
}

enum GameMode {
  manual,
  auto,
}

enum TouchAction {
  down,
  up,
  move,
}

class GameStateSnapshot {
  final double playerX;
  final double playerY;
  final double playerSize;
  final GameMode mode;
  final Direction direction;
  final bool isTouched;
  final double velocityX;
  final double velocityY;
  final int width;
  final int height;
  // ...
}

class GameEngine {
  // Opaque handle to Rust object

  static GameEngine newInstance({required int width, required int height}) { ... }

  void resize({required int width, required int height}) { ... }
  void update() { ... }
  void render() { ... }
  void setDirection({required Direction direction}) { ... }
  void setMode({required GameMode mode}) { ... }
  void touch({required double x, required double y, required TouchAction action}) { ... }
  GameStateSnapshot getState() { ... }
  double getPlayerX() { ... }
  double getPlayerY() { ... }
}
```

### `lib/src/rust/frb_generated.dart`
```dart
// Auto-generated bridge initialization

class RustLib {
  static Future<void> init() async {
    // Initialize the bridge
  }
}
```

---

## Checklist

- [ ] Add `flutter_rust_bridge = "2.0"` to `rust/Cargo.toml`
- [ ] Add `flutter_rust_bridge: ^2.0.0` to `pubspec.yaml`
- [ ] Create `rust/src/api.rs` with GameEngine and enums
- [ ] Update `rust/src/lib.rs` to export api module
- [ ] Create `flutter_rust_bridge.yaml` config
- [ ] Run `flutter_rust_bridge_codegen generate`
- [ ] Update `lib/main.dart` to use generated API
- [ ] Update Android renderer to use callbacks
- [ ] Remove old JNI code (`jni.rs`, `GameNative.kt`)
- [ ] Build and test on Android
- [ ] Build and test on iOS

---

## Benefits Summary

| Before | After |
|--------|-------|
| Manual `extern "C"` functions | Auto-generated type-safe API |
| Raw pointers (`GameHandle`) | Opaque types with methods |
| JNI boilerplate (Kotlin) | Direct Dart → Rust calls |
| MethodChannel string parsing | Native Dart enums/classes |
| Manual error handling | `Result<T, E>` → exceptions |

---

Awaiting your review and approval to implement this code.
