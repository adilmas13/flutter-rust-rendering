# Phase 1: Flutter UI Foundation

Basic Flutter app with directional controls and placeholder for native view.

---

## File: `lib/main.dart`

```dart
import 'package:flutter/material.dart';
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
          // Game view placeholder
          Expanded(
            child: Container(
              margin: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: Colors.black,
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: Colors.grey),
              ),
              child: Center(
                child: Text(
                  'Game View Placeholder\nLast direction: $_lastDirection',
                  style: const TextStyle(color: Colors.white),
                  textAlign: TextAlign.center,
                ),
              ),
            ),
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
```

---

## Summary

This phase creates:

1. **GameScreen** - Main screen with game view placeholder and direction pad
2. **DirectionPad** - Widget with four directional buttons (up, down, left, right)
3. **MethodChannel** - Set up at `com.example.flutter_con/game` for native communication
4. **Visual feedback** - Displays last pressed direction in the placeholder

The placeholder container (black box) will be replaced with a PlatformView in Phase 2.

---

## Checklist

- [x] Create direction pad widget with up/down/left/right buttons
- [x] Set up MethodChannel for native communication
- [x] Add placeholder container for PlatformView
- [x] Test button press events are captured correctly

---

Awaiting your review and approval to implement this code.
