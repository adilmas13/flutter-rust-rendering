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
