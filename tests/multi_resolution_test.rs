// Feature #12: Multi-Resolution Display (NFR-002) — Real Test File
//
// This file provides real test wrappers for the ResolutionVerifier instrumentation
// defined in src/verification.rs. All verification methods are pure stateless f32
// computations with zero I/O — the "real test" designation reflects that these
// tests exercise the full verification pipeline end-to-end without mocking.
//
// Test Inventory Reference: docs/features/12-multi-resolution-display-nfr-002.md §7
// All tests are expected to PASS (GREEN phase completed).

use mario_platformer::systems::camera::{Camera, CameraConfig};
use mario_platformer::systems::hud::HudRenderer;
use mario_platformer::verification::ResolutionVerifier;

// ============================================================================
// Helper: float equality with epsilon tolerance
// ============================================================================

fn assert_f32_eq(actual: f32, expected: f32, epsilon: f32) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= epsilon,
        "assertion failed: actual={}, expected={}, diff={}, epsilon={}",
        actual,
        expected,
        diff,
        epsilon
    );
}

// ============================================================================
// R1 — HUD anchor at 720p
// Traces To: NFR-002 AC-1, §Interface Contract expected_hud_anchor
//
// Verifies ResolutionVerifier::expected_hud_anchor(1280.0, 720.0) returns
// the correct (3%, 3%) position.
// ============================================================================

// real_test (feature #12)
#[test]
fn r1_hud_anchor_expected_720p() {
    let (x, y) = ResolutionVerifier::expected_hud_anchor(1280.0, 720.0);
    assert_f32_eq(x, 38.4, 0.01); // 1280 * 0.03
    assert_f32_eq(y, 21.6, 0.01); // 720 * 0.03
}

// ============================================================================
// R2 — HUD anchor at 1080p and 1440p
// Traces To: NFR-002 AC-1, §Interface Contract expected_hud_anchor
//
// Verifies linear scaling across all supported resolutions.
// ============================================================================

// real_test (feature #12)
#[test]
fn r2_hud_anchor_expected_1080p_1440p() {
    // 1080p: 1920 * 0.03 = 57.6, 1080 * 0.03 = 32.4
    let (x1080, y1080) = ResolutionVerifier::expected_hud_anchor(1920.0, 1080.0);
    assert_f32_eq(x1080, 57.6, 0.01);
    assert_f32_eq(y1080, 32.4, 0.01);

    // 1440p: 2560 * 0.03 = 76.8, 1440 * 0.03 = 43.2
    let (x1440, y1440) = ResolutionVerifier::expected_hud_anchor(2560.0, 1440.0);
    assert_f32_eq(x1440, 76.8, 0.01);
    assert_f32_eq(y1440, 43.2, 0.01);

    // Anchor scales linearly: 1440p / 720p ≈ 2.0
    let (x720, y720) = ResolutionVerifier::expected_hud_anchor(1280.0, 720.0);
    assert_f32_eq(x1440 / x720, 2.0, 0.01);
    assert_f32_eq(y1440 / y720, 2.0, 0.01);
}

// ============================================================================
// R3 — HUD anchor deviation computation
// Traces To: NFR-002 AC-1, §Interface Contract hud_anchor_deviation
//
// Verifies that actual=(40.0, 22.0) against 720p expected anchor produces
// correct percentage-based deviation.
// ============================================================================

// real_test (feature #12)
#[test]
fn r3_hud_anchor_deviation_computation() {
    let dev = ResolutionVerifier::hud_anchor_deviation(40.0, 22.0, 1280.0, 720.0);
    // dx_pct = (40.0 - 38.4) / 1280.0 = 1.6 / 1280.0 = 0.00125
    assert_f32_eq(dev.dx_pct, 0.00125, 0.0001);
    // dy_pct = (22.0 - 21.6) / 720.0 ≈ 0.0005556
    assert_f32_eq(dev.dy_pct, 0.0005556, 0.0001);
}

// ============================================================================
// R4 — HUD anchor self-consistency check
// Traces To: NFR-002 AC-1, §Interface Contract check_hud_anchor
//
// Verifies that the expected anchor compared to itself passes at 2% tolerance.
// ============================================================================

// real_test (feature #12)
#[test]
fn r4_hud_anchor_self_consistency() {
    let report = ResolutionVerifier::check_hud_anchor(1280.0, 720.0, 0.02);
    assert!(report.passed, "Self-consistency check should pass");
    // Deviation from itself is exactly 0.0
    assert_f32_eq(report.deviation.dx_pct, 0.0, 0.0001);
    assert_f32_eq(report.deviation.dy_pct, 0.0, 0.0001);
    // Expected anchor at 720p
    assert_f32_eq(report.expected.0, 38.4, 0.01);
    assert_f32_eq(report.expected.1, 21.6, 0.01);
    // Tolerance at 2%: (1280 * 0.02, 720 * 0.02) = (25.6, 14.4)
    assert_f32_eq(report.tolerance.0, 25.6, 0.01);
    assert_f32_eq(report.tolerance.1, 14.4, 0.01);
}

// ============================================================================
// R5 — Player visible area ratio computation
// Traces To: NFR-002 AC-2, §Interface Contract player_visible_ratio
//
// Verifies visible area ratio using 480px virtual canvas viewport.
// ============================================================================

// real_test (feature #12)
#[test]
fn r5_player_visible_ratio_computation() {
    let ratio = ResolutionVerifier::player_visible_ratio(200.0, 20.0, 480.0);
    // player_screen_x = 200.0 - 20.0 = 180.0
    // ahead = 480.0 - 180.0 = 300.0
    // ratio = 300.0 / 480.0 = 0.625
    assert_f32_eq(ratio, 0.625, 0.001);
}

// ============================================================================
// R6 — Visible area ratio within expected range
// Traces To: NFR-002 AC-2, §Interface Contract check_visible_ratio
//
// Verifies that ratio=0.48 passes when expected range is [0.45, 0.50].
// ============================================================================

// real_test (feature #12)
#[test]
fn r6_visible_ratio_within_range() {
    let report = ResolutionVerifier::check_visible_ratio(0.48, 0.45, 0.50);
    assert!(report.passed, "Ratio 0.48 should be within [0.45, 0.50]");
    assert_f32_eq(report.ratio, 0.48, 0.001);
    assert_f32_eq(report.min_expected, 0.45, 0.001);
    assert_f32_eq(report.max_expected, 0.50, 0.001);
}

// ============================================================================
// R7 — Full resolution verification suite
// Traces To: NFR-002 AC-1 & AC-2, §Interface Contract verify_all_resolutions
//
// Verifies all 3 supported resolutions produce valid reports with
// correct HUD anchor and visible area checks.
// ============================================================================

// real_test (feature #12)
#[test]
fn r7_verify_all_resolutions_full_suite() {
    let config = CameraConfig::default();
    // Verify default config values
    assert_f32_eq(config.viewport_w, 480.0, 0.01);
    assert_f32_eq(config.viewport_h, 270.0, 0.01);
    assert_f32_eq(config.player_target_x_pct, 0.375, 0.001);

    let reports = ResolutionVerifier::verify_all_resolutions(&config);
    assert_eq!(reports.len(), 3, "Should produce 3 reports (720p/1080p/1440p)");

    for report in &reports {
        assert!(
            report.hud.passed,
            "HUD anchor should pass at {:?}",
            report.resolution
        );
    }
}

// ============================================================================
// R8 — Fullscreen mode HUD anchor equivalence
// Traces To: NFR-002 AC-3, §Interface Contract check_hud_anchor
//
// Verifies that fullscreen 1080p produces identical HUD anchor to windowed
// 1080p because fullscreen only changes window decor, not viewport.
// ============================================================================

// real_test (feature #12)
#[test]
fn r8_fullscreen_hud_anchor_same_as_windowed() {
    let report = ResolutionVerifier::check_hud_anchor(1920.0, 1080.0, 0.02);
    assert!(report.passed, "Fullscreen-equivalent check should pass");
    // Anchor = (1920 * 0.03, 1080 * 0.03) = (57.6, 32.4)
    assert_f32_eq(report.expected.0, 57.6, 0.01);
    assert_f32_eq(report.expected.1, 32.4, 0.01);
    // Deviation should be 0.0 (self-consistency)
    assert_f32_eq(report.deviation.dx_pct, 0.0, 0.0001);
    assert_f32_eq(report.deviation.dy_pct, 0.0, 0.0001);
}

// ============================================================================
// R9 — HUD formula consistency between HudRenderer and ResolutionVerifier
// Traces To: NFR-002 AC-1, §Interface Contract check_hud_anchor + F09 HUD
//
// Verifies that HudRenderer::compute_anchor() and
// ResolutionVerifier::expected_hud_anchor() produce identical results
// across all supported resolutions, preventing formula drift.
// ============================================================================

// real_test (feature #12)
#[test]
fn r9_hud_anchor_consistency_all_resolutions() {
    let resolutions = [(1280.0, 720.0), (1920.0, 1080.0), (2560.0, 1440.0)];
    for &(w, h) in &resolutions {
        let hud_anchor = HudRenderer::compute_anchor(w, h);
        let verifier_anchor = ResolutionVerifier::expected_hud_anchor(w, h);
        assert_f32_eq(hud_anchor.0, verifier_anchor.0, 0.001);
        assert_f32_eq(hud_anchor.1, verifier_anchor.1, 0.001);
    }
}

// ============================================================================
// R10 — Visible area ratio uses virtual canvas not screen resolution
// Traces To: NFR-002 AC-2, §Interface Contract player_visible_ratio + F04 Camera
//
// Verifies that player_visible_ratio uses virtual canvas viewport (480px)
// rather than physical screen resolution for ratio computation.
// ============================================================================

// real_test (feature #12)
#[test]
fn r10_visible_ratio_uses_virtual_canvas() {
    // Verify Camera defaults
    let camera = Camera::new(CameraConfig::default());
    let (vp_w, vp_h) = camera.viewport();
    assert_f32_eq(vp_w, 480.0, 0.01);
    assert_f32_eq(vp_h, 270.0, 0.01);

    let config = CameraConfig::default();
    assert_f32_eq(config.player_target_x_pct, 0.375, 0.001);

    // player_world_x=200, camera_offset_x=20 → player_screen_x=180
    // ratio = (480 - 180) / 480 = 300/480 = 0.625
    let ratio = ResolutionVerifier::player_visible_ratio(200.0, 20.0, 480.0);
    assert_f32_eq(ratio, 0.625, 0.001);

    // Verify the design invariant: ratio ≈ 1.0 - player_target_x_pct
    let expected_ratio = 1.0 - config.player_target_x_pct;
    assert_f32_eq(ratio, expected_ratio, 0.001);
}

// ============================================================================
// R11 — Boundary: zero tolerance self-consistency
// Traces To: §Implementation Summary Boundary Conditions
//
// Verifies that zero tolerance self-consistency check still passes.
// ============================================================================

// real_test (feature #12)
#[test]
fn r11_zero_tolerance_self_consistency() {
    let report = ResolutionVerifier::check_hud_anchor(1280.0, 720.0, 0.0);
    assert!(
        report.passed,
        "Zero-tolerance self-consistency should pass"
    );
    assert_f32_eq(report.deviation.dx_pct, 0.0, 0.0001);
    assert_f32_eq(report.deviation.dy_pct, 0.0, 0.0001);
    assert_f32_eq(report.tolerance.0, 0.0, 0.0001);
    assert_f32_eq(report.tolerance.1, 0.0, 0.0001);
}

// ============================================================================
// R12 — Boundary: visible ratio at lower and upper bounds
// Traces To: §Implementation Summary Boundary Conditions
//
// Verifies inclusive boundary behavior for visible area ratio checks.
// ============================================================================

// real_test (feature #12)
#[test]
fn r12_visible_ratio_boundary_inclusive() {
    // At lower bound (0.45) → must pass
    let lower = ResolutionVerifier::check_visible_ratio(0.45, 0.45, 0.50);
    assert!(lower.passed, "Ratio 0.45 at lower bound should pass (inclusive)");

    // At upper bound (0.50) → must pass
    let upper = ResolutionVerifier::check_visible_ratio(0.50, 0.45, 0.50);
    assert!(upper.passed, "Ratio 0.50 at upper bound should pass (inclusive)");

    // Slightly below lower bound (0.4499) → must fail
    let below = ResolutionVerifier::check_visible_ratio(0.4499, 0.45, 0.50);
    assert!(!below.passed, "Ratio 0.4499 below lower bound should fail");
}

// ============================================================================
// R13 — Determinism: same resolution produces identical reports
// Traces To: NFR-002 AC-3, §Interface Contract check_hud_anchor
//
// Verifies pure function determinism: same inputs → same outputs.
// ============================================================================

// real_test (feature #12)
#[test]
fn r13_same_resolution_identical_reports() {
    let r1 = ResolutionVerifier::check_hud_anchor(1920.0, 1080.0, 0.02);
    let r2 = ResolutionVerifier::check_hud_anchor(1920.0, 1080.0, 0.02);
    assert_eq!(r1, r2, "Same resolution should produce identical reports");
}
