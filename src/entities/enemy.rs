// Feature #7: Patrol Enemy — Enemy entity and EnemyConfig
// Design Reference: docs/features/7-patrol-enemy.md §4, §6, §8
//
// Enemy walks between two waypoints at constant speed (reverses at endpoints).
// Collision detection (stomp vs contact) is handled by Physics::enemy_check.
// Event consumption (alive=false, player bounce, death trigger) is handled by PlayingState.

use crate::level::{AABB, Vec2};

// ============================================================================
// EnemyConfig
// ============================================================================

/// Configuration for a patrol enemy: patrol speed and stomp bounce velocity.
#[derive(Debug, Clone, Copy)]
pub struct EnemyConfig {
    /// Horizontal patrol speed in pixels per second. Default: 50.0.
    pub speed: f32,
    /// Upward velocity applied to the player on stomp (negative = upward in Y-down). Default: -200.0.
    pub bounce_velocity: f32,
}

impl Default for EnemyConfig {
    /// Returns `EnemyConfig { speed: 50.0, bounce_velocity: -200.0 }`.
    fn default() -> Self {
        Self {
            speed: 50.0,
            bounce_velocity: -200.0,
        }
    }
}

// ============================================================================
// Enemy
// ============================================================================

/// A patrol enemy that walks horizontally between two waypoints.
///
/// Does not interact with terrain (no gravity, no platform collision).
/// Collision with the player is handled by `Physics::enemy_check`.
///
/// # Coordinate conventions
/// - `pos` is the foot position (bottom-center of the collider), same as Player.
/// - `vel` uses Y-down convention: negative Y = upward.
/// - `collider()` returns a 16x16 AABB anchored at the foot position.
pub struct Enemy {
    /// World-space foot position (bottom-center of 16x16 collider).
    pub pos: Vec2,
    /// Current velocity (px/s). Y is always 0 (no vertical movement).
    pub vel: Vec2,
    /// Whether this enemy is still alive. Dead enemies are skipped by collision detection.
    pub alive: bool,
    /// Left patrol waypoint (usually the lower X coordinate).
    pub waypoint_a: Vec2,
    /// Right patrol waypoint (usually the higher X coordinate).
    pub waypoint_b: Vec2,
    /// Patrol and stomp configuration.
    pub config: EnemyConfig,
}

/// Half-width of the enemy's 16x16 collider box (centered on foot position).
const COLLIDER_HALF_W: f32 = 8.0;
/// Height of the enemy's 16x16 collider box (extends upward from foot position).
const COLLIDER_H: f32 = 16.0;

impl Enemy {
    /// Creates a new Enemy at the given foot position, patrolling between waypoints.
    ///
    /// # Postconditions (from §4 Interface Contract)
    /// - `alive = true`
    /// - `vel.x = config.speed` (initial direction: right)
    /// - `vel.y = 0.0`
    pub fn new(pos: Vec2, waypoint_a: Vec2, waypoint_b: Vec2, config: EnemyConfig) -> Self {
        Self {
            pos,
            vel: Vec2 {
                x: config.speed,
                y: 0.0,
            },
            alive: true,
            waypoint_a,
            waypoint_b,
            config,
        }
    }

    /// Advances the enemy patrol by one fixed timestep.
    ///
    /// # Postconditions (from §4 Interface Contract)
    /// - If `alive == false` → no-op.
    /// - `pos.x += vel.x * dt`
    /// - If `pos.x >= waypoint_b.x` AND `vel.x > 0.0`:
    ///   `vel.x = -config.speed`, `pos.x = waypoint_b.x` (clamped)
    /// - If `pos.x <= waypoint_a.x` AND `vel.x < 0.0`:
    ///   `vel.x = config.speed`, `pos.x = waypoint_a.x` (clamped)
    /// - Y coordinate never changes.
    pub fn update(&mut self, dt: f32) {
        if !self.alive {
            return;
        }

        self.pos.x += self.vel.x * dt;

        // Check right waypoint boundary (moving right)
        if self.pos.x >= self.waypoint_b.x && self.vel.x > 0.0 {
            self.vel.x = -self.config.speed;
            self.pos.x = self.waypoint_b.x;
        }

        // Check left waypoint boundary (moving left)
        if self.pos.x <= self.waypoint_a.x && self.vel.x < 0.0 {
            self.vel.x = self.config.speed;
            self.pos.x = self.waypoint_a.x;
        }
    }

    /// Returns the enemy's 16x16 collision box, anchored at the foot position.
    ///
    /// AABB origin is at top-left:
    /// - `x = pos.x - COLLIDER_HALF_W` (horizontal center)
    /// - `y = pos.y - COLLIDER_H` (top edge, foot at bottom)
    /// - `w = 16.0`, `h = 16.0`
    pub fn collider(&self) -> AABB {
        AABB {
            x: self.pos.x - COLLIDER_HALF_W,
            y: self.pos.y - COLLIDER_H,
            w: 16.0,
            h: 16.0,
        }
    }

    /// Returns the enemy's current foot position (immutable borrow).
    pub fn pos(&self) -> Vec2 {
        self.pos
    }

    /// Immediately kills this enemy.
    pub fn kill(&mut self) {
        self.alive = false;
    }

    /// Draws the enemy as a brown rectangle with eyes and feet.
    /// Only draws if alive.
    pub fn draw(
        &self,
        sx: f32,
        sy: f32,
        ws: &impl Fn(f32, f32) -> (f32, f32),
    ) {
        if !self.alive {
            return;
        }
        let ep = self.pos();
        let (sex, sey) = ws(ep.x, ep.y - 16.0);
        macroquad::shapes::draw_rectangle(sex, sey, 16.0 * sx, 16.0 * sy, macroquad::color::BROWN);
        // Eyes
        macroquad::shapes::draw_circle(sex + 4.0 * sx, sey + 3.0 * sy, 2.0 * sx.min(sy), macroquad::color::WHITE);
        macroquad::shapes::draw_circle(sex + 12.0 * sx, sey + 3.0 * sy, 2.0 * sx.min(sy), macroquad::color::WHITE);
        // Feet
        macroquad::shapes::draw_rectangle(sex + 2.0 * sx, sey + 12.0 * sy, 5.0 * sx, 4.0 * sy, macroquad::color::BLACK);
        macroquad::shapes::draw_rectangle(sex + 9.0 * sx, sey + 12.0 * sy, 5.0 * sx, 4.0 * sy, macroquad::color::BLACK);
    }
}
