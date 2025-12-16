# Flutter Rust Game Engine Demo

> **Presentation**: This repository contains the source code for the presentation **"BUILDING A CROSS‑PLATFORM NATIVE RENDERING ENGINE IN RUST FOR FLUTTER"** presented at **Flutter Con India 2025**.
>
> 📊 [View Presentation Slides](https://docs.google.com/presentation/d/1Dd35OlN2G9vk5JDsuf4MVLU_YNhSkXjGuGCe3JSJmsw/edit?usp=sharing)

A cross-platform demo app showcasing Flutter and Rust interoperability for game rendering. The Rust game engine handles OpenGL ES rendering via egui/egui_glow, while Flutter provides the UI layer.

## Features

- **Flutter ↔ Rust FFI**: Bidirectional communication between Flutter UI and Rust game engine
- **OpenGL ES Rendering**: Hardware-accelerated graphics via glow + egui_glow
- **Touch Input**: Drag interaction passed from Flutter through native layer to Rust
- **Image Textures**: PNG sprite rendering with dynamic tint effects
- **Two Game Modes**:
  - Manual: Control movement with D-pad
  - Auto: Bouncing physics with color changes on wall collision

## Architecture

```
Flutter (Dart UI)
       │
       │ MethodChannel
       ▼
┌──────────────────────────────────────┐
│  Android: Kotlin → JNI → Rust (.so)  │
│  iOS: Swift → C FFI → Rust (.a)      │
└──────────────────────────────────────┘
       │
       ▼
Rust Game Engine (glow + egui_glow)
       │
       ▼
OpenGL ES Rendering (GLSurfaceView / GLKView)
```

## Prerequisites

### General
- Flutter SDK 3.x+
- Rust toolchain via [rustup](https://rustup.rs/)

### Android
- Android Studio
- Android NDK (install via SDK Manager)
- cargo-ndk: `cargo install cargo-ndk`
- Rust Android targets:
  ```bash
  rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
  ```

### iOS
- Xcode (macOS only)
- Rust iOS targets:
  ```bash
  rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
  ```

## Build Instructions

### Android

```bash
# Build Rust library for Android
./build_android.sh

# Run Flutter app
flutter run
```

### iOS

```bash
# Build Rust library for iOS
./build_ios.sh

# Run Flutter app
flutter run
```

### Release Build

```bash
# Android APK
./build_android.sh
flutter build apk --release

# iOS
./build_ios.sh
flutter build ios --release
```

## Project Structure

```
├── lib/                  # Flutter Dart code
│   └── main.dart         # UI with D-pad and mode controls
├── rust/                 # Rust game engine
│   ├── src/
│   │   ├── lib.rs        # Core game logic, rendering, FFI exports
│   │   └── jni.rs        # Android JNI bindings
│   ├── assets/
│   │   └── player.png    # Player sprite
│   └── Cargo.toml
├── android/              # Android native code
│   └── app/src/main/kotlin/
│       └── com/example/flutter_con/
│           ├── GameNative.kt        # JNI declarations
│           ├── GameGLRenderer.kt    # OpenGL renderer
│           └── GameGLSurfaceFactory.kt
├── ios/                  # iOS native code
│   └── Runner/
│       ├── GameGLView.swift         # GLKView wrapper
│       ├── GamePlatformViewFactory.swift
│       └── game_engine.h            # C FFI header
├── build_android.sh      # Android Rust build script
└── build_ios.sh          # iOS Rust build script
```
