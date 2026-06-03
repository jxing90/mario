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
use crate::systems::hud::HudRenderer;
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
    pub hud: HudRenderer,
    /// Actual window dimensions for screen-space coordinate scaling.
    /// Set by main.rs after window creation. (0,0) = test mode → render is no-op.
    pub screen_w: f32,
    pub screen_h: f32,
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
            hud: HudRenderer::new(),
            screen_w: 0.0,
            screen_h: 0.0,
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

        // 11. Update camera to follow player (IAPI-007 + IAPI-008)
        self.camera.update(self.player.pos(), self.level_bounds, dt);
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

    /// Renders the playing state: sky → parallax → level → entities → HUD.
    ///
    /// Uses Macroquad shape primitives (programmer art). World coordinates are
    /// transformed to screen coordinates via camera offset + viewport scaling.
    /// HUD is rendered in screen space after resetting the camera transform.
    pub fn render(&mut self, _alpha: f32) {
        // Guard: screen_w/screen_h are 0.0 during tests (no GL context).
        // main.rs sets them to actual window size before the first frame.
        let sw = self.screen_w;
        let sh = self.screen_h;
        if sw <= 0.0 || sh <= 0.0 {
            return;
        }

        let (vp_w, vp_h) = self.camera.viewport();
        let sx = sw / vp_w;
        let sy = sh / vp_h;
        let cam = self.camera.offset();

        // World → screen coordinate helper
        let ws = |wx: f32, wy: f32| -> (f32, f32) {
            ((wx - cam.x) * sx, (wy - cam.y) * sy)
        };

        // ── 1. Sky background ──
        macroquad::prelude::clear_background(macroquad::color::Color::new(
            0.35, 0.65, 0.95, 1.0,
        ));

        // ── 2. Ground & elevated platforms ──
        use macroquad::shapes::draw_rectangle;
        let ground_color = macroquad::color::Color::new(0.40, 0.75, 0.30, 1.0);
        let plat_color = macroquad::color::Color::new(0.55, 0.35, 0.15, 1.0);

        // Ground: (0, 600, 2000, 40)
        {
            let (gx, gy) = ws(0.0, 600.0);
            draw_rectangle(gx, gy, 2000.0 * sx, 40.0 * sy, ground_color);
        }
        // Elevated platform at (400, 450, 200, 30)
        {
            let (px, py) = ws(400.0, 450.0);
            draw_rectangle(px, py, 200.0 * sx, 30.0 * sy, plat_color);
        }
        // Elevated platform at (1000, 320, 250, 30)
        {
            let (px, py) = ws(1000.0, 320.0);
            draw_rectangle(px, py, 250.0 * sx, 30.0 * sy, plat_color);
        }

        // ── 3. Spikes (red triangles approximated as rectangles) ──
        let spike_color = macroquad::color::Color::new(0.9, 0.2, 0.1, 1.0);
        for &spike_x in &[300.0_f32, 700.0, 1100.0] {
            let (sx_pos, sy_pos) = ws(spike_x, 592.0);
            draw_rectangle(sx_pos, sy_pos, 16.0 * sx, 8.0 * sy, spike_color);
        }

        // ── 4. Coins (yellow circles) ──
        use macroquad::shapes::draw_circle;
        let coin_color = macroquad::color::YELLOW;
        // Coin positions from Level::new
        for &(cx, cy) in &[
            (200.0, 550.0),
            (240.0, 550.0),
            (280.0, 550.0),
            (500.0, 400.0),
            (540.0, 400.0),
            (580.0, 400.0),
            (1100.0, 270.0),
            (1140.0, 270.0),
        ] {
            let (scx, scy) = ws(cx, cy);
            draw_circle(scx, scy, 6.0 * sx.min(sy), coin_color);
        }

        // ── 5. Question blocks (orange with "?") ──
        use macroquad::text::draw_text;
        let qblock_color = macroquad::color::Color::new(1.0, 0.65, 0.0, 1.0);
        let qblock_text_color = macroquad::color::WHITE;
        for &(bx, by) in &[
            (450.0, 400.0),
            (1050.0, 270.0),
            (1400.0, 350.0),
        ] {
            let (sbx, sby) = ws(bx, by);
            draw_rectangle(sbx, sby, 32.0 * sx, 32.0 * sy, qblock_color);
            draw_text("?", sbx + 8.0 * sx, sby + 24.0 * sy, 24.0 * sx.min(sy), qblock_text_color);
        }

        // ── 6. Checkpoints ──
        let cp_color = macroquad::color::Color::new(0.2, 0.8, 0.2, 1.0);
        for cp in &self.checkpoints {
            let (cpx, cpy) = ws(cp.pos.x, cp.pos.y);
            let cp_h = if cp.activated { 48.0 } else { 32.0 };
            draw_rectangle(cpx, cpy - cp_h * sy, 8.0 * sx, cp_h * sy, cp_color);
        }

        // ── 7. Flagpole ──
        let fp = self.flagpole.pos;
        let (fpx, fpy) = ws(fp.x, fp.y);
        draw_rectangle(fpx, fpy - 80.0 * sy, 8.0 * sx, 80.0 * sy, macroquad::color::GRAY);
        draw_rectangle(fpx - 4.0 * sx, fpy - 80.0 * sy, 16.0 * sx, 16.0 * sy, macroquad::color::GREEN);

        // ── 8. Enemies ──
        for enemy in &self.enemies {
            if enemy.alive {
                let ep = enemy.pos();
                let (sex, sey) = ws(ep.x, ep.y);
                draw_rectangle(sex, sey, 16.0 * sx, 16.0 * sy, macroquad::color::BROWN);
                // Eyes
                draw_circle(sex + 4.0 * sx, sey + 4.0 * sy, 2.0 * sx.min(sy), macroquad::color::WHITE);
                draw_circle(sex + 12.0 * sx, sey + 4.0 * sy, 2.0 * sx.min(sy), macroquad::color::WHITE);
            }
        }

        // ── 9. Player ──
        let p = self.player.pos();
        let (spx, spy) = ws(p.x, p.y);
        let player_h = match self.player.state {
            crate::entities::player::PlayerState::Small => 16.0,
            _ => 32.0, // Super or Fire — double height
        };
        let player_color = if self.invuln_timer > 0.0 {
            // Flicker effect during invulnerability
            let phase = (self.flicker_phase * 4.0) as u32;
            if phase % 2 == 0 {
                macroquad::color::RED
            } else {
                macroquad::color::Color::new(1.0, 1.0, 1.0, 0.3)
            }
        } else {
            macroquad::color::RED
        };
        draw_rectangle(spx, spy - (player_h - 16.0) * sy, 16.0 * sx, player_h * sy, player_color);
        // Hat
        draw_rectangle(spx, spy - (player_h - 12.0) * sy, 16.0 * sx, 6.0 * sy, macroquad::color::RED);

        // ── 10. HUD (screen-space overlay) ──
        let font_size = 18.0 * sx.min(sy);
        let stats = self.player.stats();
        draw_text(
            &format!("COINS: {}   LIVES: {}", stats.coins, stats.lives),
            10.0,
            30.0 * sy,
            font_size,
            macroquad::color::WHITE,
        );
    }
}
