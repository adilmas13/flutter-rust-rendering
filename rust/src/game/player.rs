//! Player entity

use notan_app::Graphics;
use notan_draw::{Draw, DrawImages, DrawShapes};
use notan_graphics::{color::Color, Texture};

/// Direction for movement
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    #[default]
    None,
    Up,
    Down,
    Left,
    Right,
}

impl From<i32> for Direction {
    fn from(value: i32) -> Self {
        match value {
            1 => Direction::Up,
            2 => Direction::Down,
            3 => Direction::Left,
            4 => Direction::Right,
            _ => Direction::None,
        }
    }
}

/// Game mode
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Manual,
    Auto,
}

impl From<i32> for Mode {
    fn from(value: i32) -> Self {
        match value {
            1 => Mode::Auto,
            _ => Mode::Manual,
        }
    }
}

/// Player entity
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub direction: Direction,
    pub mode: Mode,
    pub velocity: (f32, f32),
    pub tint: Color,
    pub texture: Option<Texture>,
    // Touch drag state
    is_touched: bool,
    drag_offset: (f32, f32),
}

impl Player {
    pub fn new(gfx: &mut Graphics, x: f32, y: f32, size: f32) -> Self {
        let texture = gfx
            .create_texture()
            .from_image(include_bytes!("../../assets/player.png"))
            .build()
            .ok();

        Self {
            x,
            y,
            size,
            direction: Direction::None,
            mode: Mode::Manual,
            velocity: (0.0, 0.0),
            tint: Color::WHITE,
            texture,
            is_touched: false,
            drag_offset: (0.0, 0.0),
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        if mode == Mode::Auto && self.mode != Mode::Auto {
            // Start bouncing with initial velocity
            self.velocity = (250.0, 200.0);
        }
        self.mode = mode;
    }

    pub fn handle_touch(&mut self, x: f32, y: f32, action: i32, bounds: (f32, f32)) {
        let half = self.size / 2.0;
        let on_player = x >= self.x - half
            && x <= self.x + half
            && y >= self.y - half
            && y <= self.y + half;

        match action {
            0 if on_player => {
                self.is_touched = true;
                self.drag_offset = (self.x - x, self.y - y);
            }
            1 => self.is_touched = false,
            2 if self.is_touched => {
                self.x = (x + self.drag_offset.0).clamp(half, bounds.0 - half);
                self.y = (y + self.drag_offset.1).clamp(half, bounds.1 - half);
            }
            _ => {}
        }
    }

    pub fn update(&mut self, delta: f32, bounds: (f32, f32)) {
        // Guard against zero/invalid bounds
        if bounds.0 <= 0.0 || bounds.1 <= 0.0 {
            return;
        }

        let half = self.size / 2.0;

        match self.mode {
            Mode::Manual => {
                let speed = 300.0 * delta;
                match self.direction {
                    Direction::Up => self.y -= speed,
                    Direction::Down => self.y += speed,
                    Direction::Left => self.x -= speed,
                    Direction::Right => self.x += speed,
                    Direction::None => {}
                }
            }
            Mode::Auto => {
                self.x += self.velocity.0 * delta;
                self.y += self.velocity.1 * delta;

                // Bounce off walls
                if self.x <= half || self.x >= bounds.0 - half {
                    self.velocity.0 = -self.velocity.0;
                    self.tint = random_color();
                }
                if self.y <= half || self.y >= bounds.1 - half {
                    self.velocity.1 = -self.velocity.1;
                    self.tint = random_color();
                }
            }
        }

        // Clamp position (ensure min < max)
        let min_x = half.min(bounds.0 - half);
        let max_x = half.max(bounds.0 - half);
        let min_y = half.min(bounds.1 - half);
        let max_y = half.max(bounds.1 - half);
        self.x = self.x.clamp(min_x, max_x);
        self.y = self.y.clamp(min_y, max_y);
    }

    pub fn draw(&self, draw: &mut Draw) {
        log::info!("Drawing player at ({}, {})", self.x, self.y);
        let tint = if self.is_touched {
            Color::from_rgb(1.0, 0.6, 0.2)
        } else {
            self.tint
        };

        let pos = (self.x - self.size / 2.0, self.y - self.size / 2.0);

        if let Some(ref tex) = self.texture {
            draw.image(tex)
                .position(pos.0, pos.1)
                .size(self.size, self.size)
                .color(tint);
        } else {
            draw.rect(pos, (self.size, self.size))
                .color(tint)
                .corner_radius(8.0);
        }
    }
}

fn random_color() -> Color {
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    let r = 0.5 + ((time & 0xFF) as f32 / 255.0) * 0.5;
    let g = 0.5 + (((time >> 8) & 0xFF) as f32 / 255.0) * 0.5;
    let b = 0.5 + (((time >> 16) & 0xFF) as f32 / 255.0) * 0.5;

    Color::from_rgb(r, g, b)
}
