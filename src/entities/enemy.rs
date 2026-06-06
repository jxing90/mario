// Feature #7: Patrol Enemy — Enemy entity and EnemyConfig
// Design Reference: docs/features/7-patrol-enemy.md §4, §6, §8
//
// Enemy walks between two waypoints at constant speed, reverses at endpoints
// and at cliff edges. Subject to gravity — falls when no platform is below.
// Collision detection (stomp vs contact) is handled by Physics::enemy_check.
// Event consumption (alive=false, player bounce, death trigger) is handled by PlayingState.

use crate::level::{AABB, Tile, Vec2};

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

/// A patrol enemy that walks horizontally between two waypoints, subject to
/// gravity and terrain collision. Reverses direction at cliff edges.
///
/// # Coordinate conventions
/// - `pos` is the foot position (bottom-center of the collider), same as Player.
/// - `vel` uses Y-down convention: negative Y = upward.
/// - `collider()` returns a 16x16 AABB anchored at the foot position.
pub struct Enemy {
    /// World-space foot position (bottom-center of 16x16 collider).
    pub pos: Vec2,
    /// Current velocity (px/s).
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
/// Gravity applied every frame (px/s^2).
const GRAVITY: f32 = 900.0;
/// Max fall speed to prevent tunneling.
const MAX_FALL_SPEED: f32 = 600.0;
/// Distance to probe ahead of the enemy for cliff detection.
const EDGE_PROBE_OFFSET: f32 = 10.0;

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
    /// Applies gravity, moves horizontally, resolves terrain collisions,
    /// and reverses direction at waypoints and cliff edges.
    ///
    /// # Postconditions
    /// - If `alive == false` → no-op.
    /// - `pos` updated by velocity and terrain resolution.
    /// - Direction reversed at waypoints or if cliff edge detected ahead.
    pub fn update(&mut self, dt: f32, terrain: &[Tile]) {
        if !self.alive {
            return;
        }

        // 1. Apply gravity (only when terrain is provided)
        if !terrain.is_empty() {
            self.vel.y += GRAVITY * dt;
            if self.vel.y > MAX_FALL_SPEED {
                self.vel.y = MAX_FALL_SPEED;
            }
        }

        // 2. Integrate position
        let new_x = self.pos.x + self.vel.x * dt;
        let new_y = self.pos.y + self.vel.y * dt;

        // 3. Cliff edge detection: probe ahead in movement direction
        let dir = if self.vel.x > 0.0 { 1.0 } else { -1.0 };
        let at_cliff = if terrain.is_empty() {
            false // no terrain = no cliff (legacy patrol behavior)
        } else {
            let probe_x = self.pos.x + dir * EDGE_PROBE_OFFSET;
            let probe_foot = Vec2 { x: probe_x, y: self.pos.y };
            let ahead = Self::has_ground_at(probe_foot, terrain);
            let current_foot = Vec2 { x: self.pos.x, y: self.pos.y };
            let on_platform = Self::has_ground_at(current_foot, terrain);
            on_platform && !ahead // cliff: standing on ground, none ahead
        };

        // 4. Horizontal movement with wall collision
        let mut resolved_x = new_x;
        let col = AABB {
            x: new_x - COLLIDER_HALF_W,
            y: self.pos.y - COLLIDER_H,
            w: 16.0,
            h: COLLIDER_H,
        };
        for tile in terrain {
            if let Tile::Platform(p) = tile {
                if col.intersects(p)
                    // Skip if enemy is standing ON this platform (vertical overlap only)
                    && (self.pos.y - p.y).abs() > 2.0
                {
                    if self.vel.x > 0.0 {
                        resolved_x = p.x - COLLIDER_HALF_W;
                    } else if self.vel.x < 0.0 {
                        resolved_x = p.x + p.w + COLLIDER_HALF_W;
                    }
                    self.vel.x = -self.vel.x;
                    break;
                }
            }
        }
        self.pos.x = resolved_x;

        // 5. Waypoint reversal
        if self.pos.x >= self.waypoint_b.x && self.vel.x > 0.0 {
            self.vel.x = -self.config.speed;
            self.pos.x = self.waypoint_b.x;
        }
        if self.pos.x <= self.waypoint_a.x && self.vel.x < 0.0 {
            self.vel.x = self.config.speed;
            self.pos.x = self.waypoint_a.x;
        }

        // 6. Cliff edge reversal: turn around if standing on platform with no ground ahead
        if at_cliff && self.vel.y >= 0.0 {
            self.vel.x = -self.vel.x;
            self.pos.x -= dir * 4.0;
        }

        // 7. Vertical movement with ground collision
        let col2 = AABB {
            x: self.pos.x - COLLIDER_HALF_W,
            y: new_y - COLLIDER_H,
            w: 16.0,
            h: COLLIDER_H,
        };
        let mut on_ground = false;
        for tile in terrain {
            if let Tile::Platform(p) = tile {
                if col2.intersects(p) {
                    // Landing on top of platform
                    if self.vel.y >= 0.0 {
                        self.pos.y = p.y;
                        self.vel.y = 0.0;
                        on_ground = true;
                        break;
                    } else {
                        // Hitting ceiling from below
                        self.pos.y = p.y + p.h + COLLIDER_H;
                        self.vel.y = 0.0;
                        break;
                    }
                }
            }
        }
        if !on_ground {
            self.pos.y = new_y;
        }
    }

    /// Checks if there is a platform tile directly at or slightly below the given world point.
    /// A point has ground if there's a platform whose top surface is within 4px of the point.
    fn has_ground_at(point: Vec2, terrain: &[Tile]) -> bool {
        for tile in terrain {
            if let Tile::Platform(p) = tile {
                if point.x >= p.x && point.x <= p.x + p.w {
                    if (p.y - point.y).abs() <= 4.0 {
                        return true;
                    }
                }
            }
        }
        false
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
