// Feature #8: Collectibles & Blocks — Coin entity
// Design Reference: docs/features/8-collectibles-blocks.md §4, §6, §8

use crate::level::{AABB, Vec2};

/// A collectible coin placed in the level.
///
/// §8 Data Model: `pos` = world-space center position, `collected` = whether
/// the coin has been picked up, `frame` = animation frame counter.
#[derive(Debug, Clone, Copy)]
pub struct Coin {
    /// World-space center position of the coin.
    pub pos: Vec2,
    /// Whether this coin has been collected (true = invisible & non-interactive).
    pub collected: bool,
    /// Animation frame counter.
    pub frame: u8,
}

impl Coin {
    /// Creates a new coin at the given world-space center position.
    ///
    /// # Postconditions (§4)
    /// - `collected = false`
    /// - `frame = 0`
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            collected: false,
            frame: 0,
        }
    }

    /// Returns a 16x16 AABB centered at `self.pos`.
    ///
    /// # Postconditions (§4)
    /// - AABB has width 16.0 and height 16.0
    /// - Top-left corner is `(pos.x - 8.0, pos.y - 8.0)`
    pub fn collider(&self) -> AABB {
        AABB {
            x: self.pos.x - 8.0,
            y: self.pos.y - 8.0,
            w: 16.0,
            h: 16.0,
        }
    }

    /// Resets the coin to its initial (uncollected) state.
    ///
    /// # Postconditions (§4)
    /// - `collected = false`
    /// - `frame = 0`
    pub fn reset(&mut self) {
        self.collected = false;
        self.frame = 0;
    }

    /// Draws the coin as an elliptical yellow shape with a "$" symbol.
    /// Only draws if not yet collected.
    pub fn draw(
        &self,
        sx: f32,
        sy: f32,
        ws: &impl Fn(f32, f32) -> (f32, f32),
    ) {
        if self.collected {
            return;
        }
        use macroquad::color::Color;
        use macroquad::shapes::draw_ellipse;
        use macroquad::text::draw_text;
        let (scx, scy) = ws(self.pos.x, self.pos.y);
        let rx = 7.0 * sx;
        let ry = 5.0 * sy; // slightly flattened → elliptical
        // Outer yellow oval
        draw_ellipse(scx, scy, rx, ry, 0.0, Color::new(0.95, 0.80, 0.15, 1.0));
        // Inner dark yellow rim
        draw_ellipse(scx, scy, rx * 0.75, ry * 0.75, 0.0, Color::new(0.85, 0.65, 0.05, 1.0));
        // "$" symbol in center
        let font_size = 10.0 * sx.min(sy);
        let text_w = font_size * 0.5;
        let text_h = font_size * 0.7;
        draw_text("$", scx - text_w, scy + text_h, font_size, Color::new(0.9, 0.75, 0.1, 1.0));
    }
}
