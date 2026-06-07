// Feature: Key — collectible key that unlocks matching-color locked portals.
// Keys are spawned from level JSON, rendered in-world, and collected on overlap.

use crate::level::{AABB, KeyColor, Vec2};

/// A collectible key with a color that matches locked portals.
#[derive(Debug, Clone, Copy)]
pub struct Key {
    pub pos: Vec2,
    pub color: KeyColor,
    pub collected: bool,
}

impl Key {
    pub fn new(pos: Vec2, color: KeyColor) -> Self {
        Self {
            pos,
            color,
            collected: false,
        }
    }

    /// Returns a 16x16 AABB centered on pos for overlap detection.
    pub fn collider(&self) -> AABB {
        AABB {
            x: self.pos.x - 8.0,
            y: self.pos.y - 8.0,
            w: 16.0,
            h: 16.0,
        }
    }

    /// Draws the key at its world position using its color.
    pub fn draw(
        &self,
        sx: f32,
        sy: f32,
        ws: &impl Fn(f32, f32) -> (f32, f32),
    ) {
        if self.collected {
            return;
        }
        let (px, py) = ws(self.pos.x, self.pos.y);
        let half = 8.0 * sx.min(sy);
        let color = key_color_rgba(self.color);

        // Key body — circle with stem
        macroquad::shapes::draw_circle(px, py - 2.0 * sy, half * 0.6, color);
        // Stem
        macroquad::shapes::draw_rectangle(
            px - 1.5 * sx,
            py,
            3.0 * sx,
            half * 1.3,
            color,
        );
        // Teeth
        macroquad::shapes::draw_rectangle(
            px + 1.5 * sx,
            py + half * 0.5,
            3.0 * sx,
            2.0 * sy,
            color,
        );
        macroquad::shapes::draw_rectangle(
            px + 1.5 * sx,
            py + half * 0.9,
            3.0 * sx,
            2.0 * sy,
            color,
        );
    }
}

/// Returns the RGBA color for a given key color.
pub fn key_color_rgba(c: KeyColor) -> macroquad::color::Color {
    use macroquad::color::Color;
    match c {
        KeyColor::Red    => Color::new(1.0, 0.15, 0.15, 1.0),
        KeyColor::Orange => Color::new(1.0, 0.55, 0.05, 1.0),
        KeyColor::Yellow => Color::new(1.0, 0.90, 0.10, 1.0),
        KeyColor::Green  => Color::new(0.15, 0.85, 0.15, 1.0),
        KeyColor::Blue   => Color::new(0.15, 0.40, 1.0, 1.0),
        KeyColor::Indigo => Color::new(0.30, 0.15, 0.80, 1.0),
        KeyColor::Violet => Color::new(0.75, 0.25, 0.95, 1.0),
    }
}
