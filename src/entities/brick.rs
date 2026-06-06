// Feature #8: Collectibles & Blocks — Brick entity
// Design Reference: docs/features/8-collectibles-blocks.md

use crate::level::{AABB, Vec2};

/// A breakable brick block. Solid platform; shatters when Super/Fire Mario
/// hits it from below. Small Mario just bounces off.
#[derive(Debug, Clone)]
pub struct Brick {
    pub pos: Vec2,
    pub broken: bool,
}

impl Brick {
    /// Creates a new brick at the given top-left position.
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            broken: false,
        }
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
    /// - Broken: nothing drawn.
    pub fn draw(
        &self,
        sx: f32,
        sy: f32,
        ws: &impl Fn(f32, f32) -> (f32, f32),
    ) {
        if self.broken {
            return;
        }
        let (sbx, sby) = ws(self.pos.x, self.pos.y);
        let brick_color = macroquad::color::Color::new(0.65, 0.40, 0.20, 1.0);
        macroquad::shapes::draw_rectangle(sbx, sby, 32.0 * sx, 32.0 * sy, brick_color);
        // Mortar lines
        let line_color = macroquad::color::Color::new(0.35, 0.20, 0.10, 1.0);
        macroquad::shapes::draw_rectangle(sbx, sby + 15.0 * sy, 32.0 * sx, 2.0 * sy, line_color);
        macroquad::shapes::draw_rectangle(sbx + 15.0 * sx, sby, 2.0 * sx, 15.0 * sy, line_color);
    }
}
