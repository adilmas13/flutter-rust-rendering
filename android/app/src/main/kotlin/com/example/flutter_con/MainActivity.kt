package com.example.flutter_con

import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

class MainActivity : FlutterActivity() {
    private val CHANNEL = "com.example.flutter_con/game"
    private lateinit var gameFactory: GameGLSurfaceFactory

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)

        // Create and register the PlatformView factory
        gameFactory = GameGLSurfaceFactory(flutterEngine.dartExecutor.binaryMessenger)
        flutterEngine
            .platformViewsController
            .registry
            .registerViewFactory("game-gl-surface", gameFactory)

        // Set up MethodChannel for game events
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, CHANNEL)
            .setMethodCallHandler { call, result ->
                when (call.method) {
                    "sendDirection" -> {
                        val direction = call.argument<String>("direction")
                        if (direction != null) {
                            gameFactory.setDirection(direction)
                        }
                        result.success(null)
                    }
                    "setMode" -> {
                        val mode = call.argument<Int>("mode") ?: 0
                        gameFactory.setMode(mode)
                        result.success(null)
                    }
                    else -> result.notImplemented()
                }
            }
    }
}
