// Oscillating fireball — a hazard that moves vertically between two Y positions.
// Damages the player on contact. Can be destroyed by the player's fireballs.

use crate::level::{AABB, Vec2};

/// Vertical speed of the oscillating fireball (px/s).
const OSC_SPEED: f32 = 80.0;

/// An oscillating fireball hazard that moves up and down between two Y bounds.
///
/// `pos` = center position. `top_y` / `bottom_y` = vertical travel bounds.
/// `alive` = whether the fireball is active (false = destroyed / pending removal).
#[derive(Debug, Clone, Copy)]
pub struct OscFireball {
    /// World-space center position.
    pub pos: Vec2,
    /// Upper bound for vertical movement (cannot go above this Y).
    pub top_y: f32,
    /// Lower bound for vertical movement (cannot go below this Y).
    pub bottom_y: f32,
    /// Current vertical velocity (positive = downward in Y-down coords).
    vel_y: f32,
    /// Whether the fireball is active.
    pub alive: bool,
}

impl OscFireball {
    /// Creates a new oscillating fireball at position (`x`, `top_y`).
    /// It will oscillate between `top_y` and `bottom_y`.
    /// Initially moves downward.
    pub fn new(x: f32, top_y: f32, bottom_y: f32) -> Self {
        Self {
            pos: Vec2 { x, y: top_y },
            top_y,
            bottom_y,
            vel_y: OSC_SPEED, // start moving down
            alive: true,
        }
    }

    /// Advances the fireball: moves vertically, reverses direction at bounds.
    pub fn update(&mut self, dt: f32) {
        if !self.alive {
            return;
        }
        self.pos.y += self.vel_y * dt;

        // Reverse at bottom bound
        if self.pos.y >= self.bottom_y {
            self.pos.y = self.bottom_y;
            self.vel_y = -OSC_SPEED;
        }
        // Reverse at top bound
        if self.pos.y <= self.top_y {
            self.pos.y = self.top_y;
            self.vel_y = OSC_SPEED;
        }
    }

    /// Returns a 16×16 AABB centered at `self.pos`.
    pub fn collider(&self) -> AABB {
        AABB {
            x: self.pos.x - 8.0,
            y: self.pos.y - 8.0,
            w: 16.0,
            h: 16.0,
        }
    }

    /// Immediately destroys the fireball.
    pub fn kill(&mut self) {
        self.alive = false;
    }

    /// Draw the oscillating fireball: same visual as player fireball
    /// (orange-red orb with yellow core and white highlight).
    pub fn draw(&self, sx: f32, sy: f32, ws: &dyn Fn(f32, f32) -> (f32, f32)) {
        if !self.alive {
            return;
        }
        use macroquad::prelude::*;
        let (spx, spy) = ws(self.pos.x, self.pos.y);
        let r = 8.0 * sx.min(sy);
        // Outer orange-red glow
        draw_circle(spx, spy, r, Color::new(1.0, 0.35, 0.05, 0.95));
        // Inner bright yellow core
        draw_circle(spx, spy, r * 0.55, Color::new(1.0, 0.90, 0.15, 1.0));
        // Flicker: white highlight
        draw_circle(spx - r * 0.15, spy - r * 0.15, r * 0.22, Color::new(1.0, 1.0, 1.0, 0.6));
    }
}
