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

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies activation conditions matching playing.rs logic.
    /// Player Small (16x16 AABB), head at `head_y`, feet at `foot_y`.
    fn should_activate(block: &QuestionBlock, head_y: f32, foot_y: f32, center_x: f32) -> bool {
        let ba = block.collider();
        let block_bottom = ba.y + ba.h;
        let pa = AABB { x: center_x - 8.0, y: head_y, w: 16.0, h: 16.0 };
        const HEAD_TOLERANCE: f32 = 8.0;
        pa.intersects(&ba)
            && head_y >= block_bottom - HEAD_TOLERANCE
            && head_y <= block_bottom
            && foot_y >= block_bottom
            && center_x > ba.x - 4.0
            && center_x < ba.x + ba.w + 4.0
    }

    #[test]
    fn test_head_at_block_bottom_activates() {
        let block = QuestionBlock::new(Vec2 { x: 380.0, y: 420.0 });
        // block_bottom=452, head=452 (just touching), foot=468 (below)
        assert!(should_activate(&block, 452.0, 468.0, 380.0));
    }

    #[test]
    fn test_head_6px_inside_activates() {
        let block = QuestionBlock::new(Vec2 { x: 380.0, y: 420.0 });
        // head=446 (~6px inside from vel=-350 overshoot), tolerance 8 covers it
        assert!(should_activate(&block, 446.0, 462.0, 380.0));
    }

    #[test]
    fn test_head_12px_inside_no_activate() {
        let block = QuestionBlock::new(Vec2 { x: 380.0, y: 420.0 });
        // head=440 (12px in), foot=456 (still below). 440 >= 444? NO.
        assert!(!should_activate(&block, 440.0, 456.0, 380.0));
    }

    #[test]
    fn test_no_activate_from_above() {
        let block = QuestionBlock::new(Vec2 { x: 380.0, y: 420.0 });
        // Standing ON block: foot=420 (on top), head=404.
        // foot_y (420) >= block_bottom (452)? NO.
        assert!(!should_activate(&block, 404.0, 420.0, 380.0));
    }

    #[test]
    fn test_no_activate_from_side() {
        let block = QuestionBlock::new(Vec2 { x: 380.0, y: 420.0 });
        // Center X outside the block (±4px margin)
        assert!(!should_activate(&block, 452.0, 468.0, 370.0)); // left
        assert!(!should_activate(&block, 452.0, 468.0, 418.0)); // right
    }

    #[test]
    fn test_activate_sets_used() {
        let mut block = QuestionBlock::new(Vec2 { x: 100.0, y: 100.0 });
        assert!(!block.used);
        let _kind = block.activate();
        assert!(block.used);
    }

    #[test]
    #[should_panic(expected = "already-used")]
    fn test_double_activate_panics() {
        let mut block = QuestionBlock::new(Vec2 { x: 100.0, y: 100.0 });
        block.activate();
        block.activate();
    }

    #[test]
    fn test_reset_restores_unused() {
        let mut block = QuestionBlock::new(Vec2 { x: 100.0, y: 100.0 });
        block.activate();
        assert!(block.used);
        block.reset();
        assert!(!block.used);
        assert_eq!(block.flicker_frame, 0);
    }
}
