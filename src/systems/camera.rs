// Feature #4: Camera System — camera following, dead zone, level clamping
// Design Reference: docs/features/4-camera-system.md §4, §6, §8

use crate::level::{Vec2, LevelBounds};

/// Configuration parameters for the camera follow system.
/// Default values match SRS FR-013 specifications.
#[derive(Debug, Clone, Copy)]
pub struct CameraConfig {
    /// Horizontal convergence rate (fraction of remaining distance per frame). Default: 0.08.
    pub h_convergence: f32,
    /// Vertical convergence rate (fraction of remaining distance per frame). Default: 0.05.
    pub v_convergence: f32,
    /// Fraction of viewport height covered by the vertical dead zone. Default: 0.60.
    pub dead_zone_pct: f32,
    /// Viewport width in pixels. Default: 480.0.
    pub viewport_w: f32,
    /// Viewport height in pixels. Default: 270.0.
    pub viewport_h: f32,
    /// Target horizontal position of the player within the viewport (fraction from left). Default: 0.375.
    pub player_target_x_pct: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            h_convergence: 0.08,
            v_convergence: 0.05,
            dead_zone_pct: 0.60,
            viewport_w: 480.0,
            viewport_h: 270.0,
            player_target_x_pct: 0.375,
        }
    }
}

/// Tracks the camera viewport offset in world coordinates.
/// Follows the player with horizontal lerp convergence (8%/frame) and
/// vertical dead-zone behavior (5%/frame when player exits central 60%).
#[derive(Debug, Clone)]
pub struct Camera {
    offset: Vec2,
    config: CameraConfig,
}

impl Camera {
    /// Creates a new Camera with zero offset and the given configuration.
    ///
    /// Postcondition: `self.offset() == Vec2 { x: 0.0, y: 0.0 }`.
    pub fn new(config: CameraConfig) -> Self {
        Self {
            offset: Vec2 { x: 0.0, y: 0.0 },
            config,
        }
    }

    /// Updates the camera offset for one frame.
    ///
    /// # Algorithm (per design §6 flowchart)
    /// 1. Guard: dt <= 0.0 is a no-op (defensive).
    /// 2. Horizontal: target_x = player_pos.x - viewport_w * player_target_x_pct;
    ///    offset.x += (target_x - offset.x) * h_convergence.
    /// 3. Vertical dead zone: if player_pos.y is outside [dead_top, dead_bottom],
    ///    offset.y += (target_y - offset.y) * v_convergence (target_y = player_pos.y - viewport_h/2).
    /// 4. Clamp offset.x to [bounds.min_x, max(bounds.min_x, bounds.max_x - viewport_w)].
    pub fn update(&mut self, player_pos: Vec2, bounds: LevelBounds, dt: f32) {
        // Guard: dt <= 0.0 is a no-op (prevents NaN/divergence)
        if dt <= 0.0 {
            return;
        }

        // Horizontal lerp convergence
        let target_x = player_pos.x - self.config.viewport_w * self.config.player_target_x_pct;
        self.offset.x += (target_x - self.offset.x) * self.config.h_convergence;

        // Vertical dead zone: central dead_zone_pct of viewport
        let dead_top = self.offset.y + self.config.viewport_h * (1.0 - self.config.dead_zone_pct) / 2.0;
        let dead_bottom = self.offset.y + self.config.viewport_h * (1.0 + self.config.dead_zone_pct) / 2.0;

        if player_pos.y < dead_top || player_pos.y > dead_bottom {
            let target_y = player_pos.y - self.config.viewport_h / 2.0;
            self.offset.y += (target_y - self.offset.y) * self.config.v_convergence;
        }

        // Clamp horizontal offset to level bounds
        // max_offset_x floors at bounds.min_x for narrow/degenerate levels
        let max_offset_x = (bounds.max_x - self.config.viewport_w).max(bounds.min_x);
        self.offset.x = self.offset.x.max(bounds.min_x).min(max_offset_x);
    }

    /// Returns the current camera offset (viewport top-left in world coordinates).
    ///
    /// Pure getter — no side effects. Output schema matches IAPI-010: Vec2 { x, y }.
    pub fn offset(&self) -> Vec2 {
        self.offset
    }
}
