import Flutter
import UIKit

class GamePlatformViewFactory: NSObject, FlutterPlatformViewFactory {
    private var messenger: FlutterBinaryMessenger
    private weak var lastCreatedView: GamePlatformView?

    init(messenger: FlutterBinaryMessenger) {
        self.messenger = messenger
        super.init()
    }

    func create(
        withFrame frame: CGRect,
        viewIdentifier viewId: Int64,
        arguments args: Any?
    ) -> FlutterPlatformView {
        let view = GamePlatformView(frame: frame, viewIdentifier: viewId, messenger: messenger)
        lastCreatedView = view
        return view
    }

    func createArgsCodec() -> FlutterMessageCodec & NSObjectProtocol {
        return FlutterStandardMessageCodec.sharedInstance()
    }

    func setDirection(_ direction: Int32) {
        lastCreatedView?.setDirection(direction)
    }
}

class GamePlatformView: NSObject, FlutterPlatformView {
    private var gameView: GameGLView

    init(frame: CGRect, viewIdentifier viewId: Int64, messenger: FlutterBinaryMessenger) {
        // Use a reasonable default frame if empty
        let actualFrame = frame.isEmpty ? CGRect(x: 0, y: 0, width: 300, height: 300) : frame
        gameView = GameGLView(frame: actualFrame)
        gameView.autoresizingMask = [.flexibleWidth, .flexibleHeight]
        super.init()
    }

    func view() -> UIView {
        return gameView
    }

    func setDirection(_ direction: Int32) {
        gameView.setDirection(direction)
    }
}
