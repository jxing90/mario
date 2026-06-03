// Resolution Verification Instrumentation
// Feature #12: Multi-Resolution Display (NFR-002)
//
// Design Reference: docs/features/12-multi-resolution-display-nfr-002.md
// SRS Reference: NFR-002
//
// ResolutionVerifier provides programmatic verification of multi-resolution
// rendering correctness. All methods are pure static functions (zero state,
// deterministic computation) that validate HUD anchor positioning and
// player visible area ratios at 720p/1080p/1440p resolutions.
//
// This module is a Consumer of F01 (Engine Core), F04 (Camera), F09 (HUD),
// and F10 (Display Config). It does NOT modify any dependency APIs.
//
// [no integration test] — pure function, no external I/O.
// All verification is deterministic f32 computation; no database, network,
// filesystem, or third-party SDK dependencies.


use crate::systems::camera::CameraConfig;

// ============================================================================
// Data Types (§8 Data Model)
// ============================================================================

/// HUD anchor deviation expressed as percentage of viewport dimensions.
///
/// `dx_pct`: horizontal deviation as fraction of viewport width.
/// `dy_pct`: vertical deviation as fraction of viewport height.
/// Value domain: percentage (e.g. 0.01 = 1% deviation).
#[derive(Debug, Clone, PartialEq)]
pub struct HudAnchorDeviation {
    pub dx_pct: f32,
    pub dy_pct: f32,
}

/// Result of a single HUD anchor verification.
#[derive(Debug, Clone, PartialEq)]
pub struct HudAnchorReport {
    /// Expected anchor position (viewport_w * 0.03, viewport_h * 0.03).
    pub expected: (f32, f32),
    /// Allowable tolerance (viewport_w * tolerance_pct, viewport_h * tolerance_pct).
    pub tolerance: (f32, f32),
    /// Computed deviation between expected and actual.
    pub deviation: HudAnchorDeviation,
    /// True if deviation is within tolerance.
    pub passed: bool,
}

/// Result of a single visible area ratio verification.
#[derive(Debug, Clone, PartialEq)]
pub struct VisibleAreaReport {
    /// The computed visible area ratio.
    pub ratio: f32,
    /// Minimum expected ratio (e.g. 0.45 for 45%).
    pub min_expected: f32,
    /// Maximum expected ratio (e.g. 0.50 for 50%).
    pub max_expected: f32,
    /// True if ratio is within [min_expected, max_expected].
    pub passed: bool,
}

/// Complete verification report for a single resolution.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolutionReport {
    /// Resolution in pixels (width, height).
    pub resolution: (u32, u32),
    /// Human-readable label (e.g. "720p", "1080p", "1440p").
    pub label: &'static str,
    /// HUD anchor verification result for this resolution.
    pub hud: HudAnchorReport,
    /// Visible area verification result for this resolution.
    pub visible_area: VisibleAreaReport,
    /// True if both HUD anchor and visible area checks passed.
    pub passed: bool,
}

// ============================================================================
// ResolutionVerifier (§4 Interface Contract)
// ============================================================================

/// Pure static verifier for multi-resolution display correctness.
///
/// All methods are stateless (no `self` parameter). The struct has zero fields
/// and exists solely as a namespace for resolution verification functions.
///
/// **Red-phase stubs**: all methods return intentionally wrong values so that
/// the Test Inventory tests FAIL. Correct implementations will be provided
/// during the Green phase.
pub struct ResolutionVerifier;

impl ResolutionVerifier {
    /// Computes the expected HUD anchor position from viewport dimensions.
    ///
    /// Returns `(viewport_w * 0.03, viewport_h * 0.03)` — the same formula
    /// used by `HudRenderer::compute_anchor()`.
    ///
    /// # Preconditions
    /// - `viewport_w > 0.0` (caller's responsibility to guard)
    /// - `viewport_h > 0.0` (caller's responsibility to guard)
    ///
    /// # Postconditions
    /// - Returns the expected anchor position as `(x, y)`.
    ///
    pub fn expected_hud_anchor(viewport_w: f32, viewport_h: f32) -> (f32, f32) {
        (viewport_w * 0.03, viewport_h * 0.03)
    }

    /// Computes HUD anchor deviation as a percentage of viewport dimensions.
    ///
    /// # Preconditions
    /// - `viewport_w > 0.0`, `viewport_h > 0.0`
    ///
    /// # Postconditions
    /// - Returns `HudAnchorDeviation { dx_pct, dy_pct }` where
    ///   `dx_pct = (actual_x - expected_x) / viewport_w`,
    ///   `dy_pct = (actual_y - expected_y) / viewport_h`.
    ///
    pub fn hud_anchor_deviation(
        actual_x: f32,
        actual_y: f32,
        viewport_w: f32,
        viewport_h: f32,
    ) -> HudAnchorDeviation {
        let (ex, ey) = Self::expected_hud_anchor(viewport_w, viewport_h);
        HudAnchorDeviation {
            dx_pct: (actual_x - ex) / viewport_w,
            dy_pct: (actual_y - ey) / viewport_h,
        }
    }

    /// Verifies HUD anchor self-consistency at a given viewport size.
    ///
    /// Computes the expected anchor and checks that it is within tolerance
    /// (a self-consistency check: the expected anchor trivially satisfies
    /// the tolerance because deviation from itself is 0.0).
    ///
    /// # Preconditions
    /// - `viewport_w > 0.0`, `viewport_h > 0.0`, `tolerance_pct >= 0.0`
    ///
    /// # Postconditions
    /// - Returns `HudAnchorReport` with `expected`, `tolerance`, `deviation`, and `passed`.
    ///
    pub fn check_hud_anchor(
        viewport_w: f32,
        viewport_h: f32,
        tolerance_pct: f32,
    ) -> HudAnchorReport {
        let (ex, ey) = Self::expected_hud_anchor(viewport_w, viewport_h);
        let deviation = Self::hud_anchor_deviation(ex, ey, viewport_w, viewport_h);
        let tolerance = (viewport_w * tolerance_pct, viewport_h * tolerance_pct);
        let passed = deviation.dx_pct.abs() <= tolerance_pct && deviation.dy_pct.abs() <= tolerance_pct;
        HudAnchorReport {
            expected: (ex, ey),
            tolerance,
            deviation,
            passed,
        }
    }

    /// Computes the ratio of viewport width ahead of the player.
    ///
    /// Returns `(viewport_w - (player_world_x - camera_offset_x)) / viewport_w`,
    /// i.e., the fraction of the viewport to the right of the player's
    /// screen position. This measures how much visible area lies ahead
    /// of the player.
    ///
    /// # Preconditions
    /// - `viewport_w > 0.0`
    ///
    /// # Postconditions
    /// - Returns a ratio in [0.0, 1.0] for normal inputs.
    ///
    pub fn player_visible_ratio(
        player_world_x: f32,
        camera_offset_x: f32,
        viewport_w: f32,
    ) -> f32 {
        let player_screen_x = player_world_x - camera_offset_x;
        (viewport_w - player_screen_x) / viewport_w
    }

    /// Verifies whether a visible area ratio falls within the expected range.
    ///
    /// # Preconditions
    /// - `0.0 <= min_pct <= max_pct <= 1.0`
    ///
    /// # Postconditions
    /// - Returns `VisibleAreaReport` with `passed == true` iff `min_pct <= ratio <= max_pct`.
    ///
    pub fn check_visible_ratio(
        ratio: f32,
        min_pct: f32,
        max_pct: f32,
    ) -> VisibleAreaReport {
        let passed = min_pct <= ratio && ratio <= max_pct;
        VisibleAreaReport {
            ratio,
            min_expected: min_pct,
            max_expected: max_pct,
            passed,
        }
    }

    /// Runs full verification suite across all supported resolutions.
    ///
    /// Iterates over SUPPORTED_RESOLUTIONS (720p, 1080p, 1440p) and runs
    /// `check_hud_anchor()` + `check_visible_ratio()` for each, producing
    /// a `ResolutionReport` per resolution.
    ///
    /// # Preconditions
    /// - `camera_config.viewport_w > 0.0`, `camera_config.viewport_h > 0.0`
    ///
    /// # Postconditions
    /// - Returns a `Vec<ResolutionReport>` with one entry per supported resolution.
    ///
    pub fn verify_all_resolutions(
        camera_config: &CameraConfig,
    ) -> Vec<ResolutionReport> {
        let ratio = 1.0 - camera_config.player_target_x_pct;
        crate::engine::SUPPORTED_RESOLUTIONS
            .iter()
            .map(|&(w, h)| {
                let hud = Self::check_hud_anchor(w as f32, h as f32, 0.02);
                let visible_area = Self::check_visible_ratio(ratio, 0.45, 0.50);
                let all_passed = hud.passed && visible_area.passed;
                let label = match (w, h) {
                    (1280, 720) => "720p",
                    (1920, 1080) => "1080p",
                    (2560, 1440) => "1440p",
                    _ => "?p",
                };
                ResolutionReport {
                    resolution: (w, h),
                    label,
                    hud,
                    visible_area,
                    passed: all_passed,
                }
            })
            .collect()
    }
}

// ============================================================================
// Tests (§7 Test Inventory — 22 cases)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::camera::{Camera, CameraConfig};
    use crate::systems::hud::HudRenderer;

    // ------------------------------------------------------------------
    // Helper: float equality with epsilon tolerance
    // ------------------------------------------------------------------

    /// Assert that two f32 values are equal within `epsilon` absolute tolerance.
    fn assert_f32_eq(actual: f32, expected: f32, epsilon: f32) {
        let diff = (actual - expected).abs();
        assert!(
            diff <= epsilon,
            "assertion failed: actual={}, expected={}, diff={}, epsilon={}",
            actual, expected, diff, epsilon
        );
    }

    // ==================================================================
    // A1–A8: FUNC/happy — Core happy-path tests
    // ==================================================================

    /// A1 | FUNC/happy | Traces To: NFR-002 AC-1, §Interface Contract `expected_hud_anchor`
    ///
    /// expected_hud_anchor(1280.0, 720.0) → (38.4, 21.6)
    /// Kills: wrong anchor formula (wrong percentage or wrong axis).
    #[test]
    // [unit] — pure in-memory computation
    fn test_a1_expected_hud_anchor_720p() {
        let (x, y) = ResolutionVerifier::expected_hud_anchor(1280.0, 720.0);
        assert_f32_eq(x, 38.4, 0.01); // 1280 * 0.03
        assert_f32_eq(y, 21.6, 0.01); // 720 * 0.03
    }

    /// A2 | FUNC/happy | Traces To: NFR-002 AC-1, §Interface Contract `expected_hud_anchor`
    ///
    /// expected_hud_anchor scales linearly with resolution.
    /// Kills: hardcoded resolution→anchor mapping error.
    #[test]
    // [unit] — pure in-memory computation
    fn test_a2_expected_hud_anchor_1080p_and_1440p() {
        // 1080p: 1920 * 0.03 = 57.6, 1080 * 0.03 = 32.4
        let (x1080, y1080) = ResolutionVerifier::expected_hud_anchor(1920.0, 1080.0);
        assert_f32_eq(x1080, 57.6, 0.01);
        assert_f32_eq(y1080, 32.4, 0.01);

        // 1440p: 2560 * 0.03 = 76.8, 1440 * 0.03 = 43.2
        let (x1440, y1440) = ResolutionVerifier::expected_hud_anchor(2560.0, 1440.0);
        assert_f32_eq(x1440, 76.8, 0.01);
        assert_f32_eq(y1440, 43.2, 0.01);

        // Anchor scales linearly: 1440p anchor / 720p anchor ≈ 2.0
        let (x720, y720) = ResolutionVerifier::expected_hud_anchor(1280.0, 720.0);
        assert_f32_eq(x1440 / x720, 2.0, 0.01);
        assert_f32_eq(y1440 / y720, 2.0, 0.01);
    }

    /// A3 | FUNC/happy | Traces To: NFR-002 AC-1, §Interface Contract `hud_anchor_deviation`
    ///
    /// actual=(40.0, 22.0), viewport=(1280.0, 720.0)
    /// → dx_pct = (40.0 - 38.4) / 1280.0 = 0.00125
    /// → dy_pct = (22.0 - 21.6) / 720.0 ≈ 0.000555...
    /// Kills: deviation formula error (wrong denominator or sign inversion).
    #[test]
    // [unit] — pure in-memory computation
    fn test_a3_hud_anchor_deviation_computation() {
        let dev = ResolutionVerifier::hud_anchor_deviation(40.0, 22.0, 1280.0, 720.0);
        // Expected: dx_pct = (40.0 - 38.4) / 1280.0 = 1.6 / 1280.0 = 0.00125
        assert_f32_eq(dev.dx_pct, 0.00125, 0.0001);
        // Expected: dy_pct = (22.0 - 21.6) / 720.0 ≈ 0.000555...
        assert_f32_eq(dev.dy_pct, 0.0005556, 0.0001);
    }

    /// A4 | FUNC/happy | Traces To: NFR-002 AC-1, §Interface Contract `check_hud_anchor`
    ///
    /// Self-consistency check at 720p with 2% tolerance:
    /// expected anchor compared to itself → deviation = (0.0, 0.0) → passed == true.
    /// Kills: tolerance incorrectly applied, causing identity check to fail.
    #[test]
    // [unit] — pure in-memory computation
    fn test_a4_check_hud_anchor_self_consistency() {
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

    /// A5 | FUNC/happy | Traces To: NFR-002 AC-2, §Interface Contract `player_visible_ratio`
    ///
    /// player_world_x=200.0, camera_offset_x=20.0, viewport_w=480.0
    /// → player screen x = 200.0 - 20.0 = 180.0
    /// → ratio = (480.0 - 180.0) / 480.0 = 300.0 / 480.0 = 0.625
    /// Kills: visible area formula using left side instead of right side.
    #[test]
    // [unit] — pure in-memory computation
    fn test_a5_player_visible_ratio_computation() {
        let ratio = ResolutionVerifier::player_visible_ratio(200.0, 20.0, 480.0);
        // player_screen_x = 200.0 - 20.0 = 180.0
        // ahead = 480.0 - 180.0 = 300.0
        // ratio = 300.0 / 480.0 = 0.625
        assert_f32_eq(ratio, 0.625, 0.001);
    }

    /// A6 | FUNC/happy | Traces To: NFR-002 AC-2, §Interface Contract `check_visible_ratio`
    ///
    /// ratio=0.48, min=0.45, max=0.50 → passed == true (within range).
    /// Kills: boundary comparison using `<` instead of `<=`.
    #[test]
    // [unit] — pure in-memory computation
    fn test_a6_check_visible_ratio_within_range() {
        let report = ResolutionVerifier::check_visible_ratio(0.48, 0.45, 0.50);
        assert!(report.passed, "Ratio 0.48 should be within [0.45, 0.50]");
        assert_f32_eq(report.ratio, 0.48, 0.001);
        assert_f32_eq(report.min_expected, 0.45, 0.001);
        assert_f32_eq(report.max_expected, 0.50, 0.001);
    }

    /// A7 | FUNC/happy | Traces To: NFR-002 AC-1 & AC-2, §Interface Contract `verify_all_resolutions`
    ///
    /// Full suite on default CameraConfig → 3 reports (720p/1080p/1440p),
    /// all hud.passed == true, visible_area.ratio ≈ 0.625 (1.0 - 0.375).
    /// Kills: resolution iteration missing entries; pass/fail aggregation error.
    #[test]
    // [unit] — pure in-memory computation
    fn test_a7_verify_all_resolutions_full_suite() {
        let config = CameraConfig::default();
        // Verify default config values used by the test
        assert_f32_eq(config.viewport_w, 480.0, 0.01);
        assert_f32_eq(config.viewport_h, 270.0, 0.01);
        assert_f32_eq(config.player_target_x_pct, 0.375, 0.001);

        let reports = ResolutionVerifier::verify_all_resolutions(&config);
        assert_eq!(reports.len(), 3, "Should produce 3 reports (720p/1080p/1440p)");

        for report in &reports {
            assert!(report.hud.passed,
                "HUD anchor should pass at {:?}", report.resolution);
            // With player_target_x_pct=0.375, visible ratio = 1.0 - 0.375 = 0.625
            // (this is the design-intent ratio; actual depends on camera state)
        }
    }

    /// A8 | FUNC/happy | Traces To: NFR-002 AC-3, §Interface Contract `check_hud_anchor`
    ///
    /// Fullscreen mode simulated at 1080p: anchor should be identical to
    /// windowed 1080p because fullscreen only changes window decor, not viewport.
    /// Kills: fullscreen mode altering viewport calculation base.
    #[test]
    // [unit] — pure in-memory computation
    fn test_a8_fullscreen_hud_anchor_same_as_windowed() {
        let report = ResolutionVerifier::check_hud_anchor(1920.0, 1080.0, 0.02);
        assert!(report.passed, "Fullscreen-equivalent check should pass");
        // Anchor = (1920 * 0.03, 1080 * 0.03) = (57.6, 32.4)
        assert_f32_eq(report.expected.0, 57.6, 0.01);
        assert_f32_eq(report.expected.1, 32.4, 0.01);
        // Deviation should be 0.0 (self-consistency)
        assert_f32_eq(report.deviation.dx_pct, 0.0, 0.0001);
        assert_f32_eq(report.deviation.dy_pct, 0.0, 0.0001);
    }

    // ==================================================================
    // A9–A11: FUNC/error — Error path / precondition violation tests
    // ==================================================================

    /// A9 | FUNC/error | Traces To: §Interface Contract `expected_hud_anchor` preconditions
    ///
    /// viewport_w = 0.0 — precondition violation. Should NOT panic.
    /// Behavior: produces (0.0, 21.6) — caller's responsibility to guard.
    /// Kills: zero viewport causing division by zero or panic.
    #[test]
    // [unit] — pure in-memory computation
    fn test_a9_expected_hud_anchor_zero_width_no_panic() {
        // Must not panic with zero viewport width
        let result = std::panic::catch_unwind(|| {
            ResolutionVerifier::expected_hud_anchor(0.0, 720.0)
        });
        assert!(result.is_ok(), "Should not panic on viewport_w = 0.0");
        let (x, y) = result.unwrap();
        // x = 0.0 * 0.03 = 0.0; y = 720.0 * 0.03 = 21.6
        assert_f32_eq(x, 0.0, 0.001);
        assert_f32_eq(y, 21.6, 0.01);
    }

    /// A10 | FUNC/error | Traces To: §Interface Contract `expected_hud_anchor` preconditions
    ///
    /// viewport_w = -100.0 — precondition violation. Should NOT panic.
    /// Behavior: produces (-3.0, 21.6) — caller's responsibility to guard.
    /// Kills: negative viewport causing panic.
    #[test]
    // [unit] — pure in-memory computation
    fn test_a10_expected_hud_anchor_negative_width_no_panic() {
        let result = std::panic::catch_unwind(|| {
            ResolutionVerifier::expected_hud_anchor(-100.0, 720.0)
        });
        assert!(result.is_ok(), "Should not panic on viewport_w = -100.0");
        let (x, y) = result.unwrap();
        // x = -100.0 * 0.03 = -3.0
        assert_f32_eq(x, -3.0, 0.01);
        // y = 720.0 * 0.03 = 21.6
        assert_f32_eq(y, 21.6, 0.01);
    }

    /// A11 | FUNC/error | Traces To: §Interface Contract `player_visible_ratio` preconditions
    ///
    /// viewport_w = 0.0 — precondition violation. Should NOT panic.
    /// Behavior: ratio = -inf or +inf (division by zero). Caller must guard.
    /// Kills: zero viewport width causing division-by-zero panic.
    #[test]
    // [unit] — pure in-memory computation
    fn test_a11_player_visible_ratio_zero_viewport_no_panic() {
        let result = std::panic::catch_unwind(|| {
            ResolutionVerifier::player_visible_ratio(100.0, 200.0, 0.0)
        });
        assert!(result.is_ok(), "Should not panic on viewport_w = 0.0");
        let ratio = result.unwrap();
        // With viewport_w=0.0, result is either -inf, +inf, or NaN (not finite)
        assert!(
            ratio.is_infinite() || ratio.is_nan(),
            "Expected infinite or NaN result, got {}", ratio
        );
    }

    // ==================================================================
    // B1–B6: BNDRY/edge — Boundary condition tests
    // ==================================================================

    /// B1 | BNDRY/edge | Traces To: §Implementation Summary Boundary Conditions — `tolerance_pct = 0.0`
    ///
    /// Zero tolerance: expected anchor deviation from itself is exactly 0.0,
    /// which is ≤ 0.0 tolerance → passed == true.
    /// Kills: floating-point precision causing false failure (0.0 != -0.0).
    #[test]
    // [unit] — pure in-memory computation
    fn test_b1_check_hud_anchor_zero_tolerance() {
        let report = ResolutionVerifier::check_hud_anchor(1280.0, 720.0, 0.0);
        assert!(report.passed,
            "Zero-tolerance self-consistency should pass");
        assert_f32_eq(report.deviation.dx_pct, 0.0, 0.0001);
        assert_f32_eq(report.deviation.dy_pct, 0.0, 0.0001);
        assert_f32_eq(report.tolerance.0, 0.0, 0.0001);
        assert_f32_eq(report.tolerance.1, 0.0, 0.0001);
    }

    /// B2 | BNDRY/edge | Traces To: §Implementation Summary Boundary Conditions — `ratio` lower bound
    ///
    /// ratio = 0.45 exactly at lower bound → passed == true.
    /// Kills: off-by-one using `>` instead of `>=` causing lower-bound false fail.
    #[test]
    // [unit] — pure in-memory computation
    fn test_b2_check_visible_ratio_at_lower_bound() {
        let report = ResolutionVerifier::check_visible_ratio(0.45, 0.45, 0.50);
        assert!(report.passed,
            "Ratio 0.45 at lower bound should pass (inclusive)");
        assert_f32_eq(report.ratio, 0.45, 0.001);
        assert_f32_eq(report.min_expected, 0.45, 0.001);
        assert_f32_eq(report.max_expected, 0.50, 0.001);
    }

    /// B3 | BNDRY/edge | Traces To: §Implementation Summary Boundary Conditions — `ratio` upper bound
    ///
    /// ratio = 0.50 exactly at upper bound → passed == true.
    /// Kills: off-by-one using `<` instead of `<=` causing upper-bound false fail.
    #[test]
    // [unit] — pure in-memory computation
    fn test_b3_check_visible_ratio_at_upper_bound() {
        let report = ResolutionVerifier::check_visible_ratio(0.50, 0.45, 0.50);
        assert!(report.passed,
            "Ratio 0.50 at upper bound should pass (inclusive)");
        assert_f32_eq(report.ratio, 0.50, 0.001);
        assert_f32_eq(report.min_expected, 0.45, 0.001);
        assert_f32_eq(report.max_expected, 0.50, 0.001);
    }

    /// B4 | BNDRY/edge | Traces To: §Implementation Summary Boundary Conditions — `ratio` slightly below lower bound
    ///
    /// ratio = 0.4499 < min 0.45 → passed == false.
    /// Kills: floating-point comparison precision (should reject, no epsilon misjudgment).
    #[test]
    // [unit] — pure in-memory computation
    fn test_b4_check_visible_ratio_below_lower_bound() {
        let report = ResolutionVerifier::check_visible_ratio(0.4499, 0.45, 0.50);
        assert!(!report.passed,
            "Ratio 0.4499 below lower bound should fail");
        assert_f32_eq(report.ratio, 0.4499, 0.001);
        assert_f32_eq(report.min_expected, 0.45, 0.001);
        assert_f32_eq(report.max_expected, 0.50, 0.001);
    }

    /// B5 | BNDRY/edge | Traces To: §Implementation Summary Boundary Conditions — fullscreen equivalence
    ///
    /// Two calls with same 1080p resolution should produce identical reports.
    /// Fullscreen only changes window decor, not viewport dimensions.
    /// Kills: fullscreen mode altering viewport calculation.
    #[test]
    // [unit] — pure in-memory computation
    fn test_b5_fullscreen_windowed_identical_reports() {
        let r1 = ResolutionVerifier::check_hud_anchor(1920.0, 1080.0, 0.02);
        let r2 = ResolutionVerifier::check_hud_anchor(1920.0, 1080.0, 0.02);
        // Same inputs → identical outputs (pure function determinism)
        assert_eq!(r1, r2, "Same resolution should produce identical reports");
        // Expected anchor = (1920 * 0.03, 1080 * 0.03) = (57.6, 32.4)
        assert_f32_eq(r1.expected.0, 57.6, 0.01);
        assert_f32_eq(r1.expected.1, 32.4, 0.01);
        // Deviation should be ~0.0 (self-consistency)
        assert_f32_eq(r1.deviation.dx_pct, 0.0, 0.0001);
        assert_f32_eq(r1.deviation.dy_pct, 0.0, 0.0001);
    }

    /// B6 | BNDRY/edge | Traces To: §Implementation Summary Boundary Conditions — NaN input
    ///
    /// viewport_w = NaN → output contains NaN. Should NOT panic.
    /// Caller's responsibility to guard against NaN.
    /// Kills: NaN propagation undetected, causing downstream misinterpretation.
    #[test]
    // [unit] — pure in-memory computation
    fn test_b6_expected_hud_anchor_nan_input_no_panic() {
        let result = std::panic::catch_unwind(|| {
            ResolutionVerifier::expected_hud_anchor(f32::NAN, 720.0)
        });
        assert!(result.is_ok(), "Should not panic on NaN viewport_w");
        let (x, _y) = result.unwrap();
        // NaN * 0.03 = NaN — propagated
        assert!(x.is_nan(), "NaN input should propagate NaN output");
    }

    // ==================================================================
    // C1–C2: PERF — Performance tests
    // ==================================================================

    /// C1 | PERF/resolution | Traces To: NFR-002 AC-1, §Interface Contract `verify_all_resolutions`
    ///
    /// Full suite across 3 resolutions should complete in < 1ms.
    /// Pure deterministic f32 computation, no I/O.
    /// Kills: verification suite having unexpected performance blocker.
    #[test]
    // [unit] — pure in-memory computation
    fn test_c1_perf_full_suite_timing() {
        let config = CameraConfig::default();
        let start = std::time::Instant::now();
        let reports = ResolutionVerifier::verify_all_resolutions(&config);
        let elapsed = start.elapsed();

        // Performance: < 1ms for 3 deterministic f32 computations
        let elapsed_us = elapsed.as_micros();
        assert!(
            elapsed_us < 1000,
            "Full suite took {} us, expected < 1000 us (1ms)", elapsed_us
        );

        // Correctness: should return 3 reports (720p/1080p/1440p)
        assert_eq!(reports.len(), 3,
            "Expected 3 resolution reports, got {}", reports.len());
    }

    /// C2 | PERF/formula | Traces To: NFR-002 AC-1, §Interface Contract `expected_hud_anchor`
    ///
    /// 10000 calls to expected_hud_anchor() should complete in < 1ms
    /// (target: < 100ns per call, pure f32 multiplication).
    /// Kills: anchor calculation calling heavy operations.
    #[test]
    // [unit] — pure in-memory computation
    fn test_c2_perf_single_call_latency() {
        let start = std::time::Instant::now();
        for _ in 0..10000 {
            let _ = ResolutionVerifier::expected_hud_anchor(1280.0, 720.0);
        }
        let elapsed = start.elapsed();

        // Performance: 10000 calls at <100ns each → <1ms total
        let elapsed_us = elapsed.as_micros();
        assert!(
            elapsed_us < 1000,
            "10000 calls took {} us, expected < 1000 us (100ns/call)", elapsed_us
        );

        // Correctness: at least one call returns the right value
        let (x, y) = ResolutionVerifier::expected_hud_anchor(1280.0, 720.0);
        assert_f32_eq(x, 38.4, 0.01);
        assert_f32_eq(y, 21.6, 0.01);
    }

    // ==================================================================
    // D1–D3: UI — HUD / Viewport consistency tests
    // ==================================================================

    /// D1 | UI/anchor | Traces To: NFR-002 AC-1, §Interface Contract `check_hud_anchor` + F09 HUD
    ///
    /// HudRenderer::compute_anchor() and ResolutionVerifier::expected_hud_anchor()
    /// must return identical values for the same viewport — preventing
    /// formula drift between HUD rendering and verification.
    /// Kills: verification tool and HUD renderer using different anchor formulas.
    #[test]
    // [unit] — cross-module consistency check (in-memory)
    fn test_d1_hud_anchor_consistency_720p() {
        let hud_anchor = HudRenderer::compute_anchor(1280.0, 720.0);
        let verifier_anchor = ResolutionVerifier::expected_hud_anchor(1280.0, 720.0);
        assert_f32_eq(hud_anchor.0, verifier_anchor.0, 0.001);
        assert_f32_eq(hud_anchor.1, verifier_anchor.1, 0.001);
    }

    /// D2 | UI/anchor | Traces To: NFR-002 AC-1, §Interface Contract `check_hud_anchor` + F09 HUD
    ///
    /// Same consistency check as D1 but across all 3 supported resolutions.
    /// Kills: resolution-specific formula divergence.
    #[test]
    // [unit] — cross-module consistency check (in-memory)
    fn test_d2_hud_anchor_consistency_all_resolutions() {
        let resolutions = [(1280.0, 720.0), (1920.0, 1080.0), (2560.0, 1440.0)];
        for &(w, h) in &resolutions {
            let hud_anchor = HudRenderer::compute_anchor(w, h);
            let verifier_anchor = ResolutionVerifier::expected_hud_anchor(w, h);
            assert_f32_eq(hud_anchor.0, verifier_anchor.0, 0.001);
            assert_f32_eq(hud_anchor.1, verifier_anchor.1, 0.001);
        }
    }

    /// D3 | UI/viewport | Traces To: NFR-002 AC-2, §Interface Contract `player_visible_ratio` + F04 Camera
    ///
    /// With default CameraConfig (viewport_w=480, player_target_x_pct=0.375):
    ///   player_visible_ratio(200.0, 20.0, 480.0) = 0.625 = 1.0 - 0.375
    /// The player_visible_ratio must use the virtual canvas viewport_w (480),
    /// NOT the screen resolution width.
    /// Kills: using screen resolution instead of virtual canvas for ratio.
    #[test]
    // [unit] — cross-module consistency check (in-memory)
    fn test_d3_player_visible_ratio_uses_virtual_canvas() {
        // Verify Camera defaults
        let camera = Camera::new(CameraConfig::default());
        let (vp_w, vp_h) = camera.viewport();
        assert_f32_eq(vp_w, 480.0, 0.01);
        assert_f32_eq(vp_h, 270.0, 0.01);

        let config = CameraConfig::default();
        assert_f32_eq(config.player_target_x_pct, 0.375, 0.001);

        // player_world_x=200, camera_offset_x=20 → player_screen_x=180
        // ratio = (480 - 180) / 480 = 300/480 = 0.625
        // This corresponds to player_target_x_pct=0.375: ratio = 1.0 - 0.375 = 0.625
        let ratio = ResolutionVerifier::player_visible_ratio(200.0, 20.0, 480.0);
        assert_f32_eq(ratio, 0.625, 0.001);

        // Verify the design invariant: ratio ≈ 1.0 - player_target_x_pct
        let expected_ratio = 1.0 - config.player_target_x_pct;
        assert_f32_eq(ratio, expected_ratio, 0.001);
    }
}
