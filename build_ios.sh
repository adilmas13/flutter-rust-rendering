#!/bin/bash
# Build Rust library for iOS (device and simulator)

set -e

cd "$(dirname "$0")/rust"

echo "Building Rust for iOS..."

# Device (arm64)
echo "  -> Building for arm64 device..."
cargo build --release --target aarch64-apple-ios

# Simulator (arm64 for Apple Silicon)
echo "  -> Building for arm64 simulator..."
cargo build --release --target aarch64-apple-ios-sim

# Simulator (x86_64 for Intel Macs)
echo "  -> Building for x86_64 simulator..."
cargo build --release --target x86_64-apple-ios

# Create universal simulator library
echo "  -> Creating universal simulator library..."
lipo -create \
  target/aarch64-apple-ios-sim/release/libgame_engine.a \
  target/x86_64-apple-ios/release/libgame_engine.a \
  -output target/libgame_engine_sim.a

# Copy to iOS project
echo "  -> Copying libraries to iOS project..."
mkdir -p ../ios/Runner/RustLib
cp target/aarch64-apple-ios/release/libgame_engine.a ../ios/Runner/RustLib/libgame_engine_device.a
cp target/libgame_engine_sim.a ../ios/Runner/RustLib/libgame_engine_sim.a

echo "Done! Libraries at:"
echo "  - ios/Runner/RustLib/libgame_engine_device.a (for device)"
echo "  - ios/Runner/RustLib/libgame_engine_sim.a (for simulator)"
