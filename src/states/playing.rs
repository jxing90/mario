// Feature #6: Life, Death & Win — PlayingState
// Design Reference: docs/features/6-life-death-win.md §4, §6, §8
//
// The primary gameplay state. Manages player physics, hazard/flagpole/checkpoint
// detection, invulnerability timers, and triggers transitions to Dead/Victory.

use crate::audio::SoundManager;
use crate::entities::coin::Coin;
use crate::entities::dart::Dart;
use crate::entities::dart_enemy::DartEnemy;
use crate::entities::enemy::{Enemy, EnemyConfig};
use crate::entities::fireball::Fireball;
use crate::entities::osc_fireball::OscFireball;
use crate::entities::player::Player;
use crate::entities::flagpole::{Flagpole, FlagpolePhase};
use crate::entities::brick::Brick;
use crate::entities::checkpoint::Checkpoint;
use crate::entities::power_up::{PowerUp, PowerUpKind};
use crate::entities::question_block::QuestionBlock;
use crate::level::{BrickSpawn, Level, LevelBounds, Vec2};
use crate::systems::camera::Camera;
use crate::systems::camera::CameraConfig;
use crate::systems::hud::HudRenderer;
use crate::systems::physics::{CollisionEvent, Physics};
use crate::states::{GameState, LifeState, VictoryState, GameOverState, DeadState};

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
    /// Active fireball projectiles (Fire state only).
    pub fireballs: Vec<Fireball>,
    /// Fireball cooldown timer (seconds until next shot allowed).
    fireball_cooldown: f32,
    /// Dart-throwing enemies (stationary turrets).
    pub dart_enemies: Vec<DartEnemy>,
    /// Active dart projectiles thrown by dart enemies.
    pub darts: Vec<Dart>,
    /// Oscillating fireball hazards (vertical movement).
    pub osc_fireballs: Vec<OscFireball>,
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
    /// Previous frame's sprint key state (for fireball edge detection).
    prev_sprint_down: bool,
    pub hud: HudRenderer,
    pub screen_w: f32,
    pub screen_h: f32,
    pub current_level: u32,
    level_bounds: LevelBounds,
    /// Xorshift64 RNG state for question block loot randomization.
    rng_state: u64,
    /// Sound effects manager (None during tests — no audio backend).
    pub sfx: Option<SoundManager>,
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
        let mut cam_cfg = CameraConfig::default();
        cam_cfg.viewport_w = 800.0;
        cam_cfg.viewport_h = 380.0;
        let camera = Camera::new(cam_cfg);
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
            .map(|b| Brick::new(Vec2 { x: b.pos.x, y: b.pos.y }))
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

        // Spawn dart enemies from level config
        let dart_enemies: Vec<DartEnemy> = level.dart_enemy_spawns.iter()
            .map(|d| DartEnemy::new(Vec2 { x: d.x, y: d.y }))
            .collect();

        // Spawn oscillating fireballs from level config
        let osc_fireballs: Vec<OscFireball> = level.osc_fireball_spawns.iter()
            .map(|o| OscFireball::new(o.x, o.top_y, o.bottom_y))
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
            fireballs: Vec::new(),
            fireball_cooldown: 0.0,
            dart_enemies,
            darts: Vec::new(),
            osc_fireballs,
            bricks,
            brick_spawns,
            invuln_timer: 0.0,
            flicker_phase: 0.0,
            time_remaining: 300.0,
            frame_count: 0,
            prev_jump_down: false,
            jump_buffer: 0,
            prev_sprint_down: false,
            hud: HudRenderer::new(),
            screen_w: 0.0,
            screen_h: 0.0,
            current_level: level_num,
            level_bounds: bounds,
            rng_state: Self::seed_rng(),
            sfx: None,
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

    /// Play a sound effect if the sound manager is initialized.
    fn sfx(&mut self, play: fn(&mut SoundManager)) {
        if let Some(ref mut s) = self.sfx {
            play(s);
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

        // Lazy-init sound effects on first real frame (requires audio backend)
        if self.screen_w > 0.0 && self.sfx.is_none() {
            self.sfx = Some(SoundManager::new());
        }

        // 0. Countdown timer — death when time runs out
        self.time_remaining -= dt;
        if self.time_remaining <= 0.0 {
            self.time_remaining = 0.0;
            self.player.lives = 0;
            self.sfx(SoundManager::play_gameover);
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
                self.sfx(SoundManager::play_victory);
                return Some(GameState::Victory(VictoryState::new(
                    self.player.coins, self.player.lives,
                    self.player.state, self.current_level,
                )));
            }
            return None; // Input locked, no player update during slide
        }

        // 3. Enemy patrol update (with terrain for gravity and cliff detection)
        let enemy_terrain: Vec<crate::level::Tile> = self.level.platforms()
            .iter()
            .map(|p| crate::level::Tile::Platform(p.aabb))
            .collect();
        for enemy in self.enemies.iter_mut() {
            enemy.update(dt, &enemy_terrain);
        }

        // 3a. Brick particle update (broken brick debris animation)
        for brick in self.bricks.iter_mut() {
            brick.update_particles(dt);
        }

        // 3b. PowerUp physics update (gravity, bounce, terrain collision)
        for pu in self.power_ups.iter_mut() {
            let pu_terrain = self.level.query_terrain(&pu.collider());
            pu.update(dt, &pu_terrain);
        }

        // 3c. Fireball cooldown timer
        if self.fireball_cooldown > 0.0 {
            self.fireball_cooldown -= dt;
        }

        // 3d. Fireball spawn: Shift edge-detect (only Fire state, max 2 on screen)
        let sprint_down = self.screen_w > 0.0 && macroquad::input::is_key_down(macroquad::input::KeyCode::LeftShift);
        let sprint_just_pressed = sprint_down && !self.prev_sprint_down;
        self.prev_sprint_down = sprint_down;
        let is_fire = matches!(self.player.state, crate::entities::player::PlayerState::Fire);
        if sprint_just_pressed && is_fire && self.fireball_cooldown <= 0.0 && self.fireballs.len() < 2 {
            let offset_x = self.player.facing as f32 * 12.0; // spawn ahead of player
            let spawn_pos = Vec2 {
                x: self.player.pos().x + offset_x,
                y: self.player.pos().y - 8.0, // chest height
            };
            self.fireballs.push(Fireball::new(spawn_pos, self.player.facing));
            self.fireball_cooldown = 0.35; // ~170ms cooldown, ~3 shots/sec
        }

        // 3e. Fireball update (move, lifetime)
        for fb in self.fireballs.iter_mut() {
            // Horizontal movement
            fb.pos.x += fb.vel.x * dt;
            fb.update(dt);
        }
        // Check fireball-enemy collisions
        let mut fb_killed_de = false;
        for fb in self.fireballs.iter_mut() {
            if !fb.alive {
                continue;
            }
            for enemy in self.enemies.iter_mut() {
                if !enemy.alive {
                    continue;
                }
                if fb.collider().intersects(&enemy.collider()) {
                    fb.kill();
                    enemy.kill();
                    break;
                }
            }
            // Also check fireball-dart_enemy collisions
            for de in self.dart_enemies.iter_mut() {
                if !de.alive || !fb.alive {
                    continue;
                }
                if fb.collider().intersects(&de.collider()) {
                    fb.kill();
                    de.kill();
                    fb_killed_de = true;
                    break;
                }
            }
        }
        if fb_killed_de {
            self.sfx(SoundManager::play_stomp);
        }
        // Clean up dead fireballs
        self.fireballs.retain(|fb| fb.alive);

        // 3f. Dart enemy update + shooting
        let player_x = self.player.pos().x;
        let player_y = self.player.pos().y;
        let mut dart_fired = false;
        for de in self.dart_enemies.iter_mut() {
            let shoot_now = de.update(dt, player_x);
            if shoot_now {
                // Spawn a dart from the enemy toward the player's current position
                let target = Vec2 { x: player_x, y: player_y };
                let spawn = de.spawn_pos();
                self.darts.push(Dart::new(spawn, target));
                dart_fired = true;
            }
        }
        if dart_fired {
            self.sfx(SoundManager::play_fireball);
        }

        // 3g. Dart update + terrain collision (darts die on platform/brick/block contact)
        let mut dart_terrain: Vec<crate::level::Tile> = self.level.platforms()
            .iter()
            .map(|p| crate::level::Tile::Platform(p.aabb))
            .collect();
        // Question blocks also block darts (used or not)
        for block in &self.question_blocks {
            dart_terrain.push(crate::level::Tile::Platform(block.collider()));
        }
        // Bricks block darts too (unless shattered)
        for brick in &self.bricks {
            if !brick.broken {
                dart_terrain.push(crate::level::Tile::Platform(brick.collider()));
            }
        }
        for dart in self.darts.iter_mut() {
            if !dart.alive { continue; }
            dart.update(dt);
            // Check terrain collision
            for tile in &dart_terrain {
                if let crate::level::Tile::Platform(p) = tile {
                    if dart.collider().intersects(p) {
                        dart.kill();
                        break;
                    }
                }
            }
        }
        // Clean up dead darts
        self.darts.retain(|d| d.alive);

        // 3h. Oscillating fireball update
        for ofb in self.osc_fireballs.iter_mut() {
            ofb.update(dt);
        }

        // 3i. Fireball vs osc-fireball: player fireballs can destroy them
        for fb in self.fireballs.iter_mut() {
            if !fb.alive { continue; }
            for ofb in self.osc_fireballs.iter_mut() {
                if !ofb.alive { continue; }
                if fb.collider().intersects(&ofb.collider()) {
                    fb.kill();
                    ofb.kill();
                    break;
                }
            }
        }

        // 4. Update player physics with real keyboard input at runtime.
        // During tests (screen_w == 0.0), use default (no input) to avoid
        // panicking in Macroquad's is_key_down() which requires a GL context.
        //
        // Build terrain with ALL level platforms unconditionally — not just
        // those intersecting the player's current collider. query_terrain
        // filtered by the PREVIOUS frame's position, so a platform the player
        // moved into this frame would be missing and the collision would pass
        // through. Also include spikes, question blocks, and bricks.
        let mut terrain: Vec<crate::level::Tile> = self.level.platforms()
            .iter()
            .map(|p| crate::level::Tile::Platform(p.aabb))
            .collect();
        for spike in self.level.spikes() {
            terrain.push(crate::level::Tile::Spike(spike.collider()));
        }
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
        // Jump sound on rising edge
        if input.jump_just {
            self.sfx(SoundManager::play_jump);
        }

        // 4a. Advance player position (forces + integration, no collision yet).
        //     Check block / brick / coin AFTER movement so the player's head
        //     actually enters the block before ceiling collision pushes it back.
        self.player.advance_position(dt, &input);

        // Coins — check with post-move collider
        let mut coin_collected = false;
        let post_aabb = self.player.collider();
        for coin in self.coins.iter_mut() {
            if !coin.collected && post_aabb.intersects(&coin.collider()) {
                coin.collected = true;
                self.player.coins += 1;
                coin_collected = true;
            }
        }
        if coin_collected {
            self.sfx(SoundManager::play_coin);
        }

        // Breakable bricks (Super/Fire only, must be hit from below)
        let mut brick_hit = false;
        if !matches!(self.player.state, crate::entities::player::PlayerState::Small) {
            for brick in self.bricks.iter_mut() {
                if brick.broken {
                    continue;
                }
                let ba = brick.collider();
                let block_bottom = ba.y + ba.h;
                if post_aabb.intersects(&ba)
                    && post_aabb.y >= block_bottom - 8.0
                    && post_aabb.y <= block_bottom
                    && self.player.pos.y >= block_bottom
                    && self.player.pos.x > ba.x - 4.0
                    && self.player.pos.x < ba.x + ba.w + 4.0
                {
                    brick.shatter();
                    self.player.vel.y = 100.0;
                    brick_hit = true;
                }
            }
        }
        if brick_hit {
            self.sfx(SoundManager::play_bump);
        }

        // Question blocks — only activate when hit from below.
        // After advance_position, the player's collider CAN intersect the block
        // (ceiling collision hasn't pushed them out yet).
        // HEAD_TOLERANCE must cover per-frame displacement (~6px at 350px/s, 60fps)
        // so the player's head doesn't skip the detection zone.
        const HEAD_TOLERANCE: f32 = 8.0;
        let block_roll = self.next_rand();
        let mut block_hit = false;
        for block in self.question_blocks.iter_mut() {
            if !block.used {
                let ba = block.collider();
                let block_bottom = ba.y + ba.h;
                if post_aabb.intersects(&ba)
                    && post_aabb.y >= block_bottom - HEAD_TOLERANCE
                    && post_aabb.y <= block_bottom
                    && self.player.pos.y >= block_bottom
                    && self.player.pos.x > ba.x - 4.0
                    && self.player.pos.x < ba.x + ba.w + 4.0
                {
                    block.used = true;
                    self.player.vel.y = 100.0;
                    block_hit = true;
                    // Random roll from xorshift64 RNG (pre-generated before loop)
                    let roll = block_roll;
                    if roll < 0.60 {
                        self.player.coins += 1;
                    } else {
                        // Spawn a PowerUp entity that bounces out of the block
                        let kind = if roll < 0.75 {
                            PowerUpKind::SuperMushroom
                        } else if roll < 0.90 {
                            PowerUpKind::FireFlower
                        } else if roll < 0.95 {
                            PowerUpKind::Starman
                        } else {
                            PowerUpKind::OneUpMushroom
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
        if block_hit {
            self.sfx(SoundManager::play_bump);
        }

        // 4b. Resolve terrain collisions (ceiling / wall / floor).
        //     This pushes the player out of blocks after activation.
        self.player.resolve_collisions(dt, &terrain);
        self.player.update_facing_from_input(&input);

        // 5. Hazard check: if player is NOT invulnerable and hazards exist → damage
        let kill_y = self.level_bounds.kill_y;
        let events = Physics::hazard_check(&self.player, &terrain, kill_y);

        if !events.is_empty() && self.invuln_timer <= 0.0 && self.player.star_timer <= 0.0 && self.player.lives > 0 {
            // take_damage(): true = fatal (Small dies), false = downgrade (Super/Fire → Small)
            let fatal = self.player.take_damage();
            if fatal {
                self.sfx(SoundManager::play_death);
                self.player.lives -= 1;
                return Some(GameState::Dead(DeadState::new(
                    self.player.lives,
                    self.player.coins,
                    self.current_level,
                    self.life_state.checkpoint,
                    self.player.pos(),
                    self.screen_w,
                    self.screen_h,
                )));
            }
            self.sfx(SoundManager::play_damage);
            self.invuln_timer = 2.0; // ~2s invulnerability window
        }

        // 5a. Dart-player collision check
        {
            let player_col = self.player.collider();
            for dart in self.darts.iter_mut() {
                if !dart.alive { continue; }
                if !player_col.intersects(&dart.collider()) { continue; }
                // Star power: destroy dart on contact (no damage)
                if self.player.star_timer > 0.0 {
                    dart.kill();
                    continue;
                }
                if self.invuln_timer <= 0.0 && self.player.lives > 0 {
                    dart.kill();
                    let fatal = self.player.take_damage();
                    if fatal {
                        self.player.lives -= 1;
                        return Some(GameState::Dead(DeadState::new(
                            self.player.lives,
                            self.player.coins,
                            self.current_level,
                            self.life_state.checkpoint,
                            self.player.pos(),
                            self.screen_w,
                            self.screen_h,
                        )));
                    }
                    self.invuln_timer = 2.0;
                }
            }
        }

        // 5b. Oscillating fireball-player collision check
        {
            let player_col = self.player.collider();
            for ofb in self.osc_fireballs.iter_mut() {
                if !ofb.alive { continue; }
                if !player_col.intersects(&ofb.collider()) { continue; }
                // Star power: destroy oscillating fireball on contact (no damage)
                if self.player.star_timer > 0.0 {
                    ofb.kill();
                    continue;
                }
                if self.invuln_timer <= 0.0 && self.player.lives > 0 {
                    let fatal = self.player.take_damage();
                    if fatal {
                        self.player.lives -= 1;
                        return Some(GameState::Dead(DeadState::new(
                            self.player.lives,
                            self.player.coins,
                            self.current_level,
                            self.life_state.checkpoint,
                            self.player.pos(),
                            self.screen_w,
                            self.screen_h,
                        )));
                    }
                    self.invuln_timer = 2.0;
                }
            }
        }

        // 6. Enemy collision detection (Step D: enemy_check)
        let enemy_events = Physics::enemy_check(&self.player, &self.enemies, dt);

        // 7. Consume enemy collision events (Step E)
        for event in enemy_events {
            match event {
                CollisionEvent::EnemyStomp(i) => {
                    self.enemies[i].alive = false;
                    self.player.vel.y = self.enemies[i].config.bounce_velocity;
                    self.sfx(SoundManager::play_stomp);
                }
                // Star power: kill enemies on contact
                CollisionEvent::EnemyContact(i) if self.player.star_timer > 0.0 => {
                    self.enemies[i].alive = false;
                    self.sfx(SoundManager::play_stomp);
                }
                CollisionEvent::EnemyContact(_)
                    if self.invuln_timer <= 0.0 && self.player.star_timer <= 0.0 && self.player.lives > 0 =>
                {
                    // take_damage(): true = fatal (Small dies), false = downgrade
                    let fatal = self.player.take_damage();
                    if fatal {
                        self.sfx(SoundManager::play_death);
                        self.player.lives -= 1;
                        return Some(GameState::Dead(DeadState::new(
                            self.player.lives,
                            self.player.coins,
                            self.current_level,
                            self.life_state.checkpoint,
                            self.player.pos(),
                            self.screen_w,
                            self.screen_h,
                        )));
                    }
                    self.invuln_timer = 2.0;
                }
                _ => {}
            }
        }

        // 7a. Dart enemy collision (stomp + contact, same logic as regular enemies)
        {
                let player_bottom_val = {
                    let pc = self.player.collider();
                    pc.y + pc.h
                };
                let prev_player_bottom = player_bottom_val - self.player.vel.y * dt;
                let player_vy = self.player.vel.y;
                let player_col = self.player.collider();
                let mut stomped = false;
                let mut damaged = false;
                let mut star_killed = false;
                for (_i, de) in self.dart_enemies.iter_mut().enumerate() {
                    if !de.alive {
                        continue;
                    }
                    let de_col = de.collider();
                    if !player_col.intersects(&de_col) {
                        continue;
                    }

                    let enemy_top = de_col.y;
                    let is_stomp = player_vy > 0.0 && prev_player_bottom <= enemy_top + 2.0;

                    if is_stomp {
                        de.kill();
                        self.player.vel.y = -200.0;
                        stomped = true;
                    } else if self.player.star_timer > 0.0 {
                        // Star power: kill dart enemy on contact
                        de.kill();
                        star_killed = true;
                    } else {
                        damaged = true;
                    }
                }
                if stomped || star_killed {
                    self.sfx(SoundManager::play_stomp);
                }
                if damaged && self.invuln_timer <= 0.0
                    && self.player.star_timer <= 0.0
                    && self.player.lives > 0
                {
                    let fatal = self.player.take_damage();
                    if fatal {
                        self.sfx(SoundManager::play_death);
                        self.player.lives -= 1;
                        return Some(GameState::Dead(DeadState::new(
                            self.player.lives,
                            self.player.coins,
                            self.current_level,
                            self.life_state.checkpoint,
                            self.player.pos(),
                            self.screen_w,
                            self.screen_h,
                        )));
                    }
                    self.invuln_timer = 2.0;
                    self.sfx(SoundManager::play_damage);
                }
            }

        // 7b. PowerUp collection: check player overlap with each power-up
        let player_col = self.player.collider();
        let mut collected_indices: Vec<usize> = Vec::new();
        let mut powerup_collected = false;
        let mut oneup_collected = false;
        for (i, pu) in self.power_ups.iter().enumerate() {
            if player_col.intersects(&pu.collider()) {
                collected_indices.push(i);
                powerup_collected = true;
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
                    PowerUpKind::OneUpMushroom => {
                        oneup_collected = true;
                        self.player.lives += 1;
                    }
                    PowerUpKind::Coin => {} // coins are handled separately
                }
            }
        }
        if powerup_collected {
            self.sfx(SoundManager::play_powerup);
        }
        if oneup_collected {
            self.sfx(SoundManager::play_oneup);
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
            down: is_key_down(KeyCode::Down) || is_key_down(KeyCode::S),
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

        // ── 3. Spikes — triangular danger zones (from level data) ──
        for spike in self.level.spikes() {
            spike.draw(sx, sy, &ws, spike_color);
        }

        // ── 4. Coins (yellow circles, only uncollected) ──
        for coin in &self.coins {
            coin.draw(sx, sy, &ws);
        }

        // ── 5. Question blocks ──
        for block in &self.question_blocks {
            block.draw(sx, sy, &ws);
        }

        // ── 5b. Breakable bricks ──
        for brick in &self.bricks {
            brick.draw(sx, sy, &ws);
        }

        // ── 6. Checkpoints ──
        for cp in &self.checkpoints {
            cp.draw(sx, sy, &ws);
        }

        // ── 7. Flagpole ──
        self.flagpole.draw(sx, sy, &ws);

        // ── 8. Enemies ──
        for enemy in &self.enemies {
            enemy.draw(sx, sy, &ws);
        }

        // ── 8d. Dart enemies (turrets) ──
        for de in &self.dart_enemies {
            de.draw(sx, sy, &ws);
        }

        // ── 8b. Power-ups ──
        for pu in &self.power_ups {
            pu.draw(sx, sy, &ws);
        }

        // ── 8c. Fireballs ──
        for fb in &self.fireballs {
            fb.draw(sx, sy, &ws);
        }

        // ── 8e. Darts (thrown projectiles) ──
        for dart in &self.darts {
            dart.draw(sx, sy, &ws);
        }

        // ── 8f. Oscillating fireballs ──
        for ofb in &self.osc_fireballs {
            ofb.draw(sx, sy, &ws);
        }

        // ── 9. Player (Mario) ──
        use macroquad::shapes::draw_circle;
        use macroquad::text::draw_text;
        let p = self.player.pos();
        let (spx, spy) = ws(p.x, p.y);
        let is_small = matches!(self.player.state, crate::entities::player::PlayerState::Small);
        let body_w = if is_small { 16.0 } else { 32.0 };
        // When crouching, the body height shrinks to 16 while width stays the same.
        let body_h = if self.player.crouching { 16.0 } else { body_w };
        // Vertical scale: uses actual body height. Horizontal scale: uses body width.
        let unit_v = body_h / 16.0;
        let sx_s = sx * body_w / 16.0;
        let sy_s = sy * unit_v;

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
            // Draw from foot upward: feet at pos.y, body extends up by body_h.
            // Offset spx left by half the body width so the sprite is centered
            // on pos.x (matching the collider which is also centered on pos.x).
            let top = spy - body_h * sy;
            // Center sprite on pos.x to match centered collider (x = pos.x - w/2).
            let spx = spx - body_w / 2.0 * sx;

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

        // ── 10. HUD (screen-space overlay, horizontal) ──
        let font_size = 14.0 * sx.min(sy);
        let stats = self.player.stats();
        let y_pos = 12.0 * sy; // single row near top
        
        // Column 1: World
        draw_text(
            &format!("WORLD 1-{}", self.current_level),
            8.0,
            y_pos,
            font_size,
            macroquad::color::WHITE,
        );
        // Column 2: Coins
        draw_text(
            &format!("COINS {}", stats.coins),
            150.0 * sx,
            y_pos,
            font_size,
            macroquad::color::Color::new(1.0, 0.85, 0.0, 1.0),
        );
        // Column 3: Lives
        draw_text(
            &format!("LIVES {}", stats.lives),
            280.0 * sx,
            y_pos,
            font_size,
            macroquad::color::Color::new(1.0, 0.3, 0.3, 1.0),
        );
        // Column 4: Time
        let time_color = if self.time_remaining <= 60.0 {
            macroquad::color::Color::new(1.0, 0.2, 0.2, 1.0) // red when urgent
        } else {
            macroquad::color::WHITE
        };
        draw_text(
            &format!("TIME {}", self.time_remaining as u32),
            430.0 * sx,
            y_pos,
            font_size,
            time_color,
        );
        // Star power countdown (inline when active)
        if self.player.star_timer > 0.0 {
            let star_color = macroquad::color::Color::new(1.0, 0.85, 0.0, 1.0);
            draw_text(
                &format!("STAR {:.1}", self.player.star_timer),
                560.0 * sx,
                y_pos,
                font_size,
                star_color,
            );
        }
    }
}
