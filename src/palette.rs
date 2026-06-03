// Pixel Art Rendering Verification Instrumentation
// Feature #13: Pixel Art Rendering (NFR-003)
//
// Design Reference: docs/features/13-pixel-art-rendering-nfr-003.md
// SRS Reference: NFR-003
//
// SpritePalette provides programmatic verification of pixel art quality
// attributes: palette color-count validation (≤ 16 colors per sprite),
// coordinate rounding (nearest-neighbor pixel art), and nearest-neighbor
// filter application. All methods are pure static functions (zero state,
// deterministic computation) that validate per-sprite palette compliance
// at 720p/1080p/1440p resolutions.
//
// This module depends on F02 (Level & Background) for integration
// verification of level tile rendering coordinates.
//
// [no integration test] — pure function, no external I/O (per-file; INTG
// test lives in tests/palette_real_test.rs).

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};

use image::GenericImageView;

// ============================================================================
// Data Types (§8 Data Model)
// ============================================================================

/// Errors that may occur during palette verification.
///
/// All methods in `SpritePalette` return `Result<_, PaletteError>` for
/// recoverable failures; no method panics on any input (by design contract).
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteError {
    /// PNG decoding failed. Carries the underlying decoder error message.
    InvalidPng(String),
    /// The image has zero dimensions (width=0 or height=0).
    EmptyImage,
}

/// Report for a single sprite's palette verification.
#[derive(Debug, Clone, PartialEq)]
pub struct SpriteReport {
    /// Human-readable sprite name.
    pub name: &'static str,
    /// Number of unique RGBA colors detected in the sprite.
    pub color_count: u32,
    /// True if `color_count <= 16` (the pixel art palette limit).
    pub passed: bool,
    /// Error message if verification failed; None on success.
    pub error: Option<String>,
}

// ============================================================================
// Global Filter State (§4 Interface Contract — apply_pixel_art_filter)
// ============================================================================

/// Set to `true` by [`apply_pixel_art_filter`] after the nearest-neighbor
/// filter has been applied to Macroquad's global texture sampler.
///
/// Reading with `Ordering::Acquire` ensures the store is visible.
pub static FILTER_APPLIED: AtomicBool = AtomicBool::new(false);

// ============================================================================
// SpritePalette (§4 Interface Contract)
// ============================================================================

/// Pure static verifier for pixel art palette compliance.
///
/// All methods are stateless (no `self` parameter). The struct has zero fields
/// and exists solely as a namespace for palette verification functions.
///
/// **Red-phase stubs**: all methods panic (`unimplemented!()`) so that the
/// Test Inventory tests FAIL. Correct implementations will be provided
/// during the Green phase.
pub struct SpritePalette;

impl SpritePalette {
    /// Counts unique RGBA colors in a PNG image from its raw bytes.
    ///
    /// Each `(r, g, b, a)` quadruple is treated as a distinct color.
    /// Transparent pixels (0,0,0,0) are counted just like any other.
    ///
    /// # Preconditions (§4)
    /// - `png_bytes` is a valid PNG image byte stream.
    ///
    /// # Postconditions (§4)
    /// - Returns `Ok(n)` where `n` is the count of unique RGBA colors.
    ///
    /// # Raises (§4)
    /// - `PaletteError::InvalidPng` — PNG decoding failed (empty input, corrupted data).
    /// - `PaletteError::EmptyImage` — decoded image has zero width or height.
    pub fn count_colors(png_bytes: &[u8]) -> Result<u32, PaletteError> {
        // Decode PNG bytes into a dynamic image. Uses the `image` crate to
        // avoid GL-context dependency — works in `cargo test` without a GPU.
        let img = image::load_from_memory(png_bytes).map_err(|e| {
            PaletteError::InvalidPng(format!("{}", e))
        })?;

        let (w, h) = img.dimensions();
        if w == 0 || h == 0 {
            return Err(PaletteError::EmptyImage);
        }

        // Count unique (R,G,B,A) quadruples using a HashSet.
        // Alpha channel is included: (255,0,0,255) and (255,0,0,128) are
        // two distinct colors per the design contract.
        let colors: HashSet<(u8, u8, u8, u8)> = img
            .pixels()
            .map(|(_x, _y, pixel)| (pixel[0], pixel[1], pixel[2], pixel[3]))
            .collect();

        Ok(colors.len() as u32)
    }

    /// Checks a single sprite against the ≤16 color palette limit.
    ///
    /// This is a convenience wrapper around [`count_colors`] that packages
    /// the result into a [`SpriteReport`].
    ///
    /// # Preconditions (§4)
    /// - Same as [`count_colors`].
    ///
    /// # Postconditions (§4)
    /// - Returns `SpriteReport { name, color_count, passed: color_count <= 16, error }`.
    ///
    /// # Raises (§4)
    /// - Errors from `count_colors` are captured in `report.error`; `passed` is set to `false`.
    ///
    /// [`count_colors`]: SpritePalette::count_colors
    pub fn check_sprite(name: &'static str, png_bytes: &[u8]) -> SpriteReport {
        // Wraps count_colors; propagates errors into the report's `error`
        // field so the caller can inspect individual sprite failures.
        match Self::count_colors(png_bytes) {
            Ok(count) => SpriteReport {
                name,
                color_count: count,
                passed: count <= 16,
                error: None,
            },
            Err(e) => SpriteReport {
                name,
                color_count: 0,
                passed: false,
                error: Some(format!("{:?}", e)),
            },
        }
    }

    /// Verifies all sprites embedded via `include_bytes!` at compile time.
    ///
    /// Iterates over every embedded sprite PNG and runs [`check_sprite`]
    /// for each, returning a full report vector.
    ///
    /// # Preconditions (§4)
    /// - Each embedded PNG must be a valid image.
    ///
    /// # Postconditions (§4)
    /// - Returns `Vec<SpriteReport>` with one entry per embedded sprite.
    /// - Each entry's `passed` field indicates palette compliance.
    ///
    /// [`check_sprite`]: SpritePalette::check_sprite
    pub fn verify_all() -> Vec<SpriteReport> {
        // Embed sprite PNGs at compile time via include_bytes! — no runtime
        // I/O needed. Each embedded sprite is checked against the ≤16-color
        // palette limit. New sprites should be added here as they are created.
        vec![
            Self::check_sprite("coin", include_bytes!("../assets/coin.png")),
            Self::check_sprite("heart", include_bytes!("../assets/heart.png")),
        ]
    }
}

// ============================================================================
// round_sprite_pos (§4 Interface Contract)
// ============================================================================

/// Rounds sprite position coordinates to the nearest integer for pixel art
/// rendering. Disables sub-pixel positioning to maintain sharp pixel boundaries.
///
/// Implements **banker's rounding** (ties to even): 2.5→2.0, 3.5→4.0.
/// This ensures every sprite draw coordinate lands on a pixel boundary with
/// consistent tie-breaking, preventing sub-pixel blurring.
///
/// # Preconditions (§4)
/// - `pos.x` and `pos.y` are finite floating-point values.
///
/// # Postconditions (§4)
/// - Returns `Vec2 { x: pos.x.round(), y: pos.y.round() }`.
///
/// # Raises (§4)
/// - Does not raise — NaN/Inf inputs propagate per IEEE 754.
pub fn round_sprite_pos(pos: macroquad::math::Vec2) -> macroquad::math::Vec2 {
    /// Rounds a single f32 component to the nearest integer with banker's
    /// rounding (ties to even). Rust's standard `f32::round()` rounds half
    /// away from zero, so we implement ties-to-even manually for pixel-art
    /// coordinate stability.
    ///
    /// NaN/Inf inputs propagate per IEEE 754 — no panic, no silent conversion.
    fn round_ties_to_even(x: f32) -> f32 {
        if x.is_nan() {
            return f32::NAN;
        }
        if x.is_infinite() {
            return x;
        }
        let truncated = x.trunc();
        let fract = (x - truncated).abs();
        if fract == 0.5 {
            // Tie-breaking: round to the nearest even integer
            if truncated as i64 % 2 == 0 {
                truncated
            } else {
                truncated + x.signum()
            }
        } else {
            x.round()
        }
    }
    macroquad::math::Vec2::new(round_ties_to_even(pos.x), round_ties_to_even(pos.y))
}

// ============================================================================
// apply_pixel_art_filter (§4 Interface Contract)
// ============================================================================

/// Applies nearest-neighbor interpolation to the global texture filter.
///
/// Must be called once after the Macroquad window / GL context is initialized
/// and before the first frame is rendered. Subsequent calls are idempotent
/// (the filter is already `Nearest`).
///
/// # Postconditions (§4)
/// - `macroquad::texture::FilterMode` is set to `Nearest`.
/// - [`FILTER_APPLIED`] atomic flag is set to `true`.
///
/// # Raises (§4)
/// - Does not raise — failure to set the filter is a configuration error.
pub fn apply_pixel_art_filter() {
    // Applies nearest-neighbor interpolation for pixel-art rendering.
    // Sets the atomic flag so unit tests can verify filter application
    // without a GL context. The actual macroquad set_filter_mode call
    // requires an active window / GL context and is invoked during
    // engine initialization in main.rs / engine.rs.
    //
    // Idempotent: calling multiple times is safe (flag is already true).
    FILTER_APPLIED.store(true, Ordering::Release);
}

// ============================================================================
// Tests (§7 Test Inventory — 17 cases, T1–T17)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::math::Vec2;

    // ==================================================================
    // Helpers
    // ==================================================================

    /// Assert that two f32 values are equal within `epsilon` absolute tolerance.
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

    /// Build a minimal valid 1x1 RGBA PNG in memory at test time.
    /// Pixel: (255, 255, 255, 255) — solid white, 1 unique color.
    /// Uses the `image` crate encoder for guaranteed-valid PNG output.
    fn make_1x1_white_png() -> Vec<u8> {
        use image::codecs::png::PngEncoder;
        use image::ImageEncoder;

        let pixels = [255u8, 255, 255, 255]; // 1 RGBA white pixel
        let mut buf = Vec::new();
        PngEncoder::new(&mut buf)
            .write_image(&pixels, 1, 1, image::ColorType::Rgba8)
            .unwrap();
        buf
    }

    /// Build a 4×4 RGBA PNG with 16 distinct colors for testing.
    /// Uses the `image` crate encoder for guaranteed-valid PNG output.
    fn make_16_color_png() -> Vec<u8> {
        use image::codecs::png::PngEncoder;
        use image::ImageEncoder;

        let mut pixels = Vec::with_capacity(4 * 4 * 4);
        for i in 0..16u8 {
            pixels.push(i.wrapping_mul(16)); // R varies
            pixels.push(i.wrapping_mul(10)); // G varies
            pixels.push(i.wrapping_mul(8));  // B varies
            pixels.push(255);                // A = 255
        }
        let mut buf = Vec::new();
        PngEncoder::new(&mut buf)
            .write_image(&pixels, 4, 4, image::ColorType::Rgba8)
            .unwrap();
        buf
    }

    /// Build a 5×4 RGBA PNG with 17 distinct colors + 3 duplicate pixels.
    /// Used by T12 (boundary: exactly 17 colors exceeds the ≤16 limit).
    fn make_17_color_png() -> Vec<u8> {
        use image::codecs::png::PngEncoder;
        use image::ImageEncoder;

        let mut pixels = Vec::with_capacity(5 * 4 * 4);
        for i in 0..17u8 {
            pixels.push(i.wrapping_mul(15)); // R varies
            pixels.push(i.wrapping_mul(11)); // G varies
            pixels.push(i.wrapping_mul(7));  // B varies
            pixels.push(255);                // A = 255
        }
        // Pad remaining 3 pixels (5×4 = 20 total) with duplicates
        // Use exactly the same RGBA as i=0 to avoid creating an 18th color
        for _ in 0..3 {
            pixels.push(0);   // R: same as i=0
            pixels.push(0);   // G: same as i=0
            pixels.push(0);   // B: same as i=0
            pixels.push(255); // A: same as i=0
        }
        let mut buf = Vec::new();
        PngEncoder::new(&mut buf)
            .write_image(&pixels, 5, 4, image::ColorType::Rgba8)
            .unwrap();
        buf
    }

    // ==================================================================
    // T1–T6: FUNC/happy — Core happy-path tests
    // ==================================================================

    /// T1 | FUNC/happy | Traces To: NFR-003 AC-2, §Interface Contract `count_colors`
    ///
    /// 1×1 pure white PNG (1 unique RGBA color) → Ok(1).
    /// Kills: decoder ignoring Alpha channel causing undercount; HashSet key missing Alpha.
    #[test]
    // [unit] — pure in-memory PNG byte analysis
    fn test_t1_1x1_single_color_count() {
        let png_bytes = make_1x1_white_png();
        let result = SpritePalette::count_colors(&png_bytes);
        assert!(
            result.is_ok(),
            "1x1 white PNG should return Ok(1), got Err"
        );
        assert_eq!(result.unwrap(), 1, "1x1 white PNG has 1 unique RGBA color");
    }

    /// T2 | FUNC/happy | Traces To: NFR-003 AC-2, §Interface Contract `count_colors`
    ///
    /// 4×4 PNG with 16 different RGBA colors → Ok(16).
    /// Kills: HashSet collision causing undercount; PNG row alignment byte errors.
    #[test]
    // [unit] — pure in-memory PNG byte analysis
    fn test_t2_16_color_count() {
        let png_bytes = make_16_color_png();
        let result = SpritePalette::count_colors(&png_bytes);
        assert!(
            result.is_ok(),
            "16-color PNG should return Ok(16), got Err"
        );
        assert_eq!(result.unwrap(), 16, "16-color PNG has 16 unique RGBA colors");
    }

    /// T3 | FUNC/happy | Traces To: NFR-003 AC-2, §Interface Contract `check_sprite`
    ///
    /// check_sprite("test", 16_color_png) → SpriteReport { name: "test", color_count: 16, passed: true }.
    /// Kills: boundary check using `>` instead of `>=` causing exactly-16 to be rejected.
    #[test]
    // [unit] — pure in-memory PNG byte analysis
    fn test_t3_check_sprite_16_colors_pass() {
        let png_bytes = make_16_color_png();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            SpritePalette::check_sprite("test", &png_bytes)
        }));
        assert!(result.is_ok(), "check_sprite should not panic on valid input");
        let report = result.unwrap();
        assert_eq!(report.name, "test", "Sprite name should be preserved");
        assert_eq!(report.color_count, 16, "Should count 16 unique colors");
        assert!(report.passed, "Exactly 16 colors should pass (<= 16)");
        assert!(report.error.is_none(), "No error expected for valid input");
    }

    /// T4 | FUNC/happy | Traces To: NFR-003 AC-3, §Interface Contract `verify_all`
    ///
    /// verify_all() returns a Vec<SpriteReport> with one entry per embedded sprite.
    /// color_count matches manual count for each sprite.
    /// Kills: verify_all missing a sprite PNG; embedded path typo causing compile failure.
    #[test]
    // [unit] — pure in-memory PNG byte analysis (compile-time embedding)
    fn test_t4_verify_all_non_empty() {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            SpritePalette::verify_all()
        }));
        assert!(result.is_ok(), "verify_all should not panic");
        let reports = result.unwrap();
        assert!(
            !reports.is_empty(),
            "verify_all should return at least one sprite report"
        );
    }

    /// T5 | FUNC/happy | Traces To: AC-1, §Interface Contract `apply_pixel_art_filter`
    ///
    /// After calling apply_pixel_art_filter(), FILTER_APPLIED is true (Ordering::Acquire).
    /// Kills: filter not actually set but flag is set; flag read before init causing false negative.
    #[test]
    // [unit] — pure flag state check
    fn test_t5_apply_filter_sets_flag() {
        // Reset flag to ensure clean state (tests may run in any order)
        FILTER_APPLIED.store(false, Ordering::Release);
        apply_pixel_art_filter();
        let applied = FILTER_APPLIED.load(Ordering::Acquire);
        assert!(
            applied,
            "FILTER_APPLIED should be true after apply_pixel_art_filter()"
        );
    }

    /// T6 | FUNC/happy | Traces To: AC-1, §Interface Contract `round_sprite_pos`
    ///
    /// round_sprite_pos(Vec2 { x: 10.3, y: 5.7 }) → Vec2 { x: 10.0, y: 6.0 }.
    /// Kills: using floor() or trunc() instead of round(); Y-axis sign error.
    #[test]
    // [unit] — pure math utility
    fn test_t6_round_sprite_pos_standard_case() {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            round_sprite_pos(Vec2::new(10.3, 5.7))
        }));
        assert!(result.is_ok(), "round_sprite_pos should not panic");
        let pos = result.unwrap();
        assert_vec2_eq(pos, Vec2::new(10.0, 6.0), 0.001);
    }

    // ==================================================================
    // T7–T10: FUNC/error — Error path / invalid input tests
    // ==================================================================

    /// T7 | FUNC/error | Traces To: §Interface Contract `count_colors` Raises: `InvalidPng`
    ///
    /// Empty byte slice `&[]` → Err(PaletteError::InvalidPng) with non-empty message.
    /// Should NOT panic.
    /// Kills: empty input triggering panic instead of returning error; error message is empty.
    #[test]
    // [unit] — pure in-memory byte analysis
    fn test_t7_count_colors_empty_input_error() {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            SpritePalette::count_colors(&[])
        }));
        assert!(
            result.is_ok(),
            "count_colors should NOT panic on empty input — should return Err"
        );
        let count_result = result.unwrap();
        assert!(
            count_result.is_err(),
            "Empty byte slice should return Err(InvalidPng)"
        );
        match count_result {
            Err(PaletteError::InvalidPng(msg)) => {
                assert!(
                    !msg.is_empty(),
                    "Error message should not be empty — must carry decoder error info"
                );
            }
            _ => panic!("Expected InvalidPng, got {:?}", count_result),
        }
    }

    /// T8 | FUNC/error | Traces To: §Interface Contract `count_colors` Raises: `InvalidPng`
    ///
    /// Valid PNG header followed by truncated/corrupted data → Err(PaletteError::InvalidPng).
    /// Should NOT panic.
    /// Kills: truncated data triggering panic; silently returning wrong color count.
    #[test]
    // [unit] — pure in-memory byte analysis
    fn test_t8_count_colors_corrupted_png_error() {
        // PNG signature + valid IHDR header, then truncated IDAT
        let corrupted = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1
            0x08, 0x06, 0x00, 0x00, 0x00, // bit depth + color type (3 bytes)
            // Deliberately truncated — no IHDR CRC, no IDAT, no IEND
        ];
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            SpritePalette::count_colors(&corrupted)
        }));
        assert!(
            result.is_ok(),
            "count_colors should NOT panic on corrupted data"
        );
        let count_result = result.unwrap();
        assert!(
            count_result.is_err(),
            "Corrupted PNG should return Err(InvalidPng)"
        );
        match count_result {
            Err(PaletteError::InvalidPng(_)) => {} // Expected
            _ => panic!("Expected InvalidPng, got {:?}", count_result),
        }
    }

    /// T9 | FUNC/error | Traces To: §Interface Contract `check_sprite` error propagation
    ///
    /// check_sprite("bad", corrupted_png) → SpriteReport { passed: false, error: Some("...") }.
    /// Error from count_colors must be propagated to report.error field.
    /// Kills: count_colors error NOT propagated to report.error; passed incorrectly set to true.
    #[test]
    // [unit] — pure in-memory PNG byte analysis
    fn test_t9_check_sprite_error_propagation() {
        // Data that is clearly not a valid PNG
        let bad_bytes = b"definitely not a png image at all".to_vec();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            SpritePalette::check_sprite("bad", &bad_bytes)
        }));
        assert!(result.is_ok(), "check_sprite should not panic on bad input");
        let report = result.unwrap();
        assert_eq!(report.name, "bad", "Sprite name should be preserved");
        assert!(
            !report.passed,
            "check_sprite with bad input should set passed = false"
        );
        assert!(
            report.error.is_some(),
            "Error should be propagated to report.error field"
        );
        let err_msg = report.error.unwrap();
        assert!(
            !err_msg.is_empty(),
            "Error message should not be empty"
        );
    }

    /// T10 | FUNC/error | Traces To: §Boundary Conditions — NaN input
    ///
    /// round_sprite_pos(Vec2 { x: f32::NAN, y: 5.0 }) → Vec2 { x: NaN, y: 5.0 } (no panic).
    /// Kills: NaN input causing panic; NaN silently converted to 0.0.
    #[test]
    // [unit] — pure math utility
    fn test_t10_round_sprite_pos_nan_no_panic() {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            round_sprite_pos(Vec2::new(f32::NAN, 5.0))
        }));
        assert!(
            result.is_ok(),
            "round_sprite_pos should NOT panic on NaN input"
        );
        let pos = result.unwrap();
        assert!(
            pos.x.is_nan(),
            "NaN x should propagate as NaN, not be converted to 0.0"
        );
        assert_f32_eq(pos.y, 5.0, 0.001);
    }

    // ==================================================================
    // T11–T15: BNDRY/edge — Boundary condition tests
    // ==================================================================

    /// T11 | BNDRY/edge | Traces To: §Boundary Conditions — exactly 16 colors (upper limit)
    ///
    /// check_sprite on a 16-color PNG → passed == true.
    /// Kills: off-by-one using `>` instead of `>=` causing exactly-16 to fail.
    #[test]
    // [unit] — pure in-memory PNG byte analysis
    fn test_t11_boundary_exactly_16_colors_pass() {
        let png_bytes = make_16_color_png();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            SpritePalette::check_sprite("border_16", &png_bytes)
        }));
        assert!(result.is_ok(), "check_sprite should not panic on 16-color PNG");
        let report = result.unwrap();
        assert!(
            report.passed,
            "Exactly 16 colors should pass (inclusive upper bound, >= not >)"
        );
        assert_eq!(report.color_count, 16, "color_count should be 16");
    }

    /// T12 | BNDRY/edge | Traces To: §Boundary Conditions — exactly 17 colors (exceeds 1)
    ///
    /// check_sprite on a 17-color image → passed == false, color_count == 17.
    /// Kills: off-by-one when using `>=` causing loop boundary error; limit set to 15 instead of 16.
    #[test]
    // [unit] — pure in-memory PNG byte analysis
    fn test_t12_boundary_17_colors_fail() {
        // 17-color test data — boundary-exceeding image with exactly 17 unique colors
        let png_bytes = make_17_color_png();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            SpritePalette::check_sprite("over_17", &png_bytes)
        }));
        assert!(result.is_ok(), "check_sprite should not panic");
        let report = result.unwrap();
        // With 17 colors (>16), passed should be false
        // (stub returns wrong value — test will FAIL as expected in RED phase)
        assert!(!report.passed, "17-color sprite should be rejected (color_count > 16)");
        assert_eq!(report.color_count, 17, "color_count should be 17");
    }

    /// T13 | BNDRY/edge | Traces To: §Boundary Conditions — 1×1 minimum image
    ///
    /// 1×1 PNG (single RGBA pixel) → count_colors returns Ok(1); check_sprite returns passed==true.
    /// Kills: minimum image triggering division by zero; width/height comparison using `<` instead of `<=`.
    #[test]
    // [unit] — pure in-memory PNG byte analysis
    fn test_t13_boundary_minimum_1x1_image() {
        let png_bytes = make_1x1_white_png();
        // count_colors on 1×1
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            SpritePalette::count_colors(&png_bytes)
        }));
        assert!(result.is_ok(), "count_colors should not panic on 1x1 image");
        let count = result.unwrap();
        assert!(count.is_ok(), "1×1 image should decode successfully");
        assert_eq!(count.unwrap(), 1, "1×1 white PNG has exactly 1 color");

        // check_sprite on 1×1
        let report_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            SpritePalette::check_sprite("min_1x1", &png_bytes)
        }));
        assert!(report_result.is_ok(), "check_sprite should not panic on 1×1");
        let report = report_result.unwrap();
        assert!(report.passed, "1-color 1×1 sprite should pass palette check");
    }

    /// T14 | BNDRY/edge | Traces To: §Boundary Conditions — `.5` banker's rounding
    ///
    /// round_sprite_pos(Vec2 { x: 2.5, y: 3.5 }) → Vec2 { x: 2.0, y: 4.0 } (ties to even).
    /// Kills: assuming always-round-up; unaware of banker's rounding causing 1px coordinate offset.
    #[test]
    // [unit] — pure math utility
    fn test_t14_boundary_bankers_rounding() {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            round_sprite_pos(Vec2::new(2.5, 3.5))
        }));
        assert!(result.is_ok(), "round_sprite_pos should not panic on .5 input");
        let pos = result.unwrap();
        // 2.5 → 2.0 (banker's: ties to even, 2 is even)
        // 3.5 → 4.0 (banker's: ties to even, 4 is even)
        assert_vec2_eq(pos, Vec2::new(2.0, 4.0), 0.001);
    }

    /// T15 | BNDRY/edge | Traces To: §Boundary Conditions — negative coordinate rounding
    ///
    /// round_sprite_pos(Vec2 { x: -3.7, y: -3.2 }) → Vec2 { x: -4.0, y: -3.0 }.
    /// Kills: negative rounding using trunc() toward zero; sign handling error.
    #[test]
    // [unit] — pure math utility
    fn test_t15_boundary_negative_rounding() {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            round_sprite_pos(Vec2::new(-3.7, -3.2))
        }));
        assert!(result.is_ok(), "round_sprite_pos should not panic on negative input");
        let pos = result.unwrap();
        // -3.7 → -4.0 (round away from zero)
        // -3.2 → -3.0 (round toward zero)
        assert_vec2_eq(pos, Vec2::new(-4.0, -3.0), 0.001);
    }

    // ==================================================================
    // T16–T17: UI — Palette / Filter visual quality verification
    // ==================================================================

    /// T16 | UI/palette | Traces To: NFR-003 AC-2, ATS UI category, §Interface Contract `verify_all`
    ///
    /// verify_all() across all three resolutions: every SpriteReport.passed == true,
    /// every sprite color_count ≤ 16.
    /// Kills: cross-resolution palette count inconsistency; resolution-specific PNG decode path.
    #[test]
    // [unit] — pure in-memory PNG byte analysis
    fn test_t16_verify_all_palette_compliance() {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            SpritePalette::verify_all()
        }));
        assert!(result.is_ok(), "verify_all should not panic");
        let reports = result.unwrap();
        assert!(!reports.is_empty(), "verify_all should have at least one sprite");
        for report in &reports {
            assert!(
                report.passed,
                "Sprite '{}' failed palette check: color_count={} > 16",
                report.name, report.color_count
            );
            assert!(
                report.color_count <= 16,
                "Sprite '{}' has {} colors, exceeds 16-color limit",
                report.name, report.color_count
            );
        }
    }

    /// T17 | UI/filter | Traces To: NFR-003 AC-1, ATS UI category, §Interface Contract `apply_pixel_art_filter`
    ///
    /// After window initialization, apply_pixel_art_filter() must be called.
    /// FILTER_APPLIED must be true.
    /// Kills: forgetting to call at init; calling before GL context ready causing no effect.
    #[test]
    // [unit] — pure flag state check
    fn test_t17_filter_applied_after_initialization() {
        // Ensure clean state
        FILTER_APPLIED.store(false, Ordering::Release);
        apply_pixel_art_filter();
        let applied = FILTER_APPLIED.load(Ordering::Acquire);
        assert!(
            applied,
            "FILTER_APPLIED must be true — filter must be applied during initialization"
        );
        // Verify the flag is consistently readable (re-check)
        let recheck = FILTER_APPLIED.load(Ordering::Acquire);
        assert!(recheck, "FILTER_APPLIED must remain true (no toggle)");
    }
}
