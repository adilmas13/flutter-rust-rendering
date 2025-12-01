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
