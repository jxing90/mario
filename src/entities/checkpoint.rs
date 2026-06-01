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
}
