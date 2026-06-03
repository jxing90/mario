// Feature #6: Life, Death & Win — PlayingState
// Design Reference: docs/features/6-life-death-win.md §4, §6, §8
//
// The primary gameplay state. Manages player physics, hazard/flagpole/checkpoint
// detection, invulnerability timers, and triggers transitions to Dead/Victory.

use crate::entities::coin::Coin;
use crate::entities::enemy::{Enemy, EnemyConfig};
use crate::entities::player::Player;
use crate::entities::flagpole::{Flagpole, FlagpolePhase};
use crate::entities::checkpoint::Checkpoint;
use crate::entities::question_block::QuestionBlock;
use crate::level::{Level, LevelBounds, Vec2};
use crate::systems::camera::Camera;
use crate::systems::camera::CameraConfig;
use crate::systems::hud::HudRenderer;
use crate::systems::physics::{CollisionEvent, Physics};
use crate::states::{GameState, LifeState, VictoryState, GameOverState};

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
    pub coins: Vec<Coin>,
    pub question_blocks: Vec<QuestionBlock>,
    pub invuln_timer: f32,
    pub flicker_phase: f32,
    pub frame_count: u64,
    pub hud: HudRenderer,
    pub screen_w: f32,
    pub screen_h: f32,
    pub current_level: u32,
    level_bounds: LevelBounds,
}

impl PlayingState {
    /// Creates a new PlayingState with the given player and life_state.
    ///
    /// Level, camera, flagpole, and checkpoints are initialized with defaults.
    /// Flagpole is placed at the end of the level (x=1800, y=560).
    pub fn new(player: Player, life_state: LifeState) -> Self {
        Self::with_level(player, life_state, 1)
    }

    /// Creates a PlayingState for a specific level number (1-4).
    pub fn with_level(player: Player, life_state: LifeState, level_num: u32) -> Self {
        let level = Level::load(level_num);
        let camera = Camera::new(CameraConfig::default());
        let bounds = level.bounds();

        // Spawn flagpole from level config
        let flagpole = Flagpole::new(Vec2 {
            x: level.flagpole_spawn.pos.x,
            y: level.flagpole_spawn.pos.y,
        });

        // Spawn checkpoints from level config
        let checkpoints: Vec<Checkpoint> = level.checkpoint_spawns.iter()
            .map(|c| Checkpoint::new(Vec2 { x: c.pos.x, y: c.pos.y }))
            .collect();

        // Spawn coins from level config
        let coins: Vec<Coin> = level.coin_spawns.iter()
            .map(|c| Coin::new(Vec2 { x: c.pos.x, y: c.pos.y }))
            .collect();

        // Spawn question blocks from level config
        let question_blocks: Vec<QuestionBlock> = level.block_spawns.iter()
            .map(|b| QuestionBlock::new(Vec2 { x: b.pos.x, y: b.pos.y }))
            .collect();

        // Spawn enemies from level config
        let enemies: Vec<Enemy> = level.enemy_spawns.iter()
            .map(|e| Enemy::new(
                Vec2 { x: e.pos.x, y: e.pos.y },
                Vec2 { x: e.waypoint_a.x, y: e.waypoint_a.y },
                Vec2 { x: e.waypoint_b.x, y: e.waypoint_b.y },
                EnemyConfig::default(),
            ))
            .collect();

        Self {
            player,
            level,
            camera,
            life_state,
            flagpole,
            checkpoints,
            enemies,
            coins,
            question_blocks,
            invuln_timer: 0.0,
            flicker_phase: 0.0,
            frame_count: 0,
            hud: HudRenderer::new(),
            screen_w: 0.0,
            screen_h: 0.0,
            current_level: level_num,
            level_bounds: bounds,
        }
    }

    /// Advance the simulation by one fixed timestep.
    ///
    /// Runs: invuln countdown → enemy patrol → player physics → hazard check → enemy check
    /// → flagpole check → checkpoint activation → event consumption. Transitions to Dead/Victory when triggered.
    /// Returns Some(GameState::Victory) when flagpole slide completes,
    /// Some(GameState::GameOver) when lives reach 0, or None to continue playing.
    pub fn update(&mut self, dt: f32) -> Option<GameState> {
        self.frame_count = self.frame_count.wrapping_add(1);

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
            if self.flagpole.phase == FlagpolePhase::Done {
                return Some(GameState::Victory(VictoryState::new(
                    self.player.coins, self.current_level,
                )));
            }
            return None; // Input locked, no player update during slide
        }

        // 3. Enemy patrol update (Step A: enemy.update(dt) for each living enemy)
        for enemy in self.enemies.iter_mut() {
            enemy.update(dt);
        }

        // 4. Update player physics with real keyboard input at runtime.
        // During tests (screen_w == 0.0), use default (no input) to avoid
        // panicking in Macroquad's is_key_down() which requires a GL context.
        let terrain = self.level.query_terrain(&self.player.collider());
        let input = if self.screen_w > 0.0 {
            self.read_input()
        } else {
            crate::input::InputState::default()
        };
        self.player.update(dt, &input, &terrain);

        // 4b. Coin collection: player overlaps uncollected coin → collect
        for coin in self.coins.iter_mut() {
            if !coin.collected && self.player.collider().intersects(&coin.collider()) {
                coin.collected = true;
                self.player.coins += 1;
            }
        }

        // 4c. Question block hit: player head hits block from below
        for block in self.question_blocks.iter_mut() {
            if !block.used {
                let block_aabb = block.collider();
                let player_aabb = self.player.collider();
                // Hit from below: player bottom is below block top, player was moving upward
                if player_aabb.intersects(&block_aabb)
                    && self.player.vel.y < 0.0
                    && (player_aabb.y + player_aabb.h) > block_aabb.y
                    && player_aabb.y < block_aabb.y + block_aabb.h
                {
                    block.used = true;
                    self.player.vel.y = 100.0;
                    // Pseudo-random roll from frame counter
                    let roll = ((self.frame_count.wrapping_mul(6364136223846793005).wrapping_add(1)) as f64
                        / u64::MAX as f64) as f32;
                    if roll < 0.70 {
                        self.player.coins += 1;
                    } else if roll < 0.85 {
                        self.player.apply_powerup(crate::entities::player::PlayerState::Super);
                    } else {
                        self.player.apply_powerup(crate::entities::player::PlayerState::Fire);
                    }
                }
            }
        }

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

        // 11b. Clamp player to level bounds (invisible walls at left/right edges)
        let bw = self.player.collider().w;
        self.player.pos.x = self.player.pos.x.clamp(
            self.level_bounds.min_x,
            self.level_bounds.max_x - bw,
        );

        // 12. Check Game Over: lives exhausted
        if self.player.lives == 0 {
            return Some(GameState::GameOver(GameOverState::new(self.player.coins)));
        }

        None // Continue playing
    }

    /// Reads keyboard state from Macroquad and returns an InputState snapshot.
    ///
    /// Arrow keys or WASD for movement, Space for jump, Shift for sprint,
    /// Escape for menu toggle, Enter for confirm.
    fn read_input(&self) -> crate::input::InputState {
        use macroquad::input::{is_key_down, is_key_pressed, KeyCode};
        crate::input::InputState {
            left: is_key_down(KeyCode::Left) || is_key_down(KeyCode::A),
            right: is_key_down(KeyCode::Right) || is_key_down(KeyCode::D),
            jump: is_key_down(KeyCode::Space),
            jump_just: is_key_pressed(KeyCode::Space),
            sprint: is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift),
            esc_just: is_key_pressed(KeyCode::Escape),
            confirm: is_key_pressed(KeyCode::Enter),
        }
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

        // ── 1. Background (themed per level) ──
        use macroquad::color::Color;
        let (bg_color, ground_color, plat_color, spike_color) = match self.current_level {
            1 => (  // Green Plains: blue sky, green ground, brown bricks
                Color::new(0.35, 0.65, 0.95, 1.0),
                Color::new(0.40, 0.75, 0.30, 1.0),
                Color::new(0.55, 0.35, 0.15, 1.0),
                Color::new(0.9, 0.2, 0.1, 1.0),
            ),
            2 => (  // Underground: dark cavern, gray stone, blue-gray bricks
                Color::new(0.05, 0.05, 0.12, 1.0),
                Color::new(0.25, 0.25, 0.30, 1.0),
                Color::new(0.40, 0.45, 0.55, 1.0),
                Color::new(0.85, 0.15, 0.05, 1.0),
            ),
            3 => (  // Sky World: light blue, white clouds, golden platforms
                Color::new(0.55, 0.80, 1.0, 1.0),
                Color::new(0.85, 0.90, 0.95, 1.0),
                Color::new(0.95, 0.75, 0.30, 1.0),
                Color::new(0.7, 0.15, 0.55, 1.0),
            ),
            _ => (  // Castle: dark red-black, dark stone, gray bricks
                Color::new(0.08, 0.02, 0.04, 1.0),
                Color::new(0.30, 0.25, 0.25, 1.0),
                Color::new(0.50, 0.45, 0.45, 1.0),
                Color::new(1.0, 0.25, 0.05, 1.0),
            ),
        };
        macroquad::prelude::clear_background(bg_color);

        // ── 2. Platforms (drawn from Level data) ──
        use macroquad::shapes::draw_rectangle;
        for platform in self.level.platforms() {
            let aabb = &platform.aabb;
            let (px, py) = ws(aabb.x, aabb.y);
            let color = if aabb.y >= 590.0 { ground_color } else { plat_color };
            draw_rectangle(px, py, aabb.w * sx, aabb.h * sy, color);
        }

        // Boundary walls (invisible collision + visual markers at level edges)
        let bounds = self.level.bounds();
        let wall_color = macroquad::color::Color::new(0.5, 0.5, 0.5, 0.6);
        // Left wall
        {
            let (wx, wy) = ws(bounds.min_x - 4.0, 0.0);
            draw_rectangle(wx, wy, 4.0 * sx, (bounds.kill_y - 0.0) * sy, wall_color);
        }
        // Right wall
        {
            let (wx, wy) = ws(bounds.max_x, 0.0);
            draw_rectangle(wx, wy, 4.0 * sx, (bounds.kill_y - 0.0) * sy, wall_color);
        }

        // ── 3. Spikes (drawn from Level terrain query at ground level) ──
        for spike_x in &[300.0_f32, 700.0, 1100.0, 1500.0] {
            let (sx_pos, sy_pos) = ws(*spike_x, 592.0);
            draw_rectangle(sx_pos, sy_pos, 16.0 * sx, 8.0 * sy, spike_color);
        }

        // ── 4. Coins (yellow circles, only uncollected) ──
        use macroquad::shapes::draw_circle;
        let coin_color = macroquad::color::YELLOW;
        for coin in &self.coins {
            if !coin.collected {
                let (scx, scy) = ws(coin.pos.x, coin.pos.y);
                draw_circle(scx, scy, 6.0 * sx.min(sy), coin_color);
            }
        }

        // ── 5. Question blocks (orange "?" or dark "used") ──
        use macroquad::text::draw_text;
        for block in &self.question_blocks {
            let (sbx, sby) = ws(block.pos.x, block.pos.y);
            if block.used {
                draw_rectangle(sbx, sby, 32.0 * sx, 32.0 * sy, macroquad::color::DARKGRAY);
            } else {
                draw_rectangle(sbx, sby, 32.0 * sx, 32.0 * sy, macroquad::color::Color::new(1.0, 0.65, 0.0, 1.0));
                draw_text("?", sbx + 8.0 * sx, sby + 24.0 * sy, 24.0 * sx.min(sy), macroquad::color::WHITE);
            }
        }

        // ── 6. Checkpoints ──
        let cp_color = macroquad::color::Color::new(0.2, 0.8, 0.2, 1.0);
        for cp in &self.checkpoints {
            let (cpx, cpy) = ws(cp.pos.x, cp.pos.y);
            let cp_h = if cp.activated { 48.0 } else { 32.0 };
            draw_rectangle(cpx, cpy - cp_h * sy, 8.0 * sx, cp_h * sy, cp_color);
        }

        // ── 7. Flagpole (prominent goal marker) ──
        let fp = self.flagpole.pos;
        let (fpx, fpy) = ws(fp.x, fp.y);
        let pole_w = 6.0 * sx;
        let pole_h = 120.0 * sy;
        // Pole shadow
        draw_rectangle(fpx + 2.0 * sx, fpy - pole_h + 2.0 * sy, pole_w, pole_h, macroquad::color::DARKGRAY);
        // Main pole
        draw_rectangle(fpx, fpy - pole_h, pole_w, pole_h, macroquad::color::GRAY);
        // Green flag (triangular-ish)
        let flag_w = 24.0 * sx;
        let flag_h = 18.0 * sy;
        draw_rectangle(fpx + pole_w, fpy - pole_h, flag_w, flag_h, macroquad::color::GREEN);
        // Star on flag
        draw_text("*", fpx + pole_w + 6.0 * sx, fpy - pole_h + 14.0 * sy, 16.0 * sx.min(sy), macroquad::color::YELLOW);
        // "GOAL" label
        draw_text("GOAL", fpx - 8.0 * sx, fpy - pole_h - 20.0 * sy, 20.0 * sx.min(sy), macroquad::color::GOLD);
        // Ground base
        draw_rectangle(fpx - 8.0 * sx, fpy, pole_w + 16.0 * sx, 8.0 * sy, macroquad::color::DARKGRAY);

        // ── 8. Enemies (drawn from foot upward, same as player) ──
        for enemy in &self.enemies {
            if enemy.alive {
                let ep = enemy.pos();
                // Enemy collider bottom is at pos.y; draw from foot going up 16px
                let (sex, sey) = ws(ep.x, ep.y - 16.0);
                draw_rectangle(sex, sey, 16.0 * sx, 16.0 * sy, macroquad::color::BROWN);
                // Eyes in upper portion
                draw_circle(sex + 4.0 * sx, sey + 3.0 * sy, 2.0 * sx.min(sy), macroquad::color::WHITE);
                draw_circle(sex + 12.0 * sx, sey + 3.0 * sy, 2.0 * sx.min(sy), macroquad::color::WHITE);
                // Feet
                draw_rectangle(sex + 2.0 * sx, sey + 12.0 * sy, 5.0 * sx, 4.0 * sy, macroquad::color::BLACK);
                draw_rectangle(sex + 9.0 * sx, sey + 12.0 * sy, 5.0 * sx, 4.0 * sy, macroquad::color::BLACK);
            }
        }

        // ── 9. Player (Mario) ──
        let p = self.player.pos();
        let (spx, spy) = ws(p.x, p.y);
        let is_small = matches!(self.player.state, crate::entities::player::PlayerState::Small);
        let body_h = if is_small { 16.0 } else { 32.0 };
        let unit = body_h / 16.0; // scale factor: 1.0 for Small, 2.0 for Super/Fire
        let sx_s = sx * unit;
        let sy_s = sy * unit;

        // Invulnerability flicker
        let flicker = self.invuln_timer > 0.0 && ((self.flicker_phase * 4.0) as u32) % 2 == 1;

        if !flicker {
            let hat_color = macroquad::color::Color::new(0.85, 0.15, 0.1, 1.0);
            let skin_color = macroquad::color::Color::new(1.0, 0.75, 0.55, 1.0);
            let overall_color = macroquad::color::Color::new(0.1, 0.3, 0.9, 1.0);
            let shoe_color = macroquad::color::Color::new(0.45, 0.25, 0.15, 1.0);
            let eye_color = macroquad::color::BLACK;
            let button_color = macroquad::color::YELLOW;
            let top = spy - (body_h - 16.0) * sy;

            // Hat (top 5 units)
            draw_rectangle(spx, top, 16.0 * sx_s, 5.0 * sy_s, hat_color);
            // Brim
            draw_rectangle(spx - 2.0 * sx_s, top + 3.0 * sy_s, 20.0 * sx_s, 3.0 * sy_s, hat_color);
            // Face (units 5-9)
            draw_rectangle(spx, top + 5.0 * sy_s, 16.0 * sx_s, 4.0 * sy_s, skin_color);
            // Eye
            draw_rectangle(spx + 10.0 * sx_s, top + 5.5 * sy_s, 3.0 * sx_s, 2.0 * sy_s, eye_color);
            // Overall (units 9-14)
            draw_rectangle(spx, top + 9.0 * sy_s, 16.0 * sx_s, 5.0 * sy_s, overall_color);
            // Buttons
            draw_circle(spx + 8.0 * sx_s, top + 10.5 * sy_s, 1.5 * sx_s.min(sy_s), button_color);
            draw_circle(spx + 8.0 * sx_s, top + 12.5 * sy_s, 1.5 * sx_s.min(sy_s), button_color);
            // Shoes (bottom 2 units)
            draw_rectangle(spx, top + 14.0 * sy_s, 7.0 * sx_s, 2.0 * sy_s, shoe_color);
            draw_rectangle(spx + 9.0 * sx_s, top + 14.0 * sy_s, 7.0 * sx_s, 2.0 * sy_s, shoe_color);
        }

        // ── 10. HUD (screen-space overlay) ──
        let font_size = 18.0 * sx.min(sy);
        let stats = self.player.stats();
        draw_text(
            &format!("WORLD 1-{}", self.current_level),
            10.0,
            30.0 * sy,
            font_size,
            macroquad::color::WHITE,
        );
        draw_text(
            &format!("COINS: {}   LIVES: {}", stats.coins, stats.lives),
            10.0,
            55.0 * sy,
            font_size,
            macroquad::color::WHITE,
        );
    }
}
