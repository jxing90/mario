// Feature #6: Life, Death & Win — PlayingState
// Design Reference: docs/features/6-life-death-win.md §4, §6, §8
//
// The primary gameplay state. Manages player physics, hazard/flagpole/checkpoint
// detection, invulnerability timers, and triggers transitions to Dead/Victory.

use crate::entities::enemy::{Enemy, EnemyConfig};
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
    pub enemies: Vec<Enemy>,
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

        // Hardcoded patrol enemies placed on the ground (y=584 for 16x16 foot-anchored collider on ground at y=600)
        let enemies: Vec<Enemy> = vec![
            Enemy::new(
                Vec2 { x: 200.0, y: 584.0 },
                Vec2 { x: 100.0, y: 584.0 },
                Vec2 { x: 300.0, y: 584.0 },
                EnemyConfig::default(),
            ),
            Enemy::new(
                Vec2 { x: 800.0, y: 584.0 },
                Vec2 { x: 700.0, y: 584.0 },
                Vec2 { x: 900.0, y: 584.0 },
                EnemyConfig::default(),
            ),
        ];

        Self {
            player,
            level,
            camera,
            life_state,
            flagpole,
            checkpoints,
            enemies,
            invuln_timer: 0.0,
            flicker_phase: 0.0,
            level_bounds: bounds,
        }
    }

    /// Advance the simulation by one fixed timestep.
    ///
    /// Runs: invuln countdown → enemy patrol → player physics → hazard check → enemy check
    /// → flagpole check → checkpoint activation → event consumption. Transitions to Dead/Victory when triggered.
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

        // 3. Enemy patrol update (Step A: enemy.update(dt) for each living enemy)
        for enemy in self.enemies.iter_mut() {
            enemy.update(dt);
        }

        // 4. Update player physics (normal gameplay)
        let terrain = self.level.query_terrain(&self.player.collider());
        let no_input = crate::input::InputState::default();
        self.player.update(dt, &no_input, &terrain);

        // 5. Hazard check: if player is NOT invulnerable and hazards exist → death
        let kill_y = self.level_bounds.kill_y;
        let events = Physics::hazard_check(&self.player, &terrain, kill_y);

        if !events.is_empty() && self.invuln_timer <= 0.0 && self.player.lives > 0 {
            self.player.lives -= 1;
        }

        // 6. Enemy collision detection (Step D: enemy_check)
        let enemy_events = Physics::enemy_check(&self.player, &self.enemies, dt);

        // 7. Consume enemy collision events (Step E)
        for event in enemy_events {
            match event {
                CollisionEvent::EnemyStomp(i) => {
                    self.enemies[i].alive = false;
                    self.player.vel.y = self.enemies[i].config.bounce_velocity;
                }
                CollisionEvent::EnemyContact(_)
                    if self.invuln_timer <= 0.0 && self.player.lives > 0 =>
                {
                    self.player.lives -= 1;
                }
                _ => {}
            }
        }

        // 8. Flagpole check: if player overlaps flagpole collider → victory slide
        if self.flagpole.phase == FlagpolePhase::Idle
            && self.player.collider().intersects(&self.flagpole.collider())
        {
            self.flagpole.phase = FlagpolePhase::Sliding;
        }

        // 9. Checkpoint activation: overlap with inactive checkpoint → activate
        for cp in self.checkpoints.iter_mut() {
            if !cp.activated && self.player.collider().intersects(&cp.collider()) {
                cp.activated = true;
                self.life_state.checkpoint = Some(cp.pos);
            }
        }

        // 10. Sync LifeState coins from player
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
