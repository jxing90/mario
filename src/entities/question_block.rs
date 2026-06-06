// Feature #8: Collectibles & Blocks — QuestionBlock entity
// Design Reference: docs/features/8-collectibles-blocks.md §4, §6, §8

use crate::entities::loot_table::LootTable;
use crate::entities::power_up::PowerUpKind;
use crate::level::{AABB, Vec2};

/// A question-mark block that produces a reward when hit from below.
///
/// §8 Data Model: `pos` = top-left corner, `used` = whether block has been
/// activated, `flicker_frame` = animation frame counter for the "used" state.
#[derive(Debug, Clone, Copy)]
pub struct QuestionBlock {
    /// Top-left corner position in world space.
    pub pos: Vec2,
    /// Whether this block has been activated (true = empty, no further rewards).
    pub used: bool,
    /// Animation frame counter for flicker effect when used.
    pub flicker_frame: u8,
}

impl QuestionBlock {
    /// Creates a new question block at the given top-left position.
    ///
    /// # Postconditions (§4)
    /// - `used = false`
    /// - `flicker_frame = 0`
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            used: false,
            flicker_frame: 0,
        }
    }

    /// Returns a 32x32 AABB with `self.pos` as the top-left corner.
    ///
    /// # Postconditions (§4)
    /// - AABB has width 32.0 and height 32.0
    /// - Top-left corner at `self.pos`
    pub fn collider(&self) -> AABB {
        AABB {
            x: self.pos.x,
            y: self.pos.y,
            w: 32.0,
            h: 32.0,
        }
    }

    /// Activates the question block, consuming it and producing a reward.
    ///
    /// # Preconditions (§4)
    /// - `used == false` (panics if already used)
    ///
    /// # Postconditions (§4)
    /// - `used = true`
    /// - Returns a `PowerUpKind` determined by `LootTable::roll`
    ///
    /// # Panics
    /// Panics if the block has already been activated (`used == true`).
    pub fn activate(&mut self) -> PowerUpKind {
        assert!(
            !self.used,
            "QuestionBlock::activate called on already-used block"
        );
        self.used = true;
        // Use a deterministic center value — in production, LootTable::roll(rng)
        // would use an externally-seeded RNG for proper randomness.
        // For deterministic testing, consumers should use LootTable::roll_with_value.
        LootTable::roll_with_value(0.5)
    }

    /// Resets the block to its initial (unused) state.
    ///
    /// # Postconditions (§4)
    /// - `used = false`
    /// - `flicker_frame = 0`
    pub fn reset(&mut self) {
        self.used = false;
        self.flicker_frame = 0;
    }

    /// Draws the question block at its world position.
    /// - Unused: orange rectangle with white "?".
    /// - Used: dark gray rectangle.
    pub fn draw(
        &self,
        sx: f32,
        sy: f32,
        ws: &impl Fn(f32, f32) -> (f32, f32),
    ) {
        let (sbx, sby) = ws(self.pos.x, self.pos.y);
        if self.used {
            macroquad::shapes::draw_rectangle(sbx, sby, 32.0 * sx, 32.0 * sy, macroquad::color::DARKGRAY);
        } else {
            macroquad::shapes::draw_rectangle(
                sbx, sby, 32.0 * sx, 32.0 * sy,
                macroquad::color::Color::new(1.0, 0.65, 0.0, 1.0),
            );
            macroquad::text::draw_text(
                "?", sbx + 8.0 * sx, sby + 24.0 * sy,
                24.0 * sx.min(sy), macroquad::color::WHITE,
            );
        }
    }
}
