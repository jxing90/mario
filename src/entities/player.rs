// Feature #3: Player Controller — Player entity, physics config, power-up state machine
// Design Reference: docs/features/3-player-controller.md

use crate::level::{AABB, Tile, Vec2};
use crate::input::InputState;

// ============================================================================
// PlayerConfig
// ============================================================================

/// Physics tuning parameters for the player character.
/// All parameters are `pub` to satisfy the SRS "configurable" requirement.
/// Default values approximate original Super Mario Bros. hand-feel at 60fps.
#[derive(Debug, Clone, Copy)]
pub struct PlayerConfig {
    /// Horizontal acceleration on the ground (px/s^2). Default: 667.0 (~0.3s to max speed).
    pub acceleration: f32,
    /// Maximum horizontal speed on the ground (px/s). Default: 200.0.
    pub max_speed: f32,
    /// Horizontal deceleration when no input (px/s^2). Default: 1000.0 (~0.2s to stop).
    pub friction: f32,
    /// Initial upward velocity on jump (px/s, negative = upward in Macroquad Y-down coords).
    /// Default: -420.0.
    pub jump_initial_velocity: f32,
    /// Maximum time the jump sustain force is applied while Space is held (seconds).
    /// Default: 0.35.
    pub max_jump_duration: f32,
    /// Multiplier applied to max_speed while sprinting (Shift held). Default: 1.5.
    pub sprint_multiplier: f32,
    /// Fraction of ground acceleration available while airborne. Default: 0.6.
    pub air_control_factor: f32,
    /// Downward acceleration due to gravity (px/s^2). Default: 900.0.
    pub gravity: f32,
    /// Maximum downward speed (px/s). Prevents tunneling through thin platforms.
    /// Default: 600.0.
    pub max_fall_speed: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            acceleration: 200.0 / 0.3, // ≈ 666.67 → reach max_speed in 0.3s
            max_speed: 200.0,
            friction: 200.0 / 0.2, // 1000.0 → stop from max_speed in 0.2s
            jump_initial_velocity: -420.0, // strong impulse (~98px short tap)
            max_jump_duration: 0.35,       // hold window for variable-height jump
            sprint_multiplier: 1.5,
            air_control_factor: 0.6,
            gravity: 900.0,
            max_fall_speed: 600.0,
        }
    }
}

// ============================================================================
// PlayerState — power-up state machine
// ============================================================================

/// The player's current power-up tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Small,
    Super,
    Fire,
}

// ============================================================================
// PlayerStats — HUD data contract (IAPI-009)
// ============================================================================

/// Read-only snapshot of player statistics for HUD display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerStats {
    pub coins: u32,
    pub lives: u32,
}

// ============================================================================
// Player
// ============================================================================

/// The player entity: position, velocity, state, and physics config.
///
/// Created once by the Playing state at game start, then updated every simulation
/// step via `Player::update(dt, input, terrain)`.
///
/// # Coordinate conventions
/// - `pos` is the foot position (bottom-center of the character collider).
/// - `vel` uses Macroquad's Y-down convention: negative Y = upward.
/// - `collider()` returns an AABB with top-left origin.
pub struct Player {
    /// World-space foot position (bottom-center of collider).
    pub pos: Vec2,
    /// Current velocity (px/s). Y-down: negative = upward.
    pub vel: Vec2,
    /// Whether the player is standing on a platform surface.
    pub on_ground: bool,
    /// Facing direction: 1 = right, -1 = left.
    pub facing: i8,
    /// Current power-up tier.
    pub state: PlayerState,
    /// Physics tuning parameters (stored by value).
    pub config: PlayerConfig,
    /// Elapsed time since jump initiation (seconds). Reset to 0 on landing.
    pub jump_timer: f32,
    /// Whether the jump button is still being held for variable-height jump.
    pub jump_held: bool,
    /// Coin count (modified externally by Feature #8 Collectibles).
    pub coins: u32,
    /// Remaining lives (modified externally by Feature #6 Life/Death).
    pub lives: u32,
    /// Previous-frame left key state for dual-key debounce (last-pressed priority).
    prev_left: bool,
    /// Previous-frame right key state for dual-key debounce (last-pressed priority).
    prev_right: bool,
    /// Persistent direction when both keys are held (0.0, -1.0, or 1.0).
    dual_dir: f32,
    /// Tracks reversal acceleration phase (2x accel until target speed reached).
    reversing: bool,
}

impl Player {
    /// Creates a new Player with the given physics configuration.
    ///
    /// Postconditions:
    /// - `pos = (100.0, 100.0)` (near level start)
    /// - `vel = (0.0, 0.0)`, `on_ground = false`
    /// - `facing = 1` (facing right)
    /// - `state = PlayerState::Small`
    /// - `coins = 0`, `lives = 3`
    /// - `jump_timer = 0.0`, `jump_held = false`
    pub fn new(config: PlayerConfig) -> Self {
        Self {
            pos: Vec2 { x: 100.0, y: 100.0 },
            vel: Vec2 { x: 0.0, y: 0.0 },
            on_ground: false,
            facing: 1,
            state: PlayerState::Small,
            config,
            jump_timer: 0.0,
            jump_held: false,
            coins: 0,
            lives: 3,
            prev_left: false,
            prev_right: false,
            dual_dir: 0.0,
            reversing: false,
        }
    }

    /// Advances the player simulation by one fixed timestep.
    ///
    /// # Preconditions
    /// - `dt = 1.0/60.0` (fixed timestep)
    /// - `input` is the current frame's keyboard snapshot
    /// - `terrain` comes from `Level::query_terrain(player.collider())`
    ///
    /// Internally delegates to:
    /// 1. `apply_horizontal(dt, input)` — acceleration/friction/sprint/air control
    /// 2. `apply_jump(dt, input)` — jump initiation/sustain/ceiling collision
    /// 3. `apply_gravity(dt)` — free-fall acceleration
    /// 4. `resolve_terrain_collision(dt, terrain)` — AABB collision with platforms
    /// 5. `update_facing(input)` — facing direction
    pub fn update(&mut self, dt: f32, input: &InputState, terrain: &[Tile]) {
        // Early-return guard: dt=0 or negative is a no-op (no panic, no NaN)
        if dt <= 0.0 {
            return;
        }

        self.apply_horizontal(dt, input);
        self.apply_jump(dt, input);
        self.apply_gravity(dt);
        self.resolve_terrain_collision(dt, terrain);
        self.update_facing(input);
    }

    // ------------------------------------------------------------------
    // apply_horizontal — acceleration, friction, sprint, air control
    // ------------------------------------------------------------------
    fn apply_horizontal(&mut self, dt: f32, input: &InputState) {
        // Determine input direction
        let dir = Self::input_direction(input, &mut self.prev_left, &mut self.prev_right, &mut self.dual_dir);

        // Effective max speed (sprint modifies cap, not acceleration)
        let sprint_factor = if input.sprint {
            self.config.sprint_multiplier
        } else {
            1.0
        };
        let effective_max = self.config.max_speed * sprint_factor;

        // Snap to non-sprint max speed when sprint is released (FR-003 AC-3)
        if !input.sprint && self.vel.x.abs() > self.config.max_speed {
            self.vel.x = self.config.max_speed * self.vel.x.signum();
        }

        // Air control reduces acceleration, not max speed
        let air_factor = if self.on_ground {
            1.0
        } else {
            self.config.air_control_factor
        };

        if dir != 0.0 {
            let accel = self.config.acceleration * air_factor;

            // Enter reversal phase if input opposes current velocity
            if !self.reversing && self.vel.x * dir < 0.0 {
                self.reversing = true;
            }
            // Exit reversal if velocity reaches effective_max in input direction
            if self.reversing {
                let reached = (dir > 0.0 && self.vel.x >= effective_max)
                    || (dir < 0.0 && self.vel.x <= -effective_max);
                if reached {
                    self.reversing = false;
                }
            }

            let accel_mult = if self.reversing { 2.0 } else { 1.0 };
            self.vel.x += accel * dt * dir * accel_mult;

            // Clamp to effective max speed
            if self.vel.x.abs() > effective_max {
                self.vel.x = effective_max * self.vel.x.signum();
            }
        } else {
            // No directional input — exit reversal, apply linear friction
            self.reversing = false;
            let friction_amount = self.config.friction * dt;
            if self.vel.x.abs() <= friction_amount {
                // Snap to zero to avoid crossing zero and oscillating
                self.vel.x = 0.0;
            } else {
                self.vel.x -= friction_amount * self.vel.x.signum();
            }
        }
    }

    /// Resolves directional input, including dual-key debounce.
    /// Returns -1.0 (left), 1.0 (right), or 0.0 (neutral).
    fn input_direction(
        input: &InputState,
        prev_left: &mut bool,
        prev_right: &mut bool,
        dual_dir: &mut f32,
    ) -> f32 {
        if input.left && !input.right {
            *prev_left = true;
            *prev_right = false;
            *dual_dir = 0.0;
            -1.0
        } else if input.right && !input.left {
            *prev_left = false;
            *prev_right = true;
            *dual_dir = 0.0;
            1.0
        } else if input.left && input.right {
            // Both keys held — last-pressed priority (FR-001 AC-4)
            let left_just = !*prev_left;
            let right_just = !*prev_right;

            *prev_left = true;
            *prev_right = true;

            let dir = if left_just && !right_just {
                -1.0 // left just pressed, right was already held
            } else if right_just && !left_just {
                1.0 // right just pressed, left was already held
            } else if left_just && right_just {
                // Both just pressed simultaneously — left priority (deterministic tie-breaker)
                -1.0
            } else {
                // Both held from previous frame — persist last dual-key direction
                *dual_dir
            };
            *dual_dir = dir;
            dir
        } else {
            *prev_left = false;
            *prev_right = false;
            *dual_dir = 0.0;
            0.0
        }
    }

    // ------------------------------------------------------------------
    // apply_jump — initiation, sustain, and ceiling collision guard
    // ------------------------------------------------------------------
    fn apply_jump(&mut self, dt: f32, input: &InputState) {
        // Jump initiation: edge-triggered, only from ground
        if input.jump_just && self.on_ground {
            self.vel.y = self.config.jump_initial_velocity; // -420 = upward in Y-down
            self.jump_timer = 0.0;
            self.jump_held = true;
            self.on_ground = false;
        }

        // Jump sustain: hold Space within max_jump_duration.
        // Counteracts a fraction of gravity so holding gives noticeably more height.
        if self.jump_held && input.jump && self.jump_timer < self.config.max_jump_duration {
            let sustain = self.config.gravity * 0.72;
            self.vel.y -= sustain * dt; // push upward (more negative Y)
            self.jump_timer += dt;
            if self.jump_timer >= self.config.max_jump_duration {
                self.jump_held = false;
            }
        } else {
            // Release jump or exceed max duration → stop sustain
            self.jump_held = false;
        }
    }

    // ------------------------------------------------------------------
    // apply_gravity — free-fall acceleration when airborne
    // ------------------------------------------------------------------
    fn apply_gravity(&mut self, dt: f32) {
        self.vel.y += self.config.gravity * dt;
        // Cap fall speed to prevent tunneling through thin platforms
        if self.vel.y > self.config.max_fall_speed {
            self.vel.y = self.config.max_fall_speed;
        }
    }

    // ------------------------------------------------------------------
    // resolve_terrain_collision — AABB collision detection and response
    // ------------------------------------------------------------------
    fn resolve_terrain_collision(&mut self, dt: f32, terrain: &[Tile]) {
        // Integrate position from current velocity
        self.pos.x += self.vel.x * dt;
        self.pos.y += self.vel.y * dt;

        // Reset on_ground — will be set true by floor collision
        self.on_ground = false;

        let player_w = self.collider_width();
        let player_h = self.collider_height();

        for tile in terrain {
            if let Tile::Platform(platform_aabb) = tile {
                let player_aabb = self.collider();
                if !player_aabb.intersects(platform_aabb) {
                    continue;
                }

                // Compute overlap amounts on all four sides
                let overlap_left = (player_aabb.x + player_aabb.w) - platform_aabb.x;
                let overlap_right = (platform_aabb.x + platform_aabb.w) - player_aabb.x;
                let overlap_top = (player_aabb.y + player_aabb.h) - platform_aabb.y;
                let overlap_bottom = (platform_aabb.y + platform_aabb.h) - player_aabb.y;

                let min_overlap = overlap_left
                    .min(overlap_right)
                    .min(overlap_top)
                    .min(overlap_bottom);

                // Resolve based on direction of movement and minimum overlap axis
                if min_overlap == overlap_bottom && self.vel.y < 0.0 {
                    // Ceiling collision — player moving upward, head hits platform bottom
                    self.vel.y = 0.0;
                    self.pos.y = platform_aabb.y + platform_aabb.h + player_h;
                } else if min_overlap == overlap_top && self.vel.y >= 0.0 {
                    // Floor collision — player landing on platform top
                    self.vel.y = 0.0;
                    self.on_ground = true;
                    self.pos.y = platform_aabb.y; // foot aligned to platform surface
                } else if min_overlap == overlap_left && self.vel.x > 0.0 {
                    // Right-side wall — player moving right, hits wall from left
                    self.vel.x = 0.0;
                    self.pos.x = platform_aabb.x - player_w / 2.0;
                } else if min_overlap == overlap_right && self.vel.x < 0.0 {
                    // Left-side wall — player moving left, hits wall from right
                    self.vel.x = 0.0;
                    self.pos.x = platform_aabb.x + platform_aabb.w + player_w / 2.0;
                } else if min_overlap == overlap_top {
                    // Floor collision while vel.y < 0 (rare: ceiling+floor sandwich)
                    // Prioritize floor — place player on top
                    self.vel.y = 0.0;
                    self.on_ground = true;
                    self.pos.y = platform_aabb.y;
                } else if min_overlap == overlap_bottom {
                    // Ceiling collision while vel.y >= 0 (rare)
                    self.vel.y = 0.0;
                    self.pos.y = platform_aabb.y + platform_aabb.h + player_h;
                } else if min_overlap == overlap_left {
                    self.vel.x = 0.0;
                    self.pos.x = platform_aabb.x - player_w / 2.0;
                } else if min_overlap == overlap_right {
                    self.vel.x = 0.0;
                    self.pos.x = platform_aabb.x + platform_aabb.w + player_w / 2.0;
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // update_facing — directional facing from input
    // ------------------------------------------------------------------
    fn update_facing(&mut self, input: &InputState) {
        if input.left && !input.right {
            self.facing = -1;
        } else if input.right && !input.left {
            self.facing = 1;
        }
        // If both or neither are held, keep previous facing (don't reset)
    }

    // ------------------------------------------------------------------
    // Helper: collider dimensions based on power-up state
    // ------------------------------------------------------------------
    fn collider_width(&self) -> f32 {
        16.0
    }

    fn collider_height(&self) -> f32 {
        match self.state {
            PlayerState::Small => 16.0,
            PlayerState::Super | PlayerState::Fire => 32.0,
        }
    }

    /// Returns the player's current world-space foot position (IAPI-007).
    /// Pure getter — no side effects.
    pub fn pos(&self) -> Vec2 {
        Vec2 {
            x: self.pos.x,
            y: self.pos.y,
        }
    }

    /// Returns the player's current statistics for HUD display (IAPI-009).
    /// Pure getter — no side effects.
    pub fn stats(&self) -> PlayerStats {
        PlayerStats {
            coins: self.coins,
            lives: self.lives,
        }
    }

    /// Returns the player's collision box as an AABB.
    ///
    /// The AABB is anchored at the player's foot position:
    /// - `x = pos.x - w / 2` (horizontal center)
    /// - `y = pos.y - h` (top edge, with pos.y at the bottom)
    ///
    /// Size depends on power-up state:
    /// - Small: 16 x 16
    /// - Super / Fire: 16 x 32
    pub fn collider(&self) -> AABB {
        let w = self.collider_width();
        let h = self.collider_height();
        AABB {
            x: self.pos.x - w / 2.0,
            y: self.pos.y - h,
            w,
            h,
        }
    }

    /// Applies a power-up state change (called by Feature #8 Collectibles).
    ///
    /// # Preconditions
    /// - `state` must be `Super` or `Fire`
    /// - Called only when upgrading from `Small`
    ///
    /// # Side effects
    /// - Updates `self.state`
    /// - Recalculates collider size (16x16 → 16x32, foot-aligned)
    pub fn apply_powerup(&mut self, state: PlayerState) {
        self.state = state;
    }

    /// Processes damage taken by the player.
    ///
    /// # Returns
    /// - `false` if the player downgrades (Super/Fire → Small) but survives
    /// - `true` if the player dies (Small → death)
    pub fn take_damage(&mut self) -> bool {
        match self.state {
            PlayerState::Small => true,
            PlayerState::Super | PlayerState::Fire => {
                self.state = PlayerState::Small;
                false
            }
        }
    }
}
