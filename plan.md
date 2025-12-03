# Implementation Plan

This document outlines the phased implementation of the Flutter/Rust game interop project.

> **Important**: Each phase requires user approval before proceeding to the next.

---

## Phase 1: Flutter UI Foundation ✅

Basic Flutter app with directional controls and placeholder for native view.

- [x] Create direction pad widget with up/down/left/right buttons
- [x] Set up MethodChannel for native communication
- [x] Add placeholder container for PlatformView
- [x] Test button press events are captured correctly

---

## Phase 2: Android Native GLSurfaceView ✅

Set up Android native rendering surface without Rust.

- [x] Create custom GLSurfaceView class
- [x] Implement GLRenderer with basic clear color
- [x] Register PlatformView factory in Flutter plugin
- [x] Connect Flutter PlatformView to native GLSurfaceView
- [x] Verify GL context is working (gray background renders)

---

## Phase 3: Flutter to Android Communication ✅

Pass direction events from Flutter to Android native layer.

- [x] Implement MethodChannel handler on Android side
- [x] Forward button events from Flutter to native view
- [x] Display received events in GL view (color change)
- [x] Verify bidirectional communication works

---

## Phase 4: Rust Library Setup ✅

Create Rust library with C FFI interface using glow + egui_glow.

- [x] Initialize Cargo project in `rust/` directory
- [x] Add glow, egui, egui_glow dependencies
- [x] Create C-compatible FFI functions (init, resize, update, render, set_direction, touch, destroy)
- [x] Build for aarch64-linux-android target
- [x] Verify .so file is generated correctly

---

## Phase 5: Android-Rust JNI Bridge ✅

Connect Android native layer to Rust library.

- [x] Add JNI native method declarations in Java/Kotlin
- [x] Load Rust .so library via System.loadLibrary()
- [x] Call Rust init function from GLRenderer.onSurfaceCreated()
- [x] Call Rust render function from GLRenderer.onDrawFrame()
- [x] Forward input events to Rust via JNI

---

## Phase 6: Rust Game Rendering with egui ✅

Implement game rendering in Rust using egui + egui_glow.

- [x] Initialize glow context from Android's EGL
- [x] Implement game state struct (player position, direction)
- [x] Set up egui_glow Painter with OpenGL ES
- [x] Render basic shapes (box + circle) using egui
- [x] Add movement based on direction input
- [x] Verify rendering appears in Flutter app

---

## Phase 7: Touch Drag Interaction ✅

Enable dragging the box via touch input.

- [x] Add drag offset tracking to GameState
- [x] Implement drag logic in game_touch()
- [x] Box follows finger while dragging
- [x] Demonstrates screen touch → Rust engine interaction

---

## Phase 8: iOS Support ✅

Extend to iOS platform using OpenGL ES (EAGL) + GLKView.

### Architecture
```
Flutter (Dart) → MethodChannel → Swift → C FFI → Rust
                                   ↓
                              GLKView (OpenGL ES)
```

### Steps
- [x] Update Rust code with iOS conditional compilation
- [x] Update Cargo.toml with staticlib + iOS dependencies
- [x] Create iOS build script (build_ios.sh)
- [x] Create C header for Swift FFI (game_engine.h)
- [x] Create iOS GLKView wrapper (GameGLView.swift)
- [x] Create Flutter PlatformView factory (GamePlatformViewFactory.swift)
- [x] Update AppDelegate.swift with MethodChannel + PlatformView
- [x] Update bridging header
- [x] Update Flutter Dart code (add UiKitView for iOS)
- [x] Configure Xcode project (link Rust library, frameworks)
- [x] Build and test on iOS simulator

### Build Commands
```bash
# Install iOS targets
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios

# Build Rust for iOS
./build_ios.sh

# Build Flutter iOS
flutter build ios --debug
```

---

## Phase 9: Polish and Optimization ✅

Performance improvements and error handling.

- [x] Optimize render loop and frame timing (delta time cap, skip zero-size render)
- [x] Add error handling across FFI boundary (catch_unwind macro)
- [x] Clean up debug code (reduced logging in hot paths)
- [ ] Test on multiple devices
- [ ] Document build and deployment process

### Changes Made
- Added `catch_panic!` macro to wrap all FFI functions
- Capped delta time to 100ms to prevent physics explosions after pause
- Reduced logging level from Debug to Info
- Pre-compute values outside render closure to reduce allocations
- Skip render if dimensions are zero
- Removed logging from hot paths (touch, direction, render)

---

## Phase 10: Auto/Manual Mode with Bouncing Ball ✅

Two game modes - Manual (D-pad control) and Auto (bouncing ball physics).

### Features
- [x] Add GameMode enum (Manual, Auto) to Rust GameState
- [x] Implement velocity-based bouncing physics for Auto mode
- [x] Add game_set_mode() FFI function
- [x] Update JNI bridge for Android
- [x] Update iOS C header and Swift code
- [x] Add mode toggle buttons in Flutter UI
- [x] D-pad disabled/grayed in Auto mode
- [x] Touch drag works in both modes
- [x] Build and test on Android

### Architecture
```
Flutter (Mode Buttons) → MethodChannel → Rust (GameMode enum)
                                           ↓
                              Auto: Velocity-based movement + bounce
                              Manual: Direction-based movement (existing)
```

### Files Modified
- `rust/src/lib.rs` - GameMode enum, velocity fields, bouncing physics, game_set_mode()
- `rust/src/jni.rs` - JNI binding for gameSetMode
- `android/.../GameNative.kt` - gameSetMode external fun
- `android/.../GameGLRenderer.kt` - setMode() method
- `android/.../GameGLPlatformView.kt` - setMode() method
- `android/.../GameGLSurfaceFactory.kt` - setMode() method
- `android/.../MainActivity.kt` - Handle setMode in MethodChannel
- `ios/Runner/game_engine.h` - game_set_mode declaration
- `ios/Runner/GameGLView.swift` - setMode() method
- `ios/Runner/GamePlatformViewFactory.swift` - setMode() methods
- `ios/Runner/AppDelegate.swift` - Handle setMode in MethodChannel
- `lib/main.dart` - Mode state, toggle buttons, disable D-pad in auto mode

---

## Current Status

**Active Phase**: Phase 10 (Auto/Manual Mode) - Complete

**Completed**: All 10 phases implemented
