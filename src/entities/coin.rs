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
}
