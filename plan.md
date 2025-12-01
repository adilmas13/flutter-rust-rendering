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

## Phase 2: Android Native GLSurfaceView

Set up Android native rendering surface without Rust.

- [ ] Create custom GLSurfaceView class
- [ ] Implement GLRenderer with basic clear color
- [ ] Register PlatformView factory in Flutter plugin
- [ ] Connect Flutter PlatformView to native GLSurfaceView
- [ ] Verify GL context is working (changing background color)

---

## Phase 3: Flutter to Android Communication

Pass direction events from Flutter to Android native layer.

- [ ] Implement MethodChannel handler on Android side
- [ ] Forward button events from Flutter to native view
- [ ] Display received events in GL view (debug text or color change)
- [ ] Verify bidirectional communication works

---

## Phase 4: Rust Library Setup

Create Rust library with C FFI interface.

- [ ] Initialize Cargo project in `rust/` directory
- [ ] Add Notan and egui dependencies
- [ ] Create C-compatible FFI functions (init, update, render, handle_input)
- [ ] Build for aarch64-linux-android target
- [ ] Verify .so file is generated correctly

---

## Phase 5: Android-Rust JNI Bridge

Connect Android native layer to Rust library.

- [ ] Add JNI native method declarations in Java/Kotlin
- [ ] Load Rust .so library via System.loadLibrary()
- [ ] Call Rust init function from GLRenderer.onSurfaceCreated()
- [ ] Call Rust render function from GLRenderer.onDrawFrame()
- [ ] Forward input events to Rust via JNI

---

## Phase 6: Rust Game Engine with Notan

Implement basic game rendering in Rust.

- [ ] Initialize Notan with external GL context (no window creation)
- [ ] Implement game state struct
- [ ] Render basic shapes responding to input
- [ ] Add movement based on direction input
- [ ] Verify rendering appears in Flutter app

---

## Phase 7: egui Integration

Add immediate-mode UI via egui in Rust.

- [ ] Integrate egui with Notan using notan_egui
- [ ] Create simple debug panel showing game state
- [ ] Add in-game UI elements (score, status)
- [ ] Handle egui input if needed

---

## Phase 8: iOS Support

Extend to iOS platform.

- [ ] Create iOS GLKit/Metal view
- [ ] Build Rust static library for aarch64-apple-ios
- [ ] Link Rust library in Xcode project
- [ ] Implement C FFI calls from Swift/Objective-C
- [ ] Register iOS PlatformView factory
- [ ] Verify game runs on iOS

---

## Phase 9: Polish and Optimization

Final improvements and cleanup.

- [ ] Optimize render loop and frame timing
- [ ] Add error handling across FFI boundary
- [ ] Clean up debug code
- [ ] Test on multiple devices
- [ ] Document build and deployment process

---

## Current Status

**Active Phase**: Phase 1 completed

**Next Step**: Awaiting approval to begin Phase 2
