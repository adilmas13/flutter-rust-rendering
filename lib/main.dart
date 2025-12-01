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

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: const Text('Flutter Rust Game'),
      ),
      body: Column(
        children: [
          // Game view - Android PlatformView with Hybrid Composition
          Expanded(
            child: Container(
              margin: const EdgeInsets.all(16),
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
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Text('Last direction: $_lastDirection'),
          ),
          // Direction pad
          Padding(
            padding: const EdgeInsets.all(24),
            child: DirectionPad(onDirectionPressed: _sendDirection),
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
