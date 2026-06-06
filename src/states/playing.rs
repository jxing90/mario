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
use crate::entities::power_up::{PowerUp, PowerUpKind};
use crate::entities::question_block::QuestionBlock;
use crate::level::{BrickSpawn, Level, LevelBounds, Vec2};
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
    /// Spawned power-up entities (mushrooms, flowers) bouncing in the world.
    pub power_ups: Vec<PowerUp>,
    /// Breakable bricks: when hit from below by Super/Fire Mario, they shatter.
    pub bricks: Vec<Brick>,
    /// Spawn positions for brick reset on death (bricks are restored).
    #[allow(dead_code)]
    brick_spawns: Vec<BrickSpawn>,
    pub invuln_timer: f32,
    pub flicker_phase: f32,
    /// Countdown timer in seconds (classic Mario: 300s per level).
    pub time_remaining: f32,
    pub frame_count: u64,
    /// Previous frame's Space key state (for reliable edge detection).
    prev_jump_down: bool,
    /// Jump input buffer: frames remaining until buffered jump expires.
    jump_buffer: u8,
    pub hud: HudRenderer,
    pub screen_w: f32,
    pub screen_h: f32,
    pub current_level: u32,
    level_bounds: LevelBounds,
    /// Xorshift64 RNG state for question block loot randomization.
    rng_state: u64,
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

        // Spawn breakable bricks from level config
        let brick_spawns: Vec<BrickSpawn> = level.brick_spawns.clone();
        let bricks: Vec<Brick> = brick_spawns.iter()
            .map(|b| Brick { pos: Vec2 { x: b.pos.x, y: b.pos.y }, broken: false })
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
            power_ups: Vec::new(),
            bricks,
            brick_spawns,
            invuln_timer: 0.0,
            flicker_phase: 0.0,
            time_remaining: 300.0,
            frame_count: 0,
            prev_jump_down: false,
            jump_buffer: 0,
            hud: HudRenderer::new(),
            screen_w: 0.0,
            screen_h: 0.0,
            current_level: level_num,
            level_bounds: bounds,
            rng_state: Self::seed_rng(),
        }
    }

    /// Seed the xorshift64 RNG from system time (fallback: 1).
    fn seed_rng() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64 | 1)
            .unwrap_or(1)
    }

    /// Xorshift64: returns a pseudo-random f32 in [0.0, 1.0).
    fn next_rand(&mut self) -> f32 {
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        (x as f64 / u64::MAX as f64) as f32
    }

    /// Advance the simulation by one fixed timestep.
    ///
    /// Runs: invuln countdown → enemy patrol → player physics → hazard check → enemy check
    /// → flagpole check → checkpoint activation → event consumption. Transitions to Dead/Victory when triggered.
    /// Returns Some(GameState::Victory) when flagpole slide completes,
    /// Some(GameState::GameOver) when lives reach 0, or None to continue playing.
    pub fn update(&mut self, dt: f32) -> Option<GameState> {
        self.frame_count = self.frame_count.wrapping_add(1);

        // 0. Countdown timer — death when time runs out
        self.time_remaining -= dt;
        if self.time_remaining <= 0.0 {
            self.time_remaining = 0.0;
            self.player.lives = 0;
            return Some(GameState::GameOver(GameOverState::new(
                self.player.coins, self.current_level,
            )));
        }

        // 1. Advance invulnerability timer
        if self.invuln_timer > 0.0 {
            self.invuln_timer -= dt;
            self.flicker_phase += dt;
            if self.invuln_timer < 0.0 {
                self.invuln_timer = 0.0;
            }
        }

        // 1b. Advance star power timer
        if self.player.star_timer > 0.0 {
            self.player.star_timer -= dt;
            if self.player.star_timer < 0.0 {
                self.player.star_timer = 0.0;
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

        // 3b. PowerUp physics update (gravity, bounce, terrain collision)
        for pu in self.power_ups.iter_mut() {
            let pu_terrain = self.level.query_terrain(&pu.collider());
            pu.update(dt, &pu_terrain);
        }

        // 4. Update player physics with real keyboard input at runtime.
        // During tests (screen_w == 0.0), use default (no input) to avoid
        // panicking in Macroquad's is_key_down() which requires a GL context.
        let mut terrain = self.level.query_terrain(&self.player.collider());
        // Question blocks act as solid platforms (used or not)
        for block in &self.question_blocks {
            terrain.push(crate::level::Tile::Platform(block.collider()));
        }
        // Bricks are solid platforms too (unless broken)
        for brick in &self.bricks {
            if !brick.broken {
                terrain.push(crate::level::Tile::Platform(brick.collider()));
            }
        }
        let input = if self.screen_w > 0.0 {
            self.read_input()
        } else {
            crate::input::InputState::default()
        };
        // 4a. Hit detection: check block/brick/coin overlap BEFORE terrain
        // collision resolves and pushes the player away.
        let player_aabb = self.player.collider();

        // Coins
        for coin in self.coins.iter_mut() {
            if !coin.collected && player_aabb.intersects(&coin.collider()) {
                coin.collected = true;
                self.player.coins += 1;
            }
        }

        // Breakable bricks (Super/Fire only)
        // Same head-proximity fix as question blocks: vel.y check fails
        // because ceiling collision zeroes it in the previous frame.
        if !matches!(self.player.state, crate::entities::player::PlayerState::Small) {
            for brick in self.bricks.iter_mut() {
                if brick.broken {
                    continue;
                }
                let ba = brick.collider();
                let block_bottom = ba.y + ba.h;
                if player_aabb.intersects(&ba)
                    && (player_aabb.y - block_bottom).abs() <= 4.0
                {
                    brick.broken = true;
                    self.player.vel.y = 100.0;
                }
            }
        }

        // Question blocks
        // Hit detection uses head-proximity instead of velocity check,
        // because the previous frame's ceiling collision may have already
        // zeroed vel.y (player head pushed to block bottom = vel.y=0).
        // HEAD_TOLERANCE (4px) from Physics::question_block_check.
        const HEAD_TOLERANCE: f32 = 4.0;
        // Generate random roll once per frame (used if a block is hit)
        let block_roll = self.next_rand();
        for block in self.question_blocks.iter_mut() {
            if !block.used {
                let ba = block.collider();
                let block_bottom = ba.y + ba.h;
                if player_aabb.intersects(&ba)
                    && (player_aabb.y - block_bottom).abs() <= HEAD_TOLERANCE
                {
                    block.used = true;
                    self.player.vel.y = 100.0;
                    // Random roll from xorshift64 RNG (pre-generated before loop)
                    let roll = block_roll;
                    if roll < 0.65 {
                        self.player.coins += 1;
                    } else {
                        // Spawn a PowerUp entity that bounces out of the block
                        let kind = if roll < 0.80 {
                            PowerUpKind::SuperMushroom
                        } else if roll < 0.95 {
                            PowerUpKind::FireFlower
                        } else {
                            PowerUpKind::Starman
                        };
                        let spawn_pos = Vec2 {
                            x: ba.x + ba.w / 2.0,
                            y: ba.y,
                        };
                        self.power_ups.push(PowerUp::new(kind, spawn_pos));
                    }
                }
            }
        }

        // 4b. Apply player physics AFTER hit detection (so blocks can be hit
        // before terrain collision pushes the player away).
        self.player.update(dt, &input, &terrain);

        // 5. Hazard check: if player is NOT invulnerable and hazards exist → damage
        let kill_y = self.level_bounds.kill_y;
        let events = Physics::hazard_check(&self.player, &terrain, kill_y);

        if !events.is_empty() && self.invuln_timer <= 0.0 && self.player.star_timer <= 0.0 && self.player.lives > 0 {
            // take_damage(): true = fatal (Small dies), false = downgrade (Super/Fire → Small)
            if self.player.take_damage() {
                self.player.lives -= 1;
            }
            self.invuln_timer = 2.0; // ~2s invulnerability window
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
                    if self.invuln_timer <= 0.0 && self.player.star_timer <= 0.0 && self.player.lives > 0 =>
                {
                    // take_damage(): true = fatal (Small dies), false = downgrade
                    if self.player.take_damage() {
                        self.player.lives -= 1;
                    }
                    self.invuln_timer = 2.0;
                }
                _ => {}
            }
        }

        // 7b. PowerUp collection: check player overlap with each power-up
        let player_col = self.player.collider();
        let mut collected_indices: Vec<usize> = Vec::new();
        for (i, pu) in self.power_ups.iter().enumerate() {
            if player_col.intersects(&pu.collider()) {
                collected_indices.push(i);
                match pu.kind {
                    PowerUpKind::SuperMushroom => {
                        self.player.apply_powerup(crate::entities::player::PlayerState::Super);
                    }
                    PowerUpKind::FireFlower => {
                        self.player.apply_powerup(crate::entities::player::PlayerState::Fire);
                    }
                    PowerUpKind::Starman => {
                        self.player.activate_star();
                    }
                    PowerUpKind::Coin => {} // coins are handled separately
                }
            }
        }
        // Remove collected power-ups (reverse order to preserve indices)
        for i in collected_indices.into_iter().rev() {
            self.power_ups.remove(i);
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
            return Some(GameState::GameOver(GameOverState::new(self.player.coins, self.current_level)));
        }

        None // Continue playing
    }

    /// Reads keyboard state with reliable edge detection.
    ///
    /// Uses own prev_jump_down tracking instead of Macroquad's is_key_pressed()
    /// which can miss edges at variable frame rates. Also implements a 6-frame
    /// jump buffer: if Space is pressed while airborne, the jump triggers
    /// automatically upon landing within the buffer window (~100ms).
    fn read_input(&mut self) -> crate::input::InputState {
        use macroquad::input::{is_key_down, is_key_pressed, KeyCode};

        let jump_down = is_key_down(KeyCode::Space);

        // Edge detection: rising edge on Space
        let jump_just_now = jump_down && !self.prev_jump_down;

        // Feed the jump buffer: if Space pressed while airborne, remember it
        if jump_just_now {
            self.jump_buffer = 6; // ~100ms at 60fps
        }

        // Consume buffer: trigger jump_just on the first grounded frame after buffered press
        let buffered_jump = self.jump_buffer > 0 && self.player.on_ground;

        self.prev_jump_down = jump_down;

        // Decrement buffer each frame
        if self.jump_buffer > 0 && self.player.on_ground {
            self.jump_buffer = 0; // consumed
        } else if self.jump_buffer > 0 {
            self.jump_buffer -= 1;
        }

        crate::input::InputState {
            left: is_key_down(KeyCode::Left) || is_key_down(KeyCode::A),
            right: is_key_down(KeyCode::Right) || is_key_down(KeyCode::D),
            jump: jump_down,
            jump_just: jump_just_now || buffered_jump,
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

        // ── 5b. Breakable bricks (brown, or gone if broken) ──
        let brick_color = macroquad::color::Color::new(0.65, 0.40, 0.20, 1.0);
        for brick in &self.bricks {
            if !brick.broken {
                let (sbx, sby) = ws(brick.pos.x, brick.pos.y);
                draw_rectangle(sbx, sby, 32.0 * sx, 32.0 * sy, brick_color);
                // Brick lines
                let line_color = macroquad::color::Color::new(0.35, 0.20, 0.10, 1.0);
                draw_rectangle(sbx, sby + 15.0 * sy, 32.0 * sx, 2.0 * sy, line_color);
                draw_rectangle(sbx + 15.0 * sx, sby, 2.0 * sx, 15.0 * sy, line_color);
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

        // ── 8b. Power-ups (mushroom = green, flower = red/orange) ──
        for pu in &self.power_ups {
            let (px, py) = ws(pu.pos.x, pu.pos.y);
            let half = 8.0 * sx.min(sy);
            match pu.kind {
                PowerUpKind::SuperMushroom => {
                    // Green mushroom cap
                    draw_rectangle(px - half, py - half, half * 2.0, half * 2.0,
                        macroquad::color::GREEN);
                    // White spots
                    draw_circle(px - 3.0 * sx, py - 3.0 * sy, 2.0 * sx.min(sy),
                        macroquad::color::WHITE);
                    draw_circle(px + 3.0 * sx, py + 3.0 * sy, 2.0 * sx.min(sy),
                        macroquad::color::WHITE);
                }
                PowerUpKind::FireFlower => {
                    // Orange/red flower
                    draw_circle(px, py, half, macroquad::color::Color::new(1.0, 0.4, 0.0, 1.0));
                    draw_circle(px, py, half * 0.5, macroquad::color::YELLOW);
                }
                PowerUpKind::Starman => {
                    // Yellow star — draw a simple star shape with cross + diagonals
                    draw_circle(px, py, half, macroquad::color::YELLOW);
                    draw_text("*", px - half * 0.5, py + half * 0.5, half * 1.6, macroquad::color::BLACK);
                }
                PowerUpKind::Coin => {}
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

        // Invulnerability flicker (damage) & star power rainbow
        let flicker = self.invuln_timer > 0.0 && ((self.flicker_phase * 4.0) as u32) % 2 == 1;
        let star_active = self.player.star_timer > 0.0;

        if !flicker {
            // Rainbow cycling during star power
            let (hat_color, overall_color, skin_color) = if star_active {
                let hue = ((self.frame_count as f32 * 0.05) % 1.0) * 6.2832; // cycle ~1s
                let r = (hue.sin() * 0.5 + 0.5).clamp(0.0, 1.0);
                let g = ((hue + 2.094).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
                let b = ((hue + 4.189).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
                let rainbow = macroquad::color::Color::new(r, g, b, 1.0);
                (rainbow, rainbow, rainbow)
            } else {
                (
                    macroquad::color::Color::new(0.85, 0.15, 0.1, 1.0),
                    macroquad::color::Color::new(0.1, 0.3, 0.9, 1.0),
                    macroquad::color::Color::new(1.0, 0.75, 0.55, 1.0),
                )
            };
            let shoe_color = macroquad::color::Color::new(0.45, 0.25, 0.15, 1.0);
            let eye_color = macroquad::color::BLACK;
            let button_color = macroquad::color::YELLOW;
            // Draw from foot upward: feet at pos.y, body extends up by body_h
            let top = spy - body_h * sy;

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
        let font_size = 14.0 * sx.min(sy);
        let stats = self.player.stats();
        draw_text(
            &format!("WORLD 1-{}", self.current_level),
            8.0,
            22.0 * sy,
            font_size,
            macroquad::color::WHITE,
        );
        draw_text(
            &format!("COINS:{}  LIVES:{}", stats.coins, stats.lives),
            8.0,
            40.0 * sy,
            font_size,
            macroquad::color::WHITE,
        );
        // Time countdown
        let time_color = if self.time_remaining <= 60.0 {
            macroquad::color::Color::new(1.0, 0.2, 0.2, 1.0) // red when urgent
        } else {
            macroquad::color::WHITE
        };
        draw_text(
            &format!("TIME:{}", self.time_remaining as u32),
            8.0,
            58.0 * sy,
            font_size,
            time_color,
        );
        // Star power countdown (only visible when active)
        if self.player.star_timer > 0.0 {
            let star_color = macroquad::color::Color::new(1.0, 0.85, 0.0, 1.0); // gold
            draw_text(
                &format!("STAR:{:.1}", self.player.star_timer),
                8.0,
                76.0 * sy,
                font_size,
                star_color,
            );
        }
    }
}

/// Breakable brick entity. Solid platform; shatters when Super/Fire Mario
/// hits it from below. Small Mario just bounces off.
#[derive(Debug, Clone)]
pub struct Brick {
    pub pos: Vec2,
    pub broken: bool,
}

impl Brick {
    pub fn collider(&self) -> crate::level::AABB {
        crate::level::AABB { x: self.pos.x, y: self.pos.y, w: 32.0, h: 32.0 }
    }
}
