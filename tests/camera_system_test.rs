// Feature #4: Camera System — TDD Red Phase
//
// Test Inventory Reference: docs/features/4-camera-system.md §7
// All tests are expected to FAIL (RED phase) — implementation not yet written.
//
// Category coverage (Rule 1):
//   FUNC/happy:   T01-T07, T20-T21  (horizontal convergence, stability, clamps, dead-zone)
//   FUNC/error:   T08-T09          (dt defense: zero, negative)
//   BNDRY/edge:   T10-T15          (dead-zone boundaries, clamp edges, narrow level, epsilon)
//   BNDRY/invalid: T16             (degenerate bounds min_x >= max_x)
//   SEC: N/A — internal utility, no user-facing input injection surface
//   INTG/player:   T17  (#[real_test (feature #4)])
//   INTG/level:    T18  (#[real_test (feature #4)])
//   INTG/parallax: T19  (#[real_test (feature #4)])
//
// Negative test ratio: FUNC/error (2) + BNDRY/edge (6) + BNDRY/invalid (1) = 9 / 21 = 42.9% ≥ 40%
// Rule 2 ✓
//
// Real tests (Rule 5): T17-T19 verify actual Camera integration with Player::pos(),
// Level::bounds(), and ParallaxLayer::update_scroll() without mocking primary
// dependencies. Marker: "real_test (feature #4)" in function comment blocks.

use mario_platformer::entities::player::{Player, PlayerConfig};
use mario_platformer::level::{Level, LevelBounds, Vec2};
use mario_platformer::parallax::ParallaxLayer;
use mario_platformer::systems::camera::{Camera, CameraConfig};

// ============================================================================
// Constants
// ============================================================================

/// Floating-point tolerance for camera convergence comparisons.
const EPSILON: f32 = 0.001;

/// Fixed timestep: 1/60 second.
const DT: f32 = 1.0 / 60.0;

/// Default viewport dimensions per design doc §Implementation Summary.
const VIEWPORT_W: f32 = 480.0;
const VIEWPORT_H: f32 = 270.0;

/// Default convergence rates and config values per SRS FR-013.
const H_CONVERGENCE: f32 = 0.08;
const V_CONVERGENCE: f32 = 0.05;
const DEAD_ZONE_PCT: f32 = 0.60;
const PLAYER_TARGET_X_PCT: f32 = 0.375;

// ============================================================================
// Helper utilities
// ============================================================================

/// Returns true if two f32 values are within EPSILON of each other.
fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < EPSILON
}

/// Creates a CameraConfig with default SRS values.
fn default_camera_config() -> CameraConfig {
    CameraConfig {
        h_convergence: H_CONVERGENCE,
        v_convergence: V_CONVERGENCE,
        dead_zone_pct: DEAD_ZONE_PCT,
        viewport_w: VIEWPORT_W,
        viewport_h: VIEWPORT_H,
        player_target_x_pct: PLAYER_TARGET_X_PCT,
    }
}

/// Creates default LevelBounds for testing (min_x=0, max_x=2000, min_y=0, kill_y=2500).
fn default_level_bounds() -> LevelBounds {
    LevelBounds {
        min_x: 0.0,
        max_x: 2000.0,
        min_y: 0.0,
        kill_y: 2500.0,
    }
}

// ============================================================================
// FUNC/happy — Tests T01-T07
// ============================================================================

/// T01 | FUNC/happy | FR-013 AC-1 | Traces To: §Design Alignment seq msg#3 (Camera::update)
///
/// Verifies horizontal lerp convergence at 8%/frame.
/// Given player at (200, 100) and camera at origin,
/// After one frame: offset.x = (200 - 480*0.375 - 0) * 0.08 = 1.6.
#[test]
fn test_camera_horizontal_convergence_single_frame() {
    let config = default_camera_config();
    let camera = Camera::new(config);
    let player_pos = Vec2 { x: 200.0, y: 100.0 };
    let bounds = default_level_bounds();
    let mut camera = camera;

    camera.update(player_pos, bounds, DT);

    // target_x = 200 - 480 * 0.375 = 200 - 180 = 20
    // delta = (20 - 0) * 0.08 = 1.6
    let expected_x = (player_pos.x - VIEWPORT_W * PLAYER_TARGET_X_PCT) * H_CONVERGENCE;
    // = (200 - 180) * 0.08 = 1.6
    assert!(
        approx_eq(camera.offset().x, expected_x),
        "offset.x should converge 8% toward target; expected {}, got {}",
        expected_x,
        camera.offset().x
    );
}

/// T02 | FUNC/happy | FR-013 AC-1 | Traces To: §Design Alignment seq msg#3
///
/// Verifies steady-state lag stays ≤ 80px after convergence.
/// At max speed (200 px/s), steady-state lag = delta_per_frame / h_convergence
/// where delta_per_frame = 200/60 ≈ 3.33 px/frame.
/// lag = 3.33 / 0.08 ≈ 41.67 px, well under 80px.
/// We converge for many frames to reach steady state, then measure.
#[test]
fn test_camera_horizontal_convergence_steady_state_lag() {
    let config = default_camera_config();
    let mut camera = Camera::new(config);
    let bounds = default_level_bounds();
    // Player moves right at max speed: 200 px/s, 200/60 ≈ 3.33 px/frame
    let mut player_x = 200.0;
    let player_y = 100.0;

    // Converge over 120 frames (2 seconds) to reach steady state
    for _frame in 0..120 {
        player_x += 200.0 * DT; // 200 px/s
        camera.update(
            Vec2 { x: player_x, y: player_y },
            bounds,
            DT,
        );
    }

    let target_x = player_x - VIEWPORT_W * PLAYER_TARGET_X_PCT;
    let lag = target_x - camera.offset().x;
    // Steady-state lag must not exceed 80 pixels (FR-013 AC-1)
    assert!(
        lag.abs() <= 80.0,
        "steady-state lag {} exceeds 80px max",
        lag.abs()
    );
    // Lag should be positive (camera trails behind moving player)
    assert!(
        lag >= 0.0,
        "lag should be non-negative for rightward movement; got {}",
        lag
    );
}

/// T03 | FUNC/happy | FR-013 AC-2 | Traces To: §Design Alignment seq msg#3
///
/// Verifies camera does not oscillate when converged.
/// Given camera already at target offset, repeated updates
/// should produce zero delta (within epsilon).
#[test]
fn test_camera_no_oscillation_when_converged() {
    let config = default_camera_config();
    let mut camera = Camera::new(config);
    let player_x = 400.0;
    let player_y = 100.0;
    let bounds = default_level_bounds();

    // Target: target_x = 400 - 480*0.375 = 400 - 180 = 220
    // Converge camera to target over many frames
    for _frame in 0..300 {
        camera.update(
            Vec2 { x: player_x, y: player_y },
            bounds,
            DT,
        );
    }

    let converged_x = camera.offset().x;
    let target_x = player_x - VIEWPORT_W * PLAYER_TARGET_X_PCT;

    // Camera should be within epsilon of target
    assert!(
        approx_eq(converged_x, target_x),
        "camera should converge to target_x {}; got {}",
        target_x,
        converged_x
    );

    // Run 10 more frames — offset must not drift
    for _frame in 0..10 {
        camera.update(
            Vec2 { x: player_x, y: player_y },
            bounds,
            DT,
        );
    }
    assert!(
        approx_eq(camera.offset().x, converged_x),
        "camera drifted from {} to {} after convergence (must be zero oscillation)",
        converged_x,
        camera.offset().x
    );
}

/// T04 | FUNC/happy | FR-013 AC-3 | Traces To: §Implementation Summary flow branch#2 (clamp below min)
///
/// Verifies camera clamps to left level boundary.
/// Player at far left; camera must not show area left of bounds.min_x.
#[test]
fn test_camera_left_boundary_clamp() {
    let config = default_camera_config();
    let mut camera = Camera::new(config);
    let bounds = LevelBounds {
        min_x: 0.0,
        max_x: 2000.0,
        min_y: 0.0,
        kill_y: 2500.0,
    };
    // Player at left edge; initial camera offset = 100
    // After convergence + clamping, offset.x must be >= 0
    let player_x = 0.0;
    let player_y = 100.0;

    // Run many frames to converge toward target (which is negative)
    for _frame in 0..300 {
        camera.update(
            Vec2 { x: player_x, y: player_y },
            bounds,
            DT,
        );
    }

    // offset.x must be clamped to >= bounds.min_x (= 0)
    assert!(
        camera.offset().x >= bounds.min_x - EPSILON,
        "camera offset.x {} must not go below left boundary {}",
        camera.offset().x,
        bounds.min_x
    );
    // After full convergence, should be exactly at min_x
    assert!(
        approx_eq(camera.offset().x, bounds.min_x),
        "camera should converge to left boundary {}; got {}",
        bounds.min_x,
        camera.offset().x
    );
}

/// T05 | FUNC/happy | FR-013 AC-4 | Traces To: §Implementation Summary flow branch#3 (clamp above max)
///
/// Verifies camera clamps to right level boundary.
/// Player at far right; camera must not show area right of bounds.max_x - viewport_w.
#[test]
fn test_camera_right_boundary_clamp() {
    let config = default_camera_config();
    let mut camera = Camera::new(config);
    let viewport_w = VIEWPORT_W;
    let bounds = LevelBounds {
        min_x: 0.0,
        max_x: 2000.0,
        min_y: 0.0,
        kill_y: 2500.0,
    };
    let max_offset = bounds.max_x - viewport_w;
    // Player at far right edge
    let player_x = 1990.0;
    let player_y = 100.0;

    for _frame in 0..300 {
        camera.update(
            Vec2 { x: player_x, y: player_y },
            bounds,
            DT,
        );
    }

    assert!(
        camera.offset().x <= max_offset + EPSILON,
        "camera offset.x {} must not exceed max offset {} (right boundary)",
        camera.offset().x,
        max_offset
    );
}

/// T06 | FUNC/happy | FR-013 AC-5a | Traces To: §Implementation Summary flow branch#1 (dead-zone exit → yes)
///
/// Verifies vertical tracking when player exits the dead-zone above.
/// dead_top = offset.y + viewport_h * 0.2 = 0 + 54 = 54
/// Player at y=30 (above dead zone) triggers 5%/frame vertical convergence.
#[test]
fn test_camera_vertical_tracking_above_dead_zone() {
    let config = default_camera_config();
    let mut camera = Camera::new(config);
    let bounds = default_level_bounds();
    // Player above dead zone: y=30, dead_top = 0 + 270*0.2 = 54
    let player_pos = Vec2 { x: 200.0, y: 30.0 };

    let old_offset_y = camera.offset().y;
    camera.update(player_pos, bounds, DT);

    // target_y = 30 - 270/2 = 30 - 135 = -105
    // delta_y = (-105 - 0) * 0.05 = -5.25
    let expected_target_y = player_pos.y - VIEWPORT_H / 2.0;
    let expected_delta = (expected_target_y - old_offset_y) * V_CONVERGENCE;
    assert!(
        approx_eq(camera.offset().y, old_offset_y + expected_delta),
        "offset.y should converge 5% toward player center when above dead zone; expected {}, got {}",
        old_offset_y + expected_delta,
        camera.offset().y
    );
    // offset.y must have moved (player is outside dead zone)
    assert!(
        (camera.offset().y - old_offset_y).abs() > EPSILON,
        "camera must track vertically when player exits dead zone"
    );
}

/// T07 | FUNC/happy | FR-013 AC-5b | Traces To: §Implementation Summary flow branch#1 (dead-zone stay → no)
///
/// Verifies camera does NOT track vertically when player is inside dead zone.
/// dead_zone for offset.y=0: [54, 216]; player at y=135 is inside.
#[test]
fn test_camera_vertical_still_in_dead_zone() {
    let config = default_camera_config();
    let mut camera = Camera::new(config);
    let bounds = default_level_bounds();
    // Player inside dead zone: center of viewport
    let player_pos = Vec2 { x: 200.0, y: 135.0 };
    // dead_top = 0 + 270*0.2 = 54, dead_bottom = 0 + 270*0.8 = 216
    // player.y = 135 is inside [54, 216]

    let old_offset_y = camera.offset().y;
    for _frame in 0..30 {
        camera.update(player_pos, bounds, DT);
    }

    // offset.y must stay unchanged within epsilon
    assert!(
        approx_eq(camera.offset().y, old_offset_y),
        "offset.y should NOT change when player is inside dead zone; started at {}, ended at {}",
        old_offset_y,
        camera.offset().y
    );
}

// ============================================================================
// FUNC/error — Tests T08-T09
// ============================================================================

/// T08 | FUNC/error | Interface Contract dt defense | Traces To: §Implementation Summary flow branch#0 (dt guard)
///
/// Verifies dt=0 is a no-op: camera offset must not change, must not panic.
#[test]
fn test_camera_dt_zero_no_op() {
    let config = default_camera_config();
    let mut camera = Camera::new(config);
    let player_pos = Vec2 { x: 500.0, y: 200.0 };
    let bounds = default_level_bounds();
    let old_offset = camera.offset();
    let old_x = old_offset.x;
    let old_y = old_offset.y;

    // This must not panic and must not change state
    camera.update(player_pos, bounds, 0.0);

    assert!(
        approx_eq(camera.offset().x, old_x),
        "dt=0 must be no-op for offset.x; changed from {} to {}",
        old_x,
        camera.offset().x
    );
    assert!(
        approx_eq(camera.offset().y, old_y),
        "dt=0 must be no-op for offset.y; changed from {} to {}",
        old_y,
        camera.offset().y
    );
}

/// T09 | FUNC/error | Interface Contract dt defense | Traces To: §Implementation Summary flow branch#0 (dt guard)
///
/// Verifies negative dt is a no-op: camera offset must not change, must not reverse direction.
#[test]
fn test_camera_dt_negative_no_op() {
    let config = default_camera_config();
    let mut camera = Camera::new(config);
    let player_pos = Vec2 { x: 500.0, y: 200.0 };
    let bounds = default_level_bounds();
    let old_x = camera.offset().x;
    let old_y = camera.offset().y;

    // Negative dt must be treated as no-op
    camera.update(player_pos, bounds, -0.016);

    assert!(
        approx_eq(camera.offset().x, old_x),
        "negative dt must not change offset.x; changed from {} to {}",
        old_x,
        camera.offset().x
    );
    assert!(
        approx_eq(camera.offset().y, old_y),
        "negative dt must not change offset.y; changed from {} to {}",
        old_y,
        camera.offset().y
    );
}

// ============================================================================
// BNDRY/edge — Tests T10-T16
// ============================================================================

/// T10 | BNDRY/edge | dead_zone top boundary | Traces To: §Interface Contract state dead_top→stay
///
/// Player exactly at dead_zone top boundary (y=54 with offset.y=0, viewport_h=270).
/// Boundary-inclusive check: player at the line should be treated as inside dead zone.
#[test]
fn test_camera_dead_zone_top_boundary_inclusive() {
    let config = default_camera_config();
    let mut camera = Camera::new(config);
    let bounds = default_level_bounds();
    // dead_top = 0 + 270 * 0.2 = 54
    let dead_top_y = VIEWPORT_H * (1.0 - DEAD_ZONE_PCT) / 2.0; // = 270 * 0.2 = 54
    let player_pos = Vec2 { x: 200.0, y: dead_top_y };

    let old_offset_y = camera.offset().y;
    for _frame in 0..10 {
        camera.update(player_pos, bounds, DT);
    }

    // At exact boundary, player is considered inside dead zone → no vertical movement
    assert!(
        approx_eq(camera.offset().y, old_offset_y),
        "at dead_zone top boundary (y={}), offset.y should stay unchanged; was {}, now {}",
        dead_top_y,
        old_offset_y,
        camera.offset().y
    );
}

/// T11 | BNDRY/edge | dead_zone bottom boundary | Traces To: §Interface Contract state dead_bottom→stay
///
/// Player exactly at dead_zone bottom boundary (y=216 with offset.y=0, viewport_h=270).
/// Boundary-inclusive check: player at the line should be treated as inside dead zone.
#[test]
fn test_camera_dead_zone_bottom_boundary_inclusive() {
    let config = default_camera_config();
    let mut camera = Camera::new(config);
    let bounds = default_level_bounds();
    // dead_bottom = 0 + 270 * 0.8 = 216
    let dead_bottom_y = VIEWPORT_H * (1.0 + DEAD_ZONE_PCT) / 2.0; // = 270 * 0.8 = 216
    let player_pos = Vec2 { x: 200.0, y: dead_bottom_y };

    let old_offset_y = camera.offset().y;
    for _frame in 0..10 {
        camera.update(player_pos, bounds, DT);
    }

    assert!(
        approx_eq(camera.offset().y, old_offset_y),
        "at dead_zone bottom boundary (y={}), offset.y should stay unchanged; was {}, now {}",
        dead_bottom_y,
        old_offset_y,
        camera.offset().y
    );
}

/// T12 | BNDRY/edge | left clamp at boundary | Traces To: §Implementation Summary flow branch#2 (clamp below min)
///
/// Camera already at left boundary (offset.x=0); player moves further left.
/// offset.x must stay clamped at min_x, not follow player out of bounds.
#[test]
fn test_camera_left_clamp_already_at_boundary() {
    let config = default_camera_config();
    // Create camera and converge to left boundary
    let mut camera = Camera::new(config);
    let bounds = LevelBounds {
        min_x: 0.0,
        max_x: 2000.0,
        min_y: 0.0,
        kill_y: 2500.0,
    };

    // First converge camera to left boundary
    for _frame in 0..300 {
        camera.update(Vec2 { x: 0.0, y: 100.0 }, bounds, DT);
    }

    // Now player moves left off the level (x=-50)
    for _frame in 0..10 {
        camera.update(Vec2 { x: -50.0, y: 100.0 }, bounds, DT);
    }

    // offset.x must stay at 0 (cannot go negative)
    assert!(
        camera.offset().x >= 0.0 - EPSILON,
        "offset.x {} must not go negative when already at left boundary",
        camera.offset().x
    );
}

/// T13 | BNDRY/edge | right clamp at boundary | Traces To: §Implementation Summary flow branch#3 (clamp above max)
///
/// Camera already at right boundary (offset.x = max_x - viewport_w);
/// player moves further right. offset.x must stay clamped.
#[test]
fn test_camera_right_clamp_already_at_boundary() {
    let config = default_camera_config();
    let mut camera = Camera::new(config);
    let viewport_w = VIEWPORT_W;
    let bounds = LevelBounds {
        min_x: 0.0,
        max_x: 2000.0,
        min_y: 0.0,
        kill_y: 2500.0,
    };
    let max_offset = bounds.max_x - viewport_w; // = 1520

    // Converge camera to right boundary
    for _frame in 0..300 {
        camera.update(Vec2 { x: 1990.0, y: 100.0 }, bounds, DT);
    }

    // Now player moves further right (beyond level)
    for _frame in 0..10 {
        camera.update(Vec2 { x: 2500.0, y: 100.0 }, bounds, DT);
    }

    assert!(
        camera.offset().x <= max_offset + EPSILON,
        "offset.x {} must not exceed max_offset {} when already at right boundary",
        camera.offset().x,
        max_offset
    );
}

/// T14 | BNDRY/edge | narrow level (viewport wider than level) | Traces To: §Implementation Summary flow branch#2
///
/// Level narrower than viewport: max_x=300, viewport_w=480,
/// max_offset = 300-480 = -180. offset.x must clamp to min_x=0.
#[test]
fn test_camera_narrow_level_clamp_to_min() {
    let config = default_camera_config();
    let mut camera = Camera::new(config);
    let bounds = LevelBounds {
        min_x: 0.0,
        max_x: 300.0,  // narrower than 480px viewport
        min_y: 0.0,
        kill_y: 2500.0,
    };
    let player_pos = Vec2 { x: 150.0, y: 100.0 };

    for _frame in 0..60 {
        camera.update(player_pos, bounds, DT);
    }

    // max_offset = 300 - 480 = -180, which is < min_x.
    // offset.x must be clamped to min_x (= 0), not allowed to go negative.
    assert!(
        camera.offset().x >= -EPSILON,
        "narrow level: offset.x {} must not go below min_x (0); max_offset would be -180",
        camera.offset().x
    );
    // Should be at or very near min_x
    assert!(
        approx_eq(camera.offset().x, bounds.min_x),
        "narrow level: offset.x should clamp to min_x {}; got {}",
        bounds.min_x,
        camera.offset().x
    );
}

/// T15 | BNDRY/edge | convergence epsilon termination | Traces To: FR-013 AC-2
///
/// When camera is within 1e-6 of target, convergence should stop.
/// No floating-point drift across 60 consecutive frames.
#[test]
fn test_camera_convergence_epsilon_termination() {
    let config = default_camera_config();
    let mut camera = Camera::new(config);
    let bounds = default_level_bounds();
    let player_pos = Vec2 { x: 400.0, y: 100.0 };
    let target_x = player_pos.x - VIEWPORT_W * PLAYER_TARGET_X_PCT;

    // Converge fully (300 frames is plenty)
    for _frame in 0..300 {
        camera.update(player_pos, bounds, DT);
    }

    let converged_x = camera.offset().x;
    // Must be within epsilon of target
    assert!(
        (converged_x - target_x).abs() < 0.01,
        "camera should converge to target {}; got {}",
        target_x,
        converged_x
    );

    // Run 60 more frames — must not drift
    let mut max_delta: f32 = 0.0;
    for _frame in 0..60 {
        let before = camera.offset().x;
        camera.update(player_pos, bounds, DT);
        let after = camera.offset().x;
        let delta = (after - before).abs();
        if delta > max_delta {
            max_delta = delta;
        }
    }

    assert!(
        max_delta < 1e-5,
        "after convergence, max frame delta {} must be < 1e-5 (no drift over 60 frames)",
        max_delta
    );
}

/// T16 | BNDRY/invalid | Boundary Conditions degenerate bounds | Traces To: §Implementation Summary flow branch#2
///
/// Degenerate level bounds: min_x >= max_x (logically invalid).
/// Camera must not panic; should clamp offset.x to min_x.
/// This is a negative test — verifying graceful degradation on invalid input.
#[test]
fn test_camera_degenerate_bounds_no_panic() {
    let config = default_camera_config();
    let mut camera = Camera::new(config);
    // Degenerate bounds: min_x >= max_x is logically invalid
    let bounds = LevelBounds {
        min_x: 100.0,
        max_x: 50.0,  // max_x < min_x — invalid
        min_y: 0.0,
        kill_y: 2500.0,
    };
    let player_pos = Vec2 { x: 75.0, y: 100.0 };

    // Must not panic on degenerate bounds
    camera.update(player_pos, bounds, DT);

    // offset must be finite (no NaN or infinity)
    assert!(
        camera.offset().x.is_finite() && camera.offset().y.is_finite(),
        "camera offset must remain finite with degenerate bounds; got ({}, {})",
        camera.offset().x,
        camera.offset().y
    );
    // offset.x must not go below min_x
    assert!(
        camera.offset().x >= bounds.min_x - EPSILON,
        "with degenerate bounds, offset.x {} must not go below min_x {}",
        camera.offset().x,
        bounds.min_x
    );
}

// ============================================================================
// INTG/player — Test T17 (real_test)
// ============================================================================

/// T17 | INTG/player | IAPI-007 + Interface Contract update() | Traces To: §Design Alignment seq msg#1
///
/// # real_test (feature #4)
///
/// Integration test: Camera correctly consumes Player::pos() through IAPI-007.
/// Creates a real Player instance, reads its position, and verifies camera
/// update uses the actual player coordinates for horizontal targeting.
/// No mocking of primary dependency (Player).
#[test]
fn test_camera_reads_player_position_via_iapi_007() {
    let config = default_camera_config();
    let mut camera = Camera::new(config);
    // Use bounds wide enough that clamping does not interfere:
    // this test validates IAPI-007 (camera reads Player::pos()), not clamping.
    let bounds = LevelBounds {
        min_x: -1000.0,
        max_x: 2000.0,
        min_y: 0.0,
        kill_y: 2500.0,
    };
    let player_config = PlayerConfig::default();
    let player = Player::new(player_config);

    // Get real player position via IAPI-007
    let player_pos = player.pos();
    // Player::new initializes pos at (100, 100)
    assert!(
        approx_eq(player_pos.x, 100.0),
        "player should start at x=100; got {}",
        player_pos.x
    );

    // Camera update using real player position
    camera.update(player_pos, bounds, DT);

    // Camera target = player.x - viewport_w * player_target_x_pct
    let expected_target_x = player_pos.x - VIEWPORT_W * PLAYER_TARGET_X_PCT;
    let expected_delta = (expected_target_x - 0.0) * H_CONVERGENCE;
    assert!(
        approx_eq(camera.offset().x, expected_delta),
        "camera offset.x should reflect player.pos().x (IAPI-007); expected {}, got {}",
        expected_delta,
        camera.offset().x
    );

    // Vertical: player at y=100, dead_zone=[54, 216] for offset.y=0
    // 100 is inside dead zone → no vertical movement
    assert!(
        approx_eq(camera.offset().y, 0.0),
        "camera offset.y should be 0 (player in dead zone); got {}",
        camera.offset().y
    );
}

// ============================================================================
// INTG/level — Test T18 (real_test)
// ============================================================================

/// T18 | INTG/level | IAPI-008 + Interface Contract update() | Traces To: §Design Alignment seq msg#2
///
/// # real_test (feature #4)
///
/// Integration test: Camera correctly consumes Level::bounds() through IAPI-008.
/// Uses a real Level instance and verifies camera clamping respects
/// the level's min_x and max_x boundaries. No mocking of Level.
#[test]
fn test_camera_respects_level_bounds_via_iapi_008() {
    let config = default_camera_config();
    let mut camera = Camera::new(config);
    let level = Level::new();
    let bounds = level.bounds();

    // Verify real level bounds
    assert!(
        approx_eq(bounds.min_x, 0.0),
        "level.min_x must be 0.0"
    );
    assert!(
        approx_eq(bounds.max_x, 2000.0),
        "level.max_x must be 2000.0"
    );

    // Move player far right, verify camera clamps at level's max_x boundary
    for _frame in 0..300 {
        camera.update(Vec2 { x: 2500.0, y: 100.0 }, bounds, DT);
    }

    let max_offset = bounds.max_x - VIEWPORT_W; // = 1520.0
    assert!(
        camera.offset().x <= max_offset + EPSILON,
        "camera must clamp at level.max_x - viewport_w ({}); got {}",
        max_offset,
        camera.offset().x
    );

    // Move player far left, verify camera clamps at level's min_x boundary
    for _frame in 0..300 {
        camera.update(Vec2 { x: -100.0, y: 100.0 }, bounds, DT);
    }

    assert!(
        camera.offset().x >= bounds.min_x - EPSILON,
        "camera must clamp at level.min_x ({}); got {}",
        bounds.min_x,
        camera.offset().x
    );
}

// ============================================================================
// INTG/parallax — Test T19 (real_test)
// ============================================================================

/// T19 | INTG/parallax | IAPI-010 + Interface Contract offset() | Traces To: §Design Alignment seq msg#4
///
/// # real_test (feature #4)
///
/// Integration test: ParallaxLayer correctly consumes Camera::offset() through IAPI-010.
/// Verifies scroll_offset.x = camera_offset.x * speed and scroll_offset.y = 0.0.
/// No mocking of Camera or ParallaxLayer.
#[test]
fn test_parallax_consumes_camera_offset_via_iapi_010() {
    let config = default_camera_config();
    let mut camera = Camera::new(config);
    let bounds = default_level_bounds();

    // Position camera at a known offset by converging to a player position
    for _frame in 0..300 {
        camera.update(Vec2 { x: 600.0, y: 100.0 }, bounds, DT);
    }

    let cam_offset = camera.offset();
    // Verify camera offset is a valid Vec2
    assert!(
        cam_offset.x.is_finite() && cam_offset.y.is_finite(),
        "camera offset must be finite; got ({}, {})",
        cam_offset.x,
        cam_offset.y
    );

    // Test with three different parallax speeds (matching Level's 3-layer setup)
    let speeds = [0.1, 0.3, 0.6];
    for speed in speeds {
        let mut layer = ParallaxLayer::new(speed);
        layer.update_scroll(cam_offset);

        let expected_scroll_x = cam_offset.x * speed;
        assert!(
            approx_eq(layer.scroll_offset.x, expected_scroll_x),
            "parallax speed={}: scroll_offset.x should be {} * {} = {}; got {}",
            speed,
            cam_offset.x,
            speed,
            expected_scroll_x,
            layer.scroll_offset.x
        );
        assert!(
            approx_eq(layer.scroll_offset.y, 0.0),
            "parallax speed={}: scroll_offset.y must be 0.0; got {}",
            speed,
            layer.scroll_offset.y
        );
    }

    // Verify camera offset is NOT mutated by being read (pure getter)
    let offset_after = camera.offset();
    assert!(
        approx_eq(offset_after.x, cam_offset.x)
            && approx_eq(offset_after.y, cam_offset.y),
        "Camera::offset() must be pure getter (no side effects)"
    );
}

// ============================================================================
// Additional edge: horizontal convergence from non-zero starting offset
// ============================================================================

/// T20 | FUNC/happy | FR-013 AC-1 | Traces To: §Design Alignment seq msg#3
///
/// Verifies convergence works correctly regardless of starting offset direction.
/// Camera starts at offset.x = 500, player at x = 200 (camera needs to move left).
/// target_x = 200 - 180 = 20; delta = (20 - 500) * 0.08 = -38.4
#[test]
fn test_camera_horizontal_convergence_backward() {
    let config = default_camera_config();
    let mut camera = Camera::new(config);
    let bounds = default_level_bounds();

    // First move camera far right
    for _frame in 0..300 {
        camera.update(Vec2 { x: 1000.0, y: 100.0 }, bounds, DT);
    }

    let old_offset_x = camera.offset().x;
    // Now move player to the left — camera must converge leftward
    let player_pos = Vec2 { x: 200.0, y: 100.0 };
    camera.update(player_pos, bounds, DT);

    let target_x = player_pos.x - VIEWPORT_W * PLAYER_TARGET_X_PCT;
    // If old_offset_x > target_x, offset.x must decrease
    if old_offset_x > target_x {
        assert!(
            camera.offset().x < old_offset_x,
            "camera must move left when target is left of current; old={}, new={}",
            old_offset_x,
            camera.offset().x
        );
    }

    // delta should be 8% of remaining distance
    let expected_delta = (target_x - old_offset_x) * H_CONVERGENCE;
    assert!(
        approx_eq(camera.offset().x, old_offset_x + expected_delta),
        "convergence delta should be {}; expected={}, got={}",
        expected_delta,
        old_offset_x + expected_delta,
        camera.offset().x
    );
}

// ============================================================================
// Additional edge: vertical dead-zone exit below
// ============================================================================

/// T21 | FUNC/happy | FR-013 AC-5a | Traces To: §Implementation Summary flow branch#1 (dead-zone exit → yes)
///
/// Verifies vertical tracking when player exits dead zone below.
/// dead_bottom = 0 + 270*0.8 = 216; player at y=250 triggers 5%/frame convergence.
#[test]
fn test_camera_vertical_tracking_below_dead_zone() {
    let config = default_camera_config();
    let mut camera = Camera::new(config);
    let bounds = default_level_bounds();
    // Player below dead zone: y=250, dead_bottom = 216
    let player_pos = Vec2 { x: 200.0, y: 250.0 };

    let old_offset_y = camera.offset().y;
    camera.update(player_pos, bounds, DT);

    // target_y = 250 - 135 = 115
    // delta_y = (115 - 0) * 0.05 = 5.75
    let expected_target_y = player_pos.y - VIEWPORT_H / 2.0;
    let expected_delta = (expected_target_y - old_offset_y) * V_CONVERGENCE;
    assert!(
        approx_eq(camera.offset().y, old_offset_y + expected_delta),
        "offset.y should converge 5% toward player center when below dead zone; expected {}, got {}",
        old_offset_y + expected_delta,
        camera.offset().y
    );
}
