import Flutter
import UIKit

@main
@objc class AppDelegate: FlutterAppDelegate {
    private var gameViewFactory: GamePlatformViewFactory?

    override func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {
        let controller = window?.rootViewController as! FlutterViewController

        // Register platform view factory
        gameViewFactory = GamePlatformViewFactory(messenger: controller.binaryMessenger)
        registrar(forPlugin: "GamePlugin")?.register(
            gameViewFactory!,
            withId: "game-gl-surface"
        )

        // Set up method channel for direction input
        let channel = FlutterMethodChannel(
            name: "com.example.flutter_con/game",
            binaryMessenger: controller.binaryMessenger
        )

        channel.setMethodCallHandler { [weak self] call, result in
            switch call.method {
            case "sendDirection":
                if let args = call.arguments as? [String: Any],
                   let direction = args["direction"] as? String {
                    let dirValue: Int32 = {
                        switch direction {
                        case "up": return 1
                        case "down": return 2
                        case "left": return 3
                        case "right": return 4
                        default: return 0
                        }
                    }()
                    self?.gameViewFactory?.setDirection(dirValue)
                    result(nil)
                } else {
                    result(FlutterError(code: "INVALID_ARGS", message: "Missing direction", details: nil))
                }
            case "setMode":
                if let args = call.arguments as? [String: Any],
                   let mode = args["mode"] as? Int32 {
                    self?.gameViewFactory?.setMode(mode)
                    result(nil)
                } else {
                    result(FlutterError(code: "INVALID_ARGS", message: "Missing mode", details: nil))
                }
            default:
                result(FlutterMethodNotImplemented)
            }
        }

        GeneratedPluginRegistrant.register(with: self)
        return super.application(application, didFinishLaunchingWithOptions: launchOptions)
    }
}
