// Feature #8: Collectibles & Blocks — Brick entity
// Design Reference: docs/features/8-collectibles-blocks.md

use crate::level::{AABB, Vec2};

/// A breakable brick block. Solid platform; shatters when Super/Fire Mario
/// hits it from below. Small Mario just bounces off.
#[derive(Debug, Clone)]
pub struct Brick {
    pub pos: Vec2,
    pub broken: bool,
    /// Break particles: (x, y, vx, vy, lifetime) — active only during shatter animation.
    pub particles: Vec<(f32, f32, f32, f32, f32)>,
}

impl Brick {
    /// Creates a new brick at the given top-left position.
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            broken: false,
            particles: Vec::new(),
        }
    }

    /// Shatters the brick, spawning 4 debris particles that fly outward.
    pub fn shatter(&mut self) {
        self.broken = true;
        let cx = self.pos.x + 16.0;
        let cy = self.pos.y + 16.0;
        // 4 pieces flying diagonally outward
        self.particles = vec![
            (cx - 8.0, cy - 8.0, -120.0, -180.0, 0.6),
            (cx + 8.0, cy - 8.0, 120.0, -160.0, 0.6),
            (cx - 8.0, cy + 8.0, -100.0, 140.0, 0.6),
            (cx + 8.0, cy + 8.0, 110.0, 150.0, 0.6),
        ];
    }

    /// Updates particle positions and lifetimes. Removes dead particles.
    pub fn update_particles(&mut self, dt: f32) {
        for (x, y, vx, vy, life) in self.particles.iter_mut() {
            *x += *vx * dt;
            *vy += 600.0 * dt; // gravity
            *y += *vy * dt;
            *life -= dt;
        }
        self.particles.retain(|(_, _, _, _, life)| *life > 0.0);
    }

    /// Returns a 32x32 AABB with `self.pos` as the top-left corner.
    pub fn collider(&self) -> AABB {
        AABB {
            x: self.pos.x,
            y: self.pos.y,
            w: 32.0,
            h: 32.0,
        }
    }

    /// Draws the brick at its world position, translated to screen coordinates.
    /// - Unbroken: brown rectangle with mortar lines.
    /// - Broken: debris particles.
    pub fn draw(
        &self,
        sx: f32,
        sy: f32,
        ws: &impl Fn(f32, f32) -> (f32, f32),
    ) {
        use macroquad::color::Color;
        use macroquad::shapes::draw_rectangle;

        if self.broken {
            // Draw debris particles
            for &(px, py, _, _, life) in &self.particles {
                if life > 0.0 {
                    let alpha = life / 0.6;
                    let (spx, spy) = ws(px, py);
                    let size = 8.0 * sx.min(sy) * alpha;
                    draw_rectangle(spx, spy, size, size, Color::new(0.65, 0.40, 0.20, alpha));
                }
            }
            return;
        }
        let (sbx, sby) = ws(self.pos.x, self.pos.y);
        let brick_color = Color::new(0.65, 0.40, 0.20, 1.0);
        draw_rectangle(sbx, sby, 32.0 * sx, 32.0 * sy, brick_color);
        // Mortar lines
        let line_color = Color::new(0.35, 0.20, 0.10, 1.0);
        draw_rectangle(sbx, sby + 15.0 * sy, 32.0 * sx, 2.0 * sy, line_color);
        draw_rectangle(sbx + 15.0 * sx, sby, 2.0 * sx, 15.0 * sy, line_color);
    }
}
