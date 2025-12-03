package com.example.flutter_con

import android.content.Context
import io.flutter.plugin.common.BinaryMessenger
import io.flutter.plugin.common.StandardMessageCodec
import io.flutter.plugin.platform.PlatformView
import io.flutter.plugin.platform.PlatformViewFactory

class GameGLSurfaceFactory(
    private val messenger: BinaryMessenger
) : PlatformViewFactory(StandardMessageCodec.INSTANCE) {

    private var currentView: GameGLPlatformView? = null

    override fun create(context: Context, viewId: Int, args: Any?): PlatformView {
        val view = GameGLPlatformView(context, viewId, messenger)
        currentView = view
        return view
    }

    fun setDirection(direction: String) {
        currentView?.setDirection(direction)
    }

    fun setMode(mode: Int) {
        currentView?.setMode(mode)
    }
}
