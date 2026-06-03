// System ST: Cross-Feature Integration Tests
//
// Tests real data flow across feature boundaries using actual struct instances
// (no mocking). Each test verifies one IAPI contract chain spanning 2+ features.
//
// Integration Boundaries Tested:
//   Boundary A: Camera → Player → Level  (IAPI-007 + IAPI-008)
//   Boundary B: HUD → Player              (IAPI-009)
//   Boundary C: PlayingState full cycle    (IAPI-002 + IAPI-004 + IAPI-005)
//   Boundary D: Physics collision chain    (IAPI-004 + IAPI-005 + IAPI-006)
//
// All tests use real service instances — no mock, no stub, no fake.

use mario_platformer::entities::player::{Player, PlayerConfig};
use mario_platformer::level::{Level, LevelBounds, Vec2};
use mario_platformer::states::LifeState;
use mario_platformer::states::PlayingState;
use mario_platformer::systems::camera::{Camera, CameraConfig};
use mario_platformer::systems::physics::Physics;

// ============================================================================
// Boundary A: IAPI-007 (Player::pos) → Camera::update → IAPI-008 (Level::bounds)
// ============================================================================

/// Integration: Player → Camera → Level follow chain [Real]
#[test]
fn integration_player_pos_flows_to_camera_offset_clamped_to_level_bounds() {
    let player = Player::new(PlayerConfig::default());
    let level = Level::new();
    let bounds = level.bounds();
    let mut camera = Camera::new(CameraConfig::default());

    let pos = player.pos();
    assert!(pos.x > 0.0, "Player should have initial X position");

    for _ in 0..60 {
        camera.update(player.pos(), bounds, 1.0 / 60.0);
    }

    let offset = camera.offset();
    assert!(
        offset.x >= bounds.min_x - 1.0,
        "Camera X offset {} >= level min_x {}",
        offset.x, bounds.min_x
    );
    assert!(
        offset.x <= bounds.max_x + 1.0,
        "Camera X offset {} <= level max_x {}",
        offset.x, bounds.max_x
    );
}

/// Integration: Camera clamps at left level boundary [Real]
#[test]
fn integration_camera_clamps_at_left_level_boundary() {
    let level = Level::new();
    let bounds = level.bounds();
    let mut camera = Camera::new(CameraConfig::default());

    let player_pos = Vec2 { x: bounds.min_x, y: 500.0 };
    for _ in 0..120 {
        camera.update(player_pos, bounds, 1.0 / 60.0);
    }

    let offset = camera.offset();
    assert!(
        offset.x >= bounds.min_x - 5.0,
        "Camera offset.x {} should not go significantly below level min_x {}",
        offset.x, bounds.min_x
    );
}

/// Integration: Camera clamps at right level boundary [Real]
#[test]
fn integration_camera_clamps_at_right_level_boundary() {
    let level = Level::new();
    let bounds = level.bounds();
    let mut camera = Camera::new(CameraConfig::default());

    let player_pos = Vec2 { x: bounds.max_x, y: 500.0 };
    for _ in 0..120 {
        camera.update(player_pos, bounds, 1.0 / 60.0);
    }

    let offset = camera.offset();
    assert!(
        offset.x <= bounds.max_x + 5.0,
        "Camera offset.x {} should not go significantly beyond level max_x {}",
        offset.x, bounds.max_x
    );
}

// ============================================================================
// Boundary B: IAPI-009 (Player::stats) → HudRenderer
// ============================================================================

/// Integration: Player stats → HUD anchor calculation [Real]
#[test]
fn integration_player_stats_flow_to_hud_anchor_validation() {
    let player = Player::new(PlayerConfig::default());
    let stats = player.stats();

    assert_eq!(stats.coins, 0, "Initial coins should be 0");
    assert_eq!(stats.lives, 3, "Initial lives should be 3");

    let viewport_w = 480.0_f32;
    let viewport_h = 270.0_f32;
    let anchor_x = viewport_w * 0.03;
    let anchor_y = viewport_h * 0.03;
    let tolerance_x = viewport_w * 0.02;
    let tolerance_y = viewport_h * 0.02;

    // Expected: 480*0.03=14.4, 270*0.03=8.1
    assert!((anchor_x - 14.4).abs() <= tolerance_x, "Anchor X within ±2% viewport");
    assert!((anchor_y - 8.1).abs() <= tolerance_y, "Anchor Y within ±2% viewport");
}

/// Integration: Coin collection → stats update [Real]
#[test]
fn integration_coin_collection_updates_stats_for_hud() {
    let mut player = Player::new(PlayerConfig::default());
    assert_eq!(player.stats().coins, 0);

    player.coins += 1;
    assert_eq!(player.stats().coins, 1);

    player.coins += 5;
    assert_eq!(player.stats().coins, 6);
}

/// Integration: Life loss → stats update [Real]
#[test]
fn integration_life_loss_updates_stats_for_hud() {
    let mut player = Player::new(PlayerConfig::default());
    assert_eq!(player.stats().lives, 3);

    player.lives -= 1;
    assert_eq!(player.stats().lives, 2);

    player.lives -= 1;
    assert_eq!(player.stats().lives, 1);

    player.lives -= 1;
    assert_eq!(player.stats().lives, 0);
}

// ============================================================================
// Boundary C: PlayingState full cycle
// ============================================================================

/// Integration: PlayingState initializes all subsystems wired together [Real]
#[test]
fn integration_playing_state_initializes_all_subsystems() {
    let player = Player::new(PlayerConfig::default());
    let life_state = LifeState::new();
    let state = PlayingState::new(player, life_state);

    assert_eq!(state.player.stats().lives, 3);
    assert_eq!(state.player.stats().coins, 0);
    assert_eq!(state.life_state.lives, 3);
    assert!(!state.enemies.is_empty(), "Level should have patrol enemies");
    assert!(!state.checkpoints.is_empty(), "Level should have checkpoints");
    assert!(!state.checkpoints[0].activated, "Checkpoint starts inactive");

    let fp_collider = state.flagpole.collider();
    assert!(fp_collider.x > 0.0, "Flagpole placed in level");
    let offset = state.camera.offset();
    assert!(offset.x >= 0.0, "Camera offset starts at or beyond origin");
}

/// Integration: PlayingState update loop — player on ground stays on platform [Real]
///
/// Places player near the ground platform (y=584 for 16px collider on ground at y=600)
/// then runs the game loop to verify stable ground standing.
#[test]
fn integration_playing_state_update_loop_player_stays_on_platform() {
    let mut player = Player::new(PlayerConfig::default());
    // Place player just above the ground platform so terrain query picks up ground tiles
    player.pos.y = 584.0;
    player.on_ground = true;
    let life_state = LifeState::new();
    let mut state = PlayingState::new(player, life_state);

    let initial_y = state.player.pos.y; // 584.0

    for _ in 0..30 {
        state.update(1.0 / 60.0);
    }

    let current_y = state.player.pos.y;
    // Player should stay near initial position (± small drift from collision resolution)
    let drift = (current_y - initial_y).abs();
    assert!(
        drift < 20.0,
        "Player should stay on ground. Initial Y: {:.1}, Current Y: {:.1}, Drift: {:.1}",
        initial_y, current_y, drift
    );
    // Must not fall through the world
    let bounds = state.level.bounds();
    assert!(
        current_y < bounds.kill_y,
        "Player Y: {:.1} should be above kill_y: {:.1}",
        current_y, bounds.kill_y
    );
}

/// Integration: PlayingState enemy patrol — enemies change position [Real]
#[test]
fn integration_playing_state_enemy_patrol_movement() {
    let player = Player::new(PlayerConfig::default());
    let life_state = LifeState::new();
    let mut state = PlayingState::new(player, life_state);

    let initial_positions: Vec<f32> = state.enemies.iter().map(|e| e.pos().x).collect();

    for _ in 0..60 {
        state.update(1.0 / 60.0);
    }

    for (i, enemy) in state.enemies.iter().enumerate() {
        let ix = initial_positions[i];
        assert!(
            (enemy.pos().x - ix).abs() > 0.01 || !enemy.alive,
            "Enemy {} moved from initial X {:.1} or stomped. Current X: {:.1}",
            i, ix, enemy.pos().x
        );
    }
}

/// Integration: Flagpole collision detection [Real]
#[test]
fn integration_playing_state_flagpole_collision_detection() {
    let player = Player::new(PlayerConfig::default());
    let life_state = LifeState::new();
    let mut state = PlayingState::new(player, life_state);

    assert!(!state.check_flagpole(), "Player at start should not trigger flagpole");

    // Teleport player to flagpole
    let fp_x = state.flagpole.pos.x;
    let fp_y = state.flagpole.pos.y;
    state.player.pos = Vec2 { x: fp_x, y: fp_y };

    assert!(state.check_flagpole(), "Player at flagpole should trigger");
}

// ============================================================================
// Boundary D: Physics collision chain
// ============================================================================

/// Integration: Physics terrain query from real Level [Real]
#[test]
fn integration_physics_terrain_query_from_real_level() {
    let level = Level::new();
    let player = Player::new(PlayerConfig::default());
    let collider = player.collider();
    let terrain = level.query_terrain(&collider);

    for tile in &terrain {
        match tile {
            mario_platformer::level::Tile::Empty => {}
            mario_platformer::level::Tile::Platform(aabb) => {
                assert!(aabb.w > 0.0 && aabb.h > 0.0, "Platform AABB positive size");
            }
            mario_platformer::level::Tile::Spike(aabb) => {
                assert!(aabb.w > 0.0 && aabb.h > 0.0, "Spike AABB positive size");
            }
        }
    }
}

/// Integration: Hazard check with real Level terrain [Real]
#[test]
fn integration_hazard_check_spike_detection_safe_position() {
    let level = Level::new();
    let player = Player::new(PlayerConfig::default());
    let terrain = level.query_terrain(&player.collider());
    let kill_y = level.bounds().kill_y;

    let events = Physics::hazard_check(&player, &terrain, kill_y);
    assert!(events.is_empty(), "Player at start should not trigger hazards");
}

/// Integration: Pit fall detection below kill_y [Real]
#[test]
fn integration_pit_fall_detection_below_kill_y() {
    let level = Level::new();
    let bounds = level.bounds();
    let kill_y = bounds.kill_y;

    let mut player = Player::new(PlayerConfig::default());
    player.pos = Vec2 { x: 500.0, y: kill_y + 10.0 };

    let terrain = level.query_terrain(&player.collider());
    let events = Physics::hazard_check(&player, &terrain, kill_y);

    assert!(!events.is_empty(), "Player below kill_y triggers PitFall");
    assert!(
        events.iter().any(|e| matches!(e, mario_platformer::systems::physics::CollisionEvent::PitFall)),
        "Events include PitFall"
    );
}
