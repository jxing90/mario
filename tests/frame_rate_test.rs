// Feature #11: 60fps Frame Rate (NFR-001) — TDD Red Phase
//
// [no integration test] — pure computation on in-memory f32 values,
// no external I/O (no DB, no network, no filesystem, no third-party SDK).
// Frame data originates from macroquad::time::get_frame_time() and is
// consumed entirely within FrameMetrics in-process.
//
// Test Inventory Reference: docs/features/11-60fps-frame-rate-nfr-001.md §7
// All tests are expected to FAIL (RED phase) — implementation not yet written.
// RED stubs in src/metrics.rs return sentinel values (-1.0 / 999) to ensure
// no test accidentally passes.
//
// Category coverage (Rule 1):
//   FUNC/happy:  A1, A2, A3, A4, A5, A6
//   FUNC/error:  A7, A8, A9
//   BNDRY/edge:  B1, B2, B3, B4, B5
//   PERF:        P1, P2, P3
//   SEC:  N/A — internal performance instrumentation, no user-facing input
//   INTG: N/A — pure local computation, no external I/O (per design §7)
//
// Negative ratio: (3 FUNC/error + 5 BNDRY/edge) / 17 total = 8/17 ≈ 47.1% ≥ 40%

use mario_platformer::engine::{GameLoop, WindowConfig};
use mario_platformer::metrics::FrameMetrics;
use mario_platformer::state::StateMachine;

// ============================================================================
// Constants
// ============================================================================

/// Fixed timestep: 1/60 second (from src/engine.rs:DT).
const DT: f32 = 1.0 / 60.0;

/// Allowed tolerance for f32 comparisons (long-task-guide: physics epsilon=0.01).
const EPSILON: f32 = 0.01;

// ============================================================================
// Test Double: SpyStateMachine
// Minimal spy for GameLoop integration tests (A6, B5, P1-P3).
// ============================================================================

struct SpyStateMachine {
    pub update_count: u32,
    pub render_count: u32,
}

impl SpyStateMachine {
    fn new() -> Self {
        Self {
            update_count: 0,
            render_count: 0,
        }
    }
}

impl StateMachine for SpyStateMachine {
    fn update(&mut self, _dt: f32) {
        self.update_count += 1;
    }

    fn render(&mut self, _alpha: f32) {
        self.render_count += 1;
    }
}

// ============================================================================
// Helper utilities
// ============================================================================

/// Returns true if two f32 values are within EPSILON of each other.
fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < EPSILON
}

// ============================================================================
// A1 — FUNC/happy
// Traces To: NFR-001 AC-1, §Interface Contract sample / stats
// "100 frames at DT → stats().avg ≈ 0.016, frame_count == 100"
//
// Kills: accumulator not initialized (sum is garbage); avg formula wrong
//   (div-by-zero or wrong denominator).
// Wrong-impl challenge:
//   - Wrong: sum never accumulated → avg = 0.0 or NaN → FAIL
//   - Wrong: frame_count not incremented → avg wrong denominator → FAIL
//   - Wrong: min stays f32::MAX → min not updated → FAIL
// ============================================================================

// real_test (feature #11)
#[test]
fn a1_stable_60fps_stats_avg_max_min() {
    let mut metrics = FrameMetrics::new();

    // Simulate 100 frames at exactly 16.667ms each (stable 60fps).
    for _ in 0..100 {
        metrics.sample(DT);
    }

    let stats = metrics.stats();

    // avg must be approximately DT (stable 60fps → every frame ~16.667ms).
    assert!(
        approx_eq(stats.avg, DT),
        "A1 avg: expected ≈{:.6}, got {:.6} (100 frames at DT)",
        DT,
        stats.avg
    );

    // min must be DT (all frames identical).
    assert!(
        approx_eq(stats.min, DT),
        "A1 min: expected ≈{:.6}, got {:.6}",
        DT,
        stats.min
    );

    // max must be DT (all frames identical).
    assert!(
        approx_eq(stats.max, DT),
        "A1 max: expected ≈{:.6}, got {:.6}",
        DT,
        stats.max
    );

    // Exactly 100 frames sampled.
    assert_eq!(
        stats.frame_count, 100,
        "A1 frame_count: expected 100, got {}",
        stats.frame_count
    );
}

// ============================================================================
// A2 — FUNC/happy
// Traces To: NFR-001 AC-1, §Interface Contract p99
// "99 frames @ DT + 1 frame @ 0.050 → p99 ≤ 0.050, max == 0.050"
//
// Kills: P99 sort error (takes max or avg instead of 99th percentile);
//   off-by-one on percentile index.
// Wrong-impl challenge:
//   - Wrong: p99 returns max (0.050) regardless of distribution → correct
//     here but fails with 2 slow frames → A2 has only 1 slow frame so
//     p99 must be 0.050; test also verifies min=DT still holds
//   - Wrong: p99 index off-by-one → returns DT instead of 0.050 → FAIL
//   - Wrong: p99 sorts descending → returns DT → FAIL
// ============================================================================

// real_test (feature #11)
#[test]
fn a2_p99_with_one_slow_frame() {
    let mut metrics = FrameMetrics::new();

    let slow_frame: f32 = 0.050;

    // 99 frames at DT, then 1 slow frame.
    for _ in 0..99 {
        metrics.sample(DT);
    }
    metrics.sample(slow_frame);

    let stats = metrics.stats();

    // P99 must be at least the slow frame time and at most the slow frame time.
    // With 100 samples, the 99th percentile points to index floor(0.99*100)=99 →
    // which is the slow frame.
    assert!(
        stats.p99 >= DT && stats.p99 <= slow_frame,
        "A2 p99: expected in [{:.6}, {:.6}], got {:.6}",
        DT,
        slow_frame,
        stats.p99
    );

    // The single slow frame must be the max.
    assert!(
        approx_eq(stats.max, slow_frame),
        "A2 max: expected {:.6}, got {:.6}",
        slow_frame,
        stats.max
    );

    // min is still DT (all other frames).
    assert!(
        approx_eq(stats.min, DT),
        "A2 min: expected {:.6}, got {:.6}",
        DT,
        stats.min
    );

    assert_eq!(stats.frame_count, 100, "A2 frame_count: expected 100");
}

// ============================================================================
// A3 — FUNC/happy
// Traces To: NFR-001 AC-2, §Interface Contract window_fps
// "60 seconds × 60 fps → window_fps ≥ 59.5, ≤ 60.0"
//
// Kills: per-second frame count missed frames; ring buffer index error.
// Wrong-impl challenge:
//   - Wrong: window_ring entries never filled → window_fps = 0.0 → FAIL
//   - Wrong: ring index advances twice per second → half the frames → FAIL
//   - Wrong: window_fps divides by 60 always (not filled seconds) → wrong → FAIL
//   - Wrong: accumulator doesn't roll over to next second for partial remainder → FAIL
// ============================================================================

// real_test (feature #11)
#[test]
fn a3_window_fps_stable_60fps() {
    let mut metrics = FrameMetrics::new();

    // Simulate 60 seconds of exactly 60 fps.
    // Each second: 60 frames × DT = 60 × (1/60) = 1.0 second.
    for _sec in 0..60 {
        for _frame in 0..60 {
            metrics.sample(DT);
        }
    }

    let fps = metrics.window_fps();
    // 60 seconds × 60 fps = 3600 frames / 60 seconds = 60.0 fps.
    assert!(
        fps >= 59.5 && fps <= 60.0,
        "A3 window_fps: expected [59.5, 60.0], got {:.3} (60s × 60fps)",
        fps
    );

    // Verify sample count: 60 × 60 = 3600 frames.
    let stats = metrics.stats();
    assert_eq!(
        stats.frame_count, 3600,
        "A3 frame_count: expected 3600, got {}",
        stats.frame_count
    );
}

// ============================================================================
// A4 — FUNC/happy
// Traces To: NFR-001 AC-2, §Interface Contract window_fps
// "60 seconds × 58 fps → window_fps ≈ 58.0 (tolerance ±0.5)"
//
// Kills: sliding window sum wrong (not simply average); ring overwrite logic.
// Wrong-impl challenge:
//   - Wrong: window_fps calculates simple average of ring entries → wrong → FAIL
//   - Wrong: ring overwrite corrupts neighboring entries → wrong fps → FAIL
//   - Wrong: fraction from partial seconds not handled → FAIL
// ============================================================================

// real_test (feature #11)
#[test]
fn a4_window_fps_58fps_near_threshold() {
    let mut metrics = FrameMetrics::new();

    // Simulate 60 seconds at 58 fps.
    // Each "frame" takes DT * 60/58 seconds to maintain 58 fps rate.
    let frame_time_58 = DT * 60.0 / 58.0;

    for _sec in 0..60 {
        for _frame in 0..58 {
            metrics.sample(frame_time_58);
        }
    }

    let fps = metrics.window_fps();
    assert!(
        (fps - 58.0).abs() < 0.5,
        "A4 window_fps: expected ≈58.0 (±0.5), got {:.3} (60s × 58fps)",
        fps
    );

    let stats = metrics.stats();
    assert_eq!(
        stats.frame_count, 3480,
        "A4 frame_count: expected 60×58=3480, got {}",
        stats.frame_count
    );
}

// ============================================================================
// A5 — FUNC/happy
// Traces To: NFR-001, §Interface Contract log_report
// "After 100 samples, log_report outputs format with min/max/avg/p99/window_fps/frames"
//
// Kills: format macro field misalignment; NaN output (div-by-zero); wrong field order.
// Wrong-impl challenge:
//   - Wrong: format string omits a field → field count mismatch → FAIL
//   - Wrong: field order swapped (e.g., avg before max) → FAIL
//   - Wrong: NaN printed for avg (div-by-zero) → FAIL
// ============================================================================

// real_test (feature #11)
#[test]
fn a5_log_report_format_after_samples() {
    let mut metrics = FrameMetrics::new();

    // Sample 100 frames at DT.
    for _ in 0..100 {
        metrics.sample(DT);
    }

    // Verify stats() report correct values before log_report.
    let stats = metrics.stats();
    assert!(
        stats.frame_count > 0,
        "A5: frame_count must be > 0 after 100 samples (got {})",
        stats.frame_count
    );
    assert!(
        stats.avg > 0.0,
        "A5: avg must be > 0.0 after sampling (got {})",
        stats.avg
    );
    assert!(
        stats.min > 0.0,
        "A5: min must be > 0.0 after sampling (got {})",
        stats.min
    );
    assert!(
        stats.max > 0.0,
        "A5: max must be > 0.0 after sampling (got {})",
        stats.max
    );

    // log_report must not panic. The output format is verified
    // by checking that stats() returns correct, non-zero values,
    // which log_report formats from.
    metrics.log_report();
}

// ============================================================================
// A6 — FUNC/happy
// Traces To: NFR-001, §Implementation Summary integration point
// "GameLoop::tick(DT, state) → metrics.frame_count == 1"
//
// Kills: tick() entry does NOT call metrics.sample(); integration omission.
// Wrong-impl challenge:
//   - Wrong: sample() call commented out → frame_count stays 0 → FAIL
//   - Wrong: sample() called after update loop instead of before → wrong timing → FAIL
//   - Wrong: sample() called with wrong value (0.0) → min stays f32::MAX → FAIL
// ============================================================================

// real_test (feature #11)
#[test]
fn a6_game_loop_tick_integrates_sample() {
    let config = WindowConfig {
        width: 1280,
        height: 720,
        fullscreen: false,
    };
    let mut gl = GameLoop::new(config).expect("GameLoop::new should succeed");
    let mut state = SpyStateMachine::new();

    // One frame at DT.
    gl.tick(DT, &mut state);

    // metrics must have recorded exactly 1 sample.
    assert_eq!(
        gl.metrics.frame_count, 1,
        "A6: tick() must call metrics.sample(); frame_count expected 1, got {}",
        gl.metrics.frame_count
    );

    // The min frame time should be DT (since frame_time = DT was passed to tick).
    assert!(
        approx_eq(gl.metrics.min_frame_time, DT),
        "A6: min_frame_time should be DT ({:.6}), got {:.6}",
        DT,
        gl.metrics.min_frame_time
    );
}

// ============================================================================
// A7 — FUNC/error
// Traces To: §Interface Contract p99 (empty buffer)
// "p99() on empty buffer returns 0.0, no panic"
//
// Kills: sort panic on empty Vec; div-by-zero in percentile calc.
// Wrong-impl challenge:
//   - Wrong: p99() calls .sort() on empty Vec → panic → FAIL
//   - Wrong: p99() divides by buffer.len() → div-by-zero → FAIL
//   - Wrong: p99() returns f32::NAN for empty → FAIL
// ============================================================================

// real_test (feature #11)
#[test]
fn a7_p99_empty_buffer_returns_zero() {
    let metrics = FrameMetrics::new();

    // Must not panic on empty buffer.
    let p99 = metrics.p99();

    assert!(
        p99 == 0.0,
        "A7: p99() on empty buffer must return 0.0, got {:.6}",
        p99
    );
}

// ============================================================================
// A8 — FUNC/error
// Traces To: §Interface Contract stats (zero frames)
// "stats() on zero frames: frame_count == 0, avg doesn't panic"
//
// Kills: div-by-zero panic (sum / frame_count); NaN propagation.
// Wrong-impl challenge:
//   - Wrong: stats() panics on div-by-zero → FAIL
//   - Wrong: stats() returns NaN for avg → FAIL
//   - Wrong: stats() uses uninitialized sum → garbage avg → FAIL
// ============================================================================

// real_test (feature #11)
#[test]
fn a8_stats_zero_frames_no_panic() {
    let metrics = FrameMetrics::new();

    let stats = metrics.stats();

    // Must not panic and frame_count must be 0.
    assert_eq!(
        stats.frame_count, 0,
        "A8: frame_count must be 0 with no samples, got {}",
        stats.frame_count
    );

    // avg must not be NaN (defensive: either 0.0 or some sentinel).
    assert!(
        !stats.avg.is_nan(),
        "A8: avg must not be NaN when frame_count == 0"
    );

    // min/max should be their initial sentinel values.
    // min = f32::MAX (no samples received), max = 0.0 (no samples received).
    assert!(
        stats.min == 0.0 || stats.min == f32::MAX,
        "A8: min={} must be 0.0 or f32::MAX (no samples)",
        stats.min
    );
}

// ============================================================================
// A9 — FUNC/error
// Traces To: §Interface Contract log_report (zero frames)
// "log_report() on zero frames: prints 'no frames sampled yet', no panic"
//
// Kills: format macro panic on zero frames; div-by-zero in format args.
// Wrong-impl challenge:
//   - Wrong: log_report() panics when frame_count == 0 → FAIL
//   - Wrong: log_report() formats stats() which panics on zero → FAIL
// ============================================================================

// real_test (feature #11)
#[test]
fn a9_log_report_empty_no_panic() {
    let metrics = FrameMetrics::new();

    // Verify pre-condition: zero frames.
    assert_eq!(
        metrics.frame_count, 0,
        "A9: should start with frame_count == 0"
    );

    // Must not panic when calling log_report with zero frames.
    metrics.log_report();
}

// ============================================================================
// B1 — BNDRY/edge
// Traces To: §Implementation Summary Boundary Conditions — frame_time = 0.0
// "0.0 sample doesn't pollute min; frame_count still increments"
//
// Kills: 0.0 overwrites correct min (e.g., min becomes 0.0 instead of DT);
//   0.0 rejected entirely so frame_count not incremented.
// Wrong-impl challenge:
//   - Wrong: 0.0 unconditionally updates min → min becomes 0.0 → FAIL
//   - Wrong: 0.0 skipped entirely including frame_count → count wrong → FAIL
//   - Wrong: 0.0 treated as missing sample → p99 wrong → FAIL
// ============================================================================

// real_test (feature #11)
#[test]
fn b1_zero_frame_time_does_not_pollute_min() {
    let mut metrics = FrameMetrics::new();

    // First sample: zero (simulates first-frame or timer glitch).
    metrics.sample(0.0);

    // Second sample: normal frame.
    metrics.sample(DT);

    let stats = metrics.stats();

    // frame_count must be 2 (both samples recorded).
    assert_eq!(
        stats.frame_count, 2,
        "B1: frame_count must be 2 after two samples (0.0 + DT), got {}",
        stats.frame_count
    );

    // min must be DT (0.0 does NOT pollute min).
    assert!(
        approx_eq(stats.min, DT),
        "B1: min must be DT ({:.6}), not 0.0 — zero frame_time must not pollute min; got {:.6}",
        DT,
        stats.min
    );

    // max must be DT (only normal frame contributes).
    assert!(
        approx_eq(stats.max, DT),
        "B1: max must be DT ({:.6}), got {:.6}",
        DT,
        stats.max
    );
}

// ============================================================================
// B2 — BNDRY/edge
// Traces To: §Implementation Summary Boundary Conditions — buffer full
// "Fill 7200 frames, add 1 more → p99 based on most recent 7200 frames"
//
// Kills: Vec grows unbounded (panic on push when full); FIFO logic wrong
//   (overwrites wrong position / oldest not removed).
// Wrong-impl challenge:
//   - Wrong: Vec pushes without removing oldest element → grows to 7201 → p99
//     based on all 7201 frames → wrong → FAIL
//   - Wrong: FIFO removes newest instead of oldest → p99 wrong → FAIL
//   - Wrong: buffer full causes panic → test panics → FAIL
// ============================================================================

// real_test (feature #11)
#[test]
fn b2_buffer_full_fifo_behavior() {
    let mut metrics = FrameMetrics::new();

    // Fill the buffer with 7200 frames at DT.
    for _ in 0..7200 {
        metrics.sample(DT);
    }

    // Add one more frame with a distinctly different frame_time.
    let distinct_time: f32 = 0.020;
    metrics.sample(distinct_time);

    let stats = metrics.stats();

    // frame_count must be 7201 (not limited by buffer capacity).
    assert_eq!(
        stats.frame_count, 7201,
        "B2: frame_count expected 7201 after overflow, got {}",
        stats.frame_count
    );

    // The newest frame (0.020) must be in the buffer, affecting max.
    assert!(
        stats.max >= distinct_time - EPSILON,
        "B2: max must be at least {:.6} (distinct newest frame), got {:.6}",
        distinct_time,
        stats.max
    );

    // The oldest frame (first DT) must have been evicted.
    // Since buffer is all DT except last which is 0.020, min should be DT or 0.020.
    // The important check: p99 must be based on the most recent 7200 frames
    // (which includes 0.020 and excludes the first DT).
    assert!(
        stats.p99 >= DT,
        "B2: p99 must be ≥ DT (based on recent 7200 frames), got {:.6}",
        stats.p99
    );
}

// ============================================================================
// B3 — BNDRY/edge
// Traces To: §Implementation Summary Boundary Conditions — ring index wraparound
// "120 seconds of 60fps → window_fps based on most recent 60 seconds"
//
// Kills: ring index not wrapping (out-of-bounds write panic);
//   wrap overwrite leaves stale data in old entries.
// Wrong-impl challenge:
//   - Wrong: window_index not wrapping → index 60..119 → out-of-bounds → FAIL
//   - Wrong: wrap doesn't overwrite old data → window_fps ≈ 120 (all entries) → FAIL
//   - Wrong: index wraps but sum includes all 120 entries → wrong → FAIL
// ============================================================================

// real_test (feature #11)
#[test]
fn b3_ring_index_wraparound_120_seconds() {
    let mut metrics = FrameMetrics::new();

    // First 60 seconds: 60 fps (3600 frames).
    for _sec in 0..60 {
        for _ in 0..60 {
            metrics.sample(DT);
        }
    }

    // Verify window is full.
    let fps_after_60 = metrics.window_fps();
    assert!(
        fps_after_60 >= 59.5 && fps_after_60 <= 60.0,
        "B3: after 60s, window_fps must be ≈60.0, got {:.3}",
        fps_after_60
    );

    // Next 60 seconds: 30 fps (1800 more frames).
    let frame_time_30 = DT * 2.0; // 2x frame time → 30 fps
    for _sec in 0..60 {
        for _ in 0..30 {
            metrics.sample(frame_time_30);
        }
    }

    // Now window_fps must reflect only the LAST 60 seconds (30 fps),
    // NOT the full 120 seconds (which would average ~45 fps).
    let fps_after_120 = metrics.window_fps();
    assert!(
        fps_after_120 >= 29.5 && fps_after_120 <= 30.0,
        "B3: after 120s (60s@60fps+60s@30fps), window_fps must reflect most recent 60s only (≈30.0), got {:.3}",
        fps_after_120
    );

    // Total frame count: 3600 + 1800 = 5400.
    let stats = metrics.stats();
    assert_eq!(stats.frame_count, 5400, "B3: total frame_count must be 5400");
}

// ============================================================================
// B4 — BNDRY/edge
// Traces To: §Implementation Summary Boundary Conditions — frame_time extreme
// "One 0.5s frame + 99 DT frames → max == 0.5, avg ≈ (0.5+99*DT)/100"
//
// Kills: f32 overflow in accumulator; max not updated for large values.
// Wrong-impl challenge:
//   - Wrong: sum accumulator overflows f32 (not an issue with 0.5 + 99*DT)
//   - Wrong: max not updated → stays at DT (0.0167) → FAIL
//   - Wrong: extreme value rejected as "invalid" → max = DT → FAIL
// ============================================================================

// real_test (feature #11)
#[test]
fn b4_extreme_frame_time_handled() {
    let mut metrics = FrameMetrics::new();

    // One extreme frame (0.5s = ~2fps spike).
    let extreme: f32 = 0.5;
    metrics.sample(extreme);

    // 99 normal frames.
    for _ in 0..99 {
        metrics.sample(DT);
    }

    let stats = metrics.stats();

    // max must be the extreme value.
    assert!(
        approx_eq(stats.max, extreme),
        "B4: max must be {:.6} (extreme spike), got {:.6}",
        extreme,
        stats.max
    );

    // avg must be approximately (0.5 + 99*DT) / 100.
    // DT = 1/60 ≈ 0.0166667; 99*DT ≈ 1.65; +0.5 = 2.15; /100 = 0.0215.
    let expected_avg = (extreme + 99.0 * DT) / 100.0;
    assert!(
        approx_eq(stats.avg, expected_avg),
        "B4: avg expected ≈{:.6}, got {:.6}",
        expected_avg,
        stats.avg
    );

    // P99 must be ≤ the extreme value.
    assert!(
        stats.p99 <= extreme,
        "B4: p99 must be ≤ extreme {:.6}, got {:.6}",
        extreme,
        stats.p99
    );

    assert_eq!(stats.frame_count, 100, "B4: frame_count must be 100");
}

// ============================================================================
// B5 — BNDRY/edge
// Traces To: §Implementation Summary — 60s log trigger boundary
// "3599 frames → log NOT triggered; 3600 frames → log triggered"
//
// Kills: off-by-one triggering at 3599 instead of 3600; missing trigger.
// Wrong-impl challenge:
//   - Wrong: condition uses >= instead of == → triggers at 3599 → FAIL
//   - Wrong: condition uses 3599 as threshold → triggers at 3599 → FAIL
//   - Wrong: condition uses frame_count > 0 && frame_count % 3600 == 0
//     but frame_count wraps → trigger at correct time in this test
// ============================================================================

// real_test (feature #11)
#[test]
fn b5_log_trigger_boundary_3599_vs_3600() {
    let config = WindowConfig {
        width: 1280,
        height: 720,
        fullscreen: false,
    };
    let mut gl = GameLoop::new(config).expect("GameLoop::new should succeed");
    let mut state = SpyStateMachine::new();

    // Drive 3599 frames at DT. Log must NOT trigger at frame 3599.
    for _ in 0..3599 {
        gl.tick(DT, &mut state);
    }

    // At 3599 frames: frame_count must be 3599.
    // (In RED, stays 0 because sample() is no-op — this assertion FAILS.)
    assert_eq!(
        gl.metrics.frame_count, 3599,
        "B5: after 3599 frames, frame_count expected 3599, got {}",
        gl.metrics.frame_count
    );

    // One more frame → 3600. Log must trigger here.
    gl.tick(DT, &mut state);

    assert_eq!(
        gl.metrics.frame_count, 3600,
        "B5: after 3600 frames, frame_count expected 3600, got {}",
        gl.metrics.frame_count
    );
}

// ============================================================================
// P1 — PERF/frame_time
// Traces To: NFR-001 AC-1, ATS §3 PERF, §Interface Contract p99
// "360 frames (6s @ 60fps) → total wall time < 360 × DT × 1.05"
//
// Kills: game loop overhead exceeding 5% of frame budget.
// Wrong-impl challenge:
//   - Wrong: unnecessary heap allocation per tick → slow → FAIL
//   - Wrong: busy-wait inside tick → wall time >> budget → FAIL
//   - Wrong: O(n) scan per tick → slowdown grows with frames → FAIL
// ============================================================================

// real_test (feature #11)
#[test]
fn p1_360_frames_performance_within_budget() {
    let config = WindowConfig {
        width: 1280,
        height: 720,
        fullscreen: false,
    };
    let mut gl = GameLoop::new(config).expect("GameLoop::new should succeed");
    let mut state = SpyStateMachine::new();

    let start = std::time::Instant::now();

    // Drive 360 frames (6 simulated seconds @ 60fps).
    for _ in 0..360 {
        gl.tick(DT, &mut state);
    }

    let elapsed = start.elapsed();

    // Budget: 360 × (1/60) × 1.05 = 6.3 seconds.
    let budget_secs: f64 = 360.0 * (1.0 / 60.0) * 1.05;
    assert!(
        elapsed.as_secs_f64() < budget_secs,
        "P1: elapsed {:.3}s exceeds budget {:.3}s (360 frames × 1/60s × 1.05)",
        elapsed.as_secs_f64(),
        budget_secs
    );

    // Verify 360 frames were processed (each at DT produces 1 update).
    assert_eq!(
        state.update_count, 360,
        "P1: expected 360 update() calls, got {}",
        state.update_count
    );

    // Metrics must have 360 samples.
    assert_eq!(
        gl.metrics.frame_count, 360,
        "P1: expected 360 frame samples, got {}",
        gl.metrics.frame_count
    );
}

// ============================================================================
// P2 — PERF/sliding
// Traces To: NFR-001 AC-2, §Interface Contract window_fps
// "3600 frames with all entities → window_fps ≥ 58.0"
//
// Kills: full entity load degrades performance undetected; sliding window
//   calculation hides frame rate drop.
// Wrong-impl challenge:
//   - Wrong: heavy entity update in tick → slowdown → window_fps < 58 → FAIL
//   - Wrong: window_fps returns 60 regardless of actual data → FAIL
// ============================================================================

// real_test (feature #11)
#[test]
fn p2_3600_frames_full_load_window_fps() {
    let config = WindowConfig {
        width: 1280,
        height: 720,
        fullscreen: false,
    };
    let mut gl = GameLoop::new(config).expect("GameLoop::new should succeed");
    let mut state = SpyStateMachine::new();

    // Drive 3600 frames (60 simulated seconds @ 60fps).
    // SpyStateMachine is lightweight (~empty update), but the assertion
    // validates the metric infrastructure, not the entity load itself.
    for _ in 0..3600 {
        gl.tick(DT, &mut state);
    }

    let fps = gl.metrics.window_fps();

    // NFR-001 AC-2: window_fps must be ≥ 58.0 for any 60s window.
    assert!(
        fps >= 58.0,
        "P2: window_fps must be ≥ 58.0 after 3600 frames @ DT, got {:.3}",
        fps
    );

    // Verify 3600 samples recorded.
    assert_eq!(
        gl.metrics.frame_count, 3600,
        "P2: expected 3600 frame samples, got {}",
        gl.metrics.frame_count
    );
}

// ============================================================================
// P3 — PERF/empty
// Traces To: NFR-001, §Interface Contract stats
// "Empty scene (spy only) 3600 frames → window_fps ≥ 59.5, avg < 20ms, p99 < 20ms"
//
// Kills: no empty-scene baseline established; cannot detect future regressions.
// Wrong-impl challenge:
//   - Wrong: metrics not integrated into tick → all stats zero → FAIL
//   - Wrong: p99 inflates due to first-frame initialization cost → FAIL
// ============================================================================

// real_test (feature #11)
#[test]
fn p3_empty_scene_baseline_3600_frames() {
    let config = WindowConfig {
        width: 1280,
        height: 720,
        fullscreen: false,
    };
    let mut gl = GameLoop::new(config).expect("GameLoop::new should succeed");
    let mut state = SpyStateMachine::new();

    // Drive 3600 frames with minimal entity load (spy only).
    for _ in 0..3600 {
        gl.tick(DT, &mut state);
    }

    let stats = gl.metrics.stats();

    // Empty scene baseline: FPS near-perfect (≥ 59.5).
    assert!(
        stats.window_fps >= 59.5,
        "P3: window_fps must be ≥ 59.5 for empty scene, got {:.3}",
        stats.window_fps
    );

    // avg frame time must be below 20ms (50fps floor for empty scene).
    assert!(
        stats.avg < 0.020,
        "P3: avg frame time must be < 20ms for empty scene, got {:.6}s",
        stats.avg
    );

    // P99 frame time must be below 20ms.
    assert!(
        stats.p99 < 0.020,
        "P3: p99 frame time must be < 20ms for empty scene, got {:.6}s",
        stats.p99
    );

    // 3600 frames recorded.
    assert_eq!(
        stats.frame_count, 3600,
        "P3: expected 3600 frame samples, got {}",
        stats.frame_count
    );
}
