# Phase 7: egui UI Enhancements

Add immediate-mode UI elements via egui.

---

## File: `rust/src/lib.rs` (updated sections)

### Updated GameState struct

Add score tracking:

```rust
/// Game state held across FFI boundary
pub struct GameState {
    gl: Arc<glow::Context>,
    width: u32,
    height: u32,

    // egui
    egui_ctx: egui::Context,
    egui_painter: egui_glow::Painter,

    // Player state
    player_x: f32,
    player_y: f32,
    player_size: f32,
    current_direction: Direction,

    // Touch state
    is_player_touched: bool,
    touch_pos: Option<Pos2>,

    // Game stats
    score: u32,
    moves: u32,

    // UI state
    show_debug: bool,

    // Time tracking
    last_frame_time: std::time::Instant,
}
```

### Updated game_init

Initialize new fields:

```rust
let state = Box::new(GameState {
    gl,
    width,
    height,
    egui_ctx,
    egui_painter,
    player_x: width as f32 / 2.0,
    player_y: height as f32 / 2.0,
    player_size,
    current_direction: Direction::None,
    is_player_touched: false,
    touch_pos: None,
    score: 0,
    moves: 0,
    show_debug: true,
    last_frame_time: std::time::Instant::now(),
});
```

### Updated game_update

Track moves:

```rust
#[no_mangle]
pub extern "C" fn game_update(handle: GameHandle) {
    if handle.is_null() {
        return;
    }
    let state = unsafe { &mut *handle };

    // Calculate delta time
    let now = std::time::Instant::now();
    let delta = now.duration_since(state.last_frame_time).as_secs_f32();
    state.last_frame_time = now;

    // Move player based on direction
    let speed = 300.0 * delta;
    let moved = state.current_direction != Direction::None;

    match state.current_direction {
        Direction::Up => state.player_y -= speed,
        Direction::Down => state.player_y += speed,
        Direction::Left => state.player_x -= speed,
        Direction::Right => state.player_x += speed,
        Direction::None => {}
    }

    // Clamp to bounds
    let half = state.player_size / 2.0;
    let old_x = state.player_x;
    let old_y = state.player_y;
    state.player_x = state.player_x.clamp(half, state.width as f32 - half);
    state.player_y = state.player_y.clamp(half, state.height as f32 - half);

    // Increment score when hitting edges
    if moved && (state.player_x != old_x || state.player_y != old_y) {
        // Hit a boundary
    }
}
```

### Updated game_render with UI panels

```rust
#[no_mangle]
pub extern "C" fn game_render(handle: GameHandle) {
    if handle.is_null() {
        return;
    }
    let state = unsafe { &mut *handle };

    // Clear background
    unsafe {
        state.gl.clear_color(0.1, 0.1, 0.15, 1.0);
        state.gl.clear(glow::COLOR_BUFFER_BIT);
    }

    let screen_rect = Rect::from_min_size(
        Pos2::ZERO,
        Vec2::new(state.width as f32, state.height as f32),
    );

    // Build egui input with touch
    let mut raw_input = egui::RawInput {
        screen_rect: Some(screen_rect),
        ..Default::default()
    };

    // Add touch/pointer events
    if let Some(pos) = state.touch_pos {
        raw_input.events.push(egui::Event::PointerMoved(pos));
        if state.is_player_touched {
            raw_input.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            });
        }
    }

    let full_output = state.egui_ctx.run(raw_input, |ctx| {
        // === Status bar at top ===
        egui::TopBottomPanel::top("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("Score: {}", state.score))
                        .size(20.0)
                        .color(Color32::WHITE),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(if state.show_debug { "Hide Debug" } else { "Show Debug" }).clicked() {
                        state.show_debug = !state.show_debug;
                    }
                });
            });
        });

        // === Debug panel ===
        if state.show_debug {
            egui::Window::new("Debug")
                .anchor(egui::Align2::RIGHT_BOTTOM, [-10.0, -10.0])
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label(format!("Position: ({:.0}, {:.0})", state.player_x, state.player_y));
                    ui.label(format!("Direction: {:?}", state.current_direction));
                    ui.label(format!("Moves: {}", state.moves));
                    ui.label(format!("Screen: {}x{}", state.width, state.height));
                    ui.separator();
                    if ui.button("Reset Position").clicked() {
                        state.player_x = state.width as f32 / 2.0;
                        state.player_y = state.height as f32 / 2.0;
                        state.score = 0;
                        state.moves = 0;
                    }
                });
        }

        // === Draw player shape on background layer ===
        let painter = ctx.layer_painter(egui::LayerId::background());

        let fill_color = if state.is_player_touched {
            Color32::from_rgb(255, 150, 50)
        } else {
            match state.current_direction {
                Direction::Up => Color32::from_rgb(50, 200, 50),
                Direction::Down => Color32::from_rgb(200, 50, 50),
                Direction::Left => Color32::from_rgb(50, 50, 200),
                Direction::Right => Color32::from_rgb(200, 200, 50),
                Direction::None => Color32::from_rgb(150, 150, 150),
            }
        };

        let stroke_color = Color32::WHITE;
        let center = Pos2::new(state.player_x, state.player_y);
        let half = state.player_size / 2.0;

        // Draw rounded rectangle
        let rect = Rect::from_center_size(center, Vec2::splat(state.player_size));
        painter.rect(rect, Rounding::same(12.0), fill_color, Stroke::new(3.0, stroke_color));

        // Draw circle inside
        painter.circle(center, half * 0.5, fill_color.gamma_multiply(0.6), Stroke::new(2.0, stroke_color));
    });

    // Tessellate and paint
    let pixels_per_point = 1.0;
    let clipped_primitives = state.egui_ctx.tessellate(full_output.shapes, pixels_per_point);

    state.egui_painter.paint_and_update_textures(
        [state.width, state.height],
        pixels_per_point,
        &clipped_primitives,
        &full_output.textures_delta,
    );
}
```

### Updated game_touch for egui interaction

```rust
#[no_mangle]
pub extern "C" fn game_touch(handle: GameHandle, x: f32, y: f32, action: i32) {
    if handle.is_null() {
        return;
    }
    let state = unsafe { &mut *handle };
    let touch_action = TouchAction::from(action);

    let pos = Pos2::new(x, y);

    match touch_action {
        TouchAction::Down => {
            state.touch_pos = Some(pos);
            // Check if touch is within player box
            let half = state.player_size / 2.0;
            let is_on_player = x >= state.player_x - half
                && x <= state.player_x + half
                && y >= state.player_y - half
                && y <= state.player_y + half;
            if is_on_player {
                state.is_player_touched = true;
                state.score += 10;
                log::info!("Player touched! Score: {}", state.score);
            }
        }
        TouchAction::Up => {
            state.touch_pos = None;
            state.is_player_touched = false;
        }
        TouchAction::Move => {
            state.touch_pos = Some(pos);
        }
    }
}
```

### Updated game_set_direction to track moves

```rust
#[no_mangle]
pub extern "C" fn game_set_direction(handle: GameHandle, direction: i32) {
    if handle.is_null() {
        return;
    }
    let state = unsafe { &mut *handle };
    let new_direction = Direction::from(direction);

    // Count direction changes as moves
    if new_direction != Direction::None && new_direction != state.current_direction {
        state.moves += 1;
    }

    state.current_direction = new_direction;
    log::debug!("game_set_direction: {:?}", state.current_direction);
}
```

---

## Summary

Phase 7 adds:

1. **Status Bar** (top) - Shows score, toggle debug button
2. **Debug Panel** (bottom-right) - Shows position, direction, moves, screen size, reset button
3. **Score System** - +10 points when touching the player shape
4. **Move Counter** - Tracks direction changes
5. **Touch Integration** - Touch events passed to egui for button interactions
6. **Reset Button** - Centers player and resets stats

---

## Checklist

- [ ] Create simple debug panel showing game state
- [ ] Add in-game UI elements (score, status)
- [ ] Handle touch input for egui interactions

---

Awaiting your review and approval to implement this code.
