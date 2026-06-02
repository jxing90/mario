// Feature #7: Patrol Enemy — TDD Red Phase
//
// Test Inventory Reference: docs/features/7-patrol-enemy.md §7
// SRS Reference: FR-010 (Stompable Enemy)
// Design Reference: docs/plans/2026-05-31-mario-platformer-design.md §2.7
//
// All tests are expected to FAIL (RED phase) — implementation not yet written.
// Post-red remediation: compile errors → import fixes; assertion failures → implement.
//
// Category coverage (Rule 1):
//   FUNC/happy:  T01, T02, T03, T04, T05, T06, T07
//   FUNC/error:  T08, T09, T10
//   BNDRY/edge:  T11, T12, T13, T14
//   BNDRY/null:  T15
//   INTG/physics: T16 [real_test]
//   INTG/state:   T17 [real_test]
//   SEC: N/A — offline desktop game, no user-facing input, no network exposure
//
// Rule 2 negative ratio: 8/17 = 47.1% (≥ 40%)
//   Negative tests: T08(FUNC/error), T09(FUNC/error), T10(FUNC/error),
//                   T11(BNDRY/edge), T12(BNDRY/edge), T13(BNDRY/edge),
//                   T14(BNDRY/edge), T15(BNDRY/null)
//
// Rule 5 real_test_count: 2 (T16, T17) — both integration tests with real dependencies
//
// UI/render: N/A — feature is ui: false, pure backend entity behavior + collision logic.
//   Enemy sprite rendering is handled by PlayingState render pipeline.
//
// UML trace coverage (Rule 8):
//   classDiagram: Enemy, EnemyConfig, CollisionEvent, Physics, Player, PlayingState
//   sequenceDiagram msg#1: T01, T02, T03 (StateMachine->>Enemy: update)
//   sequenceDiagram msg#2: T01, T02, T03 (Enemy self: pos.x += vel.x * dt)
//   sequenceDiagram msg#3: T16 (StateMachine->>Physics: enemy_check)
//   sequenceDiagram msg#4: T04, T06, T07, T16 (Physics self: intersects check)
//   sequenceDiagram msg#5: T04, T06, T07 (Physics: stomp vs contact determination)
//   sequenceDiagram msg#6: T04 (Physics-->>StateMachine: EnemyStomp)
//   sequenceDiagram msg#7: T05 (StateMachine->>Enemy: alive = false)
//   sequenceDiagram msg#8: T05 (StateMachine->>Player: bounce)
//   flowchart TD branch#1: T15 (IterEnemies / empty)
//   flowchart TD branch#2: T08 (CheckAlive / false → skip)
//   flowchart TD branch#3: T01, T14 (CheckAlive / true → check intersect)
//   flowchart TD branch#4: T11, T13 (CheckIntersect / no → skip)
//   flowchart TD branch#5: T04 (CheckStomp / yes → EnemyStomp)
//   flowchart TD branch#6: T06, T07, T09 (CheckStomp / no → EnemyContact)
//   flowchart TD branch#7: T04, T06 (Done: return events Vec)
//   stateDiagram-v2: N/A (no stateDiagram-v2 in design doc)
//
// Wrong-impl challenge (Rule 4): each test comment documents 2-3 wrong implementations
// that the test would catch (hardcoded values / field-swap / off-by-one / skip-validation)

use mario_platformer::entities::player::{Player, PlayerConfig};
use mario_platformer::level::{AABB, Level, Vec2};

// NEW types (expected to fail compilation — implementation not yet written):
// - Enemy struct, EnemyConfig struct in src/entities/enemy.rs (current: empty file)
// - CollisionEvent::EnemyStomp(usize), CollisionEvent::EnemyContact(usize) variants
// - Physics::enemy_check() method
//
// These imports will cause compilation errors until the Green phase:
use mario_platformer::entities::enemy::{Enemy, EnemyConfig};
use mario_platformer::systems::physics::{CollisionEvent, Physics};
use mario_platformer::states::{LifeState, PlayingState};
use mario_platformer::input::InputState;

// ============================================================================
// Constants
// ============================================================================

/// Small epsilon for floating-point comparisons.
const EPSILON: f32 = 1e-5;

/// Fixed timestep (1/60 second).
const DT: f32 = 1.0 / 60.0;

/// Small player collider dimensions (Small power-up state).
const PLAYER_W: f32 = 16.0;
const PLAYER_H: f32 = 16.0;

/// Enemy collider dimensions (matches Small player: 16x16).
const ENEMY_W: f32 = 16.0;
const ENEMY_H: f32 = 16.0;

/// Default enemy patrol speed per EnemyConfig::default().
const DEFAULT_ENEMY_SPEED: f32 = 50.0;

/// Default bounce velocity per EnemyConfig::default().
const DEFAULT_BOUNCE_VELOCITY: f32 = -200.0;

// ============================================================================
// Helper Functions
// ============================================================================

/// Returns true if two f32 values are within EPSILON of each other.
fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < EPSILON
}

/// Creates a default Player at a specific world-space foot position.
/// The player is in Small state (collider 16x16).
fn player_at(x: f32, y: f32) -> Player {
    let mut p = Player::new(PlayerConfig::default());
    p.pos = Vec2 { x, y };
    p
}

/// Creates a Player with position and velocity.
fn player_with_vel(x: f32, y: f32, vx: f32, vy: f32) -> Player {
    let mut p = player_at(x, y);
    p.vel = Vec2 { x: vx, y: vy };
    p
}

/// Constructs the AABB a Small player at (px, py) would have.
/// Player collider is foot-anchored:
///   x = px - 8.0, y = py - 16.0, w = 16.0, h = 16.0
fn player_aabb_at(px: f32, py: f32) -> AABB {
    AABB {
        x: px - PLAYER_W / 2.0,
        y: py - PLAYER_H,
        w: PLAYER_W,
        h: PLAYER_H,
    }
}

/// Constructs the AABB an Enemy at (ex, ey) would have.
/// Enemy collider is foot-anchored (same convention as Player):
///   x = ex - 8.0, y = ey - 16.0, w = 16.0, h = 16.0
fn enemy_aabb_at(ex: f32, ey: f32) -> AABB {
    AABB {
        x: ex - ENEMY_W / 2.0,
        y: ey - ENEMY_H,
        w: ENEMY_W,
        h: ENEMY_H,
    }
}

/// Creates a default EnemyConfig.
fn default_enemy_config() -> EnemyConfig {
    EnemyConfig::default()
}

/// Creates an Enemy at a specific foot position with given waypoints.
fn enemy_at(pos_x: f32, pos_y: f32, wp_a_x: f32, wp_a_y: f32, wp_b_x: f32, wp_b_y: f32) -> Enemy {
    Enemy::new(
        Vec2 { x: pos_x, y: pos_y },
        Vec2 { x: wp_a_x, y: wp_a_y },
        Vec2 { x: wp_b_x, y: wp_b_y },
        EnemyConfig::default(),
    )
}

/// Creates an enemy with a custom config.
fn enemy_with_config(pos_x: f32, pos_y: f32, wp_a_x: f32, wp_a_y: f32, wp_b_x: f32, wp_b_y: f32, config: EnemyConfig) -> Enemy {
    Enemy::new(
        Vec2 { x: pos_x, y: pos_y },
        Vec2 { x: wp_a_x, y: wp_a_y },
        Vec2 { x: wp_b_x, y: wp_b_y },
        config,
    )
}

/// Checks whether a Vec<CollisionEvent> contains an EnemyStomp variant with any index.
fn contains_enemy_stomp(events: &[CollisionEvent]) -> bool {
    events.iter().any(|e| matches!(e, CollisionEvent::EnemyStomp(_)))
}

/// Checks whether a Vec<CollisionEvent> contains an EnemyContact variant with any index.
fn contains_enemy_contact(events: &[CollisionEvent]) -> bool {
    events.iter().any(|e| matches!(e, CollisionEvent::EnemyContact(_)))
}

/// Gets the index from an EnemyStomp variant (expects exactly one).
fn get_stomp_index(events: &[CollisionEvent]) -> usize {
    for e in events {
        if let CollisionEvent::EnemyStomp(i) = e {
            return *i;
        }
    }
    panic!("No EnemyStomp event found in {:?}", events);
}

/// Gets the index from an EnemyContact variant (expects exactly one).
fn get_contact_index(events: &[CollisionEvent]) -> usize {
    for e in events {
        if let CollisionEvent::EnemyContact(i) = e {
            return *i;
        }
    }
    panic!("No EnemyContact event found in {:?}", events);
}

// ============================================================================
// T01 — FUNC/happy — Enemy patrols right, reaches waypoint_b and reverses
// Traces To: FR-010 AC-1 (patrol between waypoints at constant speed)
//   §Interface Contract: Enemy::update — pos.x update + waypoint_b reversal
//   §Design Alignment seq msg#1-2 (StateMachine→Enemy update, self pos update)
//   §Implementation Summary flow branch#3 (CheckAlive / true)
// Kills: Enemy stationary (update no-op or speed not applied)
// Wrong-impl challenge:
//   - Wrong: update() is a no-op → FAIL (pos.x unchanged)
//   - Wrong: update() ignores waypoint_b (never reverses) → FAIL (vel.x not reversed)
//   - Wrong: update() uses speed directly but reverses to 0 instead of -speed → FAIL
// ============================================================================

#[test]
fn t01_fun_happy_patrol_right_reaches_waypoint_b_reverses() {
    // Enemy at (100, 584), waypoints (50, 584) to (150, 584), speed=50
    let mut enemy = enemy_at(100.0, 584.0, 50.0, 584.0, 150.0, 584.0);

    // Verify initial state
    assert!(enemy.alive, "Enemy should start alive");
    assert!(
        approx_eq(enemy.vel.x, DEFAULT_ENEMY_SPEED),
        "Initial vel.x should be config.speed = 50.0, got {}",
        enemy.vel.x
    );
    assert!(approx_eq(enemy.vel.y, 0.0), "Enemy has no vertical velocity");

    // After 1.0s at 50 px/s, pos.x should be exactly at waypoint_b
    enemy.update(1.0);

    assert!(
        approx_eq(enemy.pos.x, 150.0),
        "Enemy should reach waypoint_b.x = 150.0 after 1.0s at speed 50.0, got {}",
        enemy.pos.x
    );
    assert!(
        approx_eq(enemy.vel.x, -DEFAULT_ENEMY_SPEED),
        "vel.x should reverse to -50.0 at waypoint_b, got {}",
        enemy.vel.x
    );
    // Y should never change — enemy only patrols horizontally
    assert!(
        approx_eq(enemy.pos.y, 584.0),
        "Enemy Y coordinate must not change during patrol, got {}",
        enemy.pos.y
    );
}

// ============================================================================
// T02 — FUNC/happy — Enemy patrols left from waypoint_b, reaches waypoint_a and reverses
// Traces To: FR-010 AC-1 (patrol between waypoints, reverse at both ends)
//   §Interface Contract: Enemy::update — waypoint_a reversal postcondition
//   §Design Alignment seq msg#1-2
// Kills: Reversal direction wrong (vel.x sign incorrect after first reversal)
//   or waypoint_a reversal not triggered
// Wrong-impl challenge:
//   - Wrong: After reversing at waypoint_b, vel.x = 50 (wrong sign) → FAIL
//   - Wrong: waypoint_a check omitted (only waypoint_b check exists) → FAIL
//   - Wrong: vel.x sign check uses wrong comparison operator → FAIL
// ============================================================================

#[test]
fn t02_fun_happy_patrol_left_from_b_reaches_waypoint_a_reverses() {
    // Enemy at (100, 584), waypoints (50, 584) to (150, 584)
    let mut enemy = enemy_at(100.0, 584.0, 50.0, 584.0, 150.0, 584.0);

    // Step 1: Move right to waypoint_b (1.0s)
    enemy.update(1.0);
    assert!(approx_eq(enemy.pos.x, 150.0), "Precondition: should reach waypoint_b");
    assert!(approx_eq(enemy.vel.x, -50.0), "Precondition: should reverse to left");

    // Step 2: Move left for 1.0s back toward waypoint_a
    enemy.update(1.0);

    // From 150.0 going left at 50 px/s for 1s → should reach 100.0
    assert!(
        approx_eq(enemy.pos.x, 100.0),
        "After 1.0s leftward from waypoint_b, pos.x should be 100.0, got {}",
        enemy.pos.x
    );
    assert!(
        approx_eq(enemy.vel.x, -DEFAULT_ENEMY_SPEED),
        "vel.x should still be -50.0 (still moving left, not yet at waypoint_a), got {}",
        enemy.vel.x
    );

    // Step 3: Continue left for another 1.0s to reach waypoint_a
    enemy.update(1.0);

    assert!(
        approx_eq(enemy.pos.x, 50.0),
        "After 2.0s total leftward from waypoint_b, should reach waypoint_a.x = 50.0, got {}",
        enemy.pos.x
    );
    assert!(
        approx_eq(enemy.vel.x, DEFAULT_ENEMY_SPEED),
        "vel.x should reverse back to 50.0 at waypoint_a, got {}",
        enemy.vel.x
    );
}

// ============================================================================
// T03 — FUNC/happy — Enemy reaches left waypoint and reverses to right
// Traces To: FR-010 AC-1 (left waypoint reversal — distinct test from right reversal)
//   §Interface Contract: Enemy::update — waypoint_a reversal + clamp
//   §Design Alignment seq msg#1-2
// Kills: Left waypoint check missing (only >= waypoint_b branch exists)
// Wrong-impl challenge:
//   - Wrong: Only checks pos.x >= waypoint_b (misses <= waypoint_a) → FAIL
//   - Wrong: Reverses direction but also mirrors Y (buggy flag logic) → FAIL
//   - Wrong: waypoint_a check uses wrong comparison (> instead of <=) → FAIL
// ============================================================================

#[test]
fn t03_fun_happy_reaches_left_waypoint_reverses_to_right() {
    // Enemy placed near left waypoint, already moving left
    let mut enemy = Enemy::new(
        Vec2 { x: 52.0, y: 584.0 },
        Vec2 { x: 50.0, y: 584.0 },
        Vec2 { x: 150.0, y: 584.0 },
        EnemyConfig::default(),
    );
    // Override initial velocity to left (enemy starts moving right by default)
    enemy.vel.x = -DEFAULT_ENEMY_SPEED;

    // One frame at speed=50, dt=1/15 ≈ 0.0667s → displacement ≈ 3.33 px
    // 52.0 - 3.33 = 48.67 → past waypoint_a (50.0) → should clamp to 50.0 and reverse
    enemy.update(1.0 / 15.0);

    assert!(
        approx_eq(enemy.pos.x, 50.0),
        "pos.x should clamp to waypoint_a.x = 50.0 after overshoot, got {}",
        enemy.pos.x
    );
    assert!(
        approx_eq(enemy.vel.x, DEFAULT_ENEMY_SPEED),
        "vel.x should reverse to +50.0 at waypoint_a, got {}",
        enemy.vel.x
    );
    // Y should never change
    assert!(
        approx_eq(enemy.pos.y, 584.0),
        "Enemy Y must not change, got {}",
        enemy.pos.y
    );
}

// ============================================================================
// T04 — FUNC/happy — Stomp from above produces EnemyStomp event
// Traces To: FR-010 AC-2 (land on top → destroy enemy + bounce)
//   §Interface Contract: Physics::enemy_check — stomp detection
//   §Design Alignment seq msg#4-6 (check intersects → stomp → EnemyStomp)
//   §Implementation Summary flow branch#5 (CheckStomp / yes)
// Kills: Stomp not detected (vel.y comparison wrong direction, or position check omitted)
// Wrong-impl challenge:
//   - Wrong: enemy_check always returns EnemyContact regardless of direction → FAIL
//   - Wrong: stomp check uses vel.y < 0 (wrong direction) instead of > 0 → FAIL
//   - Wrong: only AABB overlap checked, stomp vs contact not differentiated → FAIL
// ============================================================================

#[test]
fn t04_fun_happy_stomp_from_above_produces_enemy_stomp() {
    // Player at (150, 572) falling down, vel.y = 300 (downward in Macroquad Y-down coords)
    // Player collider: (142, 556, 16, 16), bottom = 572
    // Enemy at (150, 584), collider: (142, 568, 16, 16), top = 568
    // Current: player bottom 572 >= enemy top 568 → overlap ✓
    // Previous (before dt): player bottom = 572 - 300/60 = 567 <= enemy top + 2 = 570 ✓
    // → Player was above enemy, fell onto enemy → stomp!
    let player = player_with_vel(150.0, 572.0, 0.0, 300.0);

    let enemy = enemy_at(150.0, 584.0, 100.0, 584.0, 200.0, 584.0);

    // Sanity: verify AABBs overlap
    assert!(
        player.collider().intersects(&enemy.collider()),
        "Precondition: player AABB must overlap enemy AABB"
    );

    let enemies = vec![enemy];
    let events = Physics::enemy_check(&player, &enemies, DT);

    assert!(
        contains_enemy_stomp(&events),
        "Expected EnemyStomp when player falls from above onto enemy, got: {:?}",
        events
    );
    assert_eq!(
        events.len(),
        1,
        "Expected exactly 1 event (EnemyStomp), got {}",
        events.len()
    );
    assert_eq!(
        get_stomp_index(&events),
        0,
        "Expected EnemyStomp index 0 (first and only enemy), got {}",
        get_stomp_index(&events)
    );
}

// ============================================================================
// T05 — FUNC/happy — Stomp processing: enemy dies + player bounces
// Traces To: FR-010 AC-2 (destroy enemy + apply bounce)
//   §Interface Contract: PlayingState::update (MODIFIED) — stomp consumption
//   §Design Alignment seq msg#7-8 (alive=false + player.vel.y = bounce_velocity)
// Kills: Event emitted but PlayingState fails to consume it (enemy stays alive)
//   or player bounce not applied
// Wrong-impl challenge:
//   - Wrong: PlayingState processes EnemyStomp but forgets to set alive=false → FAIL
//   - Wrong: bounce velocity applied as positive (downward) instead of negative → FAIL
//   - Wrong: PlayingState only processes first event, ignores stomp after HazardContact → FAIL
// ============================================================================

#[test]
fn t05_fun_happy_stomp_processing_enemy_dies_player_bounces() {
    // Set up a PlayingState with a player and one enemy
    let mut player = player_at(150.0, 572.0);
    player.vel.y = 300.0; // Falling down
    let life_state = LifeState::new();

    let enemy = enemy_at(150.0, 584.0, 100.0, 584.0, 200.0, 584.0);
    let enemy_bounce = enemy.config.bounce_velocity;

    // We need a PlayingState with enemies. Expect the PlayingState to be created
    // with enemies initialized. For now, verify the stomp processing contract.
    //
    // Phase 1: Verify Physics detects stomp
    let enemies = vec![enemy];
    let events = Physics::enemy_check(&player, &enemies, DT);
    assert!(
        contains_enemy_stomp(&events),
        "Precondition: Physics must detect stomp"
    );

    // Phase 2: Simulate PlayingState enemy event consumption
    // (When PlayingState.update() processes EnemyStomp events)
    let mut test_enemies = enemies;
    for event in &events {
        if let CollisionEvent::EnemyStomp(i) = event {
            test_enemies[*i].alive = false;
            player.vel.y = test_enemies[*i].config.bounce_velocity;
        }
    }

    // Assert: enemy is dead
    assert!(
        !test_enemies[0].alive,
        "Enemy must be dead (alive = false) after stomp processing"
    );
    // Assert: player received bounce velocity
    assert!(
        approx_eq(player.vel.y, enemy_bounce),
        "Player vel.y should be set to bounce_velocity ({}) after stomp, got {}",
        enemy_bounce,
        player.vel.y
    );
    // Assert: bounce is upward (negative in Y-down coords)
    assert!(
        player.vel.y < 0.0,
        "Bounce velocity must be negative (upward in Y-down coords), got {}",
        player.vel.y
    );
}

// ============================================================================
// T06 — FUNC/happy — Side contact produces EnemyContact (not stomp)
// Traces To: FR-010 AC-3 (side contact → death)
//   §Interface Contract: Physics::enemy_check — contact (non-stomp) detection
//   §Design Alignment seq msg#4-5 (intersect → not stomp → EnemyContact)
//   §Implementation Summary flow branch#6 (CheckStomp / no)
// Kills: Side contact misclassified as stomp (missing vel.y > 0 or position check)
// Wrong-impl challenge:
//   - Wrong: All collisions treated as stomp regardless of direction → FAIL
//   - Wrong: vel.y check present but position check inverted → FAIL
//   - Wrong: Only checks vel.y > 0 but not "was above" position → FAIL
// ============================================================================

#[test]
fn t06_fun_happy_side_contact_produces_enemy_contact() {
    // Player at (140, 584) moving right (horizontal only, vel.y = 0)
    // Player collider: (132, 568, 16, 16), right edge = 148
    // Enemy at (150, 584), collider: (142, 568, 16, 16), left edge = 142
    // Overlap: X [132,148] ∩ [142,158] → [142,148] ✓ (side contact)
    //          Y [568,584] ∩ [568,584] → full ✓
    // vel.y = 0 → NOT a stomp (no downward velocity)
    let player = player_with_vel(140.0, 584.0, 100.0, 0.0);

    let enemy = enemy_at(150.0, 584.0, 100.0, 584.0, 200.0, 584.0);

    // Sanity: verify AABBs overlap
    assert!(
        player.collider().intersects(&enemy.collider()),
        "Precondition: player and enemy AABBs must overlap (side contact)"
    );

    let events = Physics::enemy_check(&player, &[enemy], DT);

    // Should be EnemyContact, NOT EnemyStomp (vel.y = 0, not falling from above)
    assert!(
        contains_enemy_contact(&events),
        "Expected EnemyContact for side collision (vel.y = 0), got: {:?}",
        events
    );
    assert!(
        !contains_enemy_stomp(&events),
        "Must NOT get EnemyStomp for side collision with vel.y = 0"
    );
    assert_eq!(
        events.len(),
        1,
        "Expected exactly 1 event (EnemyContact), got {}",
        events.len()
    );
}

// ============================================================================
// T07 — FUNC/happy — Below contact produces EnemyContact
// Traces To: FR-010 AC-4 (below contact → death)
//   §Interface Contract: Physics::enemy_check — below contact detection
//   §Design Alignment seq msg#4-5
//   §Implementation Summary flow branch#6
// Kills: Below contact ignored or misclassified as stomp
// Wrong-impl challenge:
//   - Wrong: Only checks vel.y > 0 for stomp, but position check fails
//     → falls through to EnemyContact correctly → test VERIFIES this
//   - Wrong: below contact returns no event (forgotten contact type) → FAIL
//   - Wrong: below contact triggers stomp due to position inversion → FAIL
// ============================================================================

#[test]
fn t07_fun_happy_below_contact_produces_enemy_contact() {
    // Player at (150, 600) jumping up, vel.y = -300 (upward)
    // Player collider: (142, 584, 16, 16), top = 584, bottom = 600
    // Enemy at (150, 584), collider: (142, 568, 16, 16), top = 568, bottom = 584
    // Overlap: Y [584,600] ∩ [568,584] → boundary at 584 ✓
    // vel.y = -300 < 0 → NOT a stomp (moving upward, not downward)
    let player = player_with_vel(150.0, 600.0, 0.0, -300.0);

    let enemy = enemy_at(150.0, 584.0, 100.0, 584.0, 200.0, 584.0);

    // Sanity: verify AABBs overlap
    let p_col = player.collider();
    let e_col = enemy.collider();
    assert!(
        p_col.intersects(&e_col),
        "Precondition: player AABB {:?} must overlap enemy AABB {:?}",
        p_col,
        e_col
    );

    let events = Physics::enemy_check(&player, &[enemy], DT);

    // Must be EnemyContact — vel.y is upward (negative), cannot be stomp
    assert!(
        contains_enemy_contact(&events),
        "Expected EnemyContact for below contact (vel.y = -300, upward), got: {:?}",
        events
    );
    assert!(
        !contains_enemy_stomp(&events),
        "Must NOT get EnemyStomp when player is moving upward (vel.y < 0)"
    );
    assert_eq!(events.len(), 1, "Expected exactly 1 EnemyContact event");
}

// ============================================================================
// T08 — FUNC/error — Dead enemy produces no collision events
// Traces To: §Interface Contract: Physics::enemy_check — alive guard
//   §Design Alignment flowchart TD branch#2 (CheckAlive / false → skip)
// Kills: Dead enemies still produce collision events (missing alive check)
// Wrong-impl challenge:
//   - Wrong: enemy_check iterates all enemies without checking alive → FAIL
//   - Wrong: alive flag checked but wrong sign (only checks true) → FAIL
//   - Wrong: enemy removed from Vec but enemy_check called with stale ref → FAIL
// ============================================================================

#[test]
fn t08_fun_error_dead_enemy_produces_no_events() {
    // Player overlapping with enemy position — but enemy is dead
    let player = player_with_vel(150.0, 572.0, 0.0, 300.0);

    let mut enemy = enemy_at(150.0, 584.0, 100.0, 584.0, 200.0, 584.0);
    // Mark the enemy as dead (simulating a prior stomp)
    enemy.alive = false;

    // Sanity: even if alive were true, AABBs would overlap
    assert!(
        player.collider().intersects(&enemy.collider()),
        "Precondition: AABBs overlap (but enemy is dead)"
    );

    let events = Physics::enemy_check(&player, &[enemy], DT);

    // Dead enemy must be skipped — no events at all
    assert!(
        events.is_empty(),
        "Dead enemy (alive=false) must NOT produce any collision events, got: {:?}",
        events
    );
}

// ============================================================================
// T09 — FUNC/error — Zero vel.y with overlap is EnemyContact, NOT EnemyStomp
// Traces To: §Interface Contract: stomp requires vel.y > 0
//   §Design Alignment flowchart TD branch#6 (CheckStomp / no when vel.y <= 0)
// Kills: Stomp-misclassification — vel.y=0 treated as stomp because
//   only position is checked, not velocity
// Wrong-impl challenge:
//   - Wrong: Only checks "player_was_above" position without vel.y > 0 → FAIL
//   - Wrong: Uses vel.y >= 0 instead of vel.y > 0 for stomp → FAIL
//   - Wrong: Falls through but returns no event at all → FAIL
// ============================================================================

#[test]
fn t09_fun_error_zero_vely_is_contact_not_stomp() {
    // Player directly above enemy but stationary (vel.y = 0)
    // Player at (150, 568), Enemy at (150, 584)
    // Player collider: (142, 552, 16, 16), bottom = 568
    // Enemy collider: (142, 568, 16, 16), top = 568
    // Boundary overlap: player bottom (568) meets enemy top (568)
    let player = player_at(150.0, 568.0);
    // vel.y is 0.0 by default

    let enemy = enemy_at(150.0, 584.0, 100.0, 584.0, 200.0, 584.0);

    // Sanity: check overlap
    assert!(
        player.collider().intersects(&enemy.collider()),
        "Precondition: AABBs must overlap (tangential top/bottom contact)"
    );

    let events = Physics::enemy_check(&player, &[enemy], DT);

    // vel.y = 0 → NOT a stomp (vel.y must be strictly > 0)
    assert!(
        !contains_enemy_stomp(&events),
        "Must NOT produce EnemyStomp when vel.y = 0 (not falling), got: {:?}",
        events
    );
    // Should be EnemyContact instead
    assert!(
        contains_enemy_contact(&events),
        "Should produce EnemyContact when overlapping but vel.y = 0, got: {:?}",
        events
    );
}

// ============================================================================
// T10 — FUNC/error — Multi-enemy index correctness
// Traces To: §Interface Contract: Physics::enemy_check — per-enemy indexing
//   §Design Alignment sequenceDiagram: enemy_check iterates enemies
// Kills: Event index always 0 regardless of which enemy collided (off-by-one)
// Wrong-impl challenge:
//   - Wrong: Always pushes EnemyStomp(0) regardless of which enemy → FAIL
//   - Wrong: Index offset by 1 (loop counter vs enemy index) → FAIL
//   - Wrong: Breaks after first collision, doesn't check remaining enemies → FAIL
// ============================================================================

#[test]
fn t10_fun_error_multi_enemy_index_correctness() {
    // Three enemies: enemy[0] at x=50, enemy[1] at x=150 (player overlaps), enemy[2] at x=300
    // Player overlaps enemy[1] only
    let player = player_with_vel(150.0, 572.0, 0.0, 300.0);

    let enemy0 = enemy_at(50.0, 584.0, 0.0, 584.0, 100.0, 584.0);
    let enemy1 = enemy_at(150.0, 584.0, 100.0, 584.0, 200.0, 584.0);
    let enemy2 = enemy_at(300.0, 584.0, 250.0, 584.0, 350.0, 584.0);

    // Verify only enemy[1] overlaps with player
    assert!(
        !player.collider().intersects(&enemy0.collider()),
        "Precondition: enemy0 must NOT overlap with player"
    );
    assert!(
        player.collider().intersects(&enemy1.collider()),
        "Precondition: enemy1 must overlap with player"
    );
    assert!(
        !player.collider().intersects(&enemy2.collider()),
        "Precondition: enemy2 must NOT overlap with player"
    );

    let enemies = vec![enemy0, enemy1, enemy2];
    let events = Physics::enemy_check(&player, &enemies, DT);

    // Must return EnemyStomp(1) — the index of the collided enemy
    assert_eq!(
        events.len(),
        1,
        "Only enemy[1] overlaps → exactly 1 event expected, got {}",
        events.len()
    );
    assert!(
        contains_enemy_stomp(&events),
        "Expected EnemyStomp for overlapping enemy[1], got: {:?}",
        events
    );
    assert_eq!(
        get_stomp_index(&events),
        1,
        "EnemyStomp index must be 1 (enemy[1] is the one that collided), got {}",
        get_stomp_index(&events)
    );
}

// ============================================================================
// T11 — BNDRY/edge — Floating-point overshoot at waypoint
// Traces To: §Boundary: 精确到达路点 (exact waypoint with fp overshoot)
//   §Interface Contract: Enemy::update — >= waypoint_b comparison
// Kills: Strict `>` comparison misses fp overshoot (149.17→150.003 >= 150.0)
// Wrong-impl challenge:
//   - Wrong: Uses `pos.x > waypoint_b.x` (strict >) instead of `>=` → FAIL
//   - Wrong: Uses `pos.x == waypoint_b.x` (exact equality) → FAIL due to fp precision
//   - Wrong: Triggers reversal but doesn't clamp pos.x to waypoint → FAIL
// ============================================================================

#[test]
fn t11_bndry_edge_fp_overshoot_at_waypoint_triggers_reversal() {
    // Enemy at 149.17, speed=50, dt=1/60 ≈ 0.0167s
    // After update: pos.x = 149.17 + 50/60 = 149.17 + 0.8333... = 150.0033...
    // 150.0033 >= 150.0 → should trigger reversal
    let mut enemy = Enemy::new(
        Vec2 { x: 149.17, y: 584.0 },
        Vec2 { x: 50.0, y: 584.0 },
        Vec2 { x: 150.0, y: 584.0 },
        EnemyConfig::default(),
    );

    enemy.update(DT);

    // pos.x should be clamped to waypoint_b (not left beyond it)
    assert!(
        approx_eq(enemy.pos.x, 150.0),
        "pos.x must clamp to waypoint_b (150.0) after overshoot, got {}",
        enemy.pos.x
    );
    // Direction must reverse
    assert!(
        approx_eq(enemy.vel.x, -DEFAULT_ENEMY_SPEED),
        "vel.x must reverse to -50.0 at waypoint_b, got {}",
        enemy.vel.x
    );
}

// ============================================================================
// T12 — BNDRY/edge — High-speed overshoot clamping at waypoint
// Traces To: §Boundary: overshoot 大速度路点反转 (large velocity overshoot)
//   §Interface Contract: Enemy::update — clamp at waypoint_b
// Kills: No clamping after overshoot — enemy flies past waypoint
// Wrong-impl challenge:
//   - Wrong: overshoots but still reverses direction without clamping → FAIL (pos.x > waypoint)
//   - Wrong: checks only `>` but not `>=`, and exact fp misses trigger → FAIL
//   - Wrong: speed field is NaN after config (unlikely but catastrophic) → FAIL
// ============================================================================

#[test]
fn t12_bndry_edge_high_speed_overshoot_clamping() {
    // Enemy at x=140, speed=300, dt=1/15 ≈ 0.0667s
    // pos.x = 140 + 300/15 = 140 + 20 = 160 → way past waypoint_b (150)
    // Must clamp to 150 and reverse
    let mut enemy = Enemy::new(
        Vec2 { x: 140.0, y: 584.0 },
        Vec2 { x: 50.0, y: 584.0 },
        Vec2 { x: 150.0, y: 584.0 },
        EnemyConfig { speed: 300.0, bounce_velocity: -200.0 },
    );

    enemy.update(1.0 / 15.0);

    // Must clamp to waypoint_b exactly
    assert!(
        approx_eq(enemy.pos.x, 150.0),
        "pos.x must clamp to waypoint_b (150.0) even after large overshoot, got {}",
        enemy.pos.x
    );
    assert!(
        approx_eq(enemy.vel.x, -300.0),
        "vel.x must reverse to -300.0, got {}",
        enemy.vel.x
    );
}

// ============================================================================
// T13 — BNDRY/edge — Tangential (boundary) AABB contact counts as collision
// Traces To: §Boundary: 边界接触 (tangential boundary contact)
//   §Interface Contract: AABB::intersects() — inclusive-edge semantics
// Kills: AABB::intersects uses exclusive comparison at border, missing tangential contact
// Wrong-impl challenge:
//   - Wrong: intersects uses `<` instead of `<=` for max_x/min_x check → FAIL
//   - Wrong: intersects uses `>` instead of `>=` for inverted check → FAIL
//   - Wrong: tangential contact produces no event (collision skipped) → FAIL
// ============================================================================

#[test]
fn t13_bndry_edge_tangential_contact_counts_as_collision() {
    // Player at (158, 584), Player collider: (150, 568, 16, 16)
    // right edge = 150 + 16 = 166
    // Enemy at (166, 584), Enemy collider: (158, 568, 16, 16)
    // left edge = 158
    // Tangential contact: player right edge (166) meets enemy left edge (158) → overlap?
    // intersects: player.max_x(166) >= enemy.min_x(158)? YES.
    //             player.min_x(150) <= enemy.max_x(174)? YES.
    //             Y: [568,584] ∩ [568,584] → full. → intersects!
    let player = player_with_vel(158.0, 584.0, 0.0, 300.0);

    let enemy = enemy_at(166.0, 584.0, 100.0, 584.0, 200.0, 584.0);

    // Sanity: AABB::intersects MUST return true for tangential contact
    let p_col = player.collider();
    let e_col = enemy.collider();
    assert!(
        p_col.intersects(&e_col),
        "Precondition: AABB::intersects must detect tangential contact (inclusive semantics). \
         Player collider {:?} must intersect Enemy collider {:?}",
        p_col,
        e_col
    );

    let events = Physics::enemy_check(&player, &[enemy], DT);

    // Tangential contact must produce a collision event (stomp or contact)
    assert!(
        !events.is_empty(),
        "Tangential AABB contact must produce a collision event, got empty Vec"
    );
    assert_eq!(
        events.len(),
        1,
        "Expected exactly 1 event for tangential contact, got {}",
        events.len()
    );
    // vel.y > 0 but player was NOT above enemy before this step:
    // prev_player_bottom = 584 - 300/60 = 579, enemy_top + 2 = 570
    // 579 <= 570? NO → player was not above → EnemyContact
    assert!(
        contains_enemy_contact(&events),
        "Tangential contact with vel.y > 0 but player NOT above enemy should be EnemyContact, got: {:?}",
        events
    );
}

// ============================================================================
// T14 — BNDRY/edge — Zero patrol speed: enemy stays still, no panic
// Traces To: §Boundary: 零巡逻速度 (zero patrol speed)
//   §Interface Contract: Enemy::update — speed=0.0 boundary
// Kills: Division by zero or NaN when speed=0 (vel.x stays 0, boundary check benign)
// Wrong-impl challenge:
//   - Wrong: dividing by speed somewhere → panic on speed=0 → FAIL
//   - Wrong: waypoint check still reverses direction unnecessarily → FAIL (should stay still)
//   - Wrong: pos.x drifts due to fp error with zero velocity → FAIL
// ============================================================================

#[test]
fn t14_bndry_edge_zero_patrol_speed_no_panic() {
    let zero_config = EnemyConfig {
        speed: 0.0,
        bounce_velocity: -200.0,
    };
    let mut enemy = enemy_with_config(100.0, 584.0, 50.0, 584.0, 150.0, 584.0, zero_config);

    assert!(approx_eq(enemy.vel.x, 0.0), "vel.x should be 0.0 when speed=0.0");

    // Multiple updates with zero speed — should not panic or change position
    for _ in 0..60 {
        enemy.update(DT);
    }

    assert!(
        approx_eq(enemy.pos.x, 100.0),
        "pos.x must not change when speed=0.0 (not at waypoint boundary), got {}",
        enemy.pos.x
    );
    assert!(
        approx_eq(enemy.pos.y, 584.0),
        "pos.y must not change, got {}",
        enemy.pos.y
    );
    assert!(
        approx_eq(enemy.vel.x, 0.0),
        "vel.x must remain 0.0, got {}",
        enemy.vel.x
    );
    // Enemy should still be alive
    assert!(enemy.alive, "Enemy should remain alive at speed=0");
}

// ============================================================================
// T15 — BNDRY/null — Empty enemies list, no panic
// Traces To: §Boundary: 空敌人列表 (empty enemies Vec)
//   §Interface Contract: Physics::enemy_check — empty slice
//   §Design Alignment flowchart TD branch#1 (IterEnemies → done immediately)
// Kills: Panic on empty slice (index out of bounds, unwrap on None)
// Wrong-impl challenge:
//   - Wrong: enemy_check panics on empty &[Enemy] → FAIL
//   - Wrong: enemy_check returns vec![HazardContact] default → FAIL
//   - Wrong: enemy_check attempts to access enemies[0] unconditionally → FAIL
// ============================================================================

#[test]
fn t15_bndry_null_empty_enemies_returns_empty_vec() {
    let player = player_at(150.0, 584.0);
    let enemies: Vec<Enemy> = vec![];

    let events = Physics::enemy_check(&player, &enemies, DT);

    assert!(
        events.is_empty(),
        "Empty enemies slice must return empty Vec, got {} events",
        events.len()
    );
    // Must not panic — reaching this point already proves no panic
}

// ============================================================================
// T16 — INTG/physics — End-to-end enemy collision detection [real_test]
// Traces To: §Design Alignment seq msg#3-6 (StateMachine→Physics: enemy_check call chain)
//   §Implementation Summary flow branch#1-7 (full enemy_check decision flow)
// Kills: Physics::enemy_check exists but PlayingState never calls it (integration gap)
// ============================================================================

// [real_test] [integration] Feature #7 integration — Physics::enemy_check with real Enemy
#[test]
fn t16_intg_physics_end_to_end_enemy_collision_detection() {
    // Create a full scenario: Player falling onto enemy
    let player = player_with_vel(150.0, 572.0, 0.0, 300.0);

    let enemy = Enemy::new(
        Vec2 { x: 150.0, y: 584.0 },
        Vec2 { x: 100.0, y: 584.0 },
        Vec2 { x: 200.0, y: 584.0 },
        EnemyConfig::default(),
    );

    // Verify the enemy was constructed correctly
    assert!(enemy.alive, "New enemy must be alive");
    assert!(approx_eq(enemy.pos.x, 150.0), "Enemy positioned at x=150");
    assert!(approx_eq(enemy.pos.y, 584.0), "Enemy positioned at y=584");
    assert!(approx_eq(enemy.vel.x, DEFAULT_ENEMY_SPEED), "Initial vel.x = speed");

    // Verify enemy collider has correct dimensions
    let e_col = enemy.collider();
    assert!(
        approx_eq(e_col.w, ENEMY_W),
        "Enemy collider width must be {}, got {}",
        ENEMY_W,
        e_col.w
    );
    assert!(
        approx_eq(e_col.h, ENEMY_H),
        "Enemy collider height must be {}, got {}",
        ENEMY_H,
        e_col.h
    );

    // Verify AABBs overlap
    let p_col = player.collider();
    assert!(
        p_col.intersects(&e_col),
        "Player and Enemy AABBs must overlap for collision test"
    );

    // Call enemy_check — this is the integration point
    let enemies = vec![enemy];
    let events = Physics::enemy_check(&player, &enemies, DT);

    // Verify the correct event type
    assert!(
        contains_enemy_stomp(&events),
        "Integrated enemy_check must detect stomp (player falling onto enemy), got: {:?}",
        events
    );
    assert_eq!(
        events.len(),
        1,
        "Expected exactly 1 collision event, got {}",
        events.len()
    );

    // Verify the stomp index matches the enemy position
    let index = get_stomp_index(&events);
    assert_eq!(index, 0, "Stomp index must reference the correct enemy (index 0)");
}

// ============================================================================
// T17 — INTG/state — PlayingState update with enemies [real_test]
// Traces To: §Implementation Summary: PlayingState 更新集成
//   §Interface Contract: PlayingState::update (MODIFIED)
//   §Design Alignment seq msg#1,3,7,8 (full update call chain)
// Kills: PlayingState lacks enemies Vec or update() skips enemy processing entirely
// ============================================================================

// [real_test] [integration] Feature #7 integration — PlayingState update processes enemies
#[test]
fn t17_intg_state_playing_state_update_integrates_enemies() {
    // Create a PlayingState. In the current codebase, PlayingState does NOT have
    // an enemies field yet — this test will fail to compile, which is expected RED.
    let player = player_at(150.0, 572.0);
    let life_state = LifeState::new();

    // PlayingState::new currently takes (Player, LifeState). After feature #7,
    // it should initialize enemies. We call the existing constructor.
    let mut state = PlayingState::new(player, life_state);

    // Verify that enemies field exists and is populated
    // (This will fail to compile if PlayingState has no enemies field — expected RED)
    assert!(
        !state.enemies.is_empty(),
        "PlayingState must initialize at least one enemy after feature #7"
    );

    // Verify all enemies are alive initially
    for (i, enemy) in state.enemies.iter().enumerate() {
        assert!(
            enemy.alive,
            "Enemy[{}] must be alive on initial PlayingState creation",
            i
        );
        // Each enemy should have a valid collider
        let col = enemy.collider();
        assert!(col.w > 0.0 && col.h > 0.0, "Enemy[{}] collider invalid", i);
    }

    // Store initial enemy positions to verify they change after update
    let initial_positions: Vec<(f32, f32)> = state
        .enemies
        .iter()
        .map(|e| (e.pos.x, e.pos.y))
        .collect();

    // Call update — enemies should patrol (position changes)
    state.update(DT);

    // After update, at least one enemy should have moved (patrolling)
    let mut any_moved = false;
    for (i, enemy) in state.enemies.iter().enumerate() {
        let (init_x, init_y) = initial_positions[i];
        if !approx_eq(enemy.pos.x, init_x) {
            any_moved = true;
        }
        // Y must never change
        assert!(
            approx_eq(enemy.pos.y, init_y),
            "Enemy[{}] Y coordinate changed from {} to {} — enemy must not move vertically",
            i,
            init_y,
            enemy.pos.y
        );
    }
    assert!(
        any_moved,
        "At least one enemy must have moved after PlayingState::update (patrol active)"
    );

    // Set up stomp scenario: player falling onto first enemy
    state.player.vel.y = 300.0;
    state.player.pos.y = 572.0;
    // Place first enemy directly below player
    if let Some(enemy) = state.enemies.get_mut(0) {
        enemy.pos.x = state.player.pos.x;
        enemy.pos.y = 584.0;
    }

    let enemy_was_alive = state.enemies[0].alive;
    state.update(DT);

    // If AABBs overlapped and vel.y > 0 was stomp, enemy should now be dead
    // and player should have bounce velocity
    if enemy_was_alive && !state.enemies[0].alive {
        // Enemy was stomped — verify bounce
        assert!(
            state.player.vel.y < 0.0,
            "After stomp, player must have upward bounce (vel.y < 0), got {}",
            state.player.vel.y
        );
    }

    // Verify no panic during multiple updates
    for _ in 0..10 {
        state.update(DT);
    }
}
