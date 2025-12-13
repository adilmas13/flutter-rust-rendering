//! Game state module with command queue architecture
//!
//! FFI calls push commands to a queue. The update loop processes them.
//! This eliminates dual-state sync issues.

mod player;

use notan_app::{App, AppState, Graphics};
use notan_draw::CreateDraw;
use notan_graphics::color::Color;
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub use player::{Direction, Mode, Player};

// ============================================================================
// Command Queue - FFI pushes commands, update() processes them
// ============================================================================

/// Commands that can be sent from FFI to the game
#[derive(Debug, Clone)]
pub enum GameCommand {
    SetDirection(i32),
    SetMode(i32),
    Touch { x: f32, y: f32, action: i32 },
    Resize { width: u32, height: u32 },
}

/// Global command queue - thread-safe
static COMMANDS: Lazy<Mutex<Vec<GameCommand>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Push a command to the queue (called from FFI)
pub fn push_command(cmd: GameCommand) {
    if let Ok(mut queue) = COMMANDS.lock() {
        queue.push(cmd);
    }
}

/// Drain all pending commands (called from update)
fn drain_commands() -> Vec<GameCommand> {
    COMMANDS
        .lock()
        .map(|mut q| q.drain(..).collect())
        .unwrap_or_default()
}

// ============================================================================
// Game State
// ============================================================================

/// Game state - implements AppState marker trait
pub struct GameState {
    player: Player,
    width: f32,
    height: f32,
}

impl AppState for GameState {}

impl GameState {
    pub fn new(gfx: &mut Graphics, width: u32, height: u32) -> Self {
        let w = width as f32;
        let h = height as f32;

        Self {
            player: Player::new(gfx, w / 2.0, h / 2.0, 200.0),
            width: w,
            height: h,
        }
    }

    /// Process a single command
    fn process_command(&mut self, cmd: GameCommand) {
        match cmd {
            GameCommand::SetDirection(dir) => {
                self.player.direction = Direction::from(dir);
            }
            GameCommand::SetMode(mode) => {
                self.player.set_mode(Mode::from(mode));
            }
            GameCommand::Touch { x, y, action } => {
                self.player.handle_touch(x, y, action, (self.width, self.height));
            }
            GameCommand::Resize { width, height } => {
                self.width = width as f32;
                self.height = height as f32;
            }
        }
    }

    pub fn player_x(&self) -> f32 {
        self.player.x
    }

    pub fn player_y(&self) -> f32 {
        self.player.y
    }
}

// ============================================================================
// Notan Callbacks
// ============================================================================

/// notan update callback
pub fn update(app: &mut App, state: &mut GameState) {
    // Get current window size (handles resize events from notan)
    let (w, h) = app.window().size();
    if w > 0 && h > 0 && (state.width != w as f32 || state.height != h as f32) {
        state.width = w as f32;
        state.height = h as f32;
        // Center player on first valid resize
        if state.player.x == 0.0 && state.player.y == 0.0 {
            state.player.x = state.width / 2.0;
            state.player.y = state.height / 2.0;
        }
    }

    // Process all pending commands from FFI
    for cmd in drain_commands() {
        state.process_command(cmd);
    }

    // Update game logic
    let delta = app.timer.delta_f32().min(0.1);
    state.player.update(delta, (state.width, state.height));
}

/// notan draw callback
pub fn draw(gfx: &mut Graphics, state: &mut GameState) {
    let mut draw = gfx.create_draw();
    draw.clear(Color::from_rgb(0.1, 0.1, 0.15));
    state.player.draw(&mut draw);
    gfx.render(&draw);
}
