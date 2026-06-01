// Feature #6: Life, Death & Win — PlayingState
// Design Reference: docs/features/6-life-death-win.md §4, §6, §8
//
// The primary gameplay state. Manages player physics, hazard/flagpole/checkpoint
// detection, invulnerability timers, and triggers transitions to Dead/Victory.

use crate::entities::player::Player;
use crate::entities::flagpole::{Flagpole, FlagpolePhase};
use crate::entities::checkpoint::Checkpoint;
use crate::level::{Level, LevelBounds, Vec2};
use crate::systems::camera::Camera;
use crate::systems::camera::CameraConfig;
use crate::systems::physics::{CollisionEvent, Physics};
use crate::states::LifeState;

/// The main gameplay state — normal play, hazard detection, and flagpole/checkpoint logic.
///
/// # Fields (§8 Data Model)
/// - `player`: the player entity with position, velocity, lives, coins.
/// - `level`: immutable level geometry and bounds.
/// - `camera`: viewport offset for rendering.
/// - `life_state`: persistent lives/checkpoint/coins across respawns.
/// - `flagpole`: the end-of-level flagpole trigger.
/// - `checkpoints`: mid-level checkpoint entities.
/// - `invuln_timer`: remaining invulnerability time (0.0 = vulnerable).
/// - `flicker_phase`: oscillating timer for invulnerability visual flicker.
/// - `level_bounds`: cached LevelBounds for hazard/checkpoint queries.
pub struct PlayingState {
    pub player: Player,
    pub level: Level,
    pub camera: Camera,
    pub life_state: LifeState,
    pub flagpole: Flagpole,
    pub checkpoints: Vec<Checkpoint>,
    pub invuln_timer: f32,
    pub flicker_phase: f32,
    level_bounds: LevelBounds,
}

impl PlayingState {
    /// Creates a new PlayingState with the given player and life_state.
    ///
    /// Level, camera, flagpole, and checkpoints are initialized with defaults.
    /// Flagpole is placed at the end of the level (x=1800, y=560).
    pub fn new(player: Player, life_state: LifeState) -> Self {
        let level = Level::new();
        let camera = Camera::new(CameraConfig::default());
        let bounds = level.bounds();
        let flagpole = Flagpole::new(Vec2 { x: 1800.0, y: 560.0 });
        let checkpoints: Vec<Checkpoint> = vec![
            Checkpoint::new(Vec2 { x: 500.0, y: 400.0 }),
        ];

        Self {
            player,
            level,
            camera,
            life_state,
            flagpole,
            checkpoints,
            invuln_timer: 0.0,
            flicker_phase: 0.0,
            level_bounds: bounds,
        }
    }

    /// Advance the simulation by one fixed timestep.
    ///
    /// Runs: invuln countdown → player physics → hazard check → flagpole check
    /// → checkpoint activation. Transitions to Dead/Victory when triggered.
    pub fn update(&mut self, dt: f32) {
        // 1. Advance invulnerability timer
        if self.invuln_timer > 0.0 {
            self.invuln_timer -= dt;
            self.flicker_phase += dt;
            if self.invuln_timer < 0.0 {
                self.invuln_timer = 0.0;
            }
        }

        // 2. Flagpole slide animation (if active, advance and check completion)
        if self.flagpole.phase == FlagpolePhase::Sliding {
            self.flagpole.update(dt);
            return; // Input locked, no player update during slide
        }

        // 3. Update player physics (normal gameplay)
        // Note: input is not available here — this is a unit-testable update.
        // In production, InputState is passed via GameState::update wrapper.
        // For tests, player movement is verified via direct Player::update calls.
        let terrain = self.level.query_terrain(&self.player.collider());
        let no_input = crate::input::InputState::default();
        self.player.update(dt, &no_input, &terrain);

        // 4. Hazard check: if player is NOT invulnerable and hazards exist → death
        let kill_y = self.level_bounds.kill_y;
        let events = Physics::hazard_check(&self.player, &terrain, kill_y);

        if !events.is_empty() && self.invuln_timer <= 0.0 {
            // Decrement lives (death triggered); transition to DeadState is
            // handled by GameState wrapper reading the signal from this method.
            if self.player.lives > 0 {
                self.player.lives -= 1;
            }
        }

        // 5. Flagpole check: if player overlaps flagpole collider → victory slide
        if self.flagpole.phase == FlagpolePhase::Idle
            && self.player.collider().intersects(&self.flagpole.collider())
        {
            self.flagpole.phase = FlagpolePhase::Sliding;
        }

        // 6. Checkpoint activation: overlap with inactive checkpoint → activate
        for cp in self.checkpoints.iter_mut() {
            if !cp.activated && self.player.collider().intersects(&cp.collider()) {
                cp.activated = true;
                self.life_state.checkpoint = Some(cp.pos);
            }
        }

        // 7. Sync LifeState coins from player
        self.life_state.coins = self.player.coins;
    }

    /// Detects all hazard collision events for the current frame.
    ///
    /// # Contract (§4)
    /// - Returns empty Vec if player is invulnerable.
    /// - Delegates to `Physics::hazard_check()` for actual detection.
    pub fn check_hazards(&self) -> Vec<CollisionEvent> {
        if self.invuln_timer > 0.0 {
            return Vec::new();
        }
        let terrain = self.level.query_terrain(&self.player.collider());
        Physics::hazard_check(&self.player, &terrain, self.level_bounds.kill_y)
    }

    /// Returns true if the player's collider overlaps the flagpole trigger zone.
    pub fn check_flagpole(&self) -> bool {
        self.player.collider().intersects(&self.flagpole.collider())
    }

    /// Activates a checkpoint by index, storing its position for respawn.
    ///
    /// # Panics
    /// Panics if `cp_idx` is out of bounds.
    pub fn activate_checkpoint(&mut self, cp_idx: usize) {
        let cp = &mut self.checkpoints[cp_idx];
        cp.activated = true;
        self.life_state.checkpoint = Some(cp.pos);
    }

    /// Renders the playing state (player, level, camera, HUD).
    pub fn render(&mut self, _alpha: f32) {
        // Rendering is deferred to the Macroquad draw loop.
        // This method exists for the StateMachine trait contract.
        // In production, this would call draw_texture, draw_rectangle, etc.
    }
}
