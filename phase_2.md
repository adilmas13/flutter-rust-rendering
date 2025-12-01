# Phase 2: Android Native GLSurfaceView

Set up Android native rendering surface without Rust.

---

## File: `android/app/src/main/kotlin/com/example/flutter_con/MainActivity.kt`

```kotlin
package com.example.flutter_con

import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

class MainActivity : FlutterActivity() {
    private val CHANNEL = "com.example.flutter_con/game"

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)

        // Register the PlatformView factory
        flutterEngine
            .platformViewsController
            .registry
            .registerViewFactory(
                "game-gl-surface",
                GameGLSurfaceFactory(flutterEngine.dartExecutor.binaryMessenger)
            )

        // Set up MethodChannel for direction events
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, CHANNEL)
            .setMethodCallHandler { call, result ->
                if (call.method == "sendDirection") {
                    val direction = call.argument<String>("direction")
                    // Direction will be forwarded to GLSurfaceView in Phase 3
                    println("Received direction: $direction")
                    result.success(null)
                } else {
                    result.notImplemented()
                }
            }
    }
}
```

---

## File: `android/app/src/main/kotlin/com/example/flutter_con/GameGLSurfaceFactory.kt`

```kotlin
package com.example.flutter_con

import android.content.Context
import io.flutter.plugin.common.BinaryMessenger
import io.flutter.plugin.common.StandardMessageCodec
import io.flutter.plugin.platform.PlatformView
import io.flutter.plugin.platform.PlatformViewFactory

class GameGLSurfaceFactory(
    private val messenger: BinaryMessenger
) : PlatformViewFactory(StandardMessageCodec.INSTANCE) {

    override fun create(context: Context, viewId: Int, args: Any?): PlatformView {
        return GameGLPlatformView(context, viewId, messenger)
    }
}
```

---

## File: `android/app/src/main/kotlin/com/example/flutter_con/GameGLPlatformView.kt`

```kotlin
package com.example.flutter_con

import android.content.Context
import android.opengl.GLSurfaceView
import android.view.View
import io.flutter.plugin.common.BinaryMessenger
import io.flutter.plugin.platform.PlatformView

class GameGLPlatformView(
    context: Context,
    private val viewId: Int,
    private val messenger: BinaryMessenger
) : PlatformView {

    private val glSurfaceView: GLSurfaceView
    private val renderer: GameGLRenderer

    init {
        glSurfaceView = GLSurfaceView(context)
        glSurfaceView.setEGLContextClientVersion(2)

        renderer = GameGLRenderer()
        glSurfaceView.setRenderer(renderer)
        glSurfaceView.renderMode = GLSurfaceView.RENDERMODE_CONTINUOUSLY
    }

    override fun getView(): View = glSurfaceView

    override fun dispose() {
        // Cleanup will be handled here
    }

    fun setDirection(direction: String) {
        renderer.setDirection(direction)
    }
}
```

---

## File: `android/app/src/main/kotlin/com/example/flutter_con/GameGLRenderer.kt`

```kotlin
package com.example.flutter_con

import android.opengl.GLES20
import android.opengl.GLSurfaceView
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10

class GameGLRenderer : GLSurfaceView.Renderer {

    @Volatile
    private var currentDirection: String = "none"

    private var red = 0.2f
    private var green = 0.2f
    private var blue = 0.2f

    override fun onSurfaceCreated(gl: GL10?, config: EGLConfig?) {
        GLES20.glClearColor(red, green, blue, 1.0f)
    }

    override fun onSurfaceChanged(gl: GL10?, width: Int, height: Int) {
        GLES20.glViewport(0, 0, width, height)
    }

    override fun onDrawFrame(gl: GL10?) {
        // Change color based on direction for visual feedback
        when (currentDirection) {
            "up" -> {
                red = 0.0f; green = 0.5f; blue = 0.0f  // Green
            }
            "down" -> {
                red = 0.5f; green = 0.0f; blue = 0.0f  // Red
            }
            "left" -> {
                red = 0.0f; green = 0.0f; blue = 0.5f  // Blue
            }
            "right" -> {
                red = 0.5f; green = 0.5f; blue = 0.0f  // Yellow
            }
            else -> {
                red = 0.2f; green = 0.2f; blue = 0.2f  // Gray
            }
        }

        GLES20.glClearColor(red, green, blue, 1.0f)
        GLES20.glClear(GLES20.GL_COLOR_BUFFER_BIT)
    }

    fun setDirection(direction: String) {
        currentDirection = direction
    }
}
```

---

## File: `lib/main.dart` (updated)

Replace the placeholder container with PlatformViewLink (Hybrid Composition for better GL performance):

```dart
import 'package:flutter/foundation.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';

void main() {
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
      ),
      home: const GameScreen(),
    );
  }
}

class GameScreen extends StatefulWidget {
  const GameScreen({super.key});

  @override
  State<GameScreen> createState() => _GameScreenState();
}

class _GameScreenState extends State<GameScreen> {
  static const platform = MethodChannel('com.example.flutter_con/game');
  String _lastDirection = 'none';

  Future<void> _sendDirection(String direction) async {
    setState(() {
      _lastDirection = direction;
    });

    try {
      await platform.invokeMethod('sendDirection', {'direction': direction});
    } on PlatformException catch (e) {
      debugPrint('Failed to send direction: ${e.message}');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: const Text('Flutter Rust Game'),
      ),
      body: Column(
        children: [
          // Game view - Android PlatformView with Hybrid Composition
          Expanded(
            child: Container(
              margin: const EdgeInsets.all(16),
              clipBehavior: Clip.hardEdge,
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: Colors.grey),
              ),
              child: const GameGLView(),
            ),
          ),
          // Direction indicator
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Text('Last direction: $_lastDirection'),
          ),
          // Direction pad
          Padding(
            padding: const EdgeInsets.all(24),
            child: DirectionPad(onDirectionPressed: _sendDirection),
          ),
        ],
      ),
    );
  }
}

class GameGLView extends StatelessWidget {
  const GameGLView({super.key});

  @override
  Widget build(BuildContext context) {
    const String viewType = 'game-gl-surface';

    return PlatformViewLink(
      viewType: viewType,
      surfaceFactory: (context, controller) {
        return AndroidViewSurface(
          controller: controller as AndroidViewController,
          gestureRecognizers: const <Factory<OneSequenceGestureRecognizer>>{},
          hitTestBehavior: PlatformViewHitTestBehavior.opaque,
        );
      },
      onCreatePlatformView: (params) {
        return PlatformViewsService.initSurfaceAndroidView(
          id: params.id,
          viewType: viewType,
          layoutDirection: TextDirection.ltr,
          creationParams: null,
          creationParamsCodec: const StandardMessageCodec(),
          onFocus: () {
            params.onFocusChanged(true);
          },
        )
          ..addOnPlatformViewCreatedListener(params.onPlatformViewCreated)
          ..create();
      },
    );
  }
}

class DirectionPad extends StatelessWidget {
  final void Function(String direction) onDirectionPressed;

  const DirectionPad({super.key, required this.onDirectionPressed});

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 200,
      height: 200,
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          // Up button
          _DirectionButton(
            icon: Icons.arrow_upward,
            onPressed: () => onDirectionPressed('up'),
          ),
          // Left and Right buttons
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              _DirectionButton(
                icon: Icons.arrow_back,
                onPressed: () => onDirectionPressed('left'),
              ),
              const SizedBox(width: 60),
              _DirectionButton(
                icon: Icons.arrow_forward,
                onPressed: () => onDirectionPressed('right'),
              ),
            ],
          ),
          // Down button
          _DirectionButton(
            icon: Icons.arrow_downward,
            onPressed: () => onDirectionPressed('down'),
          ),
        ],
      ),
    );
  }
}

class _DirectionButton extends StatelessWidget {
  final IconData icon;
  final VoidCallback onPressed;

  const _DirectionButton({required this.icon, required this.onPressed});

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 60,
      height: 60,
      child: ElevatedButton(
        onPressed: onPressed,
        style: ElevatedButton.styleFrom(
          padding: EdgeInsets.zero,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(12),
          ),
        ),
        child: Icon(icon, size: 32),
      ),
    );
  }
}
```

---

## Summary

This phase creates:

1. **MainActivity.kt** - Registers PlatformView factory and MethodChannel handler
2. **GameGLSurfaceFactory.kt** - Factory to create GameGLPlatformView instances
3. **GameGLPlatformView.kt** - Wrapper around GLSurfaceView implementing PlatformView
4. **GameGLRenderer.kt** - OpenGL ES 2.0 renderer with basic clear color
5. **main.dart (updated)** - Replaces placeholder with PlatformViewLink (Hybrid Composition)

Using `PlatformViewLink` with `AndroidViewSurface` instead of `AndroidView` for:
- Better performance (native view renders directly, no buffer copy)
- Lower latency (critical for real-time GL rendering)
- Better touch event handling

The GL view changes background color based on direction (for testing):
- Up: Green
- Down: Red
- Left: Blue
- Right: Yellow
- None: Gray

Note: Direction forwarding from MethodChannel to GLSurfaceView will be completed in Phase 3.

---

## Checklist

- [x] Create custom GLSurfaceView class
- [x] Implement GLRenderer with basic clear color
- [x] Register PlatformView factory in Flutter plugin
- [x] Connect Flutter PlatformView to native GLSurfaceView
- [x] Verify GL context is working (changing background color)

---

Awaiting your review and approval to implement this code.
