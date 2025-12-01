#!/bin/bash
set -e

echo "Building Rust library for Android ARM64..."

# Build for ARM64
cargo ndk -t arm64-v8a -o ../android/app/src/main/jniLibs build --release

echo "Build complete!"
echo "Output: ../android/app/src/main/jniLibs/arm64-v8a/libgame_engine.so"
ls -la ../android/app/src/main/jniLibs/arm64-v8a/
