// Feature #8: Collectibles & Blocks — Fireball projectile entity
// Design Reference: docs/features/8-collectibles-blocks.md §4, §6, §8

use crate::level::{AABB, Vec2};

/// Horizontal speed of a fireball in pixels per second.
const FIREBALL_SPEED: f32 = 200.0;

/// Maximum lifetime of a fireball in seconds before it auto-destructs.
const MAX_LIFETIME: f32 = 2.0;

/// A fireball projectile fired by the player in Fire state.
///
/// §8 Data Model: `pos` = world-space center position, `vel` = velocity (px/s),
/// `alive` = whether the fireball is active, `timer` = elapsed lifetime in seconds.
#[derive(Debug, Clone, Copy)]
pub struct Fireball {
    /// World-space center position.
    pub pos: Vec2,
    /// Current velocity (px/s, Y-down). Horizontal only — vel.y is always 0.
    pub vel: Vec2,
    /// Whether the fireball is active (false = pending removal).
    pub alive: bool,
    /// Elapsed lifetime in seconds since creation.
    pub timer: f32,
}

impl Fireball {
    /// Creates a new fireball at the given position, traveling in the facing direction.
    ///
    /// # Preconditions (§4)
    /// - `facing` is 1 (right) or -1 (left)
    ///
    /// # Postconditions (§4)
    /// - `vel.x = facing * FIREBALL_SPEED` (200.0 px/s)
    /// - `vel.y = 0.0` (horizontal flight only)
    /// - `alive = true`
    /// - `timer = 0.0`
    pub fn new(pos: Vec2, facing: i8) -> Self {
        Self {
            pos,
            vel: Vec2 {
                x: facing as f32 * FIREBALL_SPEED,
                y: 0.0,
            },
            alive: true,
            timer: 0.0,
        }
    }

    /// Advances the fireball simulation by one timestep.
    ///
    /// # Preconditions (§4)
    /// - `alive == true`
    ///
    /// # Postconditions (§4)
    /// - `timer += dt`
    /// - If `timer >= MAX_LIFETIME` → `alive = false`
    ///
    /// # Boundary condition (§Implementation Summary)
    /// - `timer == MAX_LIFETIME` (exact) → `alive = false`
    pub fn update(&mut self, dt: f32) {
        self.timer += dt;
        if self.timer >= MAX_LIFETIME {
            self.alive = false;
        }
    }

    /// Returns an 8x8 AABB centered at `self.pos`.
    ///
    /// # Postconditions (§4)
    /// - AABB has width 8.0 and height 8.0
    /// - Top-left corner is `(pos.x - 4.0, pos.y - 4.0)`
    pub fn collider(&self) -> AABB {
        AABB {
            x: self.pos.x - 4.0,
            y: self.pos.y - 4.0,
            w: 8.0,
            h: 8.0,
        }
    }

    /// Immediately destroys the fireball.
    ///
    /// # Postconditions (§4)
    /// - `alive = false`
    pub fn kill(&mut self) {
        self.alive = false;
    }
}
