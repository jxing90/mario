// Feature #6: Life, Death & Win — Flagpole entity
// Design Reference: docs/features/6-life-death-win.md §4, §8
//
// Flagpole marks the end of the level. When the player's collider overlaps
// the flagpole trigger zone, the slide animation plays, followed by the
// Victory state transition.

use crate::level::{AABB, Vec2};

/// Animation phase of the flagpole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagpolePhase {
    /// Waiting for player contact.
    Idle,
    /// Player is sliding down the pole; input is locked.
    Sliding,
    /// Slide animation complete; ready to transition to Victory.
    Done,
}

/// The flagpole entity at the end of the level.
///
/// # Fields (§8 Data Model)
/// - `pos`: world-space center position of the flagpole.
/// - `phase`: current animation phase.
/// - `slide_progress`: 0.0 → 1.0, animated during `Sliding` phase.
pub struct Flagpole {
    pub pos: Vec2,
    pub phase: FlagpolePhase,
    pub slide_progress: f32,
}

impl Flagpole {
    /// Creates a new Flagpole at the given world position.
    ///
    /// # Postconditions (§4)
    /// - `phase = FlagpolePhase::Idle`
    /// - `slide_progress = 0.0`
    /// - Trigger zone AABB is 16×80 px centered at `pos`.
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            phase: FlagpolePhase::Idle,
            slide_progress: 0.0,
        }
    }

    /// Returns the trigger-zone AABB for flagpole collision detection.
    ///
    /// Dimensions: w=16, h=80, centered on `pos`.
    pub fn collider(&self) -> AABB {
        AABB {
            x: self.pos.x - 8.0,
            y: self.pos.y - 40.0,
            w: 16.0,
            h: 80.0,
        }
    }

    /// Advances the slide animation by `dt` seconds.
    ///
    /// Only active when `phase == Sliding`. Slide duration is 1.0s.
    /// On completion, `slide_progress` is clamped to 1.0 and `phase` becomes `Done`.
    pub fn update(&mut self, dt: f32) {
        if self.phase == FlagpolePhase::Sliding {
            self.slide_progress += dt / 1.0; // 1.0s slide duration
            if self.slide_progress >= 1.0 {
                self.slide_progress = 1.0;
                self.phase = FlagpolePhase::Done;
            }
        }
    }
}
