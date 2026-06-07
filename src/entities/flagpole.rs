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
    /// Matches the visual: pole 120px tall above pos.y, plus 8px base below,
    /// and wide enough to cover the flag extending right of the pole.
    pub fn collider(&self) -> AABB {
        AABB {
            x: self.pos.x - 8.0,
            y: self.pos.y - 120.0,
            w: 40.0,   // pole (6) + flag (24) + margin
            h: 128.0,  // pole (120) + base (8)
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

    /// Draws the flagpole with pole, green flag, star, and "GOAL" label.
    pub fn draw(
        &self,
        sx: f32,
        sy: f32,
        ws: &impl Fn(f32, f32) -> (f32, f32),
    ) {
        let (fpx, fpy) = ws(self.pos.x, self.pos.y);
        let pole_w = 6.0 * sx;
        let pole_h = 120.0 * sy;
        // Pole shadow
        macroquad::shapes::draw_rectangle(fpx + 2.0 * sx, fpy - pole_h + 2.0 * sy, pole_w, pole_h, macroquad::color::DARKGRAY);
        // Main pole
        macroquad::shapes::draw_rectangle(fpx, fpy - pole_h, pole_w, pole_h, macroquad::color::GRAY);
        // Green flag
        let flag_w = 24.0 * sx;
        let flag_h = 18.0 * sy;
        macroquad::shapes::draw_rectangle(fpx + pole_w, fpy - pole_h, flag_w, flag_h, macroquad::color::GREEN);
        // Star on flag
        macroquad::text::draw_text("*", fpx + pole_w + 6.0 * sx, fpy - pole_h + 14.0 * sy, 16.0 * sx.min(sy), macroquad::color::YELLOW);
        // "GOAL" label
        macroquad::text::draw_text("GOAL", fpx - 8.0 * sx, fpy - pole_h - 20.0 * sy, 20.0 * sx.min(sy), macroquad::color::GOLD);
        // Ground base
        macroquad::shapes::draw_rectangle(fpx - 8.0 * sx, fpy, pole_w + 16.0 * sx, 8.0 * sy, macroquad::color::DARKGRAY);
    }
}
