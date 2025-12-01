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
