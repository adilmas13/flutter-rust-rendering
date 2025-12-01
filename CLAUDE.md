# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> **Reference [plan.md](plan.md) for implementation phases and task checklists.** Each phase requires user approval before proceeding.

## Development Workflow

1. For each phase, first write all code to `phase_<number>.md` (e.g., `phase_1.md`, `phase_2.md`)
2. Wait for user review and approval of the code in the phase file
3. Only after approval, implement the code into the actual project files
4. Mark phase checklist items complete in `plan.md`
5. Commit the code after each successful phase
6. Wait for approval before starting the next phase

## Project Overview

A Flutter game demonstrating Dart/Rust interop through native platform layers. The game renders via a Rust engine while Flutter handles UI controls.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Flutter/Dart                         │
│  - PlatformView for native rendering surface            │
│  - Four directional buttons (up, down, left, right)     │
│  - Sends events to native layer                         │
└────────────────────────┬────────────────────────────────┘
                         │ Platform Channels
┌────────────────────────▼────────────────────────────────┐
│              Native (Android/iOS)                       │
│  - Android: GLSurfaceView + GLRenderer                  │
│  - iOS: Metal/GLKit view                                │
│  - Bridges Flutter events to Rust via FFI               │
└────────────────────────┬────────────────────────────────┘
                         │ FFI (C ABI)
┌────────────────────────▼────────────────────────────────┐
│                      Rust                               │
│  - glow: OpenGL bindings (uses Android's GL context)    │
│  - egui + egui_glow: Immediate-mode UI rendering        │
│  - Game logic and rendering                             │
└─────────────────────────────────────────────────────────┘
```

### Layer Responsibilities

**Flutter/Dart (`lib/`)**
- PlatformView widget embedding native rendering surface
- Direction pad UI with up/down/left/right buttons
- MethodChannel/EventChannel for native communication

**Native Android (`android/`)**
- GLSurfaceView with custom GLRenderer
- JNI bindings to Rust library
- Event forwarding from Flutter to Rust

**Native iOS (`ios/`)**
- GLKit or Metal view for rendering
- C FFI bindings to Rust library
- Event forwarding from Flutter to Rust

**Rust (`rust/` - to be created)**
- glow for OpenGL bindings (works with external GL context from Android)
- egui + egui_glow for immediate-mode UI rendering
- Game state and logic
- Exposes C-compatible FFI for native layers
- Receives both direction pad and touch events

## Build Commands

```bash
# Flutter
flutter pub get
flutter run
flutter test
flutter analyze

# Rust (from rust/ directory)
cargo build --release --target aarch64-linux-android    # Android ARM64
cargo build --release --target aarch64-apple-ios        # iOS ARM64

# Android NDK build (generates .so files)
cargo ndk -t arm64-v8a build --release
```

## Key Dependencies

- **glow** (Rust): OpenGL bindings - works with external GL context
- **egui** (Rust): Immediate-mode GUI library
- **egui_glow** (Rust): egui OpenGL renderer using glow
- **flutter_lints**: Dart static analysis

## FFI Considerations

- Rust functions exposed via `#[no_mangle]` and `extern "C"`
- Android: Load `.so` via `System.loadLibrary()`, call through JNI
- iOS: Static library linked, called directly via C interop
