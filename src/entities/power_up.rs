// Feature #8: Collectibles & Blocks — PowerUp entity and PowerUpKind enum
// Design Reference: docs/features/8-collectibles-blocks.md §4, §6, §8

use crate::level::{AABB, Tile, Vec2};

/// Gravity constant for PowerUp entities (px/s^2).
/// Uses a higher value than Player gravity to ensure timely fall-off from
/// platforms and visible bounce arc within typical frame budgets.
const GRAVITY: f32 = 1500.0;

/// Base horizontal speed for SuperMushroom bounce movement (px/s).
const BOUNCE_SPEED: f32 = 80.0;

/// Initial upward velocity for SuperMushroom spawn burst (px/s, negative = upward in Y-down).
const JUMP_VELOCITY: f32 = -200.0;

/// Bounce velocity applied when SuperMushroom lands on a platform (px/s, negative = upward).
const BOUNCE_UP_VELOCITY: f32 = -200.0;

/// Half of the PowerUp collider side length (8.0 = 16.0 / 2), used for position
/// correction during terrain collision resolution.
const HALF_SIZE: f32 = 8.0;

/// The type of reward produced by a QuestionBlock or collected as a PowerUp.
///
/// §8 Data Model: Coin (increment counter), SuperMushroom (grow player, one-hit absorption),
/// FireFlower (grant fireball ability).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerUpKind {
    /// Coin reward — increments player.coins, no PowerUp entity spawned.
    Coin,
    /// Super Mushroom — bounces along terrain; player grows to Super size on contact.
    SuperMushroom,
    /// Fire Flower — stationary; player gains Fire state and fireball ability on contact.
    FireFlower,
    /// Starman — bounces like a mushroom; grants temporary invincibility on contact.
    Starman,
}

/// A spawned PowerUp entity in the world.
///
/// §8 Data Model: `kind` = reward type, `pos` = world-space center position,
/// `vel` = current velocity (px/s, Y-down).
#[derive(Debug, Clone, Copy)]
pub struct PowerUp {
    /// The type of power-up reward.
    pub kind: PowerUpKind,
    /// World-space center position.
    pub pos: Vec2,
    /// Current velocity (px/s, Y-down).
    pub vel: Vec2,
}

impl PowerUp {
    /// Creates a new PowerUp at the given spawn position.
    ///
    /// # Preconditions (§4)
    /// - `spawn_pos` is within level bounds
    ///
    /// # Postconditions (§4)
    /// - For SuperMushroom: `vel = (BOUNCE_SPEED, JUMP_VELOCITY)` = (80.0, -200.0)
    /// - For FireFlower: `vel = (0.0, 0.0)` (stationary)
    pub fn new(kind: PowerUpKind, spawn_pos: Vec2) -> Self {
        let vel = match kind {
            PowerUpKind::SuperMushroom | PowerUpKind::Starman => Vec2 {
                x: BOUNCE_SPEED,
                y: JUMP_VELOCITY,
            },
            PowerUpKind::FireFlower => Vec2 { x: 0.0, y: 0.0 },
            PowerUpKind::Coin => Vec2 { x: 0.0, y: 0.0 },
        };
        Self {
            kind,
            pos: spawn_pos,
            vel,
        }
    }

    /// Advances the PowerUp simulation by one fixed timestep.
    ///
    /// # Preconditions (§4)
    /// - `dt = 1.0/60.0`
    ///
    /// # Behavior
    /// - Applies gravity (vel.y += GRAVITY * dt)
    /// - Integrates position from velocity
    /// - Resolves terrain collisions with platforms (floor, wall, ceiling)
    /// - SuperMushroom: horizontal bounce on wall contact, vertical bounce on landing
    /// - FireFlower: terrain collision still applied (for resting on platforms)
    pub fn update(&mut self, dt: f32, terrain: &[Tile]) {
        // Apply gravity
        self.vel.y += GRAVITY * dt;

        // Integrate position
        self.pos.x += self.vel.x * dt;
        self.pos.y += self.vel.y * dt;

        // Resolve terrain collisions
        self.resolve_terrain_collision(terrain);
    }

    /// Resolves terrain collisions against platform tiles.
    ///
    /// Handles floor landing (with bounce), ceiling push-out, and wall direction
    /// reversal. Uses minimum-overlap disambiguation to select the primary
    /// collision axis and resolves velocity + position accordingly.
    fn resolve_terrain_collision(&mut self, terrain: &[Tile]) {
        let col = self.collider();
        for tile in terrain {
            let platform = match tile {
                Tile::Platform(p) => p,
                _ => continue,
            };
            if !col.intersects(platform) {
                continue;
            }

            // Compute overlap amounts on all four sides
            let overlap_left = (col.x + col.w) - platform.x;
            let overlap_right = (platform.x + platform.w) - col.x;
            let overlap_top = (col.y + col.h) - platform.y;
            let overlap_bottom = (platform.y + platform.h) - col.y;

            let min_overlap = overlap_left
                .min(overlap_right)
                .min(overlap_top)
                .min(overlap_bottom);

            // Floor collision: landing on platform top → bounce
            if min_overlap == overlap_top && self.vel.y >= 0.0 {
                self.vel.y = BOUNCE_UP_VELOCITY;
                self.pos.y = platform.y;
            }
            // Ceiling collision: hitting platform from below → stop
            else if min_overlap == overlap_bottom && self.vel.y < 0.0 {
                self.vel.y = 0.0;
                self.pos.y = platform.y + platform.h + HALF_SIZE;
            }
            // Wall collision from left → reverse horizontal
            else if min_overlap == overlap_left && self.vel.x > 0.0 {
                self.vel.x = -self.vel.x.abs();
                self.pos.x = platform.x - HALF_SIZE;
            }
            // Wall collision from right → reverse horizontal
            else if min_overlap == overlap_right && self.vel.x < 0.0 {
                self.vel.x = self.vel.x.abs();
                self.pos.x = platform.x + platform.w + HALF_SIZE;
            }
            // Fallback: push upward if overlapping without directional resolution
            else if min_overlap == overlap_top {
                self.pos.y = platform.y;
            }
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

    /// Draws the power-up at its world position.
    /// - SuperMushroom: green cap with white spots.
    /// - FireFlower: orange circle with yellow center.
    /// - Starman: yellow circle with black star.
    /// - Coin: not drawn (handled separately).
    pub fn draw(
        &self,
        sx: f32,
        sy: f32,
        ws: &impl Fn(f32, f32) -> (f32, f32),
    ) {
        let (px, py) = ws(self.pos.x, self.pos.y);
        let half = 8.0 * sx.min(sy);
        match self.kind {
            PowerUpKind::SuperMushroom => {
                macroquad::shapes::draw_rectangle(px - half, py - half, half * 2.0, half * 2.0, macroquad::color::GREEN);
                macroquad::shapes::draw_circle(px - 3.0 * sx, py - 3.0 * sy, 2.0 * sx.min(sy), macroquad::color::WHITE);
                macroquad::shapes::draw_circle(px + 3.0 * sx, py + 3.0 * sy, 2.0 * sx.min(sy), macroquad::color::WHITE);
            }
            PowerUpKind::FireFlower => {
                macroquad::shapes::draw_circle(px, py, half, macroquad::color::Color::new(1.0, 0.4, 0.0, 1.0));
                macroquad::shapes::draw_circle(px, py, half * 0.5, macroquad::color::YELLOW);
            }
            PowerUpKind::Starman => {
                macroquad::shapes::draw_circle(px, py, half, macroquad::color::YELLOW);
                macroquad::text::draw_text("*", px - half * 0.5, py + half * 0.5, half * 1.6, macroquad::color::BLACK);
            }
            PowerUpKind::Coin => {}
        }
    }
}
