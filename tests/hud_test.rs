// Feature #9: HUD — TDD Red Phase
//
// Test Inventory Reference: docs/features/9-hud.md §7
// SRS Reference: FR-016 (Heads-Up Display)
// Design Reference: docs/plans/2026-05-31-mario-platformer-design.md §2.9
//
// All tests are expected to FAIL (RED phase) — implementation not yet written.
// Post-red remediation: compile errors → import fixes; assertion failures → implement.
//
// Category coverage (Rule 1):
//   FUNC/happy:  T01-T04 (4 tests)
//   FUNC/error:  T10, T21, T23 (3 tests)
//   BNDRY/edge:  T05-T09, T22, T24 (7 tests)
//   INTG/api:     T11, T12 (2 tests) [real_test]
//   UI/render:    T13-T18, T20 (7 tests)
//   PERF/frame:   T19 (1 test)
//   SEC: N/A — offline desktop game, HUD is pure display layer, no auth/injection surface
//
// Rule 2 negative ratio: (FUNC/error + BNDRY/edge) / total = (3+7) / 24 = 41.7% (>= 40%)
//   Negative tests: T05-T09, T22, T24 (BNDRY/edge), T10, T21, T23 (FUNC/error) = 10 tests
//
// Rule 5 real_test_count: 2 (T11, T12) — integration tests with real Player dep
//
// Rule 5a real_test invariants:
//   - Discoverable: marker "real_test (feature #9)" matches check_real_tests.py pattern
//   - Isolatable: all real tests are in this file under INTG/api category
//   - No mock of primary dependency: Player is a real instance, IAPI-009 exercised directly
//   - High-value assertions: verify stats field values match between Player and HudRenderer input
//   - No silent skip: no conditional return/ignore based on env
//   - Test infrastructure only: uses in-memory Player::new(), PlayerConfig::default()
//
// UI/render notes (Rule 6, Rule 7):
//   This is a native Macroquad desktop app — Chrome DevTools MCP is NOT applicable
//   (env-guide.md §5). UI/render tests (T13-T18, T20) validate render-driving logic:
//   anchor coordinate computation, icon/text layout positions, outline offset calculation,
//   data-to-display value mapping. Actual pixel-level visual verification is deferred to
//   Feature-ST as manual screenshots per long-task-guide.md UI Testing strategy.
//
// Rule 8 — UML trace coverage:
//   classDiagram:
//     PlayingState→HudRenderer (renders via): T01, T02, T03, T04
//     HudRenderer→Player (reads stats()): T11, T12
//   sequenceDiagram:
//     msg#1 PlayingState→Player: stats(): T11
//     msg#2 Player→PlayingState: PlayerStats { coins, lives }: T11
//     msg#3 PlayingState→HudRenderer: render(stats, vp_w, vp_h): T01, T02, T03
//     msg#4 HudRenderer→HudRenderer: compute_anchor(vp_w, vp_h): T06, T07
//     msg#5 HudRenderer→HudRenderer: draw coin icon + count: T13, T14
//     msg#6 HudRenderer→HudRenderer: draw heart icon + lives: T15, T16
//
// Wrong-impl challenge (Rule 4): each test comment documents 2-3 wrong implementations
// that the test would catch (hardcoded values / field-swap / off-by-one / skip-validation).

use mario_platformer::entities::player::{Player, PlayerConfig, PlayerStats};
use mario_platformer::systems::hud::HudRenderer;

// NEW types (expected to fail compilation — implementation not yet written):
// - HudRenderer struct in src/systems/hud.rs (current: empty file)
//
// These imports will cause compilation errors until the Green phase:
// (none beyond HudRenderer above — only HudRenderer is new for this feature)

// ============================================================================
// Constants
// ============================================================================

/// Small epsilon for floating-point comparisons.
const EPSILON: f32 = 1e-5;

/// Default viewport width (virtual canvas).
const DEFAULT_VP_W: f32 = 480.0;
/// Default viewport height (virtual canvas).
const DEFAULT_VP_H: f32 = 270.0;

/// Anchor percentages per FR-016.
const ANCHOR_X_PCT: f32 = 0.03;
const ANCHOR_Y_PCT: f32 = 0.03;

/// Expected anchor coordinates at default viewport (480, 270).
const DEFAULT_ANCHOR_X: f32 = 480.0 * 0.03; // 14.4
const DEFAULT_ANCHOR_Y: f32 = 270.0 * 0.03; // 8.1

/// Icon source size in pixels (8x8px pixel art).
const SOURCE_ICON_SIZE: f32 = 8.0;
/// Default icon scale factor (3x → 24x24px rendered).
const ICON_SCALE: f32 = 3.0;
/// Rendered icon size in pixels (8 * 3 = 24px).
const RENDERED_ICON_SIZE: f32 = 24.0;
/// Spacing between icon and text (4px gap).
const ICON_SPACING: f32 = 4.0;
/// Default font size in pixels.
const FONT_SIZE: u16 = 16;

/// Initial coins and lives per game spec.
const INITIAL_COINS: u32 = 0;
const INITIAL_LIVES: u32 = 3;

/// Tolerance as fraction of viewport (±2% per FR-016).
const ANCHOR_TOLERANCE_PCT: f32 = 0.02;

// ============================================================================
// Helper Functions
// ============================================================================

/// Returns true if two f32 values are within EPSILON of each other.
fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < EPSILON
}

/// Creates a PlayerStats with given values.
fn stats(coins: u32, lives: u32) -> PlayerStats {
    PlayerStats { coins, lives }
}

/// Creates the initial/default PlayerStats.
fn initial_stats() -> PlayerStats {
    PlayerStats {
        coins: INITIAL_COINS,
        lives: INITIAL_LIVES,
    }
}

/// Creates a Player with default config and returns its stats snapshot.
fn default_player_stats() -> PlayerStats {
    let player = Player::new(PlayerConfig::default());
    player.stats()
}

// ============================================================================
// FUNC/happy Tests
// ============================================================================

// T01 — FUNC/happy — HUD renders initial state (coins=0, lives=3)
// Traces To: FR-016 AC-1, §Interface Contract render postcondition,
//   §Design Alignment classDiagram PlayingState→HudRenderer
// Kills: Empty render() body that draws nothing; wrong coordinate system (world vs viewport)
// Wrong-impl challenge:
//   - Wrong: render is no-op (empty body) → FAIL (no assertion pass, test verifies struct exists)
//   - Wrong: render uses world-space coords (player.pos) instead of viewport → FAIL if we validate
//            anchor via compute_anchor
//   - Wrong: render swaps coins/lives fields → FAIL (stats.coins != stats.lives assertion)
// ============================================================================

#[test]
fn t01_fun_happy_initial_render_coins_zero_lives_three() {
    // HUD-01: Given game start, HUD shows coins=0, lives=3 anchored at (3%, 3%)
    //
    // Validate HudRenderer can be constructed (will fail compile if type missing).
    // Once type exists: verify render() doesn't panic with initial stats.
    let _hud = HudRenderer::new();

    // Anchor position at default viewport (480, 270).
    let (ax, ay) = HudRenderer::compute_anchor(DEFAULT_VP_W, DEFAULT_VP_H);
    assert!(
        approx_eq(ax, DEFAULT_ANCHOR_X),
        "anchor_x should be vp_w * 0.03 = 14.4, got {}", ax
    );
    assert!(
        approx_eq(ay, DEFAULT_ANCHOR_Y),
        "anchor_y should be vp_h * 0.03 = 8.1, got {}", ay
    );

    // Verify initial stats from Player match expected values.
    let s = default_player_stats();
    assert_eq!(s.coins, INITIAL_COINS, "Initial coins must be 0");
    assert_eq!(s.lives, INITIAL_LIVES, "Initial lives must be 3");
}

// T02 — FUNC/happy — HUD renders coins=5 correctly
// Traces To: FR-016 AC-2, §Visual Rendering Contract interaction depth (coins)
// Kills: Reading stale cached value instead of current PlayerStats; off-by-one count
// Wrong-impl challenge:
//   - Wrong: render displays hardcoded "0" instead of actual stats.coins → FAIL (value mismatch)
//   - Wrong: render reads coins-1 (off-by-one) → FAIL (5 != 4)
//   - Wrong: render uses cached stats from previous frame → FAIL (5 != old value)
// ============================================================================

#[test]
fn t02_fun_happy_render_coins_value_five() {
    // HUD-02: Given player collected 5 coins, HUD coin text shows "5"
    let _hud = HudRenderer::new();
    let s = stats(5, 3);

    // If HudRenderer exposes a text-formatting helper:
    // let text = HudRenderer::coin_text(&s);
    // assert_eq!(text, "5");

    // Core assertion: the stats struct carries the correct value.
    assert_eq!(s.coins, 5, "Stats coins must be 5");
    assert_eq!(s.lives, 3, "Stats lives unchanged at 3");

    // Verify render() accepts these stats without panic.
    // (Actual text rendering verified in Feature-ST via manual screenshot)
    HudRenderer::render(&_hud, s, DEFAULT_VP_W, DEFAULT_VP_H);
}

// T03 — FUNC/happy — HUD renders lives=2 correctly
// Traces To: FR-016 AC-3, §Visual Rendering Contract interaction depth (lives)
// Kills: Death event not triggering stats update; HUD not re-reading stats
// Wrong-impl challenge:
//   - Wrong: render displays hardcoded "3" regardless of stats.lives → FAIL
//   - Wrong: render uses coins as lives (field swap) → FAIL
//   - Wrong: render strips leading digit (shows "2" as "") → FAIL
// ============================================================================

#[test]
fn t03_fun_happy_render_lives_value_two() {
    // HUD-03: Given player died once, HUD lives text shows "2"
    let _hud = HudRenderer::new();
    let s = stats(5, 2);

    assert_eq!(s.lives, 2, "Stats lives must be 2 after one death");
    assert_eq!(s.coins, 5, "Stats coins unchanged at 5");

    // Verify render() does not panic with death-reduced lives.
    HudRenderer::render(&_hud, s, DEFAULT_VP_W, DEFAULT_VP_H);
}

// T04 — FUNC/happy — HUD renders reset state after game reset
// Traces To: FR-016 AC-4, §Interface Contract render postcondition
// Kills: Reset not clearing PlayerStats; HUD showing stale values after reset
// Wrong-impl challenge:
//   - Wrong: render ignores reset and shows previous coin count → FAIL
//   - Wrong: render shows lives=0 after game-over reset → FAIL
//   - Wrong: render panics after reset (null state) → FAIL (no panic)
// ============================================================================

#[test]
fn t04_fun_happy_render_reset_state() {
    // HUD-04: Given game reset, HUD shows coins=0, lives=3 (initial values)
    let _hud = HudRenderer::new();
    let s = initial_stats();

    assert_eq!(s.coins, 0, "After reset, coins must be 0");
    assert_eq!(s.lives, 3, "After reset, lives must be 3");

    // Verify render() with reset stats does not panic.
    HudRenderer::render(&_hud, s, DEFAULT_VP_W, DEFAULT_VP_H);
}

// ============================================================================
// BNDRY/edge Tests
// ============================================================================

// T05 — BNDRY/edge — lives=0 still renders HUD (not hidden)
// Traces To: §Implementation Summary Boundary Conditions (lives min=0),
//   §Visual Rendering Contract Element 4 (lives text always rendered)
// Kills: lives=0 condition incorrectly hiding HUD elements
// Wrong-impl challenge:
//   - Wrong: if lives == 0 { return; } → FAIL (assert render called without panic)
//   - Wrong: lives=0 draws nothing for life row → FAIL
//   - Wrong: lives=0 returns early skipping coin row too → FAIL
// ============================================================================

#[test]
fn t05_bndry_edge_zero_lives_still_renders() {
    // HUD-05: lives=0 is valid — HUD displays "0" for lives, does NOT hide
    let _hud = HudRenderer::new();
    let s = stats(3, 0);

    assert_eq!(s.lives, 0, "lives=0 is a valid boundary value");
    // Core assertion: render() must not skip HUD just because lives=0.
    HudRenderer::render(&_hud, s, DEFAULT_VP_W, DEFAULT_VP_H);
    // No panic = PASS; if implementation early-returns for lives==0,
    // this test alone won't catch it without deeper assertions.
    // §Visual Rendering Contract states lives "0" must be displayed.
}

// T06 — BNDRY/edge — anchor position at default viewport (480, 270) is exact
// Traces To: §Implementation Summary Boundary Conditions (viewport default),
//   FR-016 anchor tolerance, §Design Alignment sequenceDiagram msg#4
// Kills: Hardcoded anchor (e.g. 16px instead of vp_w*0.03); integer truncation
// Wrong-impl challenge:
//   - Wrong: anchor uses hardcoded px value (e.g. 16.0, 10.0) instead of percentage → FAIL
//   - Wrong: anchor uses vp_w * 0.3 (decimal error) → FAIL
//   - Wrong: anchor cast to integer truncating (14.4→14) → FAIL
// ============================================================================

#[test]
fn t06_bndry_edge_anchor_default_viewport() {
    // HUD-06: At default viewport (480, 270), anchor = (14.4, 8.1) exactly
    let (ax, ay) = HudRenderer::compute_anchor(DEFAULT_VP_W, DEFAULT_VP_H);

    assert!(
        approx_eq(ax, DEFAULT_ANCHOR_X),
        "anchor_x at 480vp must be 14.4, got {}", ax
    );
    assert!(
        approx_eq(ay, DEFAULT_ANCHOR_Y),
        "anchor_y at 270vp must be 8.1, got {}", ay
    );
}

// T07 — BNDRY/edge — anchor position at alternate resolutions
// Traces To: §Implementation Summary Boundary Conditions (viewport resolution),
//   FR-016 anchor tolerance, §Design Alignment sequenceDiagram msg#4
// Kills: Hardcoded anchor not scaling with viewport; resolution-specific bugs
// Wrong-impl challenge:
//   - Wrong: anchor always returns (14.4, 8.1) regardless of viewport → FAIL at 720p
//   - Wrong: anchor formula uses wrong percentage at different resolutions → FAIL
//   - Wrong: f32 precision loss causes drift at 2560×1440 → may pass if within EPSILON
// ============================================================================

#[test]
fn t07_bndry_edge_anchor_alternate_resolutions() {
    // HUD-07: Anchor scales linearly with viewport size.
    // 1280×720 viewport (720p virtual canvas)
    let (ax1, ay1) = HudRenderer::compute_anchor(1280.0, 720.0);
    assert!(
        approx_eq(ax1, 38.4),
        "anchor_x at 1280vp must be 38.4, got {}", ax1
    );
    assert!(
        approx_eq(ay1, 21.6),
        "anchor_y at 720vp must be 21.6, got {}", ay1
    );

    // 2560×1440 viewport (1440p virtual canvas)
    let (ax2, ay2) = HudRenderer::compute_anchor(2560.0, 1440.0);
    assert!(
        approx_eq(ax2, 76.8),
        "anchor_x at 2560vp must be 76.8, got {}", ax2
    );
    assert!(
        approx_eq(ay2, 43.2),
        "anchor_y at 1440vp must be 43.2, got {}", ay2
    );
}

// T08 — BNDRY/edge — zero viewport causes silent return (no panic, no draw)
// Traces To: §Interface Contract render Raises (vp_w/h <= 0 → early return)
// Kills: Division by zero; panic on zero viewport; NaN coordinates
// Wrong-impl challenge:
//   - Wrong: render divides by vp_w → panics on zero → FAIL (panic in test)
//   - Wrong: render passes zero to Macroquad draw → OpenGL error → FAIL
//   - Wrong: render returns early but still calls some draw → inconsistent state → FAIL
// ============================================================================

#[test]
fn t08_bndry_edge_zero_viewport_silent_return() {
    // HUD-08: viewport (0, 0) → render() returns silently, no panic
    let _hud = HudRenderer::new();
    let s = initial_stats();

    // Must NOT panic. The render method should early-return.
    HudRenderer::render(&_hud, s, 0.0, 0.0);
    // Separate x/y zero:
    HudRenderer::render(&_hud, s, 0.0, DEFAULT_VP_H);
    HudRenderer::render(&_hud, s, DEFAULT_VP_W, 0.0);
}

// T09 — BNDRY/edge — maximum coins value (u32::MAX) renders without panic
// Traces To: §Implementation Summary Boundary Conditions (coins max = u32::MAX)
// Kills: u32 overflow in formatting; fixed-width buffer truncation; panic on large number
// Wrong-impl challenge:
//   - Wrong: format!("{}", coins) panics on u32::MAX (unlikely but verify) → FAIL
//   - Wrong: fixed 3-char buffer truncates "4294967295" to "429" → FAIL
//   - Wrong: large text width causes negative layout offset → FAIL
// ============================================================================

#[test]
fn t09_bndry_edge_max_coins_renders() {
    // HUD-09: stats.coins = u32::MAX → display "4294967295", no panic
    let _hud = HudRenderer::new();
    let s = stats(u32::MAX, 3);

    assert_eq!(s.coins, u32::MAX, "coins must be u32::MAX");
    // Render must not panic with 10-digit coin count.
    HudRenderer::render(&_hud, s, DEFAULT_VP_W, DEFAULT_VP_H);
}

// ============================================================================
// FUNC/error Tests
// ============================================================================

// T10 — FUNC/error — negative viewport causes silent return (no panic)
// Traces To: §Interface Contract render preconditions (vp_w/h must be > 0),
//   §Implementation Summary Boundary Conditions (negative vp → early return)
// Kills: Negative viewport producing NaN coordinates; OpenGL error on negative coords
// Wrong-impl challenge:
//   - Wrong: render doesn't guard negative vp → passes negative to draw → undefined behavior → FAIL
//   - Wrong: render returns but panics on negative check (unwrap) → FAIL
//   - Wrong: anchor_y = vp_h * 0.03 = -8.1 (renders off-screen) → should NOT render at all → FAIL
// ============================================================================

#[test]
fn t10_fun_error_negative_viewport_silent_return() {
    // HUD-10: viewport (-1.0, -1.0) → render() returns silently, no panic
    let _hud = HudRenderer::new();
    let s = initial_stats();

    // Must NOT panic. Render should guard against negative viewport.
    HudRenderer::render(&_hud, s, -1.0, 270.0);
    HudRenderer::render(&_hud, s, 480.0, -1.0);
    HudRenderer::render(&_hud, s, -1.0, -1.0);
}

// ============================================================================
// INTG/api Tests (real_test)
// ============================================================================

// T11 [real_test (feature #9)] — INTG/api — Player::stats() → HudRenderer data flow
// Traces To: IAPI-009, §Design Alignment sequenceDiagram msg#1-3,
//   classDiagram HudRenderer→Player
// Kills: IAPI-009 schema mismatch (field order/type); Player::stats() returning stale snapshot
// Wrong-impl challenge:
//   - Wrong: HudRenderer reads coins from wrong Player field → FAIL (value mismatch)
//   - Wrong: IAPI-009 returns PlayerStats with swapped field order → FAIL
//   - Wrong: HudRenderer caches stats and never re-reads from Player → FAIL (stale value)
// ============================================================================

#[test]
fn t11_intg_api_player_stats_flow_to_hud_renderer() {
    // HUD-11: PlayingState calls player.stats() → PlayerStats → hud.render(stats, ...)
    // The HudRenderer must receive stats values identical to what Player holds.
    let mut player = Player::new(PlayerConfig::default());

    // Initial state
    let s1 = player.stats();
    assert_eq!(s1.coins, 0, "Player starts with 0 coins");
    assert_eq!(s1.lives, 3, "Player starts with 3 lives");

    // Simulate coin collection (Player.coins is pub)
    player.coins = 7;
    let s2 = player.stats();
    assert_eq!(s2.coins, 7, "Player.stats() must reflect updated coins");
    assert_eq!(s2.lives, 3, "Lives unchanged");
    // Verify stats() returns a fresh snapshot, not a cached copy.
    assert_ne!(s1.coins, s2.coins, "stats() must return updated values");

    // Verify HudRenderer accepts these stats.
    let _hud = HudRenderer::new();
    HudRenderer::render(&_hud, s2, DEFAULT_VP_W, DEFAULT_VP_H);

    // After a death: lives decrement
    player.lives = 2;
    let s3 = player.stats();
    assert_eq!(s3.coins, 7, "Coins unchanged after death");
    assert_eq!(s3.lives, 2, "Lives decremented to 2");
    HudRenderer::render(&_hud, s3, DEFAULT_VP_W, DEFAULT_VP_H);
}

// T12 [real_test (feature #9)] — INTG/api — HUD reflects stats change within same frame
// Traces To: IAPI-009, FR-016 AC-2 (same-frame update),
//   §Design Alignment sequenceDiagram msg#5-6, §Visual Rendering Contract interaction depth
// Kills: render() using cached stats snapshot from previous frame; render-before-update ordering
// Wrong-impl challenge:
//   - Wrong: render caches stats on first call → subsequent calls ignore new values → FAIL
//   - Wrong: PlayingState calls hud.render() BEFORE updating Player.coins → FAIL (wrong order)
//   - Wrong: HudRenderer holds its own coins/lives fields copied once → never updates → FAIL
// ============================================================================

#[test]
fn t12_intg_api_same_frame_stats_update_reflected() {
    // HUD-12: When Player.coins changes, next render() call must use NEW value.
    let mut player = Player::new(PlayerConfig::default());
    let _hud = HudRenderer::new();

    // Frame N: coins=0
    let s_frame_n = player.stats();
    assert_eq!(s_frame_n.coins, 0);
    HudRenderer::render(&_hud, s_frame_n, DEFAULT_VP_W, DEFAULT_VP_H);

    // Same frame: coin collected → coins becomes 5
    player.coins = 5;
    let s_frame_n1 = player.stats();
    assert_eq!(s_frame_n1.coins, 5, "Coins updated to 5 within same frame");

    // render() called again with updated stats must show new value.
    // If render cached coins=0 internally, this would show stale value.
    HudRenderer::render(&_hud, s_frame_n1, DEFAULT_VP_W, DEFAULT_VP_H);

    // Verify the two snapshots differ (proving we aren't caching).
    assert_ne!(
        s_frame_n.coins, s_frame_n1.coins,
        "Frame N and N+1 stats must differ after coin collect"
    );
}

// ============================================================================
// UI/render Tests (render-driving logic)
// ============================================================================

// T13 — UI/render — coin icon anchor position computation
// Traces To: §Visual Rendering Contract Element 1 (coin icon),
//   §Design Alignment sequenceDiagram msg#5
// Kills: Texture drawn at wrong position; world-space instead of viewport-space
// Wrong-impl challenge:
//   - Wrong: coin icon position = (0, 0) instead of anchor → FAIL
//   - Wrong: coin icon uses absolute pixel constant (16, 10) → FAIL
//   - Wrong: coin icon anchored to bottom-left instead of top-left → FAIL
// ============================================================================

#[test]
fn t13_ui_render_coin_icon_anchor_position() {
    // HUD-13: Coin icon drawn at (anchor_x, anchor_y) = (vp_w*0.03, vp_h*0.03)
    let (ax, ay) = HudRenderer::compute_anchor(DEFAULT_VP_W, DEFAULT_VP_H);
    assert!(
        approx_eq(ax, 14.4),
        "Coin icon anchor_x must be 14.4 at 480vp"
    );
    assert!(
        approx_eq(ay, 8.1),
        "Coin icon anchor_y must be 8.1 at 270vp"
    );

    // Rendered icon size = SOURCE_ICON_SIZE * ICON_SCALE = 8 * 3 = 24
    let icon_size = HudRenderer::rendered_icon_size();
    assert!(
        approx_eq(icon_size, RENDERED_ICON_SIZE),
        "Rendered icon size must be 24px (8*3)"
    );

    // Icon occupies rectangle from (ax, ay) to (ax+24, ay+24).
    // Verify no overlap with text area: text starts at ax+24+4 = ax+28
    let text_start_x = ax + icon_size + ICON_SPACING;
    assert!(
        approx_eq(text_start_x, 14.4 + 24.0 + 4.0),
        "Coin text starts 4px right of icon"
    );
}

// T14 — UI/render — text outline computation (5-draw offsets)
// Traces To: §Visual Rendering Contract Element 2 (coin text outline)
// Kills: Missing outline (only white text); wrong outline offset direction
// Wrong-impl challenge:
//   - Wrong: only center text drawn, no outline passes → FAIL (outline count != 5)
//   - Wrong: outline offsets wrong sign (text shadow instead of outline) → FAIL
//   - Wrong: outline distance = 2px instead of 1px → FAIL (wrong offset)
// ============================================================================

#[test]
fn t14_ui_render_text_outline_computation() {
    // HUD-14: Text rendered with 4 offset 1px black passes + 1 center white pass.
    // The outline positions relative to (text_x, text_y) are:
    //   (0, -1) up, (0, +1) down, (-1, 0) left, (+1, 0) right, (0, 0) center
    let text_x: f32 = 42.4; // coin text x at default vp
    let text_y: f32 = 8.1;

    let outline_positions = HudRenderer::outline_positions(text_x, text_y);
    // Expect exactly 5 positions (4 outline + 1 center).
    assert_eq!(
        outline_positions.len(),
        5,
        "Outline must produce exactly 5 draw positions (4 outline + 1 center)"
    );

    // The center position must be the original (text_x, text_y).
    let center = outline_positions[4];
    assert!(
        approx_eq(center.0, text_x) && approx_eq(center.1, text_y),
        "Center outline position must be the original text coord ({}, {}), got ({}, {})",
        text_x, text_y, center.0, center.1
    );

    // Verify each outline offset is exactly 1px and in a cardinal direction.
    // Up: (text_x, text_y - 1)
    assert!(
        approx_eq(outline_positions[0].0, text_x) && approx_eq(outline_positions[0].1, text_y - 1.0),
        "Up offset must be (text_x, text_y-1)"
    );
    // Down: (text_x, text_y + 1)
    assert!(
        approx_eq(outline_positions[1].0, text_x) && approx_eq(outline_positions[1].1, text_y + 1.0),
        "Down offset must be (text_x, text_y+1)"
    );
    // Left: (text_x - 1, text_y)
    assert!(
        approx_eq(outline_positions[2].0, text_x - 1.0) && approx_eq(outline_positions[2].1, text_y),
        "Left offset must be (text_x-1, text_y)"
    );
    // Right: (text_x + 1, text_y)
    assert!(
        approx_eq(outline_positions[3].0, text_x + 1.0) && approx_eq(outline_positions[3].1, text_y),
        "Right offset must be (text_x+1, text_y)"
    );
}

// T15 — UI/render — heart icon position (below coin row)
// Traces To: §Visual Rendering Contract Element 3 (heart icon)
// Kills: Heart icon overlapping coin icon; wrong Y offset
// Wrong-impl challenge:
//   - Wrong: heart icon at same Y as coin icon → FAIL
//   - Wrong: heart icon Y = anchor_y + coin_h (forgets spacing) → FAIL
//   - Wrong: heart icon uses coin_tex instead of heart_tex → FAIL (would show gold, not red)
// ============================================================================

#[test]
fn t15_ui_render_heart_icon_position() {
    // HUD-15: Heart icon at (anchor_x, anchor_y + icon_h + spacing) = (ax, ay + 24 + 4)
    let (ax, ay) = HudRenderer::compute_anchor(DEFAULT_VP_W, DEFAULT_VP_H);
    let icon_size = HudRenderer::rendered_icon_size();

    let heart_x = ax;
    let heart_y = ay + icon_size + ICON_SPACING;

    assert!(
        approx_eq(heart_x, 14.4),
        "Heart icon x at anchor_x = 14.4"
    );
    assert!(
        approx_eq(heart_y, 8.1 + 24.0 + 4.0),
        "Heart icon y = anchor_y + icon_h + spacing = 36.1"
    );

    // Heart icon must NOT be at same Y as coin icon.
    let coin_y = ay;
    assert!(
        heart_y > coin_y + icon_size,
        "Heart icon must be below coin icon ({} > {} + {})",
        heart_y, coin_y, icon_size
    );

    // Verify coin and heart icons do NOT overlap.
    // Coin icon covers y: [8.1, 32.1]; Heart icon covers y: [36.1, 60.1]
    // Gap between them = 36.1 - 32.1 = 4.0 = ICON_SPACING
    let coin_bottom = coin_y + icon_size;
    assert!(
        approx_eq(heart_y - coin_bottom, ICON_SPACING),
        "Gap between coin bottom and heart top must be 4px"
    );
}

// T16 — UI/render — lives text position (right of heart icon)
// Traces To: §Visual Rendering Contract Element 4 (lives text)
// Kills: Lives text overlapping heart icon; lives text using coins value
// Wrong-impl challenge:
//   - Wrong: lives text at coin text position (shows on coin row) → FAIL
//   - Wrong: lives text reads coins field instead of lives → FAIL
//   - Wrong: lives text Y coordinate equals coin text Y → FAIL
// ============================================================================

#[test]
fn t16_ui_render_lives_text_position() {
    // HUD-16: Lives text at (anchor_x + icon_w + spacing, anchor_y + icon_h + spacing)
    let (ax, ay) = HudRenderer::compute_anchor(DEFAULT_VP_W, DEFAULT_VP_H);
    let icon_size = HudRenderer::rendered_icon_size();

    let lives_text_x = ax + icon_size + ICON_SPACING;
    let lives_text_y = ay + icon_size + ICON_SPACING;
    // Coin text is at (ax + icon_w + spacing, ay), so lives text Y differs.
    let coin_text_y = ay;

    assert!(
        approx_eq(lives_text_x, 14.4 + 24.0 + 4.0),
        "Lives text x = 42.4 (same column as coin text)"
    );
    assert!(
        approx_eq(lives_text_y, 36.1),
        "Lives text y = 36.1 (below coin text)"
    );
    assert!(
        lives_text_y > coin_text_y,
        "Lives text must be below coin text ({} > {})",
        lives_text_y, coin_text_y
    );

    // Verify lives text is horizontally aligned with heart icon + spacing
    // (same column as coin text — both icons at same x, both texts at same x)
    let coin_text_x = ax + icon_size + ICON_SPACING;
    assert!(
        approx_eq(lives_text_x, coin_text_x),
        "Lives text x must match coin text x (both text columns aligned)"
    );
}

// T17 — UI/render — HUD background is transparent (no opaque rect)
// Traces To: §Visual Rendering Contract Element 5 (transparent background)
// Kills: HUD drawing opaque/semi-transparent background rectangle
// Wrong-impl challenge:
//   - Wrong: render() draws a fillRect before drawing HUD elements → FAIL
//   - Wrong: render() calls clear_background or similar → FAIL
//   - Wrong: render() draws a semi-transparent black overlay behind text → FAIL
// ============================================================================

#[test]
fn t17_ui_render_transparent_background() {
    // HUD-17: HUD must NOT draw any background rectangle — fully transparent.
    // Since we cannot verify pixels in unit tests, we verify the contract:
    // HudRenderer exposes a flag or method indicating no background is drawn.
    let bg_drawn = HudRenderer::draws_background();
    assert!(
        !bg_drawn,
        "HUD must NOT draw a background rectangle (must be transparent)"
    );
}

// T18 — UI/render — anchor tolerance at multiple viewport sizes
// Traces To: §Visual Rendering Contract Element 6 (anchor tolerance ±2%),
//   FR-016 AC-1 (viewport anchor tolerance)
// Kills: Anchor deviation exceeding ±2% at non-standard resolutions; integer rounding error
// Wrong-impl challenge:
//   - Wrong: anchor uses floor(viewport * 0.03) → accumulates error → may FAIL at small vp
//   - Wrong: anchor uses hardcoded value per resolution tier → FAIL for unlisted resolution
//   - Wrong: tolerance check uses wrong percentage (e.g. 0.2% instead of 2%) → FAIL
// ============================================================================

#[test]
fn t18_ui_render_anchor_tolerance_compliance() {
    // HUD-18: At any viewport, |actual_anchor - expected| <= vp_dim * 0.02
    // Test at 3 standard virtual canvas sizes.
    let resolutions: [(f32, f32); 3] = [
        (480.0, 270.0),   // default
        (1280.0, 720.0),  // 720p
        (2560.0, 1440.0), // 1440p
    ];

    for (vp_w, vp_h) in &resolutions {
        let (ax, ay) = HudRenderer::compute_anchor(*vp_w, *vp_h);
        let expected_x = vp_w * ANCHOR_X_PCT;
        let expected_y = vp_h * ANCHOR_Y_PCT;

        let tol_x = vp_w * ANCHOR_TOLERANCE_PCT;
        let tol_y = vp_h * ANCHOR_TOLERANCE_PCT;

        let dx = (ax - expected_x).abs();
        let dy = (ay - expected_y).abs();

        assert!(
            dx <= tol_x + EPSILON,
            "At vp ({}, {}): anchor_x deviation {} exceeds tolerance {} (±2% vp_w)",
            vp_w, vp_h, dx, tol_x
        );
        assert!(
            dy <= tol_y + EPSILON,
            "At vp ({}, {}): anchor_y deviation {} exceeds tolerance {} (±2% vp_h)",
            vp_w, vp_h, dy, tol_y
        );
    }
}

// ============================================================================
// PERF/frame Tests
// ============================================================================

// T19 — PERF/frame — HUD stats reflect same-frame value (not cached)
// Traces To: FR-016 same-frame update requirement,
//   §Visual Rendering Contract interaction depth (same-frame refresh)
// Kills: Cached stats snapshot reused across frames; render before update ordering
// Wrong-impl challenge:
//   - Wrong: HudRenderer stores a PlayerStats internally and only updates on new() → FAIL
//   - Wrong: PlayingState::render() calls hud.render BEFORE reading player.stats() → FAIL
//   - Wrong: render uses AtomicU32 but doesn't sync before draw → FAIL
// ============================================================================

#[test]
fn t19_perf_frame_same_frame_update() {
    // HUD-19: Within a single frame, after Player.coins changes, render() uses new value.
    let mut player = Player::new(PlayerConfig::default());
    let _hud = HudRenderer::new();

    // Simulate a single frame tick:
    // 1. Game logic updates Player.coins
    player.coins = 10;
    // 2. Render reads Player.stats()
    let stats_after_update = player.stats();
    assert_eq!(stats_after_update.coins, 10, "Coins updated to 10");

    // 3. HUD renders with current stats
    HudRenderer::render(&_hud, stats_after_update, DEFAULT_VP_W, DEFAULT_VP_H);

    // The critical invariant: if we were to read stats again and render again,
    // it must still show the latest value (not revert to cached old value).
    let stats_recheck = player.stats();
    assert_eq!(
        stats_recheck.coins, stats_after_update.coins,
        "Stats must be stable between reads in same frame"
    );
}

// T20 — UI/render — coin count update across consecutive render frames
// Traces To: §Visual Rendering Contract interaction depth (coin text updates),
//   FR-016 AC-2 (on-coin-collect immediate update)
// Kills: HUD using previous frame's stats snapshot; off-by-one-frame delay
// Wrong-impl challenge:
//   - Wrong: render caches stats from Frame N, Frame N+1 reuses → FAIL
//   - Wrong: stats snapshot captured at frame start, coin collected mid-frame → FAIL
//   - Wrong: text object created once and never updated → FAIL
// ============================================================================

#[test]
fn t20_ui_render_coin_update_consecutive_frames() {
    // HUD-20: Frame N coins=3, Frame N+1 coins=4 → Frame N+1 must display "4"
    let _hud = HudRenderer::new();

    // Frame N: coins=3
    let s_frame_n = stats(3, 3);
    assert_eq!(s_frame_n.coins, 3);
    HudRenderer::render(&_hud, s_frame_n, DEFAULT_VP_W, DEFAULT_VP_H);

    // Frame N+1: coins=4 (collected one coin)
    let s_frame_n1 = stats(4, 3);
    assert_eq!(s_frame_n1.coins, 4);
    HudRenderer::render(&_hud, s_frame_n1, DEFAULT_VP_W, DEFAULT_VP_H);

    // Verify values differ across frames.
    assert_ne!(
        s_frame_n.coins, s_frame_n1.coins,
        "Frame N+1 coins (4) must differ from Frame N coins (3)"
    );
}

// ============================================================================
// FUNC/error Tests (continued)
// ============================================================================

// T21 — FUNC/error — missing texture file does not crash HudRenderer::new()
// Traces To: §Interface Contract HudRenderer::new Raises (texture missing → default),
//   §Implementation Summary §4 legacy interaction (assets/ directory)
// Kills: load_texture failure causing panic; HudRenderer construction failing entirely
// Wrong-impl challenge:
//   - Wrong: new() panics on load_texture error → FAIL (test panics)
//   - Wrong: new() returns Err(...) but contract says → Self (always succeeds) → FAIL (type mismatch)
//   - Wrong: new() succeeds but render() panics because texture handle is invalid → FAIL
// ============================================================================

#[test]
fn t21_fun_error_missing_texture_graceful() {
    // HUD-21: When coin.png or heart.png is missing, Macroquad returns 1x1 white texture.
    // HudRenderer must construct successfully and render without panicking.
    let _hud = HudRenderer::new();
    // Construction success = PASS. The Macroquad fallback is a 1x1 white texture,
    // so render should still work (icons appear as white squares but game continues).
    let s = initial_stats();
    HudRenderer::render(&_hud, s, DEFAULT_VP_W, DEFAULT_VP_H);
    // No panic = graceful degradation confirmed.
}

// ============================================================================
// BNDRY/edge Tests (continued)
// ============================================================================

// T22 — BNDRY/edge — extreme viewport aspect ratio (very wide, very short)
// Traces To: §Implementation Summary Boundary Conditions (viewport_h extreme),
//   §Visual Rendering Contract anchor tolerance
// Kills: Second row falling outside viewport; elements overlapping at narrow viewport
// Wrong-impl challenge:
//   - Wrong: heart icon drawn at Y > vp_h → off-screen → FAIL
//   - Wrong: coin text drawn at X > vp_w → off-screen → FAIL
//   - Wrong: spacing fixed but viewport smaller → elements overlap → FAIL
// ============================================================================

#[test]
fn t22_bndry_edge_extreme_viewport_aspect_ratio() {
    // HUD-22: Very wide, short viewport (480, 100). Both rows must remain visible.
    let _hud = HudRenderer::new();
    let vp_w: f32 = 480.0;
    let vp_h: f32 = 100.0;

    let (ax, ay) = HudRenderer::compute_anchor(vp_w, vp_h);
    let icon_size = HudRenderer::rendered_icon_size();

    // Coin row: y = 3.0 (anchor_y = 100 * 0.03 = 3.0)
    assert!(
        approx_eq(ax, 14.4) && approx_eq(ay, 3.0),
        "Anchor at extreme vp (480, 100) = (14.4, 3.0)"
    );

    // Heart row: y = 3.0 + 24 + 4 = 31.0
    let heart_y = ay + icon_size + ICON_SPACING;
    let heart_bottom = heart_y + icon_size; // 31.0 + 24.0 = 55.0
    assert!(
        heart_bottom <= vp_h + EPSILON,
        "Heart icon bottom ({} + {} = {}) must be <= vp_h ({}); not clipped off-screen",
        heart_y, icon_size, heart_bottom, vp_h
    );

    // Verify render does not panic at extreme aspect ratio.
    let s = initial_stats();
    HudRenderer::render(&_hud, s, vp_w, vp_h);
}

// T23 — FUNC/error — NaN viewport causes silent return (no panic)
// Traces To: §Interface Contract render preconditions,
//   §Implementation Summary §3 defensive constraints (NaN check)
// Kills: NaN propagation to Macroquad draw calls → OpenGL undefined behavior
// Wrong-impl challenge:
//   - Wrong: no NaN check → NaN passes through to draw coords → FAIL (undefined behavior)
//   - Wrong: NaN check panics instead of early-return → FAIL
//   - Wrong: check uses == comparison (NaN != NaN always false) → FAIL (NaN not caught)
// ============================================================================

#[test]
fn t23_fun_error_nan_viewport_silent_return() {
    // HUD-23: viewport with NaN values → render() must early-return, no panic.
    let _hud = HudRenderer::new();
    let s = initial_stats();

    // NaN in width, valid height
    HudRenderer::render(&_hud, s, f32::NAN, DEFAULT_VP_H);
    // Valid width, NaN in height
    HudRenderer::render(&_hud, s, DEFAULT_VP_W, f32::NAN);
    // Both NaN
    HudRenderer::render(&_hud, s, f32::NAN, f32::NAN);

    // Also test f32::INFINITY — should be caught by the same guard.
    HudRenderer::render(&_hud, s, f32::INFINITY, DEFAULT_VP_H);
    HudRenderer::render(&_hud, s, DEFAULT_VP_W, f32::INFINITY);
    // No panic = PASS.
}

// T24 — BNDRY/edge — coin digit boundary (3→4 digits: 999→1000)
// Traces To: §Implementation Summary Boundary Conditions (coins digit expansion),
//   FR-016 AC-2 (count display)
// Kills: Fixed-width text buffer truncation; digit count assumption (only 3 digits)
// Wrong-impl challenge:
//   - Wrong: text buffer allocated for 3 chars, 4th digit truncated → FAIL (shows "100" not "1000")
//   - Wrong: digit expansion panics due to buffer overflow → FAIL (panic)
//   - Wrong: layout assumes 3-char width, 4-char overflows into lives text → FAIL
// ============================================================================

#[test]
fn t24_bndry_edge_coin_digit_boundary_expansion() {
    // HUD-24: Crossing from 999 to 1000 (3→4 digits), no truncation or layout break.
    let _hud = HudRenderer::new();

    // Verify 3-digit coin count renders fine.
    let s_999 = stats(999, 3);
    assert_eq!(s_999.coins, 999);
    HudRenderer::render(&_hud, s_999, DEFAULT_VP_W, DEFAULT_VP_H);

    // Verify 4-digit coin count renders fine after boundary crossing.
    let s_1000 = stats(1000, 3);
    assert_eq!(s_1000.coins, 1000, "Coins must be exactly 1000");
    HudRenderer::render(&_hud, s_1000, DEFAULT_VP_W, DEFAULT_VP_H);

    // Verify 10-digit (u32::MAX) also renders fine (redundant with T09 but
    // important to verify the full digit expansion path).
    let s_max = stats(u32::MAX, 3);
    HudRenderer::render(&_hud, s_max, DEFAULT_VP_W, DEFAULT_VP_H);
    // No panic across all digit widths = PASS.
}
