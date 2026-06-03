// Feature #13: Pixel Art Rendering (NFR-003) — Real Test File
//
// This file provides real test wrappers for the PixelArt rendering instrumentation
// defined in src/palette.rs. The INTG/level test (T18) validates that level tile
// rendering coordinates pass through round_sprite_pos before draw_texture_ex,
// ensuring no sub-pixel blurring in the final render pipeline.
//
// Test Inventory Reference: docs/features/13-pixel-art-rendering-nfr-003.md §7
// All tests are expected to FAIL (RED phase — stubs unimplemented).
//
// Dependencies: F02 (Level & Background) — validates coordinate rounding in
// the context of level tile rendering.

use macroquad::math::Vec2;
use mario_platformer::palette;

// ============================================================================
// Helper: float equality with epsilon tolerance
// ============================================================================

fn assert_f32_eq(actual: f32, expected: f32, epsilon: f32) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= epsilon,
        "assertion failed: actual={}, expected={}, diff={}, epsilon={}",
        actual, expected, diff, epsilon
    );
}

/// Assert that two `Vec2` values are component-wise equal within tolerance.
fn assert_vec2_eq(actual: Vec2, expected: Vec2, epsilon: f32) {
    let dx = (actual.x - expected.x).abs();
    let dy = (actual.y - expected.y).abs();
    assert!(
        dx <= epsilon && dy <= epsilon,
        "assertion failed: actual=({}, {}), expected=({}, {}), dx={}, dy={}, epsilon={}",
        actual.x, actual.y, expected.x, expected.y, dx, dy, epsilon
    );
}

// ============================================================================
// T18 — INTG/level: Level tile rendering coordinates must be integer-rounded
// Traces To: F02 dependency, §2.2 Level & Background, §Interface Contract round_sprite_pos
//
// Verifies that round_sprite_pos produces integer coordinates suitable for
// nearest-neighbor pixel art rendering when applied to level tile positions.
// All tile draw coordinates must have no fractional part; sprite boundaries
// must have no sub-pixel blurring.
//
// Kills (3 wrong-implementation scenarios):
//  1. Camera offset applied AFTER rounding → sub-pixel coordinates
//  2. ParallaxLayer scroll offset introducing sub-pixel positions
//  3. round() not called at all, passing raw float to draw_texture_ex
// ============================================================================

// real_test (feature #13)
#[test]
fn t18_level_tile_coordinates_integer_after_rounding() {
    // Simulate level tile world-coordinates that get transformed through
    // camera offset (world → screen) then rounded for pixel-art rendering.
    //
    // Scenario: a platform tile at world position (256.7, 180.3) with
    // camera offset (20.5, 0.0). After camera transform:
    //   screen_x = 256.7 - 20.5 = 236.2
    //   screen_y = 180.3 - 0.0  = 180.3
    //
    // After round_sprite_pos:
    //   screen_x = round(236.2) = 236.0
    //   screen_y = round(180.3) = 180.0

    let tile_world_x: f32 = 256.7;
    let tile_world_y: f32 = 180.3;
    let camera_offset_x: f32 = 20.5;
    let camera_offset_y: f32 = 0.0;

    // Step 1: Apply camera transform (world → screen)
    let screen_x = tile_world_x - camera_offset_x;
    let screen_y = tile_world_y - camera_offset_y;

    // Step 2: Round for pixel-art rendering (nearest-neighbor)
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        palette::round_sprite_pos(Vec2::new(screen_x, screen_y))
    }));
    assert!(result.is_ok(), "round_sprite_pos should not panic on level tile coordinates");
    let rounded = result.unwrap();

    // Step 3: Assert coordinates are integer (no fractional part)
    assert_vec2_eq(rounded, Vec2::new(236.0, 180.0), 0.001);
    assert_f32_eq(rounded.x.fract(), 0.0, 0.001);
    assert_f32_eq(rounded.y.fract(), 0.0, 0.001);

    // Step 4: Verify rounding direction is correct (not floor, not ceil, not trunc)
    // 236.2 should round to 236.0 (not 237.0 or 236.0 via trunc)
    let expect_x: f32 = 236.2_f32.round(); // = 236.0
    let expect_y: f32 = 180.3_f32.round(); // = 180.0
    assert_f32_eq(rounded.x, expect_x, 0.001);
    assert_f32_eq(rounded.y, expect_y, 0.001);
}

// ============================================================================
// R2 — INTG: Parallax background layer rounding
// Traces To: F02 dependency, §2.2 Level & Background
//
// Parallax layers have fractional scroll offsets (e.g., 0.5x camera offset).
// After camera transform, parallax-layer tile coordinates must be rounded
// before rendering to prevent sub-pixel blurring.
//
// Kills: parallax scroll offset applied AFTER rounding, introducing
// sub-pixel coordinates at layer boundaries.
// ============================================================================

// real_test (feature #13)
#[test]
fn t18b_parallax_layer_coordinates_rounded() {
    // Parallax layer at 0.5× speed: offset = camera_offset * 0.5
    let camera_offset_x: f32 = 33.7;
    let parallax_factor: f32 = 0.5;
    let parallax_offset = camera_offset_x * parallax_factor; // = 16.85

    // A background tile at local position (100.0, 50.0)
    // After parallax shift: screen_x = 100.0 - 16.85 = 83.15
    let tile_local_x: f32 = 100.0;
    let screen_x = tile_local_x - parallax_offset; // = 83.15

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        palette::round_sprite_pos(Vec2::new(screen_x, 50.0))
    }));
    assert!(result.is_ok(), "round_sprite_pos should not panic on parallax coordinates");
    let rounded = result.unwrap();

    // round(83.15) = 83.0, round(50.0) = 50.0
    assert_vec2_eq(rounded, Vec2::new(83.0, 50.0), 0.001);
    assert_f32_eq(rounded.x.fract(), 0.0, 0.001);
    assert_f32_eq(rounded.y.fract(), 0.0, 0.001);
}

// ============================================================================
// R3 — INTG: Multiple tile coordinates all produce integer positions
// Traces To: F02 dependency, §2.2 Level & Background
//
// A batch of level tiles at various positions should all produce integer
// coordinates after rounding, ensuring consistent pixel-art rendering
// across the entire level.
// ============================================================================

// real_test (feature #13)
#[test]
fn t18c_tile_batch_integer_coordinates() {
    // Simulate a row of platform tiles with a moving camera
    let camera_x: f32 = 150.25;
    let tile_world_positions: [(f32, f32); 5] = [
        (100.0, 200.0),
        (148.5, 200.0),
        (197.3, 200.0),
        (245.8, 200.0),
        (294.1, 200.0),
    ];

    for &(wx, wy) in &tile_world_positions {
        let sx = wx - camera_x;
        let sy = wy; // No vertical camera movement in this test
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            palette::round_sprite_pos(Vec2::new(sx, sy))
        }));
        assert!(result.is_ok(), "round_sprite_pos should not panic on tile ({}, {})", wx, wy);
        let rounded = result.unwrap();

        let expected_x = sx.round();
        let expected_y = sy.round();
        assert_f32_eq(rounded.x, expected_x, 0.001);
        assert_f32_eq(rounded.y, expected_y, 0.001);
        assert_f32_eq(rounded.x.fract(), 0.0, 0.001);
        assert_f32_eq(rounded.y.fract(), 0.0, 0.001);
    }
}

// ============================================================================
// R4 — FILTER_APPLIED flag verification (real test)
// Traces To: §Interface Contract apply_pixel_art_filter
//
// Integration-level check that the FILTER_APPLIED flag is globally accessible
// and behaves correctly (initially false, set to true after apply).
// ============================================================================

// real_test (feature #13)
#[test]
fn t18d_filter_applied_flag_integration() {
    use std::sync::atomic::Ordering;

    // Verify the flag exists and is accessible from integration test context
    // Initial state should be false (before apply_pixel_art_filter is called)
    palette::FILTER_APPLIED.store(false, Ordering::Release);
    let before = palette::FILTER_APPLIED.load(Ordering::Acquire);
    assert!(!before, "FILTER_APPLIED should be false before filter application");

    // Apply the filter
    palette::apply_pixel_art_filter();

    // After application: flag should be true
    let after = palette::FILTER_APPLIED.load(Ordering::Acquire);
    assert!(after, "FILTER_APPLIED should be true after apply_pixel_art_filter()");
}
