package com.example.flutter_con

import android.annotation.SuppressLint
import android.content.Context
import android.opengl.GLSurfaceView
import android.view.MotionEvent
import android.view.View
import io.flutter.plugin.common.BinaryMessenger
import io.flutter.plugin.platform.PlatformView

class GameGLPlatformView(
    context: Context,
    private val viewId: Int,
    private val messenger: BinaryMessenger
) : PlatformView {

    private val glSurfaceView: GameGLSurfaceView
    private val renderer: GameGLRenderer

    init {
        renderer = GameGLRenderer()
        glSurfaceView = GameGLSurfaceView(context, renderer)
    }

    override fun getView(): View = glSurfaceView

    override fun dispose() {
        glSurfaceView.queueEvent {
            renderer.destroy()
        }
    }

    fun setDirection(direction: String) {
        renderer.setDirection(direction)
    }

    fun setMode(mode: Int) {
        glSurfaceView.queueEvent {
            renderer.setMode(mode)
        }
    }
}

@SuppressLint("ViewConstructor")
class GameGLSurfaceView(
    context: Context,
    private val renderer: GameGLRenderer
) : GLSurfaceView(context) {

    init {
        setEGLContextClientVersion(2)
        setRenderer(renderer)
        renderMode = RENDERMODE_CONTINUOUSLY
    }

    @SuppressLint("ClickableViewAccessibility")
    override fun onTouchEvent(event: MotionEvent): Boolean {
        val action = when (event.action) {
            MotionEvent.ACTION_DOWN -> GameNative.TOUCH_DOWN
            MotionEvent.ACTION_UP -> GameNative.TOUCH_UP
            MotionEvent.ACTION_MOVE -> GameNative.TOUCH_MOVE
            else -> return super.onTouchEvent(event)
        }

        // Queue touch event to be processed on GL thread
        queueEvent {
            renderer.onTouch(event.x, event.y, action)
        }

        return true
    }
}
