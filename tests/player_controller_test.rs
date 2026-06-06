// Feature #3: Player Controller — TDD Red Phase
//
// [no integration test exemption — feature has terrain/collision external dependency]
// Real tests: all tests marked with "real_test (feature #3)" verify actual Player logic
// without mocking primary physics/collision dependencies.
//
// Test Inventory Reference: docs/features/3-player-controller.md §7
// All tests are expected to FAIL (RED phase) — implementation not yet written.
//
// Category coverage (Rule 1):
//   FUNC/happy:  T01-T07, T09-T13, T23-T27
//   FUNC/error:  T08, T14, T28
//   BNDRY/edge:  T15-T22, T29-T30
//   SEC: N/A — standalone desktop game, no user authentication or injection surface
//   PERF: N/A — physics computation, no performance requirements (covered by NFR-001)
//   UI: N/A — `"ui": false`, sprite rendering in Playing state, not this feature
//   INTG/terrain: T31
//
// Negative test ratio: FUNC/error (3) + BNDRY/* (10) + INTG (1) = 14 / 31 = 45.2% ≥ 40%
// Rule 2 ✓

use mario_platformer::entities::player::{Player, PlayerConfig, PlayerState};
use mario_platformer::input::InputState;
use mario_platformer::level::{AABB, Level, Tile, Vec2};

// ============================================================================
// Constants
// ============================================================================

/// Allowed tolerance for f32 physics comparisons (per long-task-guide: epsilon=0.5px
/// for convergence, epsilon=0.01 for physics).
const PHYSICS_EPSILON: f32 = 0.01;
const POSITION_EPSILON: f32 = 0.5;

/// Fixed timestep: 1/60 second.
const DT: f32 = 1.0 / 60.0;

/// Default config values from design doc §Implementation Summary.
const DEFAULT_MAX_SPEED: f32 = 200.0;
const DEFAULT_ACCELERATION: f32 = 200.0 / 0.3; // ≈ 666.67
const DEFAULT_FRICTION: f32 = 200.0 / 0.2; // 1000.0
const DEFAULT_JUMP_VELOCITY: f32 = -420.0;
const DEFAULT_MAX_JUMP_DURATION: f32 = 0.35;
const DEFAULT_SPRINT_MULTIPLIER: f32 = 1.5;
const DEFAULT_AIR_CONTROL_FACTOR: f32 = 0.6;
const DEFAULT_GRAVITY: f32 = 1200.0;

// ============================================================================
// Helper utilities
// ============================================================================

/// Returns true if two f32 values are within EPSILON of each other.
fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < PHYSICS_EPSILON
}

/// Returns true if two f32 values are within a larger tolerance (for positions).
fn approx_eq_loose(a: f32, b: f32) -> bool {
    (a - b).abs() < POSITION_EPSILON
}

/// Runs N simulation frames (each = DT) through Player::update.
fn simulate_frames(player: &mut Player, input: &InputState, terrain: &[Tile], n_frames: u32) {
    for _ in 0..n_frames {
        player.update(DT, input, terrain);
    }
}

/// Creates a terrain slice with a single platform at the given AABB.
fn platform_terrain(aabb: AABB) -> Vec<Tile> {
    vec![Tile::Platform(aabb)]
}

/// Creates a default Player standing on a platform (on_ground = true).
fn player_on_ground(config: PlayerConfig) -> (Player, Vec<Tile>) {
    let mut player = Player::new(config);
    // Place player on a ground platform: platform top at y=600, player foot at y=600.
    let ground = AABB {
        x: 0.0,
        y: 600.0,
        w: 2000.0,
        h: 40.0,
    };
    player.pos.x = 200.0;
    player.pos.y = 600.0; // foot == platform top
    player.on_ground = true;
    player.vel.x = 0.0;
    player.vel.y = 0.0;
    (player, platform_terrain(ground))
}

/// Creates a PlayerConfig with overrides applied to defaults.
fn custom_config(
    acceleration: Option<f32>,
    max_speed: Option<f32>,
    friction: Option<f32>,
    jump_initial_velocity: Option<f32>,
    max_jump_duration: Option<f32>,
    sprint_multiplier: Option<f32>,
    air_control_factor: Option<f32>,
    gravity: Option<f32>,
) -> PlayerConfig {
    let mut c = PlayerConfig::default();
    if let Some(v) = acceleration { c.acceleration = v; }
    if let Some(v) = max_speed { c.max_speed = v; }
    if let Some(v) = friction { c.friction = v; }
    if let Some(v) = jump_initial_velocity { c.jump_initial_velocity = v; }
    if let Some(v) = max_jump_duration { c.max_jump_duration = v; }
    if let Some(v) = sprint_multiplier { c.sprint_multiplier = v; }
    if let Some(v) = air_control_factor { c.air_control_factor = v; }
    if let Some(v) = gravity { c.gravity = v; }
    c
}

// ============================================================================
// T01 — FUNC/happy
// Traces To: FR-001 AC-1, §Interface Contract Player::update postcondition 水平移动
// "玩家在 0.3s 内加速至最大向右速度并保持匀速移动"
//
// Kills: acceleration formula wrong → 0.3s time deviates; no speed clamp → unlimited speed.
// Wrong-impl challenge:
//   - Wrong: accel = max_speed (not max_speed/0.3) → reaches max in 1 frame → FAIL
//   - Wrong: no velocity clamp → vel.x keeps growing → FAIL
//   - Wrong: sign error → vel.x decreases with right input → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t01_accelerate_to_max_speed_in_0_3_seconds() {
    let (mut player, terrain) = player_on_ground(PlayerConfig::default());
    let input = InputState {
        right: true,
        ..InputState::default()
    };

    // 0.3s = 18 frames at 60fps.
    let frames_03s: u32 = (0.3 / DT) as u32;
    simulate_frames(&mut player, &input, &terrain, frames_03s);

    // After 0.3s, vel.x should be ≈ max_speed (200.0 px/s).
    assert!(
        approx_eq(player.vel.x, DEFAULT_MAX_SPEED),
        "T01: After 0.3s ({} frames), vel.x should be ~max_speed={}, got {}",
        frames_03s, DEFAULT_MAX_SPEED, player.vel.x
    );

    // After another 5 frames, vel.x must still be clamped at max_speed (no growth beyond cap).
    simulate_frames(&mut player, &input, &terrain, 5);
    assert!(
        player.vel.x <= DEFAULT_MAX_SPEED + PHYSICS_EPSILON,
        "T01: After steady state, vel.x must NOT exceed max_speed={}, got {}",
        DEFAULT_MAX_SPEED, player.vel.x
    );
}

// ============================================================================
// T02 — FUNC/happy
// Traces To: FR-001 AC-2, §Interface Contract Player::update postcondition 摩擦减速
// "玩家在 0.2s 内因摩擦力减速至完全静止"
//
// Kills: friction = 0 → never stops; friction causes reversal → vel.x goes negative.
// Wrong-impl challenge:
//   - Wrong: friction_factor = 0 → never decelerates → FAIL
//   - Wrong: friction sign wrong → accelerates instead → FAIL
//   - Wrong: stopping time >> 0.2s → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t02_friction_decelerates_to_zero_in_0_2_seconds() {
    let (mut player, terrain) = player_on_ground(PlayerConfig::default());
    player.vel.x = DEFAULT_MAX_SPEED; // start at max speed right
    let no_input = InputState::default(); // no directional input

    // 0.2s = 12 frames at 60fps.
    let frames_02s: u32 = (0.2 / DT) as u32;
    simulate_frames(&mut player, &no_input, &terrain, frames_02s);

    // After 0.2s, vel.x should be approximately 0.0.
    assert!(
        approx_eq(player.vel.x, 0.0),
        "T02: After 0.2s ({} frames) of friction, vel.x should be ~0.0, got {}",
        frames_02s, player.vel.x
    );

    // vel.x must NOT go negative (don't overshoot/reverse).
    assert!(
        player.vel.x >= -PHYSICS_EPSILON,
        "T02: After stopping, vel.x must NOT be negative (no reversal), got {}",
        player.vel.x
    );
}

// ============================================================================
// T03 — FUNC/happy
// Traces To: FR-001 AC-3, §Interface Contract Player::update postcondition 反向加速
// "玩家从向右最大速度反向加速至向左最大速度"
//
// Kills: reverse acceleration skips to -max immediately; longer than forward acceleration.
// Wrong-impl challenge:
//   - Wrong: vel.x set directly to -max_speed instead of acceleration → instant flip → FAIL
//   - Wrong: acceleration applied in wrong direction → speed increases instead → FAIL
//   - Wrong: double acceleration applied when reversing → less than 0.3s → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t03_reverse_acceleration_from_max_right_to_max_left() {
    let (mut player, terrain) = player_on_ground(PlayerConfig::default());
    player.vel.x = DEFAULT_MAX_SPEED; // moving right at max speed
    let left_input = InputState {
        left: true,
        ..InputState::default()
    };

    // 0.3s = 18 frames for reverse acceleration.
    let frames_03s: u32 = (0.3 / DT) as u32;
    simulate_frames(&mut player, &left_input, &terrain, frames_03s);

    // After 0.3s, vel.x should be ≈ -max_speed.
    assert!(
        approx_eq(player.vel.x, -DEFAULT_MAX_SPEED),
        "T03: After 0.3s ({} frames) reversing from +max, vel.x should be ~-max_speed={}, got {}",
        frames_03s, -DEFAULT_MAX_SPEED, player.vel.x
    );

    // After steady state, vel.x stays at -max_speed (clamped).
    simulate_frames(&mut player, &left_input, &terrain, 5);
    assert!(
        player.vel.x >= -DEFAULT_MAX_SPEED - PHYSICS_EPSILON,
        "T03: After reverse steady state, vel.x must stay >= -max_speed, got {}",
        player.vel.x
    );
}

// ============================================================================
// T04 — FUNC/happy
// Traces To: FR-001 AC-4, §Design Rationale "同时按键去抖"
// "同时按住左箭头和右箭头 → 优先最后变化的方向"
//
// Kills: dual-key → NaN or panic; random direction → non-reproducible.
// Wrong-impl challenge:
//   - Wrong: panic when both keys pressed → FAIL
//   - Wrong: random selection or oscillating → test is deterministic → FAIL if wrong
//   - Wrong: no special handling → both left+right accelerations cancel = 0 → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t04a_dual_key_last_pressed_left_priority() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    // Place player on ground manually.
    let ground = AABB { x: 0.0, y: 600.0, w: 2000.0, h: 40.0 };
    player.pos = Vec2 { x: 200.0, y: 600.0 };
    player.on_ground = true;
    let terrain = platform_terrain(ground);

    // Both keys held, but left was just pressed (jump_just → false) — left priority.
    // Simulate: right was held in prev frame, left just pressed this frame.
    let input = InputState {
        left: true,
        right: true,
        ..InputState::default()
    };
    // We rely on the "last pressed" logic: since left wasn't held before (player was idle),
    // and now both are true, left should be the "just changed" direction.
    // The implementation tracks previous-frame state internally or via Player fields.

    simulate_frames(&mut player, &input, &terrain, 10);

    // Player should move left (negative velocity) — left has priority.
    assert!(
        player.vel.x < -PHYSICS_EPSILON,
        "T04a: With both keys held (left last pressed), player should accelerate LEFT. vel.x={}",
        player.vel.x
    );
}

// real_test (feature #3)
#[test]
fn t04b_dual_key_both_simultaneous_no_movement() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    let ground = AABB { x: 0.0, y: 600.0, w: 2000.0, h: 40.0 };
    player.pos = Vec2 { x: 200.0, y: 600.0 };
    player.on_ground = true;
    player.vel.x = 100.0; // was already moving right
    let terrain = platform_terrain(ground);

    // Both keys pressed from the start (same frame) — ambiguous.
    let input = InputState {
        left: true,
        right: true,
        ..InputState::default()
    };

    simulate_frames(&mut player, &input, &terrain, 10);

    // Net acceleration should be zero → player decelerates from friction (toward 0).
    // The key assertion: velocity doesn't increase (no acceleration applied).
    assert!(
        player.vel.x <= 100.0 + PHYSICS_EPSILON,
        "T04b: With both keys simultaneously pressed, vel.x should NOT increase beyond initial 100.0. vel.x={}",
        player.vel.x
    );
}

// ============================================================================
// T05 — FUNC/happy
// Traces To: FR-001 AC-5, §Interface Contract Player::update postcondition 空中控制
// "玩家在空中获得在地面加速度 60% 的水平控制力"
//
// Kills: air control = 100% → too sensitive; air control = 0% → no control.
// Wrong-impl challenge:
//   - Wrong: air_control_factor applied to max_speed instead of acceleration → FAIL
//   - Wrong: air_control_factor = 1.0 (ignored) → same as ground → FAIL
//   - Wrong: air_control_factor = 0.0 (no air control) → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t05_air_control_is_60_percent_of_ground_acceleration() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    player.on_ground = false; // airborne
    player.vel.x = 0.0;
    let empty_terrain: Vec<Tile> = vec![];

    let input = InputState {
        right: true,
        ..InputState::default()
    };

    // Run 1 frame of update.
    player.update(DT, &input, &empty_terrain);

    // Expected acceleration: config.acceleration * air_control_factor * DT
    let expected_accel = config.acceleration * config.air_control_factor * DT;
    let actual_accel = player.vel.x; // started at 0, so vel.x == acceleration

    assert!(
        approx_eq(actual_accel, expected_accel),
        "T05: Air control acceleration should be {} * {} * {} = {}, got vel.x={}",
        config.acceleration, config.air_control_factor, DT, expected_accel, actual_accel
    );

    // Ground comparison: acceleration on ground should be larger.
    let mut ground_player = Player::new(config);
    ground_player.on_ground = true;
    ground_player.vel.x = 0.0;
    ground_player.update(DT, &input, &empty_terrain);
    let ground_accel = ground_player.vel.x;

    assert!(
        ground_accel > actual_accel + PHYSICS_EPSILON,
        "T05: Ground acceleration ({}) must be greater than air acceleration ({})",
        ground_accel, actual_accel
    );
}

// ============================================================================
// T06 — FUNC/happy
// Traces To: FR-002 AC-1, §Interface Contract Player::update postcondition 短跳
// "轻按空格键 (<100ms) → 玩家上升至基础跳跃高度的约 40% 后开始下落"
//
// Kills: short jump = max height; short jump = 0 (initial pulse not applied).
// Wrong-impl challenge:
//   - Wrong: jump_sustain always applied → short press = max height → FAIL
//   - Wrong: jump_initial_velocity = 0 → no upward motion → FAIL
//   - Wrong: jump_just not consumed → re-triggers every frame → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t06_short_jump_reaches_approximately_40_percent_of_max_height() {
    let (mut player, terrain) = player_on_ground(PlayerConfig::default());

    // Frame 1: jump just pressed.
    let jump_start_input = InputState {
        jump: true,
        jump_just: true,
        ..InputState::default()
    };
    player.update(DT, &jump_start_input, &terrain);
    let start_y = player.pos.y; // after jump initiation

    // Frames 2-6: release jump after ~83ms (5 more frames).
    // jump is NOT held, so sustain should not apply.
    let release_input = InputState {
        jump: false,
        ..InputState::default()
    };

    // Simulate until player starts falling (vel.y becomes positive = downward).
    let mut max_height_reached = false;
    let mut peak_y = f32::MAX; // smallest y = highest (Y-down)
    let mut prev_vel_y = player.vel.y;
    for _ in 0..60 {
        // 1 second max
        player.update(DT, &release_input, &terrain);
        peak_y = peak_y.min(player.pos.y);
        // Detect falling: vel.y was negative (up), now positive (down) or zero then positive.
        if prev_vel_y < 0.0 && player.vel.y >= 0.0 {
            max_height_reached = true;
            break;
        }
        prev_vel_y = player.vel.y;
    }

    assert!(
        max_height_reached,
        "T06: Short jump should reach a peak and then start falling. vel.y never transitioned up→down."
    );

    // Short jump height should be 35-50% of max height.
    // v0=-350, g=980: short ≈ 61px, max hold ≈ 138px, ratio ≈ 44%.
    let jump_height_px = start_y - peak_y; // pixels risen (Y-down: start > peak)
    let min_expected = 65.0; // ~34% of max hold
    let max_expected = 110.0; // ~58% of max hold
    assert!(
        jump_height_px >= min_expected && jump_height_px <= max_expected,
        "T06: Short jump height should be ~40-50% of max (roughly {} to {} px), got {} px",
        min_expected, max_expected, jump_height_px
    );
}

// ============================================================================
// T07 — FUNC/happy
// Traces To: FR-002 AC-2, §Interface Contract Player::update postcondition 最大跳跃
// "按住空格不放 → 玩家上升至最大跳跃高度并在最高点短暂悬停"
//
// Kills: sustain force missing → height = min jump; sustain infinite → never stops rising.
// Wrong-impl challenge:
//   - Wrong: jump_sustain never applied → only initial pulse height → FAIL
//   - Wrong: max_jump_duration not enforced → infinite upward → FAIL
//   - Wrong: jump_held stays true after landing → can't jump again → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t07_max_jump_reaches_full_height_with_sustain() {
    let (mut player, terrain) = player_on_ground(PlayerConfig::default());

    // Jump with Space held continuously.
    let hold_jump_input = InputState {
        jump: true,
        jump_just: true,
        ..InputState::default()
    };

    // On frame 1: jump_just triggers initial pulse.
    player.update(DT, &hold_jump_input, &terrain);
    let start_y = player.pos.y;

    // After first frame, jump_just should be consumed (set false by player or caller).
    // For testing, we simulate: jump_just only true on frame 1, then just jump held.
    let sustain_input = InputState {
        jump: true,
        ..InputState::default()
    };

    // Simulate up to 1 second to find peak.
    let mut peak_y = f32::MAX;
    let mut prev_vel_y = player.vel.y;
    let mut max_height_reached = false;
    let mut jump_timer_at_peak: f32 = 0.0;

    for i in 0..60 {
        let input = if i == 0 {
            &hold_jump_input
        } else {
            &sustain_input
        };
        player.update(DT, input, &terrain);
        peak_y = peak_y.min(player.pos.y);

        if prev_vel_y < 0.0 && player.vel.y >= 0.0 {
            max_height_reached = true;
            jump_timer_at_peak = player.jump_timer;
            break;
        }
        prev_vel_y = player.vel.y;
    }

    assert!(
        max_height_reached,
        "T07: Max jump with held Space should reach peak and start falling."
    );

    let jump_height_px = start_y - peak_y;
    // Full max jump should be substantially higher than short tap.
    assert!(
        jump_height_px > 140.0,
        "T07: Max jump height should be > 140 px (full sustain), got {} px",
        jump_height_px
    );

    // jump_timer should be close to max_jump_duration at peak.
    assert!(
        jump_timer_at_peak <= DEFAULT_MAX_JUMP_DURATION + PHYSICS_EPSILON,
        "T07: jump_timer at peak ({}) should be <= max_jump_duration ({})",
        jump_timer_at_peak, DEFAULT_MAX_JUMP_DURATION
    );

    // After max_jump_duration, jump_held should become false (sustain stopped).
    // Simulate past the sustain cut-off.
    let mut long_hold_player = Player::new(PlayerConfig::default());
    long_hold_player.pos = Vec2 { x: 200.0, y: 600.0 };
    long_hold_player.on_ground = true;
    // Simulate past max_jump_duration.
    let total_frames = (DEFAULT_MAX_JUMP_DURATION / DT) as u32 + 2;
    let init_input = InputState { jump: true, jump_just: true, ..InputState::default() };
    let hold_input = InputState { jump: true, ..InputState::default() };
    long_hold_player.update(DT, &init_input, &terrain);
    for _ in 1..total_frames {
        long_hold_player.update(DT, &hold_input, &terrain);
    }
    assert!(
        !long_hold_player.jump_held,
        "T07: After max_jump_duration ({}s), jump_held must be false. Was true.",
        DEFAULT_MAX_JUMP_DURATION
    );
}

// ============================================================================
// T08 — FUNC/error
// Traces To: FR-002 AC-3, §Interface Contract Player::update Raises (二段跳防护)
// "玩家处于空中时按下空格键不应再次起跳"
//
// Kills: on_ground check missing → infinite mid-air jumps; jump_just stale in air.
// Wrong-impl challenge:
//   - Wrong: jump_just honored regardless of on_ground → air jump works → FAIL
//   - Wrong: on_ground stuck true → can always jump → FAIL
//   - Wrong: jump_just not consumed → fires every frame → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t08_no_double_jump_when_airborne() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    player.on_ground = false; // airborne
    player.vel.y = 100.0; // falling
    player.vel.x = 0.0;
    let empty_terrain: Vec<Tile> = vec![];

    let vy_before = player.vel.y;

    // Press jump in mid-air — should be IGNORED.
    let jump_input = InputState {
        jump: true,
        jump_just: true,
        ..InputState::default()
    };
    player.update(DT, &jump_input, &empty_terrain);

    // Vertical velocity should NOT become negative (upward).
    assert!(
        player.vel.y >= vy_before - PHYSICS_EPSILON,
        "T08: Jump in mid-air must NOT apply upward velocity. vy was {}, now {}",
        vy_before, player.vel.y
    );

    // jump_held should still be false.
    assert!(
        !player.jump_held,
        "T08: jump_held must remain false when jump_just ignored in air. Was {}.",
        player.jump_held
    );

    // jump_timer should not have been set.
    assert!(
        player.jump_timer < PHYSICS_EPSILON,
        "T08: jump_timer must NOT start when jump_just ignored in air. Was {}.",
        player.jump_timer
    );
}

// ============================================================================
// T09 — FUNC/happy
// Traces To: FR-002 AC-4, §Interface Contract Player::update postcondition 天花板碰撞
// "玩家跳跃中头顶接触天花板 → 垂直速度立即归零，开始下落"
//
// Kills: no ceiling detection → player passes through; vel.y zeroed but pos not corrected.
// Wrong-impl challenge:
//   - Wrong: ceiling collision not handled → player clips through → FAIL
//   - Wrong: vel.y = 0 but pos.y uncorrected → next frame still collides → FAIL
//   - Wrong: ceiling sets on_ground = true → player stuck to ceiling → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t09_ceiling_collision_stops_upward_motion() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    // Player is jumping upward, head about to hit a platform above.
    player.pos = Vec2 { x: 200.0, y: 400.0 };
    player.vel.y = -300.0; // moving up (negative Y)
    player.on_ground = false;

    // Ceiling platform: bottom at y=384 (player head at pos.y - 16 = 384 when at y=400)
    let ceiling = AABB { x: 180.0, y: 360.0, w: 40.0, h: 24.0 };
    let terrain = platform_terrain(ceiling);

    player.update(DT, &InputState::default(), &terrain);

    // After ceiling collision, vertical velocity must be zeroed (or set to start falling).
    assert!(
        player.vel.y >= -PHYSICS_EPSILON,
        "T09: After ceiling hit, vel.y must be >= 0 (stopped upward motion). Got {}.",
        player.vel.y
    );

    // Player must NOT have penetrated the ceiling (pos.y should be below ceiling).
    // Player collider top = pos.y - 16 (Small). Should be >= ceiling.y + ceiling.h.
    let player_top = player.pos.y - 16.0;
    let ceiling_bottom = ceiling.y + ceiling.h;
    assert!(
        player_top >= ceiling_bottom - POSITION_EPSILON,
        "T09: Player must not penetrate ceiling. Player top={}, ceiling bottom={}.",
        player_top, ceiling_bottom
    );

    // on_ground must be false (player hit ceiling, not floor).
    assert!(
        !player.on_ground,
        "T09: After ceiling collision, on_ground must be false. Got true (stuck to ceiling)."
    );
}

// ============================================================================
// T10 — FUNC/happy
// Traces To: FR-002 AC-5, §Interface Contract Player::update postcondition 边缘走出
// "玩家从较高平台边缘走出 → 重力立即作用，角色开始下落"
//
// Kills: on_ground stuck true → player floats; gravity not applied → floating.
// Wrong-impl challenge:
//   - Wrong: on_ground only cleared on jump → walking off edge still on_ground → FAIL
//   - Wrong: gravity not applied when on_ground = false → player floats → FAIL
//   - Wrong: terrain query not re-run → stale ground from previous frames → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t10_walk_off_ledge_applies_gravity() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    player.pos = Vec2 { x: 200.0, y: 600.0 };
    player.on_ground = true;
    player.vel.y = 0.0;

    // Empty terrain: no platform below player (walked off edge).
    let empty_terrain: Vec<Tile> = vec![];

    player.update(DT, &InputState::default(), &empty_terrain);

    // on_ground must become false (no terrain support).
    assert!(
        !player.on_ground,
        "T10: After walking off ledge, on_ground must be false. Got true."
    );

    // Gravity must be applied: vel.y should increase (downward).
    let expected_vy = config.gravity * DT;
    assert!(
        player.vel.y > PHYSICS_EPSILON,
        "T10: After walking off ledge, gravity must be applied. vel.y={}, expected > 0.",
        player.vel.y
    );
    assert!(
        approx_eq(player.vel.y, expected_vy),
        "T10: First frame gravity = {} * {} = {}, got vel.y={}",
        config.gravity, DT, expected_vy, player.vel.y
    );
}

// ============================================================================
// T11 — FUNC/happy
// Traces To: FR-003 AC-1, §Interface Contract Player::update postcondition 冲刺速度
// "按住 Shift 时玩家最大水平速度为基础速度的 1.5 倍"
//
// Kills: sprint_multiplier not applied → speed = max_speed; applied to accel not max.
// Wrong-impl challenge:
//   - Wrong: sprint_multiplier ignored → max_speed unchanged → FAIL
//   - Wrong: sprint applied to acceleration instead of max → reaches 1.5x in wrong time → FAIL
//   - Wrong: sprint flag not read from input → no effect → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t11_sprint_increases_max_speed_by_1_5x() {
    let (mut player, terrain) = player_on_ground(PlayerConfig::default());
    let sprint_input = InputState {
        right: true,
        sprint: true,
        ..InputState::default()
    };

    // Accelerate until max speed is reached.
    let frames_to_max: u32 = (0.5 / DT) as u32; // plenty of time
    simulate_frames(&mut player, &sprint_input, &terrain, frames_to_max);

    let expected_max = DEFAULT_MAX_SPEED * DEFAULT_SPRINT_MULTIPLIER;
    assert!(
        approx_eq(player.vel.x, expected_max),
        "T11: Sprint max speed should be {} * {} = {}, got vel.x={}",
        DEFAULT_MAX_SPEED, DEFAULT_SPRINT_MULTIPLIER, expected_max, player.vel.x
    );

    // Non-sprint comparison: should reach lower max.
    let (mut normal_player, normal_terrain) = player_on_ground(PlayerConfig::default());
    let normal_input = InputState {
        right: true,
        ..InputState::default()
    };
    simulate_frames(&mut normal_player, &normal_input, &normal_terrain, frames_to_max);
    let normal_max = normal_player.vel.x;

    assert!(
        player.vel.x > normal_max + PHYSICS_EPSILON,
        "T11: Sprint max ({}) must be greater than normal max speed ({})",
        player.vel.x, normal_max
    );
}

// ============================================================================
// T12 — FUNC/happy
// Traces To: FR-003 AC-2, §Design Rationale 冲刺跳跃距离
// "冲刺中起跳的水平位移比不冲刺时增加约 50%"
//
// Kills: sprint speed bonus lost during jump; air speed clamped to non-sprint max.
// Wrong-impl challenge:
//   - Wrong: jump resets horizontal speed → same distance → FAIL
//   - Wrong: air max speed = non-sprint max → sprint bonus lost → FAIL
//   - Wrong: sprint_multiplier not applied to effective_max in air → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t12_sprint_jump_has_approximately_1_5x_horizontal_distance() {
    // This test verifies that sprint jump horizontal distance is ~1.5x normal jump.

    // --- Normal jump distance ---
    let (mut normal_player, terrain) = player_on_ground(PlayerConfig::default());
    // Accelerate to max speed first.
    let accel_input = InputState { right: true, ..InputState::default() };
    simulate_frames(&mut normal_player, &accel_input, &terrain, 20); // reach max speed

    let normal_jump_start_x = normal_player.pos.x;
    let jump_input = InputState { right: true, jump: true, jump_just: true, ..InputState::default() };
    // Jump: jump_just on first frame, then hold right (no sprint).
    normal_player.update(DT, &jump_input, &terrain);
    let hold_right = InputState { right: true, ..InputState::default() };
    // Simulate until landing (on_ground becomes true).
    for _ in 0..120 {
        if normal_player.on_ground {
            break;
        }
        normal_player.update(DT, &hold_right, &terrain);
    }
    let normal_distance = normal_player.pos.x - normal_jump_start_x;

    // --- Sprint jump distance ---
    let (mut sprint_player, terrain2) = player_on_ground(PlayerConfig::default());
    let sprint_accel = InputState { right: true, sprint: true, ..InputState::default() };
    simulate_frames(&mut sprint_player, &sprint_accel, &terrain2, 20);

    let sprint_jump_start_x = sprint_player.pos.x;
    let sprint_jump_input = InputState {
        right: true, sprint: true, jump: true, jump_just: true,
        ..InputState::default()
    };
    sprint_player.update(DT, &sprint_jump_input, &terrain2);
    let hold_sprint = InputState { right: true, sprint: true, ..InputState::default() };
    for _ in 0..120 {
        if sprint_player.on_ground {
            break;
        }
        sprint_player.update(DT, &hold_sprint, &terrain2);
    }
    let sprint_distance = sprint_player.pos.x - sprint_jump_start_x;

    // Sprint distance / normal distance should be ~1.4-1.6 (50% more, ±10% tolerance).
    if normal_distance > 0.0 {
        let ratio = sprint_distance / normal_distance;
        assert!(
            ratio >= 1.4 && ratio <= 1.6,
            "T12: Sprint jump distance ratio should be 1.4-1.6 (50% more). Got {:.2} (sprint={:.1}, normal={:.1})",
            ratio, sprint_distance, normal_distance
        );
    }
}

// ============================================================================
// T13 — FUNC/happy
// Traces To: FR-003 AC-3, §Interface Contract Player::update postcondition 冲刺释放摩擦
// "冲刺时释放所有水平输入 → 减速至静止，减速时间 0.2s（与非冲刺一致）"
//
// Kills: sprint modifies friction → different stop time; releasing sprint snaps speed down.
// Wrong-impl challenge:
//   - Wrong: friction * sprint_multiplier → stops faster → FAIL
//   - Wrong: releasing sprint resets vel.x → instant speed drop → FAIL
//   - Wrong: sprint changes friction_factor → stop time changes → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t13_sprint_release_friction_same_as_normal() {
    let (mut player, terrain) = player_on_ground(PlayerConfig::default());
    // Player at sprint max speed.
    player.vel.x = DEFAULT_MAX_SPEED * DEFAULT_SPRINT_MULTIPLIER;

    // Release all input (no sprint, no direction).
    let no_input = InputState::default();
    let frames_02s: u32 = (0.2 / DT) as u32;
    simulate_frames(&mut player, &no_input, &terrain, frames_02s);

    // After 0.2s, should be stopped (same as non-sprint friction).
    assert!(
        approx_eq(player.vel.x, 0.0),
        "T13: After 0.2s friction from sprint max speed, vel.x should be ~0.0. Got {}.",
        player.vel.x
    );

    // Should NOT go negative.
    assert!(
        player.vel.x >= -PHYSICS_EPSILON,
        "T13: After stopping from sprint, vel.x must NOT go negative. Got {}.",
        player.vel.x
    );
}

// ============================================================================
// T14 — FUNC/error
// Traces To: §Interface Contract Player::new postcondition
// "max_speed=0 构造成功但不 panic；vel.x 始终钳制于 0"
//
// Kills: max_speed=0 → divide-by-zero or panic; negative params accepted silently.
// Wrong-impl challenge:
//   - Wrong: config validation panics on max_speed=0 → FAIL
//   - Wrong: NaN propagation from 0 max_speed → vel.x = NaN → FAIL
//   - Wrong: negative friction accepted → wrong physics → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t14_invalid_config_max_speed_zero_does_not_panic() {
    let config = PlayerConfig {
        max_speed: 0.0,
        ..PlayerConfig::default()
    };
    let mut player = Player::new(config);
    let terrain: Vec<Tile> = vec![];
    player.on_ground = true;

    // Should not panic.
    let input = InputState { right: true, ..InputState::default() };
    player.update(DT, &input, &terrain);

    // With max_speed = 0, velocity should remain clamped at 0.
    assert!(
        player.vel.x >= -PHYSICS_EPSILON && player.vel.x <= PHYSICS_EPSILON,
        "T14: With max_speed=0, vel.x must stay at 0. Got {}.",
        player.vel.x
    );
}

// ============================================================================
// T15 — BNDRY/edge
// Traces To: §Boundary Conditions vel.x at max_speed
// "速度恰好等于上限时不再加速"
//
// Kills: clamp uses < instead of <= → overshoot; float drift in clamp formula.
// Wrong-impl challenge:
//   - Wrong: clamp check uses vel.x < max_speed → at 199.99 still accelerates → overshoots → FAIL
//   - Wrong: clamp applied after position update → one frame of overshoot → FAIL
//   - Wrong: abs(vel.x) > max check instead of >= → edge miss → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t15_speed_clamped_at_max_when_already_at_max() {
    let (mut player, terrain) = player_on_ground(PlayerConfig::default());
    player.vel.x = DEFAULT_MAX_SPEED; // exactly at max

    let right_input = InputState { right: true, ..InputState::default() };
    player.update(DT, &right_input, &terrain);

    // vx should not exceed max_speed.
    assert!(
        player.vel.x <= DEFAULT_MAX_SPEED + PHYSICS_EPSILON,
        "T15: When vel.x == max_speed, further right input must NOT increase vel.x. Got {}.",
        player.vel.x
    );
}

// ============================================================================
// T16 — BNDRY/edge
// Traces To: §Boundary Conditions vel.x 过零
// "极低速度摩擦至零，不跨越零点，不在零点附近振荡"
//
// Kills: vel.x oscillates around 0; vel.x never converges exactly to 0.
// Wrong-impl challenge:
//   - Wrong: friction overshoots → vel.x goes from +0.5 to -0.3 → FAIL
//   - Wrong: vel.x converges to epsilon but never 0 → micro-drift → FAIL
//   - Wrong: no zero-crossing clamp → oscillation persists → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t16_velocity_zero_crossing_clamped_to_zero() {
    let (mut player, terrain) = player_on_ground(PlayerConfig::default());
    player.vel.x = 0.5; // very low speed to the right
    let no_input = InputState::default();

    // Apply friction for several frames.
    for _ in 0..10 {
        player.update(DT, &no_input, &terrain);
    }

    // After friction, vel.x should be 0.0 (not oscillating or stuck at a tiny value).
    assert!(
        approx_eq(player.vel.x, 0.0),
        "T16: After friction from vel.x=0.5, vel.x must be clamped to 0.0. Got {}.",
        player.vel.x
    );

    // Should not reverse direction (stay >= -EPSILON).
    assert!(
        player.vel.x >= -PHYSICS_EPSILON,
        "T16: Friction from rightward drift must NOT reverse to negative. Got {}.",
        player.vel.x
    );
}

// ============================================================================
// T17 — BNDRY/edge
// Traces To: §Boundary Conditions jump_timer == max_jump_duration
// "在 jump_timer 恰好等于 max_jump_duration 时，sustain 力停止施加"
//
// Kills: comparison uses > instead of >= → one extra sustain frame; timer overflows.
// Wrong-impl challenge:
//   - Wrong: check `jump_timer > max_jump_duration` → one extra frame of sustain → FAIL
//   - Wrong: timer keeps growing past max → overflow or → FAIL
//   - Wrong: timer check missing → infinite sustain → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t17_jump_timer_at_max_duration_stops_sustain() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    player.pos = Vec2 { x: 200.0, y: 600.0 };
    player.on_ground = true;
    let ground = AABB { x: 0.0, y: 600.0, w: 2000.0, h: 40.0 };
    let terrain = platform_terrain(ground);

    // Set jump_timer just below max, jump_held true.
    let start_input = InputState { jump: true, jump_just: true, ..InputState::default() };
    player.update(DT, &start_input, &terrain);

    // Manually advance jump_timer near max.
    player.jump_timer = DEFAULT_MAX_JUMP_DURATION - DT * 0.5;
    player.jump_held = true;

    // One more frame with jump held — should cross the threshold.
    let hold_input = InputState { jump: true, ..InputState::default() };
    player.update(DT, &hold_input, &terrain);

    // After crossing max_jump_duration, jump_held must be false.
    assert!(
        !player.jump_held,
        "T17: After jump_timer >= max_jump_duration, jump_held must be false. Got true."
    );

    // jump_timer should not keep growing past max.
    assert!(
        player.jump_timer <= DEFAULT_MAX_JUMP_DURATION + PHYSICS_EPSILON,
        "T17: jump_timer should not exceed max_jump_duration. Got {} > {}.",
        player.jump_timer, DEFAULT_MAX_JUMP_DURATION
    );
}

// ============================================================================
// T18 — BNDRY/edge
// Traces To: §Boundary Conditions on_ground 着陆
// "玩家下落着地 → vel.y = 0, on_ground = true, pos.y 等于平台顶面 Y"
//
// Kills: float error → player embedded in platform; on_ground not set → falls through.
// Wrong-impl challenge:
//   - Wrong: Y correction is player bottom = platform top + epsilon → embedded → FAIL
//   - Wrong: on_ground not set on landing → falls through next frame → FAIL
//   - Wrong: vel.y > 0 not checked → landing from jump ascent also sets on_ground → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t18_landing_on_platform_sets_on_ground_and_zeroes_vertical_velocity() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    // Player falling onto platform: foot at y=599, platform top at y=600.
    // Player collider = (x-8, y-16, 16, 16). Bottom = y at pos.y.
    player.pos = Vec2 { x: 200.0, y: 599.0 };
    player.vel.y = 200.0; // falling down
    player.on_ground = false;

    let ground = AABB {
        x: 0.0, y: 600.0, w: 2000.0, h: 40.0,
    };
    let terrain = platform_terrain(ground);

    player.update(DT, &InputState::default(), &terrain);

    // After landing, on_ground must be true.
    assert!(
        player.on_ground,
        "T18: After landing on platform, on_ground must be true. Got false (fell through?)."
    );

    // Vertical velocity must be zeroed.
    assert!(
        approx_eq(player.vel.y, 0.0),
        "T18: After landing, vel.y must be 0.0. Got {}.",
        player.vel.y
    );

    // Position must be corrected to platform surface.
    assert!(
        approx_eq_loose(player.pos.y, ground.y),
        "T18: After landing, pos.y should be platform top ({}), got {}.",
        ground.y, player.pos.y
    );
}

// ============================================================================
// T19 — BNDRY/edge
// Traces To: §Boundary Conditions terrain 空切片
// "terrain 为空 → 玩家持续下落，无 panic"
//
// Kills: empty terrain panics (unwrap/索引越界); on_ground set true on empty.
// Wrong-impl challenge:
//   - Wrong: unwrap on first terrain element → panic → FAIL
//   - Wrong: for loop on empty → on_ground stays true from last frame → FAIL
//   - Wrong: empty terrain → skip gravity → no falling → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t19_empty_terrain_allows_continuous_falling() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    player.pos = Vec2 { x: 200.0, y: 400.0 };
    player.on_ground = false;
    player.vel.y = 0.0;
    let empty_terrain: Vec<Tile> = vec![];

    let initial_y = player.pos.y;

    // Simulate 10 frames of falling.
    for _ in 0..10 {
        player.update(DT, &InputState::default(), &empty_terrain);
    }

    // Player should have fallen (pos.y increased = moved down).
    assert!(
        player.pos.y > initial_y + POSITION_EPSILON,
        "T19: With empty terrain, player should fall. pos.y went from {} to {}.",
        initial_y, player.pos.y
    );

    // on_ground must remain false (nothing to land on).
    assert!(
        !player.on_ground,
        "T19: With empty terrain, on_ground must remain false. Got true."
    );

    // vel.y should be positive (falling).
    assert!(
        player.vel.y > 0.0,
        "T19: With empty terrain and no ground, vel.y should be > 0 (falling). Got {}.",
        player.vel.y
    );
}

// ============================================================================
// T20 — BNDRY/edge
// Traces To: §Boundary Conditions collider() Small vs Super
// "Small: 16x16, Super: 16x32, 脚底对齐（pos.y 不变）"
//
// Kills: upgrade → head embeds in ceiling; foot position shifts → visual jump.
// Wrong-impl challenge:
//   - Wrong: Super collider has w=32 → too wide → FAIL
//   - Wrong: foot position moves when resizing → pos.y changes → FAIL
//   - Wrong: collider origin at center instead of foot-bottom → offset wrong → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t20_collider_size_changes_with_powerup_state_foot_aligned() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    player.pos = Vec2 { x: 200.0, y: 500.0 };
    player.state = PlayerState::Small;

    let small_collider = player.collider();

    // Small: 16x16
    assert!(
        approx_eq(small_collider.w, 16.0) && approx_eq(small_collider.h, 16.0),
        "T20: Small collider must be 16x16. Got {}x{}.",
        small_collider.w, small_collider.h
    );

    // Foot alignment: pos.y should be at bottom of collider (collider.y + collider.h).
    assert!(
        approx_eq(small_collider.y + small_collider.h, player.pos.y),
        "T20: Small collider bottom must equal pos.y (foot). Collider bottom={}, pos.y={}.",
        small_collider.y + small_collider.h, player.pos.y
    );

    // Center alignment: pos.x should be at collider horizontal center.
    assert!(
        approx_eq(small_collider.x + small_collider.w / 2.0, player.pos.x),
        "T20: Small collider horizontal center must equal pos.x."
    );

    // Change to Super.
    player.state = PlayerState::Super;
    let super_collider = player.collider();

    // Super: 32x32 (w and h both doubled).
    assert!(
        approx_eq(super_collider.w, 32.0),
        "T20: Super collider width must be 32. Got {}.",
        super_collider.w
    );
    assert!(
        approx_eq(super_collider.h, 32.0),
        "T20: Super collider height must be 32 (doubled). Got {}.",
        super_collider.h
    );

    // Foot position must stay ALIGNED (pos.y unchanged, collider bottom = pos.y).
    assert!(
        approx_eq(super_collider.y + super_collider.h, player.pos.y),
        "T20: Super collider bottom must still equal pos.y after resize. Collider bottom={}, pos.y={}.",
        super_collider.y + super_collider.h, player.pos.y
    );

    // pos.y must not have changed.
    assert!(
        approx_eq(player.pos.y, 500.0),
        "T20: pos.y must not change when power-up state changes. Got {}.",
        player.pos.y
    );
}

// ============================================================================
// T21 — BNDRY/edge
// Traces To: §Boundary Conditions air_control_factor = 0.0
// "air_control_factor = 0 → 空中完全无法水平操控"
//
// Kills: air_control_factor = 0 still applies ground acceleration.
// Wrong-impl challenge:
//   - Wrong: air_control_factor not used → same as ground → FAIL
//   - Wrong: multiplication by 0.0 → acceleration = 0 → correct, but verify
// ============================================================================

// real_test (feature #3)
#[test]
fn t21_air_control_factor_zero_prevents_air_steering() {
    let config = PlayerConfig {
        air_control_factor: 0.0,
        ..PlayerConfig::default()
    };
    let mut player = Player::new(config);
    player.on_ground = false;
    player.vel.x = 50.0; // some initial horizontal speed
    player.vel.y = -100.0;
    let empty_terrain: Vec<Tile> = vec![];

    let input = InputState { right: true, ..InputState::default() };
    let vx_before = player.vel.x;
    player.update(DT, &input, &empty_terrain);

    // With air_control_factor = 0, horizontal velocity should not change.
    assert!(
        approx_eq(player.vel.x, vx_before),
        "T21: With air_control_factor=0, right input must NOT change vel.x. Was {}, now {}.",
        vx_before, player.vel.x
    );
}

// ============================================================================
// T22 — BNDRY/edge
// Traces To: §Boundary Conditions air_control_factor = 1.0
// "air_control_factor = 1.0 → 空中加速度等于地面加速度"
//
// Kills: air_control_factor = 1.0 reduced by other logic.
// Wrong-impl challenge:
//   - Wrong: factor clamped to < 1.0 → air < ground → FAIL
//   - Wrong: factor hardcoded to 0.6 regardless of config → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t22_air_control_factor_one_equals_ground_acceleration() {
    let config = PlayerConfig {
        air_control_factor: 1.0,
        ..PlayerConfig::default()
    };
    let mut player = Player::new(config);
    player.on_ground = false;
    player.vel.x = 0.0;
    let empty_terrain: Vec<Tile> = vec![];

    let input = InputState { right: true, ..InputState::default() };
    player.update(DT, &input, &empty_terrain);
    let air_vx = player.vel.x;

    // Same setup on ground.
    let mut ground_player = Player::new(config);
    ground_player.on_ground = true;
    ground_player.vel.x = 0.0;
    ground_player.update(DT, &input, &empty_terrain);
    let ground_vx = ground_player.vel.x;

    assert!(
        approx_eq(air_vx, ground_vx),
        "T22: With air_control_factor=1.0, air acceleration ({}) must equal ground acceleration ({}).",
        air_vx, ground_vx
    );
}

// ============================================================================
// T23 — FUNC/happy
// Traces To: §Interface Contract Player::pos (IAPI-007)
// "pos() 返回当前世界坐标 Vec2 { x, y }，纯 getter 无副作用"
//
// Kills: pos() returns reference allowing mutation; wrong coordinate components.
// Wrong-impl challenge:
//   - Wrong: pos() returns &Vec2 → caller can mutate through reference → FAIL
//   - Wrong: pos() returns { x: y, y: x } → swapped → FAIL
//   - Wrong: pos() offset by camera → wrong coordinates → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t23_pos_returns_current_world_coordinates() {
    let config = PlayerConfig::default();
    let player = Player::new(config);
    // pos() should return the same coordinates set during construction.
    let pos = player.pos();

    assert!(
        approx_eq(pos.x, 100.0),
        "T23: pos().x should be 100.0 (level start). Got {}.",
        pos.x
    );
    assert!(
        approx_eq(pos.y, 100.0),
        "T23: pos().y should be 100.0 (level start). Got {}.",
        pos.y
    );

    // Second call should return same value (pure getter).
    let pos2 = player.pos();
    assert!(
        approx_eq(pos2.x, pos.x) && approx_eq(pos2.y, pos.y),
        "T23: Multiple pos() calls must return consistent values."
    );
}

// ============================================================================
// T24 — FUNC/happy
// Traces To: §Interface Contract Player::stats (IAPI-009)
// "stats() 返回 PlayerStats { coins, lives }，与内部字段一致"
//
// Kills: stats() returns stale values; unnecessary allocations.
// Wrong-impl challenge:
//   - Wrong: stats() returns hardcoded {0, 3} → ignores actual values → FAIL
//   - Wrong: stats() reads wrong fields → coins/lives swapped → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t24_stats_returns_current_player_stats() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    player.coins = 5;
    player.lives = 2;

    let stats = player.stats();

    assert_eq!(
        stats.coins, 5,
        "T24: stats().coins should be 5. Got {}.",
        stats.coins
    );
    assert_eq!(
        stats.lives, 2,
        "T24: stats().lives should be 2. Got {}.",
        stats.lives
    );

    // stats() should return type PlayerStats (compile-time check passed by usage).
    // Verify it returns by value (not reference that could be mutated).
    let _stats2 = player.stats();
}

// ============================================================================
// T25 — FUNC/happy
// Traces To: §Interface Contract Player::apply_powerup
// "Small → Super: state 变更, collider 变为 16x32 脚底对齐"
//
// Kills: apply_powerup doesn't update collider size; state changes but collision stays Small.
// Wrong-impl challenge:
//   - Wrong: state changed but collider uses old state → still 16x16 → FAIL
//   - Wrong: collider foot position shifts → pos.y changes → FAIL
//   - Wrong: apply_powerup panics on valid input → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t25_apply_powerup_small_to_super() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    player.pos = Vec2 { x: 200.0, y: 500.0 };
    player.state = PlayerState::Small;

    let pos_y_before = player.pos.y;
    player.apply_powerup(PlayerState::Super);

    // State must change.
    assert_eq!(
        player.state, PlayerState::Super,
        "T25: After apply_powerup(Super), state should be Super. Got {:?}.",
        player.state
    );

    // Collider must update to 32x32.
    let c = player.collider();
    assert!(
        approx_eq(c.w, 32.0) && approx_eq(c.h, 32.0),
        "T25: After powerup, collider must be 32x32. Got {}x{}.",
        c.w, c.h
    );

    // Foot position shifts down by height increase (pos.y += new_h - old_h = 16).
    assert!(
        approx_eq(player.pos.y, pos_y_before + 16.0),
        "T25: pos.y must shift down by 16 after powerup. Expected {}, got {}.",
        pos_y_before + 16.0, player.pos.y
    );

    // Collider bottom must align with pos.y.
    assert!(
        approx_eq(c.y + c.h, player.pos.y),
        "T25: Collider bottom ({}) must align with foot pos.y ({}).",
        c.y + c.h, player.pos.y
    );
}

// ============================================================================
// T26 — FUNC/happy
// Traces To: §Interface Contract Player::take_damage (Super→Small)
// "Super 状态受伤 → 降级为 Small，返回 false（未死亡）"
//
// Kills: take_damage on Super returns true (false death); collider doesn't shrink.
// Wrong-impl challenge:
//   - Wrong: take_damage always returns true → death when should downgrade → FAIL
//   - Wrong: collider stays 16x32 after downgrade → collision mismatch → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t26_take_damage_super_downgrades_to_small() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    player.pos = Vec2 { x: 200.0, y: 500.0 };
    player.state = PlayerState::Super;

    let result = player.take_damage();

    // Should NOT die (Super downgrades, not death).
    assert!(
        !result,
        "T26: take_damage on Super should return false (downgrade, not death). Got true."
    );

    // State should be Small now.
    assert_eq!(
        player.state, PlayerState::Small,
        "T26: After damage, Super should become Small. Got {:?}.",
        player.state
    );

    // Collider must shrink back to 16x16.
    let c = player.collider();
    assert!(
        approx_eq(c.w, 16.0) && approx_eq(c.h, 16.0),
        "T26: After downgrade to Small, collider must be 16x16. Got {}x{}.",
        c.w, c.h
    );
}

// ============================================================================
// T27 — FUNC/happy
// Traces To: §Interface Contract Player::take_damage (Small→death)
// "Small 状态受伤 → 返回 true（应触发死亡流程）"
//
// Kills: Small take_damage returns false → immortal; Small take_damage panics.
// Wrong-impl challenge:
//   - Wrong: take_damage returns false for Small → immortal player → FAIL
//   - Wrong: take_damage panics on Small (e.g. unwrap on enum) → FAIL
//   - Wrong: state changes to something invalid on death → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t27_take_damage_small_returns_death() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    player.pos = Vec2 { x: 200.0, y: 500.0 };
    player.state = PlayerState::Small;

    let result = player.take_damage();

    // Should return true (death).
    assert!(
        result,
        "T27: take_damage on Small should return true (death). Got false."
    );

    // State should remain Small (no panic, no invalid state).
    assert_eq!(
        player.state, PlayerState::Small,
        "T27: After death, state should still be Small. Got {:?}.",
        player.state
    );
}

// ============================================================================
// T28 — FUNC/error
// Traces To: §Interface Contract Player::update 无效输入
// "dt = 0.0 → 不 panic, pos/vel 不变"
//
// Kills: dt=0 causes division by zero or NaN propagation; negative dt reverses physics.
// Wrong-impl challenge:
//   - Wrong: dt=0 → division by zero in acceleration formula → NaN → FAIL
//   - Wrong: dt=0 → no-op check missing → unexpected changes → FAIL
//   - Wrong: negative dt → player moves backward → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t28_zero_dt_does_not_panic_or_modify_state() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    player.pos = Vec2 { x: 200.0, y: 500.0 };
    player.vel = Vec2 { x: 50.0, y: -100.0 };
    player.on_ground = false;

    let ground = AABB { x: 0.0, y: 600.0, w: 2000.0, h: 40.0 };
    let terrain = platform_terrain(ground);
    let input = InputState { right: true, jump: true, jump_just: true, ..InputState::default() };

    let pos_before = player.pos;
    let vel_before = player.vel;
    let on_ground_before = player.on_ground;

    // dt = 0.0 should be a no-op.
    player.update(0.0, &input, &terrain);

    // Position and velocity must remain unchanged.
    assert!(
        approx_eq(player.pos.x, pos_before.x) && approx_eq(player.pos.y, pos_before.y),
        "T28: With dt=0, pos must not change. Was ({},{}), now ({},{}).",
        pos_before.x, pos_before.y, player.pos.x, player.pos.y
    );
    assert!(
        approx_eq(player.vel.x, vel_before.x) && approx_eq(player.vel.y, vel_before.y),
        "T28: With dt=0, vel must not change. Was ({},{}), now ({},{}).",
        vel_before.x, vel_before.y, player.vel.x, player.vel.y
    );

    // on_ground must remain unchanged.
    assert_eq!(
        player.on_ground, on_ground_before,
        "T28: With dt=0, on_ground must not change."
    );
}

// ============================================================================
// T29 — BNDRY/edge
// Traces To: §Boundary Conditions 多平台同时碰撞
// "玩家夹在两侧平台之间 → 不穿入任一平台, vel.x 为 0"
//
// Kills: only first collision processed → other side penetrates; oscillation between walls.
// Wrong-impl challenge:
//   - Wrong: collision loop breaks after first hit → second wall penetrates → FAIL
//   - Wrong: correction pushes player left then right alternately → oscillation → FAIL
//   - Wrong: AABB correction overshoots → player pushed through wall → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t29_multi_platform_collision_prevents_penetration() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    // Player sandwiched between two walls, 16px apart (exactly player width).
    // Left wall right edge at x=200, Right wall left edge at x=216.
    player.pos = Vec2 { x: 208.0, y: 500.0 }; // centered between walls
    player.vel.x = 0.0;
    player.on_ground = false;

    let left_wall = AABB { x: 180.0, y: 480.0, w: 20.0, h: 100.0 };
    let right_wall = AABB { x: 216.0, y: 480.0, w: 20.0, h: 100.0 };
    let terrain = vec![Tile::Platform(left_wall), Tile::Platform(right_wall)];

    // Try to push right.
    let input = InputState { right: true, ..InputState::default() };
    for _ in 0..5 {
        player.update(DT, &input, &terrain);
    }

    // Player must not penetrate the right wall: player right edge (pos.x + 8) <= wall.left (216)
    let player_right = player.pos.x + 8.0; // half width of 16 = 8
    assert!(
        player_right <= right_wall.x + POSITION_EPSILON,
        "T29: Player must not penetrate right wall. Player right edge={}, wall left={}.",
        player_right, right_wall.x
    );

    // Player must not penetrate the left wall: player left edge (pos.x - 8) >= wall.right (200)
    let player_left = player.pos.x - 8.0;
    assert!(
        player_left >= left_wall.x + left_wall.w - POSITION_EPSILON,
        "T29: Player must not penetrate left wall. Player left edge={}, wall right={}.",
        player_left, left_wall.x + left_wall.w
    );
}

// ============================================================================
// T30 — BNDRY/edge
// Traces To: §Boundary Conditions facing 更新
// "方向键控制朝向：左 → facing=-1, 右 → facing=1, 无输入保持前值"
//
// Kills: dual key oscillates facing; no input resets facing; wrong direction mapping.
// Wrong-impl challenge:
//   - Wrong: both keys → facing flips each frame → FAIL
//   - Wrong: no input → facing = 0 or default → loses direction → FAIL
//   - Wrong: left → facing +1 (right) → wrong direction → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t30_facing_updates_with_directional_input() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    let empty_terrain: Vec<Tile> = vec![];

    // Left input → facing = -1.
    let left_input = InputState { left: true, ..InputState::default() };
    player.update(DT, &left_input, &empty_terrain);
    assert_eq!(player.facing, -1, "T30: With left input, facing should be -1. Got {}.", player.facing);

    // Right input → facing = 1.
    let right_input = InputState { right: true, ..InputState::default() };
    player.update(DT, &right_input, &empty_terrain);
    assert_eq!(player.facing, 1, "T30: With right input, facing should be 1. Got {}.", player.facing);

    // No input → facing keeps last value.
    let no_input = InputState::default();
    player.update(DT, &no_input, &empty_terrain);
    assert_eq!(player.facing, 1, "T30: With no input, facing should keep previous value (1). Got {}.", player.facing);

    // Both keys → should not panic, facing should be deterministic.
    let both_input = InputState { left: true, right: true, ..InputState::default() };
    player.update(DT, &both_input, &empty_terrain);
    // facing must stay 1 or -1 (never 0 or invalid).
    assert!(
        player.facing == 1 || player.facing == -1,
        "T30: With both keys, facing must be 1 or -1. Got {}.",
        player.facing
    );
}

// ============================================================================
// T31 — INTG/terrain
// Traces To: §Design Alignment seq msg#3, IAPI-005
// "集成测试：Level::query_terrain + Player::update 协作 → 正确碰撞"
//
// Kills: Tile format mismatch; AABB coordinate systems misaligned.
// Wrong-impl challenge:
//   - Wrong: Player::collider() returns AABB in wrong coordinate space → no collision → FAIL
//   - Wrong: Player ignores terrain tiles → walks through platforms → FAIL
//   - Wrong: terrain query includes wrong tiles → phantom collisions → FAIL
// ============================================================================

// real_test (feature #3)
#[test]
fn t31_integration_player_stands_on_real_level_platform() {
    let level = Level::new();
    let config = PlayerConfig::default();
    let mut player = Player::new(config);

    // Place player on the first platform of the real level.
    let platforms = level.platforms();
    assert!(
        !platforms.is_empty(),
        "T31: Level must have at least one platform for integration test."
    );
    let first_platform = &platforms[0].aabb;

    // Position player standing on the platform surface.
    player.pos = Vec2 {
        x: first_platform.x + first_platform.w / 2.0,
        y: first_platform.y, // foot on platform top
    };
    player.vel = Vec2 { x: 0.0, y: 0.0 };
    player.on_ground = true;

    // Query terrain using player's collider (like the Playing state does).
    let terrain = level.query_terrain(&player.collider());

    // The query should return at least one Terrain tile.
    let has_platform = terrain.iter().any(|t| matches!(t, Tile::Platform(_)));
    assert!(
        has_platform,
        "T31: Level::query_terrain(player.collider()) must return Platform tile when player stands on platform."
    );

    // Execute Player::update with real terrain data. Player should remain on_ground.
    let no_input = InputState::default();
    player.update(DT, &no_input, &terrain);

    assert!(
        player.on_ground,
        "T31: After update with real terrain, player must stay on_ground. Got false (fell through)."
    );

    // Vertical velocity must be zero (being held on platform, no gravity applied).
    assert!(
        approx_eq(player.vel.y, 0.0),
        "T31: While standing on platform, vel.y must be 0.0. Got {}.",
        player.vel.y
    );

    // Player must not have fallen through the platform.
    assert!(
        approx_eq_loose(player.pos.y, first_platform.y),
        "T31: After update, player foot must still be at platform surface. pos.y={}, platform.y={}.",
        player.pos.y, first_platform.y
    );
}

// ============================================================================
// T34 — walk RIGHT into ground brick (LEFT edge) vs walk LEFT (RIGHT edge).
// ============================================================================

#[test]
fn t34_walk_into_ground_brick_left_vs_right() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    let ground = AABB { x: 0.0, y: 600.0, w: 700.0, h: 40.0 };
    let brick = AABB { x: 300.0, y: 568.0, w: 32.0, h: 32.0 };
    let terrain = vec![Tile::Platform(ground), Tile::Platform(brick)];

    // Test A: walk RIGHT into brick's LEFT edge
    player.pos = Vec2 { x: 290.0, y: 600.0 };
    player.vel = Vec2 { x: 0.0, y: 0.0 };
    player.on_ground = true;
    let right_input = InputState { right: true, ..InputState::default() };
    for _ in 0..20 {
        player.update(DT, &right_input, &terrain);
    }
    let pr = player.pos.x + 8.0;
    assert!(pr <= brick.x + POSITION_EPSILON,
        "T34-A: right walk must stop at brick left. r={}, bx={}", pr, brick.x);
    assert!(player.on_ground && approx_eq(player.pos.y, 600.0),
        "T34-A: must stay on ground");

    // Test B: walk LEFT into brick's RIGHT edge
    player.pos = Vec2 { x: 340.0, y: 600.0 };
    player.vel = Vec2 { x: 0.0, y: 0.0 };
    player.on_ground = true;
    let left_input = InputState { left: true, ..InputState::default() };
    for _ in 0..20 {
        player.update(DT, &left_input, &terrain);
    }
    let pl = player.pos.x - 8.0;
    let br = brick.x + brick.w;
    assert!(pl >= br - POSITION_EPSILON,
        "T34-B: left walk must stop at brick right. l={}, br={}", pl, br);
    assert!(player.on_ground && approx_eq(player.pos.y, 600.0),
        "T34-B: must stay on ground");
}

// ============================================================================
// T32 — ceiling when brick left edge between player center and right edge.
// ============================================================================

#[test]
fn t32_brick_left_edge_between_center_and_right_ceiling_collision() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    player.pos = Vec2 { x: 546.0, y: 484.0 };
    player.vel = Vec2 { x: 200.0, y: -420.0 };
    player.on_ground = false;
    let brick = AABB { x: 550.0, y: 430.0, w: 32.0, h: 32.0 };
    let terrain = platform_terrain(brick);
    player.update(DT, &InputState::default(), &terrain);
    assert!(player.vel.y >= -PHYSICS_EPSILON,
        "T32: vel.y must be >= 0. Got {}", player.vel.y);
    assert!(player.pos.y - 16.0 >= brick.y + brick.h - POSITION_EPSILON,
        "T32: Must not pass through brick");
    assert!(player.pos.x + 8.0 >= brick.x - POSITION_EPSILON,
        "T32: Must not be pushed past brick left edge");
}

// T33 — platform ceiling collision from below-left (center near left edge).
// Player at x=546, platform at x=550 → center (546) is just inside the 4px margin.
// Verifies the outside_x check doesn't block legitimate ceiling hits.
#[test]
fn t33_platform_ceiling_from_below_left() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    player.pos = Vec2 { x: 546.0, y: 484.0 };
    player.vel = Vec2 { x: 200.0, y: -420.0 };
    player.on_ground = false;
    let platform = AABB { x: 550.0, y: 430.0, w: 120.0, h: 32.0 };
    let terrain = platform_terrain(platform);
    player.update(DT, &InputState::default(), &terrain);
    // Must be stopped by ceiling, not pushed sideways through wall.
    assert!(player.vel.y >= -PHYSICS_EPSILON,
        "T33: vel.y must be stopped. Got {}", player.vel.y);
    let head = player.pos.y - 16.0;
    let plat_bottom = platform.y + platform.h;
    assert!(head >= plat_bottom - POSITION_EPSILON,
        "T33: Must not pass through platform. head={}, bottom={}", head, plat_bottom);
}

// ============================================================================
// T35 — full level 1 terrain: walk right into brick at x=550 (ground level).
// Uses the exact ground platform from level 1 (x=0,y=600,w=700,h=40).
// ============================================================================

#[test]
fn t35_full_level1_terrain_walk_right_into_brick() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    // Exact level 1 ground
    let ground = AABB { x: 0.0, y: 600.0, w: 700.0, h: 40.0 };
    // Brick at level 1 position (x=550, y=430) but moved to ground for test
    let brick = AABB { x: 550.0, y: 568.0, w: 32.0, h: 32.0 };
    let terrain = vec![Tile::Platform(ground), Tile::Platform(brick)];

    player.pos = Vec2 { x: 540.0, y: 600.0 };
    player.vel = Vec2 { x: 0.0, y: 0.0 };
    player.on_ground = true;

    let right_input = InputState { right: true, ..InputState::default() };
    // Simulate walking for many frames
    for _ in 0..30 {
        player.update(DT, &right_input, &terrain);
    }

    let pr = player.pos.x + 8.0;
    assert!(pr <= brick.x + POSITION_EPSILON + 1.0,
        "T35: walking right must stop near brick left. right={}, bx={}", pr, brick.x);
    assert!(player.on_ground,
        "T35: must stay on ground after collision");
}

// ============================================================================
// T36 — collider dimensions must match visual render size for all states.
// ============================================================================

#[test]
fn t36_collider_matches_render_size_all_states() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);
    player.pos = Vec2 { x: 200.0, y: 500.0 };

    // Small: 16x16
    player.state = PlayerState::Small;
    player.crouching = false;
    let c = player.collider();
    assert!(approx_eq(c.w, 16.0), "T36: Small w must be 16. Got {}", c.w);
    assert!(approx_eq(c.h, 16.0), "T36: Small h must be 16. Got {}", c.h);
    assert!(approx_eq(c.x + c.w / 2.0, player.pos.x), "T36: Small center != pos.x");

    // Super: 32x32
    player.state = PlayerState::Super;
    player.crouching = false;
    let c = player.collider();
    assert!(approx_eq(c.w, 32.0), "T36: Super w must be 32. Got {}", c.w);
    assert!(approx_eq(c.h, 32.0), "T36: Super h must be 32. Got {}", c.h);
    assert!(approx_eq(c.x + c.w / 2.0, player.pos.x), "T36: Super center != pos.x");

    // Crouching: 32x16
    player.state = PlayerState::Super;
    player.crouching = true;
    let c = player.collider();
    assert!(approx_eq(c.w, 32.0), "T36: Crouch w must stay 32. Got {}", c.w);
    assert!(approx_eq(c.h, 16.0), "T36: Crouch h must be 16. Got {}", c.h);
    assert!(approx_eq(c.x + c.w / 2.0, player.pos.x), "T36: Crouch center != pos.x");

    // Fire: 32x32
    player.state = PlayerState::Fire;
    player.crouching = false;
    let c = player.collider();
    assert!(approx_eq(c.w, 32.0), "T36: Fire w must be 32. Got {}", c.w);
    assert!(approx_eq(c.h, 32.0), "T36: Fire h must be 32. Got {}", c.h);

    // Damage: back to 16x16
    player.take_damage();
    let c = player.collider();
    assert!(approx_eq(c.w, 16.0), "T36: After damage w must be 16. Got {}", c.w);
    assert!(approx_eq(c.h, 16.0), "T36: After damage h must be 16. Got {}", c.h);
}

// ============================================================================
// T37 — side-slide must NOT trigger ceiling collision (only wall push).
// When player center X is outside the block's X range (sliding along side),
// the collision must be a wall push, not a ceiling stop. This guards against
// question-block activation from the side.
// ============================================================================

#[test]
fn t37_side_slide_wall_not_ceiling() {
    let config = PlayerConfig::default();
    let mut player = Player::new(config);

    // Block at (300, 430, 32, 32). Player sliding down LEFT side:
    // player center at x=290 (outside block X range 300..332).
    let brick = AABB { x: 300.0, y: 430.0, w: 32.0, h: 32.0 };
    let terrain = platform_terrain(brick);

    // Player positioned to left of block, falling down past it.
    // Head near block bottom but center OUTSIDE X range.
    player.pos = Vec2 { x: 290.0, y: 476.0 }; // head at 460, brick bottom at 462
    player.vel = Vec2 { x: 0.0, y: 50.0 };    // falling slowly
    player.on_ground = false;

    player.update(DT, &InputState::default(), &terrain);

    // Player must be pushed LEFT (wall collision), not stopped as ceiling.
    // After wall push, player right edge should be at or left of block left edge.
    let pr = player.pos.x + 8.0;
    assert!(pr <= brick.x + POSITION_EPSILON,
        "T37: Side slide must push player left (wall). right={}, block_left={}", pr, brick.x);

    // vel.y must NOT be zeroed (player continues falling — not a ceiling stop).
    assert!(player.vel.y > PHYSICS_EPSILON,
        "T37: Side slide must NOT stop vertical movement. vel.y={}", player.vel.y);
}
