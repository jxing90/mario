// Feature #6: Life, Death & Win — Checkpoint entity
// Design Reference: docs/features/6-life-death-win.md §4, §8
//
// Checkpoint is a trigger zone placed in the level. When the player's
// collider overlaps an inactive checkpoint, it activates and stores its
// position as the respawn point for subsequent deaths.

use crate::level::{AABB, Vec2};

/// A checkpoint trigger in the level.
///
/// # Fields (§8 Data Model)
/// - `pos`: world-space position (center of the checkpoint flag).
/// - `activated`: whether this checkpoint has been triggered by the player.
pub struct Checkpoint {
    pub pos: Vec2,
    pub activated: bool,
}

impl Checkpoint {
    /// Creates a new Checkpoint at the given world position.
    ///
    /// # Postconditions (§4)
    /// - `activated = false`
    /// - Trigger zone AABB is 16×32 px centered at `pos`.
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            activated: false,
        }
    }

    /// Returns the trigger-zone AABB for checkpoint activation detection.
    ///
    /// Dimensions: w=16, h=32, centered on `pos`.
    pub fn collider(&self) -> AABB {
        AABB {
            x: self.pos.x - 8.0,
            y: self.pos.y - 16.0,
            w: 16.0,
            h: 32.0,
        }
    }

    /// Draws the checkpoint as a flagpole with banner.
    /// - Inactive: dark green banner with "CP".
    /// - Activated: bright green banner with "OK".
    pub fn draw(
        &self,
        sx: f32,
        sy: f32,
        ws: &impl Fn(f32, f32) -> (f32, f32),
    ) {
        let (cpx, cpy) = ws(self.pos.x, self.pos.y);
        let pole_h = 40.0 * sy;
        let pole_w = 3.0 * sx;
        // Pole
        let pole_color = macroquad::color::Color::new(0.6, 0.6, 0.6, 1.0);
        macroquad::shapes::draw_rectangle(cpx, cpy - pole_h, pole_w, pole_h, pole_color);
        // Banner
        let banner_color = if self.activated {
            macroquad::color::Color::new(0.1, 0.8, 0.2, 1.0)
        } else {
            macroquad::color::Color::new(0.3, 0.5, 0.3, 1.0)
        };
        let banner_w = 16.0 * sx;
        let banner_h = 12.0 * sy;
        macroquad::shapes::draw_rectangle(cpx + pole_w, cpy - pole_h, banner_w, banner_h, banner_color);
        // Text
        let banner_text = if self.activated { "OK" } else { "CP" };
        macroquad::text::draw_text(
            banner_text,
            cpx + pole_w + 2.0 * sx,
            cpy - pole_h + 10.0 * sy,
            12.0 * sx.min(sy),
            macroquad::color::WHITE,
        );
    }
}
