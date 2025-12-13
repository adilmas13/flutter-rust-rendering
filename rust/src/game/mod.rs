//! Game state module

mod player;

use notan_app::{App, AppState, Graphics};
use notan_draw::CreateDraw;
use notan_graphics::color::Color;
use once_cell::sync::OnceCell;
use std::sync::Mutex;

pub use player::{Direction, Mode, Player};

/// Global game state access for FFI
static GAME: OnceCell<Mutex<GameState>> = OnceCell::new();

/// Get mutable access to game state from FFI
pub fn with_game<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut GameState) -> R,
{
    GAME.get().and_then(|m| m.lock().ok().map(|mut g| f(&mut g)))
}

/// Initialize the global game state (called once from game_init)
pub fn init_global(state: GameState) {
    let _ = GAME.set(Mutex::new(state));
}

/// Game state - implements AppState marker trait
pub struct GameState {
    player: Player,
    width: f32,
    height: f32,
}

// Implement the AppState marker trait for GameState
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

    pub fn set_direction(&mut self, direction: i32) {
        self.player.direction = Direction::from(direction);
    }

    pub fn set_mode(&mut self, mode: i32) {
        self.player.set_mode(Mode::from(mode));
    }

    pub fn handle_touch(&mut self, x: f32, y: f32, action: i32) {
        self.player
            .handle_touch(x, y, action, (self.width, self.height));
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width as f32;
        self.height = height as f32;
    }

    pub fn player_x(&self) -> f32 {
        self.player.x
    }

    pub fn player_y(&self) -> f32 {
        self.player.y
    }
}

/// notan update callback
pub fn update(app: &mut App, state: &mut GameState) {
    // Get current window size (handles resize events)
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

    // Sync input state FROM global GAME TO local state
    // (FFI calls update GAME, but we render using state)
    with_game(|g| {
        state.player.direction = g.player.direction;
        // Sync mode and velocity (velocity is set when switching to Auto mode)
        if state.player.mode != g.player.mode {
            state.player.mode = g.player.mode;
            state.player.velocity = g.player.velocity;
        }
        // Sync touch drag position only when GAME has a valid (non-zero) position
        // This ensures initial centering in state isn't overwritten by GAME's 0,0
        if g.player.x > 0.0 && g.player.y > 0.0 {
            state.player.x = g.player.x;
            state.player.y = g.player.y;
        }
    });

    let delta = app.timer.delta_f32().min(0.1);
    state.player.update(delta, (state.width, state.height));

    // Sync position back to global for FFI queries
    with_game(|g| {
        g.player.x = state.player.x;
        g.player.y = state.player.y;
        g.width = state.width;
        g.height = state.height;
    });
}

/// notan draw callback
pub fn draw(gfx: &mut Graphics, state: &mut GameState) {
    let mut draw = gfx.create_draw();
    draw.clear(Color::from_rgb(0.1, 0.1, 0.15));
    state.player.draw(&mut draw);
    gfx.render(&draw);
}
