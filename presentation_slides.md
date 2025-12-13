# Bridging Flutter and Rust
## Cross-Platform Native Performance for Mobile and Web

---

# SECTION 1: Introduction

---

## Slide 1: Title

**Bridging Flutter and Rust**
*Cross-Platform Native Performance for Mobile and Web*

- Speaker Name
- FlutterCon 2024

---

## Slide 2: Problem We're Trying to Solve

**When Dart Isn't Enough**

- **Performance-critical workloads**
  - Real-time graphics and game engines
  - Image/video processing
  - ML inference, cryptography

- **Cross-platform native code**
  - Want single codebase for Android, iOS, AND Web
  - Existing native libraries to integrate

- **Memory constraints**
  - Dart's GC can cause frame drops
  - Need predictable memory management

- **The Challenge**
  - Flutter is great for UI
  - But how do we integrate high-performance native code?

---

## Slide 3: Solution We Came Up With

**Rust as the Shared Core**

```
                    ┌─────────────┐
                    │  Rust Core  │
                    │  (Shared)   │
                    └──────┬──────┘
           ┌───────────────┼───────────────┐
           ▼               ▼               ▼
    ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
    │   Android   │ │     iOS     │ │     Web     │
    │    (.so)    │ │    (.a)     │ │   (.wasm)   │
    └─────────────┘ └─────────────┘ └─────────────┘
```

**Key Benefits:**
- Write Rust once, compile to all platforms
- Platform-specific bridges handle the FFI
- Flutter handles UI, Rust handles heavy lifting
- Same binary works on mobile AND web

---

# SECTION 2: Architecture Overview

---

## Slide 4: Layers in the Implementation

**The Full Stack**

```
┌─────────────────────────────────────────────────────────┐
│                   Flutter/Dart UI                       │
│              (Widgets, State Management)                │
├─────────────────────────────────────────────────────────┤
│                   Platform Views                        │
│         (Native rendering surface in Flutter)          │
├─────────────────────────────────────────────────────────┤
│                   Method Channel                        │
│            (Dart ↔ Native communication)               │
├─────────────────────────────────────────────────────────┤
│              Native Layer (per platform)               │
│         Kotlin    │    Swift    │    JavaScript        │
├─────────────────────────────────────────────────────────┤
│                   FFI Bridge                            │
│          JNI      │   C Interop │    WASM Bindgen      │
├─────────────────────────────────────────────────────────┤
│                     Rust Core                           │
│            (Game Logic, Rendering, Compute)            │
└─────────────────────────────────────────────────────────┘
```

---

## Slide 5: Data Flow - A Button Press

**Tracing Through the Layers**

- A single user interaction travels through **5 layers** to reach Rust
- Each layer transforms the data format (Dart → Native → FFI types)
- Response follows the same path in reverse for state updates
- Understanding this flow is key to debugging issues

```
User taps "UP" button in Flutter
           │
           ▼
┌──────────────────────────────────────────────────────┐
│ Dart: platform.invokeMethod('setDirection', 'up')    │
└──────────────────────────────────────────────────────┘
           │ Method Channel
           ▼
┌──────────────────────────────────────────────────────┐
│ Android: MethodCallHandler receives call             │
│ iOS: FlutterMethodChannel handler                    │
│ Web: js.context.callMethod()                         │
└──────────────────────────────────────────────────────┘
           │ Native Bridge
           ▼
┌──────────────────────────────────────────────────────┐
│ Android: GameNative.gameSetDirection(handle, 1)      │
│ iOS: game_set_direction(handle, 1)                   │
│ Web: wasm.game_set_direction(handle, 1)              │
└──────────────────────────────────────────────────────┘
           │ FFI / JNI / WASM
           ▼
┌──────────────────────────────────────────────────────┐
│ Rust: state.current_direction = Direction::Up        │
└──────────────────────────────────────────────────────┘
           │
           ▼
     Next frame: Player moves up
```

---

# SECTION 3: Flutter Integration Layer

---

## Slide 6: Platform Views

**Embedding Native Rendering Surfaces**

| Platform | Flutter Widget | Native View | Use Case |
|----------|---------------|-------------|----------|
| Android | `AndroidView` | `GLSurfaceView` | OpenGL rendering |
| iOS | `UiKitView` | `GLKView` / `MTKView` | OpenGL/Metal |
| Web | `HtmlElementView` | `<canvas>` | WebGL |

**Dart Code:**
```dart
Widget build(BuildContext context) {
  if (Platform.isAndroid) {
    return AndroidView(
      viewType: 'game-gl-surface',
      onPlatformViewCreated: _onViewCreated,
    );
  } else if (Platform.isIOS) {
    return UiKitView(
      viewType: 'game-gl-surface',
      onPlatformViewCreated: _onViewCreated,
    );
  }
  // Web uses HtmlElementView
}
```

**Key Points:**
- Platform View embeds native view in Flutter widget tree
- Native view owns the GL context
- Rust renders to this context

---

## Slide 7: Method Channel

**Dart ↔ Native Communication**

- Method Channels are Flutter's built-in way to communicate with native code
- Works like a **named pipe** - both sides must agree on the channel name
- Messages are serialized using **Standard Message Codec** (primitives, maps, lists)
- Calls are **asynchronous** - returns a Future in Dart
- Each platform implements its own handler for the same channel name

**Dart Side:**
```dart
class GameController {
  static const platform = MethodChannel('com.example.app/game');

  Future<void> setDirection(String direction) async {
    await platform.invokeMethod('setDirection', {
      'direction': direction,
    });
  }

  Future<void> setMode(int mode) async {
    await platform.invokeMethod('setMode', {'mode': mode});
  }
}
```

**Android Side (Kotlin):**
```kotlin
MethodChannel(flutterEngine.dartExecutor, "com.example.app/game")
  .setMethodCallHandler { call, result ->
    when (call.method) {
      "setDirection" -> {
        val dir = call.argument<String>("direction")
        gameView.setDirection(dir)
        result.success(null)
      }
      "setMode" -> {
        val mode = call.argument<Int>("mode")
        gameView.setMode(mode)
        result.success(null)
      }
    }
  }
```

**iOS Side (Swift):**
```swift
let channel = FlutterMethodChannel(name: "com.example.app/game",
                                   binaryMessenger: controller.binaryMessenger)
channel.setMethodCallHandler { (call, result) in
  switch call.method {
  case "setDirection":
    let args = call.arguments as! [String: Any]
    self.gameView.setDirection(args["direction"] as! String)
    result(nil)
  default:
    result(FlutterMethodNotImplemented)
  }
}
```

---

## Slide 8: Renderer

**Native Renderer Lifecycle**

- The renderer lifecycle is managed by the **platform's graphics system**, not Flutter
- Rust code responds to lifecycle events but doesn't drive them
- Critical: **GL context is only valid between create and destroy**
- Each platform has different API names for the same concepts

| Event | Android | iOS | Web |
|-------|---------|-----|-----|
| **Create** | `onSurfaceCreated()` | `glkView(_:drawIn:)` first call | `canvas.getContext()` |
| **Resize** | `onSurfaceChanged()` | `layoutSubviews()` | `resize` event |
| **Draw** | `onDrawFrame()` | `glkView(_:drawIn:)` | `requestAnimationFrame` |
| **Destroy** | `onSurfaceDestroyed()` | `deinit` | Page unload |

**Android GLSurfaceView.Renderer:**
```kotlin
class GameRenderer : GLSurfaceView.Renderer {
    private var handle: Long = 0

    override fun onSurfaceCreated(gl: GL10?, config: EGLConfig?) {
        handle = GameNative.gameInit(width, height)
    }

    override fun onSurfaceChanged(gl: GL10?, width: Int, height: Int) {
        GameNative.gameResize(handle, width, height)
    }

    override fun onDrawFrame(gl: GL10?) {
        GameNative.gameUpdate(handle)
        GameNative.gameRender(handle)
    }
}
```

**Key Point:** Rust doesn't create the GL context - it receives it from the platform and renders to it.

---

# SECTION 4: Why Rust?

---

## Slide 9: Why Rust?

**The Perfect Fit for Cross-Platform Native Code**

| Feature | Benefit |
|---------|---------|
| **Memory Safety** | No segfaults, no GC pauses |
| **Zero-Cost Abstractions** | High-level code, C-level performance |
| **Cross-Compilation** | Single codebase → all platforms |
| **C ABI Compatibility** | Works with any language via FFI |
| **WASM Support** | Same code runs on Web |
| **Cargo Ecosystem** | Rich library ecosystem |

**Comparison:**
```
┌─────────────────────────────────────────────────────┐
│                     Rust                            │
│  ✓ Memory safe    ✓ Fast    ✓ Cross-platform       │
├─────────────────────────────────────────────────────┤
│                     C/C++                           │
│  ✗ Memory safe    ✓ Fast    ~ Cross-platform       │
├─────────────────────────────────────────────────────┤
│                     Go                              │
│  ✓ Memory safe    ~ Fast    ✗ Mobile/WASM          │
└─────────────────────────────────────────────────────┘
```

---

## Slide 10: Rust's Cross-Platform Story

**One Codebase, Three Targets**

- Rust compiles to native code for each platform - not interpreted
- Each platform gets an optimized binary for its architecture
- Conditional compilation (`#[cfg(...)]`) handles platform differences
- Binary sizes are small - LTO and stripping help further

| Target | Output | Tool | Size |
|--------|--------|------|------|
| Android | `libgame.so` | cargo-ndk | ~1-2 MB |
| iOS | `libgame.a` | cargo + lipo | ~2-3 MB |
| Web | `game.wasm` | wasm-pack | ~500 KB |

**The Shared Core:**
```rust
// src/lib.rs - Runs on ALL platforms

pub fn game_update(state: &mut GameState, delta: f32) {
    // Same physics code everywhere
    state.player.x += state.velocity.x * delta;
    state.player.y += state.velocity.y * delta;

    // Bounce off walls
    if state.player.x < 0.0 || state.player.x > state.width {
        state.velocity.x = -state.velocity.x;
    }
}
```

**Platform-Specific Only Where Needed:**
```rust
#[cfg(target_os = "android")]
mod jni;  // JNI bindings

#[cfg(target_os = "ios")]
mod ios;  // iOS-specific code

#[cfg(target_arch = "wasm32")]
mod wasm; // WASM bindings
```

---

# SECTION 5: Native Bridges - JNI (Android) & iOS

---

## Slide 11: JNI - Android Bridge

**Java Native Interface**

- JNI is Android/Java's standard way to call native code
- Kotlin/Java declares `external` functions, Rust implements them
- Function names must follow **strict naming convention** or linking fails
- The `handle` pattern: Rust returns opaque pointer, Kotlin passes it back
- All JNI calls go through the Android runtime - slight overhead

**1. Load the Library (Kotlin):**
```kotlin
object GameNative {
    init {
        System.loadLibrary("game_engine")  // Loads libgame_engine.so
    }

    external fun gameInit(width: Int, height: Int): Long
    external fun gameUpdate(handle: Long)
    external fun gameRender(handle: Long)
    external fun gameSetDirection(handle: Long, direction: Int)
    external fun gameDestroy(handle: Long)
}
```

**2. JNI Naming Convention:**
```
Java_<package>_<class>_<method>

Java_com_example_flutter_1con_GameNative_gameInit
     └─────────────────────┘ └────────┘ └──────┘
           package            class      method
```

**3. Rust Implementation:**
```rust
#[no_mangle]
pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameInit(
    _env: JNIEnv,
    _class: JClass,
    width: jint,
    height: jint,
) -> jlong {
    game_init(width as u32, height as u32) as jlong
}
```

---

## Slide 12: C Interop - iOS Bridge

**Swift ↔ Rust via C**

- Swift can call C functions directly through a **bridging header**
- No runtime overhead like JNI - direct function calls
- Rust exports `extern "C"` functions with stable C ABI
- Use `UnsafeMutableRawPointer` for opaque handles in Swift
- Memory management is manual - must call `destroy()` to free Rust memory

**1. Bridging Header (game_engine.h):**
```c
#ifndef game_engine_h
#define game_engine_h

#include <stdint.h>

typedef void* GameHandle;

GameHandle game_init(uint32_t width, uint32_t height);
void game_update(GameHandle handle);
void game_render(GameHandle handle);
void game_set_direction(GameHandle handle, int32_t direction);
void game_destroy(GameHandle handle);

#endif
```

**2. Swift Usage:**
```swift
class GameBridge {
    private var handle: UnsafeMutableRawPointer?

    func initialize(width: UInt32, height: UInt32) {
        handle = game_init(width, height)
    }

    func setDirection(_ direction: Int32) {
        guard let h = handle else { return }
        game_set_direction(h, direction)
    }

    deinit {
        if let h = handle { game_destroy(h) }
    }
}
```

**3. Rust (same as Android core):**
```rust
#[no_mangle]
pub extern "C" fn game_init(width: u32, height: u32) -> *mut GameState {
    Box::into_raw(Box::new(GameState::new(width, height)))
}
```

---

## Slide 13: WASM - Web Bridge

**WebAssembly for the Browser**

- WebAssembly lets the **same Rust code** run in browsers at near-native speed
- `wasm-bindgen` generates JS glue code automatically
- Structs with `#[wasm_bindgen]` become JS classes
- Methods become callable from JavaScript
- WASM module is loaded asynchronously via `init()`

**1. Rust with wasm-bindgen:**
```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct GameEngine {
    state: GameState,
}

#[wasm_bindgen]
impl GameEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> GameEngine {
        GameEngine { state: GameState::new(width, height) }
    }

    pub fn update(&mut self) {
        self.state.update();
    }

    pub fn render(&mut self) {
        self.state.render();
    }

    pub fn set_direction(&mut self, direction: i32) {
        self.state.set_direction(direction);
    }
}
```

**2. JavaScript Usage:**
```javascript
import init, { GameEngine } from './pkg/game_engine.js';

async function start() {
    await init();
    const engine = new GameEngine(canvas.width, canvas.height);

    function gameLoop() {
        engine.update();
        engine.render();
        requestAnimationFrame(gameLoop);
    }
    gameLoop();
}
```

---

## Slide 14: The Shared Rust Core

**Platform-Agnostic Code**

- The **game logic** is 100% shared - same behavior on all platforms
- Platform-specific code is isolated in separate modules (`jni.rs`, `wasm.rs`)
- Use `#[cfg(target_os = ...)]` for conditional compilation
- Core code has no platform dependencies - pure Rust

```rust
// src/game.rs - Shared across ALL platforms

pub struct GameState {
    pub player_x: f32,
    pub player_y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub direction: Direction,
    pub mode: GameMode,
    width: f32,
    height: f32,
}

impl GameState {
    pub fn update(&mut self, delta: f32) {
        // This EXACT code runs on Android, iOS, AND Web
        match self.mode {
            GameMode::Manual => self.update_manual(delta),
            GameMode::Auto => self.update_auto(delta),
        }
    }
}
```

**Platform-specific is minimal:**
- Android: JNI wrapper (~80 lines)
- iOS: C exports (~50 lines)
- Web: wasm-bindgen (~60 lines)
- **Core game logic: 500+ lines shared**

---

# SECTION 6: flutter_rust_bridge

---

## Slide 15: What is flutter_rust_bridge?

**Auto-Generated FFI Bindings**

- A code generator that creates Dart bindings from Rust functions
- No manual JNI or C header writing required
- Automatically handles complex types (structs, enums, Vec, Option)
- Supports async/await out of the box
- Great for **business logic**, but OpenGL still needs native layer

```
┌────────────────────────────────────────────────────┐
│              Write Normal Rust Code                │
└────────────────────────────────────────────────────┘
                         │
                         ▼
┌────────────────────────────────────────────────────┐
│       flutter_rust_bridge_codegen generate         │
└────────────────────────────────────────────────────┘
                         │
                         ▼
┌────────────────────────────────────────────────────┐
│            Auto-Generated Dart Bindings            │
└────────────────────────────────────────────────────┘
                         │
                         ▼
┌────────────────────────────────────────────────────┐
│      Call Rust from Dart Like Normal Dart Code     │
└────────────────────────────────────────────────────┘
```

**Rust:**
```rust
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

pub struct Point { pub x: f64, pub y: f64 }

pub fn distance(a: Point, b: Point) -> f64 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}
```

**Dart (auto-generated):**
```dart
final greeting = await greet(name: "Flutter");

final d = await distance(
    a: Point(x: 0, y: 0),
    b: Point(x: 3, y: 4),
);
```

---

## Slide 16: When to Use Which?

**Manual FFI vs flutter_rust_bridge**

| Aspect | Manual FFI | flutter_rust_bridge |
|--------|-----------|---------------------|
| **Setup** | More work | One command |
| **Type Safety** | Manual | Automatic |
| **Complex Types** | Serialize yourself | Handled |
| **Async** | Manual threading | Built-in |
| **OpenGL/Rendering** | Full control | Still needs native |
| **Learning Curve** | Steeper | Gentler |
| **Dependencies** | Minimal | Codegen tool |

**Use Manual FFI when:**
- OpenGL/graphics rendering
- Existing C libraries
- Maximum control needed

**Use flutter_rust_bridge when:**
- Business logic in Rust
- Complex data structures
- Rapid development

---

# SECTION 7: Message Passing

---

## Slide 17: Message Passing Architecture

**Why Message Passing?**

- **Thread Safety:** GL calls must happen on GL thread - can't call directly from FFI
- **Decoupling:** Flutter UI thread is separate from Rust render thread
- **Predictability:** State updates happen at controlled points (once per frame)
- **No race conditions:** Commands queued, not executed immediately
- **Debugging:** Can log all commands passing through the queue

**Command Queue Pattern:**

```
┌─────────────┐     ┌──────────────────┐     ┌─────────────┐
│   Flutter   │────▶│  Command Queue   │────▶│    Rust     │
│  (UI Thread)│     │  (Thread-safe)   │     │ (GL Thread) │
└─────────────┘     └──────────────────┘     └─────────────┘
      │                                             │
      │              ┌──────────────────┐           │
      └─────────────◀│   Event Stream   │◀──────────┘
                     │  (Rust → Dart)   │
                     └──────────────────┘
```

**Rust Side:**
```rust
static COMMANDS: Lazy<Mutex<Vec<GameCommand>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

pub fn push_command(cmd: GameCommand) {
    COMMANDS.lock().unwrap().push(cmd);
}

fn process_commands(state: &mut GameState) {
    let commands: Vec<_> = COMMANDS.lock().unwrap().drain(..).collect();
    for cmd in commands {
        match cmd {
            GameCommand::SetDirection(d) => state.direction = d,
            GameCommand::SetMode(m) => state.mode = m,
        }
    }
}
```

- `Lazy<Mutex<Vec>>` creates a thread-safe global command queue
- `push_command()` can be called from any thread (FFI entry point)
- `process_commands()` runs on GL thread at start of each frame
- `drain(..)` clears the queue while processing - prevents re-processing

---

## Slide 18: Thread Safety Considerations

**The Golden Rules**

```
┌────────────────────────────────────────────────────────┐
│  RULE 1: GL calls MUST happen on the GL thread         │
│  RULE 2: FFI calls can come from ANY thread            │
│  RULE 3: Shared state needs synchronization            │
└────────────────────────────────────────────────────────┘
```

**Safe Pattern:**
```rust
use std::sync::atomic::{AtomicI32, Ordering};

// Atomic for simple values (lock-free)
static PENDING_DIRECTION: AtomicI32 = AtomicI32::new(0);

#[no_mangle]
pub extern "C" fn game_set_direction(direction: i32) {
    // Called from any thread - safe
    PENDING_DIRECTION.store(direction, Ordering::Relaxed);
}

fn game_update(state: &mut GameState) {
    // Called from GL thread - reads atomic
    let dir = PENDING_DIRECTION.load(Ordering::Relaxed);
    state.direction = Direction::from(dir);
}
```

- Use **atomics** for simple values (integers, bools) - no locking overhead
- `Ordering::Relaxed` is fine for game commands - no strict ordering needed
- Pattern: FFI writes to atomic, render loop reads from atomic

**For Complex Data:**
```rust
use std::sync::Mutex;

static COMMAND_QUEUE: Lazy<Mutex<Vec<Command>>> =
    Lazy::new(|| Mutex::new(Vec::new()));
```

- Use **Mutex** when you need to queue multiple commands or complex data
- Keep critical section short - lock, push, unlock immediately

---

# SECTION 8: Tools & Tips

---

## Slide 19: Build Tools - Android (cargo-ndk)

**Cross-Compiling for Android**

- Rust can cross-compile, but Android needs the **NDK toolchain**
- `cargo-ndk` wraps cargo with correct NDK compiler flags
- Build once, output to `jniLibs` folder - Android picks up automatically
- Support multiple ABIs for device coverage (arm64 is most common)

**1. Install Targets:**
```bash
rustup target add aarch64-linux-android    # ARM64 devices
rustup target add armv7-linux-androideabi  # ARM32 devices
rustup target add x86_64-linux-android     # x86_64 emulator
```

**2. Install cargo-ndk:**
```bash
cargo install cargo-ndk
```

**3. Build:**
```bash
cargo ndk -t arm64-v8a -t armeabi-v7a \
    -o ./android/app/src/main/jniLibs \
    build --release
```

**Output Structure:**
```
android/app/src/main/jniLibs/
├── arm64-v8a/
│   └── libgame_engine.so      # ARM64
├── armeabi-v7a/
│   └── libgame_engine.so      # ARM32
└── x86_64/
    └── libgame_engine.so      # Emulator
```

---

## Slide 20: Build Tools - iOS (cargo + lipo)

**Building Universal Libraries**

- iOS needs separate builds for device and simulator (different architectures)
- `lipo` combines multiple architectures into one "fat" binary
- Apple Silicon Macs use `aarch64-apple-ios-sim`, Intel uses `x86_64-apple-ios`
- Static library (`.a`) gets linked into your app at build time

**1. Install Targets:**
```bash
rustup target add aarch64-apple-ios         # Device
rustup target add aarch64-apple-ios-sim     # Simulator (Apple Silicon)
rustup target add x86_64-apple-ios          # Simulator (Intel)
```

**2. Build Script (build_ios.sh):**
```bash
#!/bin/bash
set -e

# Build for all targets
cargo build --release --target aarch64-apple-ios
cargo build --release --target aarch64-apple-ios-sim
cargo build --release --target x86_64-apple-ios

# Create universal simulator library
lipo -create \
    target/aarch64-apple-ios-sim/release/libgame_engine.a \
    target/x86_64-apple-ios/release/libgame_engine.a \
    -output target/universal-sim/libgame_engine.a

# Copy to iOS project
cp target/aarch64-apple-ios/release/libgame_engine.a \
   ios/Runner/libgame_engine_device.a
```

**3. Generate Header (optional - cbindgen):**
```bash
cargo install cbindgen
cbindgen --output ios/Runner/game_engine.h
```

---

## Slide 21: Build Tools - Web (wasm-pack)

**Compiling to WebAssembly**

- `wasm-pack` compiles Rust to WASM and generates JavaScript bindings
- Output includes JS, WASM binary, and TypeScript definitions
- Use `--target web` for browser usage (vs `--target bundler` for webpack)
- Final WASM is often smaller than native binaries after optimization

**1. Install Target:**
```bash
rustup target add wasm32-unknown-unknown
```

**2. Install wasm-pack:**
```bash
cargo install wasm-pack
```

**3. Build:**
```bash
wasm-pack build --target web --out-dir ../web/pkg
```

**Output:**
```
web/pkg/
├── game_engine.js       # JS bindings
├── game_engine_bg.wasm  # WebAssembly binary
├── game_engine.d.ts     # TypeScript definitions
└── package.json
```

**Usage in HTML:**
```html
<script type="module">
  import init, { GameEngine } from './pkg/game_engine.js';

  async function run() {
    await init();
    const engine = new GameEngine(800, 600);
    // ...
  }
  run();
</script>
```

---

## Slide 22: Precompile Hooks

**Automate Rust Builds with Flutter**

- Integrate Rust builds into platform build systems - no manual steps
- Debug builds use debug Rust, release builds use release Rust
- Developers just run `flutter run` - everything builds automatically
- CI/CD works out of the box with the same commands

**Android - Gradle (build.gradle.kts):**
```kotlin
tasks.register<Exec>("buildRustDebug") {
    workingDir = file("../../rust")
    commandLine("cargo", "ndk", "-t", "arm64-v8a",
                "-o", "../android/app/src/main/jniLibs", "build")
}

tasks.register<Exec>("buildRustRelease") {
    workingDir = file("../../rust")
    commandLine("cargo", "ndk", "-t", "arm64-v8a",
                "-o", "../android/app/src/main/jniLibs",
                "build", "--release")
}

// Hook into Android build
tasks.named("preBuild") {
    dependsOn(if (isRelease) "buildRustRelease" else "buildRustDebug")
}
```

**iOS - Xcode Build Phase:**
```bash
# Add "Run Script" build phase in Xcode
cd "$SRCROOT/../rust"

if [ "$CONFIGURATION" = "Release" ]; then
    ./build_ios.sh release
else
    ./build_ios.sh debug
fi
```

**Result:**
```bash
flutter run           # Auto-builds Rust debug
flutter build apk     # Auto-builds Rust release
flutter build ios     # Auto-builds Rust release
```

---

# SECTION 9: Wrap Up

---

## Slide 23: Key Takeaways

**What We Learned**

1. **Layered Architecture**
   - Flutter → Platform View → Method Channel → Native → FFI → Rust
   - Each layer has a purpose

2. **Rust is Ideal for Cross-Platform**
   - Single codebase: Android + iOS + Web
   - Memory safe, fast, modern tooling

3. **Platform Bridges are Thin**
   - Android: JNI (~80 lines)
   - iOS: C header + Swift (~50 lines)
   - Web: wasm-bindgen (~60 lines)
   - 90% of code is shared

4. **Tools Make It Easy**
   - `cargo-ndk` for Android
   - `lipo` for iOS universal binaries
   - `wasm-pack` for Web
   - Precompile hooks automate everything

5. **Message Passing for Thread Safety**
   - GL thread requirements
   - Command queue pattern

---

## Slide 24: Q&A / Resources

**Resources**

- **Rust FFI Guide:** https://doc.rust-lang.org/nomicon/ffi.html
- **cargo-ndk:** https://github.com/nickelc/cargo-ndk
- **wasm-pack:** https://rustwasm.github.io/wasm-pack/
- **flutter_rust_bridge:** https://cjycode.com/flutter_rust_bridge/
- **Android NDK:** https://developer.android.com/ndk

**This Project:**
- GitHub: [your-repo-link]
- Slides: [slides-link]

**Contact:**
- Twitter: @yourhandle
- Email: you@example.com

---

**Questions?**

---

# BONUS SLIDES

---

## Bonus Slide 1: Type Mapping Reference

**Cross-Platform Type Mapping**

- Primitive types map directly but have different names per platform
- Pointers become `jlong` (Android) or `UnsafeMutableRawPointer` (iOS)
- Strings require special handling - memory ownership matters
- Use `#[repr(i32)]` on enums to ensure consistent size across FFI

| Rust | Android (JNI) | iOS (Swift) | Web (JS) |
|------|---------------|-------------|----------|
| `i32` | `jint` / `Int` | `Int32` | `number` |
| `i64` | `jlong` / `Long` | `Int64` | `BigInt` |
| `f32` | `jfloat` / `Float` | `Float` | `number` |
| `f64` | `jdouble` / `Double` | `Double` | `number` |
| `bool` | `jboolean` / `Boolean` | `Bool` | `boolean` |
| `*mut T` | `jlong` | `UnsafeMutableRawPointer` | N/A |
| `String` | `JString` | `String` | `string` |

**Enums (all platforms):**
```rust
#[repr(i32)]  // Fixed size for FFI
pub enum Direction {
    None = 0,
    Up = 1,
    Down = 2,
    Left = 3,
    Right = 4,
}
```

---

## Bonus Slide 2: Debugging Tips

**Tools for Each Platform**

- Logging is essential - use platform-specific loggers for visibility
- Each platform has its own debugging ecosystem - learn the basics
- Rust panics will crash the app - use `catch_unwind` at FFI boundaries
- Profile regularly - FFI calls and GL rendering are common bottlenecks

| Platform | Logging | Debugger | Profiler |
|----------|---------|----------|----------|
| Android | `adb logcat \| grep RustGame` | Android Studio | Perfetto |
| iOS | Console.app | Xcode LLDB | Instruments |
| Web | `console.log` | Browser DevTools | Performance tab |

**Rust Logging:**
```rust
// Cargo.toml
[target.'cfg(target_os = "android")'.dependencies]
android_logger = "0.14"

[target.'cfg(target_os = "ios")'.dependencies]
oslog = "0.2"

// Code
fn init_logging() {
    #[cfg(target_os = "android")]
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("RustGame")
            .with_max_level(log::LevelFilter::Info)
    );

    #[cfg(target_os = "ios")]
    oslog::OsLogger::new("com.example.app")
        .level_filter(log::LevelFilter::Info)
        .init()
        .ok();
}

log::info!("Player position: ({}, {})", x, y);
```

---

## Bonus Slide 3: Common Pitfalls

**Things That Will Bite You**

- Most errors happen at the FFI boundary - type mismatches, wrong names
- Memory leaks are common - Rust allocates, but who deallocates?
- Threading issues cause random crashes - always use command queue pattern
- Missing Rust targets cause confusing build errors - install them first!

| Pitfall | Symptom | Solution |
|---------|---------|----------|
| Wrong JNI name | `UnsatisfiedLinkError` | Check package/class/method exactly |
| Panic across FFI | App crash (SIGABRT) | Wrap with `catch_unwind` |
| Memory leak | Growing memory | Call `destroy()` function |
| Wrong thread | GL errors, crash | Use command queue pattern |
| Missing target | Build fails | `rustup target add ...` |
| Mismatched types | Garbage values | Check type sizes match |

**Panic Safety Macro:**
```rust
macro_rules! ffi_safe {
    ($default:expr, $body:expr) => {
        match std::panic::catch_unwind(
            std::panic::AssertUnwindSafe(|| $body)
        ) {
            Ok(v) => v,
            Err(e) => {
                log::error!("Panic in FFI: {:?}", e);
                $default
            }
        }
    };
}

#[no_mangle]
pub extern "C" fn game_init(w: u32, h: u32) -> *mut GameState {
    ffi_safe!(std::ptr::null_mut(), {
        Box::into_raw(Box::new(GameState::new(w, h)))
    })
}
```

---

*End of Presentation*
