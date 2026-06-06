// Dart-throwing enemy — stationary turret that faces the player and throws darts.
// Each dart is spawned externally by PlayingState when the shoot timer elapses.

use crate::level::Vec2;

/// Cooldown between dart throws (seconds).
const SHOOT_COOLDOWN: f32 = 2.0;

/// Range within which the dart enemy will shoot (horizontal distance in px).
const SHOOT_RANGE: f32 = 400.0;

/// A stationary enemy that periodically throws darts toward the player.
///
/// `pos` = foot position (bottom-center), same convention as Enemy.
/// `alive` = whether this enemy is active (false = skip update/render).
/// `facing` = -1 (left) or 1 (right), toward the player's last known X.
#[derive(Debug, Clone)]
pub struct DartEnemy {
    /// World-space foot position (bottom-center of collider).
    pub pos: Vec2,
    /// Whether this enemy is alive. Dead ones are skipped.
    pub alive: bool,
    /// Time remaining until next dart throw (seconds).
    shoot_timer: f32,
    /// Facing direction: -1 = left, 1 = right.
    pub facing: i8,
    /// Whether the player was in range when last checked.
    player_in_range: bool,
}

impl DartEnemy {
    /// Creates a new DartEnemy at the given foot position.
    /// Starts facing right by default.
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            alive: true,
            shoot_timer: 1.0, // initial delay before first shot
            facing: 1,
            player_in_range: false,
        }
    }

    /// Updates the dart enemy: decrements the shoot timer, updates facing
    /// toward the player, and checks whether the player is in range.
    ///
    /// Returns `true` when the enemy wants to shoot (caller should spawn a Dart).
    pub fn update(&mut self, dt: f32, player_x: f32) -> bool {
        if !self.alive {
            return false;
        }

        // Update facing toward player
        self.facing = if player_x > self.pos.x { 1 } else { -1 };

        // Check if player is in range
        let dist = (player_x - self.pos.x).abs();
        self.player_in_range = dist <= SHOOT_RANGE;

        // Only count down if player is in range
        if self.player_in_range {
            self.shoot_timer -= dt;
        }

        if self.shoot_timer <= 0.0 && self.player_in_range {
            self.shoot_timer = SHOOT_COOLDOWN;
            true
        } else {
            false
        }
    }

    /// Immediately kills this enemy.
    pub fn kill(&mut self) {
        self.alive = false;
    }

    /// Returns the dart spawn position (center of the enemy body).
    pub fn spawn_pos(&self) -> Vec2 {
        let offset_x = self.facing as f32 * 12.0;
        Vec2 {
            x: self.pos.x + offset_x,
            y: self.pos.y - 8.0, // chest height
        }
    }

    /// Draws the dart enemy as a gray turret with a red eye.
    pub fn draw(
        &self,
        sx: f32,
        sy: f32,
        ws: &impl Fn(f32, f32) -> (f32, f32),
    ) {
        if !self.alive {
            return;
        }
        use macroquad::shapes::{draw_circle, draw_rectangle};
        use macroquad::color::Color;

        let (sx2, sy2) = ws(self.pos.x, self.pos.y - 16.0);
        let w = 16.0 * sx;
        let h = 16.0 * sy;

        // Body: dark gray square with border
        draw_rectangle(sx2, sy2, w, h, Color::new(0.3, 0.3, 0.35, 1.0));
        // Red glowing eye (facing direction)
        let eye_x = if self.facing > 0 {
            sx2 + w * 0.65
        } else {
            sx2 + w * 0.15
        };
        draw_circle(eye_x, sy2 + h * 0.35, 3.0 * sx.min(sy), Color::new(1.0, 0.15, 0.1, 1.0));
        // Barrel/cannon pointing toward player
        let barrel_len = 10.0 * sx;
        let barrel_y = sy2 + h * 0.55;
        let barrel_start_x = sx2 + w * 0.5;
        let barrel_end_x = barrel_start_x + self.facing as f32 * barrel_len;
        draw_rectangle(
            barrel_start_x.min(barrel_end_x),
            barrel_y - 2.0 * sy,
            barrel_len,
            4.0 * sy,
            Color::new(0.5, 0.5, 0.55, 1.0),
        );
    }
}
