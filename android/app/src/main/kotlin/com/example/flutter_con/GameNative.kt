package com.example.flutter_con

object GameNative {
    init {
        System.loadLibrary("game_engine")
    }

    // Native methods - these call into Rust FFI
    external fun gameInit(width: Int, height: Int): Long
    external fun gameResize(handle: Long, width: Int, height: Int)
    external fun gameUpdate(handle: Long)
    external fun gameRender(handle: Long)
    external fun gameSetDirection(handle: Long, direction: Int)
    external fun gameTouch(handle: Long, x: Float, y: Float, action: Int)
    external fun gameDestroy(handle: Long)

    // Direction constants matching Rust enum
    const val DIRECTION_NONE = 0
    const val DIRECTION_UP = 1
    const val DIRECTION_DOWN = 2
    const val DIRECTION_LEFT = 3
    const val DIRECTION_RIGHT = 4

    // Touch action constants matching Rust enum
    const val TOUCH_DOWN = 0
    const val TOUCH_UP = 1
    const val TOUCH_MOVE = 2
}
