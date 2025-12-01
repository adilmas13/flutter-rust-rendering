package com.example.flutter_con

import android.opengl.GLSurfaceView
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10

class GameGLRenderer : GLSurfaceView.Renderer {

    private var gameHandle: Long = 0
    private var width: Int = 0
    private var height: Int = 0

    @Volatile
    private var pendingDirection: Int = GameNative.DIRECTION_NONE

    @Volatile
    private var pendingTouch: TouchEvent? = null

    data class TouchEvent(val x: Float, val y: Float, val action: Int)

    override fun onSurfaceCreated(gl: GL10?, config: EGLConfig?) {
        // Initialize Rust game engine
        gameHandle = GameNative.gameInit(width, height)
    }

    override fun onSurfaceChanged(gl: GL10?, width: Int, height: Int) {
        this.width = width
        this.height = height
        if (gameHandle != 0L) {
            GameNative.gameResize(gameHandle, width, height)
        }
    }

    override fun onDrawFrame(gl: GL10?) {
        if (gameHandle == 0L) return

        // Process pending direction
        GameNative.gameSetDirection(gameHandle, pendingDirection)

        // Process pending touch
        pendingTouch?.let { touch ->
            GameNative.gameTouch(gameHandle, touch.x, touch.y, touch.action)
            pendingTouch = null
        }

        // Update and render
        GameNative.gameUpdate(gameHandle)
        GameNative.gameRender(gameHandle)
    }

    fun setDirection(direction: String) {
        pendingDirection = when (direction) {
            "up" -> GameNative.DIRECTION_UP
            "down" -> GameNative.DIRECTION_DOWN
            "left" -> GameNative.DIRECTION_LEFT
            "right" -> GameNative.DIRECTION_RIGHT
            else -> GameNative.DIRECTION_NONE
        }
    }

    fun onTouch(x: Float, y: Float, action: Int) {
        pendingTouch = TouchEvent(x, y, action)
    }

    fun destroy() {
        if (gameHandle != 0L) {
            GameNative.gameDestroy(gameHandle)
            gameHandle = 0
        }
    }
}
