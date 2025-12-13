import 'dart:io' show Platform;

import 'package:flutter/foundation.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Rust Game',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
      ),
      home: const GameScreen(),
    );
  }
}

class GameScreen extends StatefulWidget {
  const GameScreen({super.key});

  @override
  State<GameScreen> createState() => _GameScreenState();
}

class _GameScreenState extends State<GameScreen> {
  static const platform = MethodChannel('com.example.flutter_con/game');
  String _lastDirection = 'none';
  bool _isAutoMode = false;
  bool _fpsLimitEnabled = false;
  int _targetFps = 30;

  Future<void> _sendDirection(String direction) async {
    setState(() {
      _lastDirection = direction;
    });

    try {
      await platform.invokeMethod('sendDirection', {'direction': direction});
    } on PlatformException catch (e) {
      debugPrint('Failed to send direction: ${e.message}');
    }
  }

  Future<void> _setMode(bool auto) async {
    setState(() {
      _isAutoMode = auto;
    });

    try {
      await platform.invokeMethod('setMode', {'mode': auto ? 1 : 0});
    } on PlatformException catch (e) {
      debugPrint('Failed to set mode: ${e.message}');
    }
  }

  Future<void> _setFps(int fps) async {
    try {
      await platform.invokeMethod('setFps', {'fps': fps});
    } on PlatformException catch (e) {
      debugPrint('Failed to set FPS: ${e.message}');
    }
  }

  void _onFpsLimitToggle(bool? enabled) {
    setState(() {
      _fpsLimitEnabled = enabled ?? false;
    });
    _setFps(_fpsLimitEnabled ? _targetFps : 0);
  }

  void _onFpsSliderChange(double value) {
    setState(() {
      _targetFps = value.round();
    });
    if (_fpsLimitEnabled) {
      _setFps(_targetFps);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: const Text('Flutter Rust Game'),
      ),
      body: Column(
        children: [
          // Mode toggle buttons and FPS dropdown
          Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              children: [
                Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    ElevatedButton(
                      onPressed: () => _setMode(false),
                      style: ElevatedButton.styleFrom(
                        backgroundColor: !_isAutoMode ? Colors.blue : Colors.grey,
                        foregroundColor: Colors.white,
                      ),
                      child: const Text('Manual'),
                    ),
                    const SizedBox(width: 16),
                    ElevatedButton(
                      onPressed: () => _setMode(true),
                      style: ElevatedButton.styleFrom(
                        backgroundColor: _isAutoMode ? Colors.blue : Colors.grey,
                        foregroundColor: Colors.white,
                      ),
                      child: const Text('Auto'),
                    ),
                  ],
                ),
                Row(
                  children: [
                    Checkbox(
                      value: _fpsLimitEnabled,
                      onChanged: _onFpsLimitToggle,
                    ),
                    const Text('Limit FPS'),
                    Expanded(
                      child: Slider(
                        value: _targetFps.toDouble(),
                        min: 1,
                        max: 60,
                        divisions: 59,
                        label: '$_targetFps',
                        onChanged: _fpsLimitEnabled ? _onFpsSliderChange : null,
                      ),
                    ),
                    Text('$_targetFps fps'),
                  ],
                )
              ],
            )
          ),
          // Game view - Android PlatformView with Hybrid Composition
          Expanded(
            child: Container(
              margin: const EdgeInsets.symmetric(horizontal: 16),
              clipBehavior: Clip.hardEdge,
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: Colors.grey),
              ),
              child: const GameGLView(),
            ),
          ),
          // Direction indicator
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            child: Text(
              _isAutoMode ? 'Mode: Auto (bouncing)' : 'Last direction: $_lastDirection',
            ),
          ),
          // Direction pad (disabled in auto mode)
          Padding(
            padding: const EdgeInsets.all(24),
            child: IgnorePointer(
              ignoring: _isAutoMode,
              child: Opacity(
                opacity: _isAutoMode ? 0.3 : 1.0,
                child: DirectionPad(onDirectionPressed: _sendDirection),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class GameGLView extends StatelessWidget {
  const GameGLView({super.key});

  @override
  Widget build(BuildContext context) {
    const String viewType = 'game-gl-surface';

    if (Platform.isAndroid) {
      // Android: Use PlatformViewLink with Hybrid Composition
      return PlatformViewLink(
        viewType: viewType,
        surfaceFactory: (context, controller) {
          return AndroidViewSurface(
            controller: controller as AndroidViewController,
            gestureRecognizers: const <Factory<OneSequenceGestureRecognizer>>{},
            hitTestBehavior: PlatformViewHitTestBehavior.opaque,
          );
        },
        onCreatePlatformView: (params) {
          return PlatformViewsService.initSurfaceAndroidView(
            id: params.id,
            viewType: viewType,
            layoutDirection: TextDirection.ltr,
            creationParams: null,
            creationParamsCodec: const StandardMessageCodec(),
            onFocus: () {
              params.onFocusChanged(true);
            },
          )
            ..addOnPlatformViewCreatedListener(params.onPlatformViewCreated)
            ..create();
        },
      );
    } else if (Platform.isIOS) {
      // iOS: Use UiKitView
      return const UiKitView(
        viewType: viewType,
        creationParamsCodec: StandardMessageCodec(),
      );
    }

    // Unsupported platform
    return const Center(
      child: Text('Platform not supported'),
    );
  }
}

class DirectionPad extends StatelessWidget {
  final void Function(String direction) onDirectionPressed;

  const DirectionPad({super.key, required this.onDirectionPressed});

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 200,
      height: 200,
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          // Up button
          _DirectionButton(
            icon: Icons.arrow_upward,
            onPressed: () => onDirectionPressed('up'),
          ),
          // Left and Right buttons
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              _DirectionButton(
                icon: Icons.arrow_back,
                onPressed: () => onDirectionPressed('left'),
              ),
              const SizedBox(width: 60),
              _DirectionButton(
                icon: Icons.arrow_forward,
                onPressed: () => onDirectionPressed('right'),
              ),
            ],
          ),
          // Down button
          _DirectionButton(
            icon: Icons.arrow_downward,
            onPressed: () => onDirectionPressed('down'),
          ),
        ],
      ),
    );
  }
}

class _DirectionButton extends StatelessWidget {
  final IconData icon;
  final VoidCallback onPressed;

  const _DirectionButton({required this.icon, required this.onPressed});

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 60,
      height: 60,
      child: ElevatedButton(
        onPressed: onPressed,
        style: ElevatedButton.styleFrom(
          padding: EdgeInsets.zero,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(12),
          ),
        ),
        child: Icon(icon, size: 32),
      ),
    );
  }
}
