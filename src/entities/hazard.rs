// Feature #5: Hazards — Spike entity
// Design Reference: docs/features/5-hazards.md §4, §6, §8
//
// Spike is a static danger collider (16x8 px) placed in the level.
// Position is stored as a Vec2; the collider is computed as a fixed-size
// AABB centered at the position.

use crate::level::{AABB, Vec2};

/// A static spike hazard placed in the level.
///
/// Collider is fixed at 16x8 pixels, centered on the spike's world position.
/// Spike entities are immutable after construction — they never move or change size.
pub struct Spike {
    pub pos: Vec2,
}

impl Spike {
    /// Creates a new Spike at the given world position.
    ///
    /// # Preconditions
    /// - `pos.x >= 0.0` (within level left bound)
    /// - `pos.y >= 0.0` (within level top bound)
    ///
    /// # Postconditions
    /// - Returns a Spike instance with position = `pos`
    /// - Collider is fixed at AABB{w:16, h:8} centered at `pos`
    pub fn new(pos: Vec2) -> Self {
        Self { pos }
    }

    /// Returns the AABB collision box for this spike.
    ///
    /// The AABB is 16 pixels wide and 8 pixels tall, centered at the spike's position:
    /// - `x = pos.x - 8.0` (left edge)
    /// - `y = pos.y - 4.0` (top edge)
    /// - `w = 16.0`
    /// - `h = 8.0`
    pub fn collider(&self) -> AABB {
        AABB {
            x: self.pos.x - 8.0,
            y: self.pos.y - 4.0,
            w: 16.0,
            h: 8.0,
        }
    }

    /// Returns the spike's current world position (Copy).
    /// Pure getter — no side effects.
    pub fn pos(&self) -> Vec2 {
        self.pos
    }
}
