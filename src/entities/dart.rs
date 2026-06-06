// Dart projectile — thrown by DartEnemy toward the player.
// Moves in a straight line, lifetime-limited. Damages player on contact,
// destroyed on terrain collision.

use crate::level::{AABB, Vec2};

/// Horizontal speed of a dart in pixels per second.
const DART_SPEED: f32 = 150.0;

/// Maximum lifetime of a dart in seconds before it auto-destructs.
const MAX_LIFETIME: f32 = 3.0;

/// A dart projectile thrown by a DartEnemy toward the player.
///
/// `pos` = world-space center position, `vel` = velocity (px/s),
/// `alive` = whether the dart is active, `timer` = elapsed lifetime.
#[derive(Debug, Clone, Copy)]
pub struct Dart {
    /// World-space center position.
    pub pos: Vec2,
    /// Current velocity (px/s). Set at spawn toward player's position at that time.
    pub vel: Vec2,
    /// Whether the dart is active (false = pending removal).
    pub alive: bool,
    /// Elapsed lifetime in seconds since creation.
    pub timer: f32,
}

impl Dart {
    /// Creates a new dart at `pos`, traveling toward `target` at DART_SPEED.
    pub fn new(pos: Vec2, target: Vec2) -> Self {
        let dx = target.x - pos.x;
        let dy = target.y - pos.y;
        let dist = (dx * dx + dy * dy).sqrt().max(1.0);
        Self {
            pos,
            vel: Vec2 {
                x: dx / dist * DART_SPEED,
                y: dy / dist * DART_SPEED,
            },
            alive: true,
            timer: 0.0,
        }
    }

    /// Advances the dart simulation: moves (pos += vel*dt), increments timer.
    /// Kills the dart if lifetime exceeds MAX_LIFETIME.
    pub fn update(&mut self, dt: f32) {
        self.timer += dt;
        if self.timer >= MAX_LIFETIME {
            self.alive = false;
            return;
        }
        self.pos.x += self.vel.x * dt;
        self.pos.y += self.vel.y * dt;
    }

    /// Returns a 10×10 AABB centered at `self.pos`.
    pub fn collider(&self) -> AABB {
        AABB {
            x: self.pos.x - 5.0,
            y: self.pos.y - 5.0,
            w: 10.0,
            h: 10.0,
        }
    }

    /// Immediately destroys the dart.
    pub fn kill(&mut self) {
        self.alive = false;
    }

    /// Draw the dart as a spinning 4-pointed star (shuriken-like).
    pub fn draw(&self, sx: f32, sy: f32, ws: &dyn Fn(f32, f32) -> (f32, f32)) {
        use macroquad::prelude::*;
        let (spx, spy) = ws(self.pos.x, self.pos.y);
        let r = 5.0 * sx.min(sy);
        // Rotation based on timer for spin effect
        let angle = self.timer * 10.0;
        let pts: Vec<(f32, f32)> = (0..4)
            .map(|i| {
                let a = angle + i as f32 * std::f32::consts::PI / 2.0;
                (spx + a.cos() * r, spy + a.sin() * r)
            })
            .collect();
        // Draw as two crossing lines (4-pointed star)
        draw_line(pts[0].0, pts[0].1, pts[2].0, pts[2].1, 2.0, Color::new(0.9, 0.75, 0.1, 1.0));
        draw_line(pts[1].0, pts[1].1, pts[3].0, pts[3].1, 2.0, Color::new(0.9, 0.75, 0.1, 1.0));
        // Center dot
        draw_circle(spx, spy, 2.0, Color::new(1.0, 1.0, 1.0, 0.9));
    }
}
