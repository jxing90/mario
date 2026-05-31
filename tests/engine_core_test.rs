// Feature #1: Engine Core — TDD Red Phase
//
// [no integration test] — pure engine computation, no external I/O.
// Macroquad window/render-target integration tested via Manual: visual-judgment
// (see long-task-guide.md Real Test Convention).
//
// Test Inventory Reference: docs/features/1-engine-core.md §7
// All tests are expected to FAIL (RED phase) — implementation not yet written.
//
// Category coverage (Rule 1):
//   FUNC/happy:   T1, T2, T3, T4, T12
//   FUNC/error:   T5, T6
//   BNDRY/edge:   T7, T8, T9, T10
//   PERF/frame-time: T11
//   SEC: N/A — internal engine logic, no user-facing input
//   INTG: N/A — pure engine layer, no external I/O (per design §7)

use mario_platformer::engine::{GameLoop, WindowConfig, WindowError};
use mario_platformer::state::StateMachine;

// ============================================================================
// Constants
// ============================================================================

/// Allowed tolerance for f32 comparisons (per long-task-guide: epsilon=0.01 for physics).
const EPSILON: f32 = 1e-5;

/// The fixed timestep: 1/60 second.
const DT: f32 = 1.0 / 60.0;

/// Maximum catch-up steps per frame.
const MAX_STEPS: u32 = 5;


// ============================================================================
// Test Double: SpyStateMachine
// Records every update(dt) and render(alpha) invocation for verification.
// Used across all loop-logic tests (T1, T2, T6–T12).
// ============================================================================

struct SpyStateMachine {
    /// Number of times update(dt) was called.
    pub update_count: u32,
    /// Number of times render(alpha) was called.
    pub render_count: u32,
    /// Every dt value passed to update(), in order.
    pub update_dt_values: Vec<f32>,
    /// Every alpha value passed to render(), in order.
    pub render_alpha_values: Vec<f32>,
    /// Simulated horizontal position (vx = 100 px/s) — used in T2.
    pub position: f32,
}

impl SpyStateMachine {
    fn new() -> Self {
        Self {
            update_count: 0,
            render_count: 0,
            update_dt_values: Vec::new(),
            render_alpha_values: Vec::new(),
            position: 0.0,
        }
    }
}

impl StateMachine for SpyStateMachine {
    fn update(&mut self, dt: f32) {
        self.update_count += 1;
        self.update_dt_values.push(dt);
        // Uniform horizontal motion: vx = 100 px/s
        self.position += 100.0 * dt;
    }

    fn render(&mut self, alpha: f32) {
        self.render_count += 1;
        self.render_alpha_values.push(alpha);
    }
}

// ============================================================================
// Helper utilities
// ============================================================================

/// Returns true if two f32 values are within EPSILON of each other.
fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < EPSILON
}

/// Helper: drive N frames through GameLoop::tick with uniform frame_time.
/// Returns the SpyStateMachine so callers can inspect counts.
fn drive_frames(gl: &mut GameLoop, frame_time: f32, n_frames: u32, state: &mut SpyStateMachine) {
    for _ in 0..n_frames {
        gl.tick(frame_time, state);
    }
}

// ============================================================================
// T1 — FUNC/happy
// Traces To: FR-018 AC-1, §Interface Contract GameLoop::run postcondition
// "真实时间 1 秒内恰好调用 update 60 次"
//
// Kills: accumulator going too fast/slow; dt constant accidentally mutated.
// Wrong-impl challenge:
//   - Wrong: dt = frame_time → update count varies with injected frame_time → FAIL
//   - Wrong: accumulator uses *= instead of += → fewer updates → FAIL
//   - Wrong: dt = 1.0/30.0 → only ~30 updates → FAIL
// ============================================================================

// real_test (feature #1)
#[test]
fn t1_fixed_timestep_60_steps_per_second() {
    let config = WindowConfig {
        width: 1280,
        height: 720,
        fullscreen: false,
    };
    let mut gl = GameLoop::new(config).expect("GameLoop::new should succeed");
    let mut state = SpyStateMachine::new();

    // Drive 60 frames each lasting exactly 1/60 s → 1.0 s real time.
    drive_frames(&mut gl, DT, 60, &mut state);

    // AC-1: Exactly 60 simulation steps in 1 second of real time.
    assert_eq!(
        state.update_count, 60,
        "Expected exactly 60 update() calls in 1.0s real time, got {}",
        state.update_count
    );

    // Every dt value must be exactly DT (1/60).
    for (i, &dt_val) in state.update_dt_values.iter().enumerate() {
        assert!(
            approx_eq(dt_val, DT),
            "update_dt_values[{}] = {}, expected {} (± epsilon)",
            i,
            dt_val,
            DT
        );
    }

    // Render must be called once per frame (60 renders for 60 frames).
    assert_eq!(
        state.render_count, 60,
        "Expected exactly 60 render() calls (one per frame), got {}",
        state.render_count
    );

    // update and render interleaving: each frame has update(s) then one render.
    // For uniform DT frame_time, each frame produces exactly 1 update + 1 render.
    // total updates (60) == total renders (60) when frame_time == DT.
    // No render should have fired before the first update completes.
    assert!(
        !state.render_alpha_values.is_empty(),
        "render() must be called at least once"
    );
}

// ============================================================================
// T2 — FUNC/happy
// Traces To: FR-018 AC-2, §Design Alignment seq msg#4-5, seq msg#7
// "模拟继续以固定步长累进，物体运动不因帧率波动而变速"
//
// Kills: delta-time driving physics; alpha miscalculation causing render jitter.
// Wrong-impl challenge:
//   - Wrong: physics driven by frame_time instead of fixed dt → position varies → FAIL
//   - Wrong: update called with variable dt → distance drifts → FAIL
//   - Wrong: alpha applied to physics instead of render interpolation → position wrong → FAIL
// ============================================================================

// real_test (feature #1)
#[test]
fn t2_frame_rate_independent_motion() {
    let config = WindowConfig {
        width: 1280,
        height: 720,
        fullscreen: false,
    };
    let mut gl = GameLoop::new(config).expect("GameLoop::new should succeed");
    let mut state = SpyStateMachine::new();

    // Variable frame times that sum to ~1.0s real time.
    // 0.008 + 0.025 + 0.033 + 0.012 = 0.078 per cycle.
    // 13 cycles ≈ 1.014s → roughly 60 updates.
    let frame_times: [f32; 4] = [0.008, 0.025, 0.033, 0.012];
    let cycles: u32 = 13; // 13 * 0.078 = 1.014s → ~60.84 updates expected

    for _ in 0..cycles {
        for &ft in &frame_times {
            gl.tick(ft, &mut state);
        }
    }

    // The position should be ~100 px after 1 second of simulated time.
    // Each update advances position by 100 * DT = 100/60 ≈ 1.6667 px.
    // 60 updates → 100 px. 61 updates → 101.6667 px.
    // Allowed range: [99.5, 101.7] (ε = 0.5px position tolerance).
    let expected_min = 99.5;
    let expected_max = 101.7;
    assert!(
        state.position >= expected_min && state.position <= expected_max,
        "Frame-rate-independent position: got {:.4}, expected [{:.1}, {:.1}] (100 px at 100 px/s after ~1s)",
        state.position,
        expected_min,
        expected_max
    );

    // Verify all dt values passed to update are exactly DT (fixed timestep invariant).
    for (i, &dt_val) in state.update_dt_values.iter().enumerate() {
        assert!(
            approx_eq(dt_val, DT),
            "update_dt_values[{}] = {}, expected DT={} — variable frame_time must NOT leak into update dt",
            i,
            dt_val,
            DT
        );
    }

    // Render called exactly once per injected frame.
    let total_frames = cycles * (frame_times.len() as u32);
    assert_eq!(
        state.render_count, total_frames,
        "Expected {} render() calls (one per frame), got {}",
        total_frames,
        state.render_count
    );
}

// ============================================================================
// T3 — FUNC/happy
// Traces To: FR-018 AC-4, §Interface Contract GameLoop::new postcondition
// "时间累加器初始化为 0，模拟与渲染时基对齐"
//
// Kills: accumulator starting non-zero; RenderTarget not created; wrong max_steps.
// Wrong-impl challenge:
//   - Wrong: accumulator = 1.0 → first frame has extra catch-up → FAIL
//   - Wrong: dt = 0 → division by zero later → FAIL
//   - Wrong: max_steps = 0 → no updates ever → FAIL
// ============================================================================

// real_test (feature #1)
#[test]
fn t3_game_loop_constructor_initializes_correctly() {
    let config = WindowConfig {
        width: 1280,
        height: 720,
        fullscreen: false,
    };
    let result = GameLoop::new(config);

    // Must return Ok, not Err.
    assert!(
        result.is_ok(),
        "GameLoop::new(1280x720) should return Ok, got Err"
    );

    let gl = result.unwrap();

    // AC-4: accumulator starts at zero.
    assert!(
        approx_eq(gl.accumulator, 0.0),
        "accumulator must be 0.0 after construction, got {}",
        gl.accumulator
    );

    // dt is the fixed timestep constant 1/60.
    assert!(
        approx_eq(gl.dt, DT),
        "dt must be 1.0/60.0, got {}",
        gl.dt
    );

    // max_steps is the catch-up limit (5).
    assert_eq!(
        gl.max_steps, MAX_STEPS,
        "max_steps must be 5, got {}",
        gl.max_steps
    );
}

// ============================================================================
// T4 — FUNC/happy
// Traces To: IAPI-011, §Interface Contract GameLoop::apply_display postcondition
// "若 (w, h) 在受支持分辨率集合中：窗口调整至 w × h"
//
// Kills: config fields not updated; RenderTarget not rebuilt.
// Wrong-impl challenge:
//   - Wrong: config unchanged after apply → DISPLAY Config shows stale values → FAIL
//   - Wrong: width/height swapped → wrong resolution → FAIL
//   - Wrong: fullscreen flag ignored → can't toggle → FAIL
// ============================================================================

// real_test (feature #1)
#[test]
fn t4_apply_display_supported_resolution() {
    let config = WindowConfig {
        width: 1280,
        height: 720,
        fullscreen: false,
    };
    let mut gl = GameLoop::new(config).expect("GameLoop::new should succeed");

    // Apply a different supported resolution.
    gl.apply_display(1920, 1080, false);

    assert_eq!(
        gl.window.config.width, 1920,
        "apply_display(1920,1080): config.width must be 1920, got {}",
        gl.window.config.width
    );
    assert_eq!(
        gl.window.config.height, 1080,
        "apply_display(1920,1080): config.height must be 1080, got {}",
        gl.window.config.height
    );
    assert_eq!(
        gl.window.config.fullscreen, false,
        "apply_display(1920,1080,false): config.fullscreen must be false"
    );

    // Apply another supported resolution with fullscreen enabled.
    gl.apply_display(2560, 1440, true);

    assert_eq!(gl.window.config.width, 2560);
    assert_eq!(gl.window.config.height, 1440);
    assert_eq!(
        gl.window.config.fullscreen, true,
        "apply_display(2560,1440,true): config.fullscreen must be true"
    );
}

// ============================================================================
// T5 — FUNC/error
// Traces To: IAPI-011, §Interface Contract GameLoop::apply_display Raises
// "若 (w, h) 不匹配受支持分辨率 → 无错误抛出；行为为 no-op"
//
// Kills: unsupported resolution causing panic; throwing exception instead of no-op.
// Wrong-impl challenge:
//   - Wrong: panic on unsupported resolution → test panics → FAIL
//   - Wrong: config partially updated (e.g. only width changed) → assertions fail → FAIL
//   - Wrong: unsupported resolution creates window anyway → crash later → FAIL
// ============================================================================

// real_test (feature #1)
#[test]
fn t5_apply_display_unsupported_resolution_is_noop() {
    let config = WindowConfig {
        width: 1280,
        height: 720,
        fullscreen: false,
    };
    let mut gl = GameLoop::new(config).expect("GameLoop::new should succeed");

    let original_width = gl.window.config.width;
    let original_height = gl.window.config.height;
    let original_fullscreen = gl.window.config.fullscreen;

    // Apply an unsupported resolution — must be a no-op, not a panic.
    gl.apply_display(800, 600, true);

    // All config fields must remain unchanged.
    assert_eq!(
        gl.window.config.width, original_width,
        "apply_display(800,600): width must be unchanged (was {}, now {})",
        original_width, gl.window.config.width
    );
    assert_eq!(
        gl.window.config.height, original_height,
        "apply_display(800,600): height must be unchanged (was {}, now {})",
        original_height, gl.window.config.height
    );
    assert_eq!(
        gl.window.config.fullscreen, original_fullscreen,
        "apply_display(800,600): fullscreen must be unchanged (was {}, now {})",
        original_fullscreen, gl.window.config.fullscreen
    );
}

// ============================================================================
// ST-FUNC-001-008 — FUNC/error (added during Feature-ST)
// Traces To: IAPI-011, §Interface Contract GameLoop::new Raises
// "GameLoop::new with unsupported resolution returns WindowError::UnsupportedResolution"
//
// Kills: constructor panicking on bad input instead of returning Err.
// ============================================================================

// real_test (feature #1)
#[test]
fn t_st_func_001_008_unsupported_resolution_error() {
    // 800x600 is NOT in the supported set {1280x720, 1920x1080, 2560x1440}.
    let config = WindowConfig {
        width: 800,
        height: 600,
        fullscreen: false,
    };
    let result = GameLoop::new(config);

    assert!(
        result.is_err(),
        "GameLoop::new(800x600) should return Err"
    );
    let err = result.err().unwrap();
    assert_eq!(
        err, WindowError::UnsupportedResolution,
        "GameLoop::new(800x600) should return WindowError::UnsupportedResolution, got {:?}",
        err
    );

    // 1024x768 is also unsupported.
    let config2 = WindowConfig {
        width: 1024,
        height: 768,
        fullscreen: false,
    };
    let result2 = GameLoop::new(config2);
    assert!(result2.is_err());
    let err2 = result2.err().unwrap();
    assert_eq!(err2, WindowError::UnsupportedResolution);

    // 1280x720 IS supported — should succeed.
    let config3 = WindowConfig {
        width: 1280,
        height: 720,
        fullscreen: false,
    };
    let result3 = GameLoop::new(config3);
    assert!(
        result3.is_ok(),
        "GameLoop::new(1280x720) should succeed"
    );
}

// ============================================================================
// T6 — FUNC/error
// Traces To: FR-018 AC-3, §Implementation Summary flow branch#DoClamp,
//            §Interface Contract GameLoop::run postcondition
// "系统将单帧模拟步数限制在最大值 5 次，超出部分的时间被丢弃"
//
// Kills: no clamping → spiral-of-death hundreds of steps; off-by-one (6 steps).
// Wrong-impl challenge:
//   - Wrong: max_steps check uses <= instead of < → 6 updates → FAIL
//   - Wrong: accumulator not reduced after update → infinite loop → timeout → FAIL
//   - Wrong: no max_steps check → all 0.2s consumed as 12 updates → FAIL
// ============================================================================

// real_test (feature #1)
#[test]
fn t6_spike_frame_clamped_to_max_5_steps() {
    let config = WindowConfig {
        width: 1280,
        height: 720,
        fullscreen: false,
    };
    let mut gl = GameLoop::new(config).expect("GameLoop::new should succeed");
    let mut state = SpyStateMachine::new();

    let updates_before = state.update_count;

    // Single frame with extreme frame_time: 0.2s (far exceeding normal).
    // accumulator = 0.2 → clamped to dt*(max_steps+1) = 1/60 * 6 = 0.1.
    // Then exactly 5 updates of dt each, consuming 5/60 = 0.08333... s.
    gl.tick(0.2, &mut state);

    let updates_this_frame = state.update_count - updates_before;

    // AC-3: Exactly 5 simulation steps — no more, no less.
    assert_eq!(
        updates_this_frame, 5,
        "Spike frame (0.2s): expected exactly 5 update() calls, got {}",
        updates_this_frame
    );

    // After the spike frame, accumulator must hold only the unconsumed remainder
    // (clamped 0.1 - 5*DT = 0.1 - 0.08333... ≈ 0.016667), NOT the full 0.2s.
    // The excess beyond clamp threshold is discarded.
    let max_remainder = DT + EPSILON; // less than one full dt
    assert!(
        gl.accumulator < max_remainder,
        "After spike frame, accumulator {:.6} must be < {:.6} (excess time discarded)",
        gl.accumulator,
        max_remainder
    );

    // render() must still be called exactly once this frame.
    assert_eq!(
        state.render_count, 1,
        "Even during a spike frame, render() must be called once, got {}",
        state.render_count
    );
}

// ============================================================================
// T7 — BNDRY/edge
// Traces To: §Implementation Summary Boundary Conditions (accumulator at dt),
//            §Implementation Summary flow branch#CheckLoop
// "accumulator == 1.0/60.0 恰好 → update 1 次, accumulator -> 0.0, render(alpha=0.0)"
//
// Kills: == comparison instead of >= skipping update; floating underflow after -= dt.
// Wrong-impl challenge:
//   - Wrong: uses == instead of >= → boundary miss → 0 updates → FAIL
//   - Wrong: accumulator -= DT leaves tiny remainder (f32 error) → next frame drifts → FAIL
//   - Wrong: alpha computed before update loop instead of after → alpha ≠ 0.0 → FAIL
// ============================================================================

// real_test (feature #1)
#[test]
fn t7_accumulator_exactly_at_dt_boundary() {
    let config = WindowConfig {
        width: 1280,
        height: 720,
        fullscreen: false,
    };
    let mut gl = GameLoop::new(config).expect("GameLoop::new should succeed");
    let mut state = SpyStateMachine::new();

    // Manually set accumulator to exactly dt, then call tick with zero frame_time
    // so no additional time is added. This must trigger exactly 1 update.
    gl.accumulator = DT;
    gl.tick(0.0, &mut state);

    // At boundary: exactly 1 update executed.
    assert_eq!(
        state.update_count, 1,
        "accumulator == DT: expected exactly 1 update(), got {}",
        state.update_count
    );

    // After consuming dt, accumulator must be 0.0 (exact consumption, no float residue).
    assert!(
        approx_eq(gl.accumulator, 0.0),
        "After consuming DT from accumulator, expected 0.0, got {:.10}",
        gl.accumulator
    );

    // alpha = accumulator_after / dt = 0.0 / DT = 0.0.
    assert_eq!(
        state.render_count, 1,
        "render() must be called once per frame, got {}",
        state.render_count
    );
    let alpha = state.render_alpha_values[0];
    assert!(
        approx_eq(alpha, 0.0),
        "After consuming exactly DT, alpha must be 0.0, got {}",
        alpha
    );
}

// ============================================================================
// T8 — BNDRY/edge
// Traces To: §Implementation Summary Boundary Conditions (frame_time=0),
//            §Design Alignment seq msg#1, seq msg#7
// "frame_time = 0.0 → update 0 次, render 1 次 (alpha=0.0)"
//
// Kills: divide-by-zero on frame_time=0; skipping render when no updates.
// Wrong-impl challenge:
//   - Wrong: division by frame_time → NaN or panic → FAIL
//   - Wrong: render skipped because update_count == 0 → black first frame → FAIL
//   - Wrong: alpha uses frame_time instead of accumulator → wrong alpha → FAIL
// ============================================================================

// real_test (feature #1)
#[test]
fn t8_zero_frame_time_produces_no_update_still_renders() {
    let config = WindowConfig {
        width: 1280,
        height: 720,
        fullscreen: false,
    };
    let mut gl = GameLoop::new(config).expect("GameLoop::new should succeed");
    let mut state = SpyStateMachine::new();

    // frame_time = 0.0 simulates first frame or timer glitch.
    gl.tick(0.0, &mut state);

    // No updates when no time has passed.
    assert_eq!(
        state.update_count, 0,
        "frame_time=0.0: expected 0 update() calls, got {}",
        state.update_count
    );

    // Render MUST still happen (avoids black first frame).
    assert_eq!(
        state.render_count, 1,
        "frame_time=0.0: render() must still be called once, got {}",
        state.render_count
    );

    // alpha must be 0.0 (accumulator=0 / dt).
    assert!(
        approx_eq(state.render_alpha_values[0], 0.0),
        "frame_time=0.0: alpha must be 0.0, got {}",
        state.render_alpha_values[0]
    );

    // No crash, no NaN, no panic — system is stable.
}

// ============================================================================
// T9 — BNDRY/edge
// Traces To: §Implementation Summary Boundary Conditions (spiral clamp),
//            §Implementation Summary flow branch#DoClamp
// "frame_time = 10.0s → 钳制后恰好 5 次 update，剩余时间被丢弃"
//
// Kills: clamp formula using dt*max_steps (4) instead of dt*(max_steps+1) (6 threshold);
//        no clamp → hundreds of catch-up steps.
// Wrong-impl challenge:
//   - Wrong: clamp threshold = dt * max_steps → accumulator clamped to 0.0833 → only 4 updates → FAIL
//   - Wrong: clamp threshold missing → 600 updates (10/0.01667) → hang → FAIL
//   - Wrong: clamp resets accumulator to 0 instead of to threshold → 0 updates → FAIL
// ============================================================================

// real_test (feature #1)
#[test]
fn t9_spiral_death_recovery_clamps_accumulator() {
    let config = WindowConfig {
        width: 1280,
        height: 720,
        fullscreen: false,
    };
    let mut gl = GameLoop::new(config).expect("GameLoop::new should succeed");
    let mut state = SpyStateMachine::new();

    // Simulate system suspend/resume: 10 seconds of wall time in one frame.
    gl.tick(10.0, &mut state);

    // AC-3 guard: exactly 5 updates, NOT hundreds.
    assert_eq!(
        state.update_count, 5,
        "Spiral death (10s): expected exactly 5 update() after clamping, got {}",
        state.update_count
    );

    // Accumulator after processing must be bounded — excess time is discarded.
    // The clamp ceiling is dt*(max_steps+1) = DT*6 ≈ 0.1.
    // After 5 updates: 0.1 - 5*DT = 0.1 - 0.08333... = 0.016667.
    let clamp_ceiling = DT * ((MAX_STEPS + 1) as f32);
    assert!(
        gl.accumulator < clamp_ceiling,
        "After spiral recovery, accumulator {:.6} must be < clamp ceiling {:.6}",
        gl.accumulator,
        clamp_ceiling
    );

    // System is still alive after the spike — tick can be called again normally.
    let updates_after_recovery = state.update_count;
    gl.tick(DT, &mut state);
    assert!(
        state.update_count > updates_after_recovery,
        "Game loop should continue processing normally after spiral recovery"
    );
}

// ============================================================================
// T10 — BNDRY/edge
// Traces To: §Implementation Summary Boundary Conditions (dt constant)
// "dt 值始终等于 1.0 / 60.0，从未被修改"
//
// Kills: dt accidentally rebound to frame_time; mutable dt field.
// Wrong-impl challenge:
//   - Wrong: dt = frame_time in tick body → constant violated → FAIL
//   - Wrong: dt field is `mut` and gets modified somewhere → value changes → FAIL
//   - Wrong: dt read from config but config changes → dt drifts → FAIL
// ============================================================================

// real_test (feature #1)
#[test]
fn t10_dt_is_immutable_constant_across_many_frames() {
    let config = WindowConfig {
        width: 1280,
        height: 720,
        fullscreen: false,
    };
    let mut gl = GameLoop::new(config).expect("GameLoop::new should succeed");
    let mut state = SpyStateMachine::new();

    // Run 360 frames with varying frame times — dt must remain exactly 1/60.
    let frame_times: [f32; 6] = [0.005, 0.010, 0.016667, 0.020, 0.025, 0.033];

    for cycle in 0..60 {
        for &ft in &frame_times {
            gl.tick(ft, &mut state);
        }
        // Spot-check dt field hasn't been mutated mid-simulation.
        assert!(
            approx_eq(gl.dt, DT),
            "dt field mutated at cycle {}: got {}, expected {}",
            cycle,
            gl.dt,
            DT
        );
    }

    // Total: 360 frames, 360 renders.
    assert_eq!(state.render_count, 360);

    // All update dt values must be exactly DT.
    for (i, &dt_val) in state.update_dt_values.iter().enumerate() {
        assert!(
            approx_eq(dt_val, DT),
            "update_dt_values[{}] = {}, dt must be constant {}",
            i,
            dt_val,
            DT
        );
    }

    // dt was never mutated — it remained constant across all 360 frames.
}

// ============================================================================
// T11 — PERF/frame-time
// Traces To: FR-018 AC-1, NFR-001, §Interface Contract GameLoop::run postcondition
// "总耗时 < 360 × (1/60)s × 1.05 = 6.3s"
//
// Kills: empty loop overhead too high (> 1ms/step) consuming render budget.
// Wrong-impl challenge:
//   - Wrong: busy-wait inside tick → wall time >> 6.3s → FAIL
//   - Wrong: unnecessary allocation per update → overhead spike → FAIL
//   - Wrong: unbuffered I/O per update → slow → FAIL
// ============================================================================

// real_test (feature #1)
#[test]
fn t11_empty_update_loop_performance_within_budget() {
    let config = WindowConfig {
        width: 1280,
        height: 720,
        fullscreen: false,
    };
    let mut gl = GameLoop::new(config).expect("GameLoop::new should succeed");
    let mut state = SpyStateMachine::new();

    // The StateMachine's update() is a no-op for this test
    // (SpyStateMachine::update does a few arithmetic ops — close to empty).

    let start = std::time::Instant::now();

    // Drive 360 simulation steps worth of time (= 6 seconds simulated).
    // Each frame: ~1/60s → ~360 frames to accumulate 360 steps.
    for _ in 0..360 {
        gl.tick(DT, &mut state);
    }

    let elapsed = start.elapsed();

    // Performance budget: 360 * (1/60)s * 1.05 = 6.3 seconds max.
    let budget_secs: f64 = 360.0 * (1.0 / 60.0) * 1.05;
    assert!(
        elapsed.as_secs_f64() < budget_secs,
        "Empty update loop: elapsed {:.3}s exceeds budget {:.3}s (360 steps × 1/60s × 1.05)",
        elapsed.as_secs_f64(),
        budget_secs
    );

    // Verify the 360 steps were actually executed.
    assert_eq!(
        state.update_count, 360,
        "Expected 360 update() calls, got {}",
        state.update_count
    );
}

// ============================================================================
// T12 — FUNC/happy
// Traces To: FR-018 AC-2, §Design Alignment seq msg#7
// "总 update 次数 = 3；总 render 次数 = 3（每帧恰好 1 次 render，独立于 update 次数）"
//
// Kills: render inside while loop (per update); render skipped on frames with 0 updates.
// Wrong-impl challenge:
//   - Wrong: render() inside while(accumulator>=dt) loop → extra renders → FAIL
//   - Wrong: render() guarded by `if updates > 0` → skipped when 0 updates → FAIL
//   - Wrong: render() before update loop → alpha from previous frame → FAIL
// ============================================================================

// real_test (feature #1)
#[test]
fn t12_render_once_per_frame_regardless_of_update_count() {
    let config = WindowConfig {
        width: 1280,
        height: 720,
        fullscreen: false,
    };
    let mut gl = GameLoop::new(config).expect("GameLoop::new should succeed");
    let mut state = SpyStateMachine::new();

    // Frame 1: large frame_time → 2 updates (0.035s ➔ accumulator=0.035 > 2*DT)
    gl.tick(0.035, &mut state);
    let render_after_f1 = state.render_count;
    let update_after_f1 = state.update_count;
    assert_eq!(render_after_f1, 1, "Frame 1: must have exactly 1 render");
    assert_eq!(update_after_f1, 2, "Frame 1: expected 2 updates");

    // Frame 2: moderate frame_time → 1 update
    gl.tick(0.018, &mut state);
    let render_after_f2 = state.render_count;
    let update_after_f2 = state.update_count;
    assert_eq!(render_after_f2, 2, "Frame 2: must have exactly 1 render (2 total)");
    assert_eq!(update_after_f2 - update_after_f1, 1, "Frame 2: expected 1 update");

    // Frame 3: very small frame_time → 0 updates
    gl.tick(0.001, &mut state);
    let render_after_f3 = state.render_count;
    let update_after_f3 = state.update_count;

    // KEY ASSERTION: render still called even when 0 updates occurred.
    assert_eq!(
        render_after_f3, 3,
        "Frame 3 (0 updates): render() MUST still be called (3 total), got {}",
        render_after_f3
    );
    // 0 additional updates.
    assert_eq!(
        update_after_f3 - update_after_f2, 0,
        "Frame 3 (0.001s): expected 0 updates, got {}",
        update_after_f3 - update_after_f2
    );

    // Summary: total updates = 3, total renders = 3.
    // render() is called exactly once per frame, independently of update count.
    assert_eq!(state.update_count, 3, "Total updates across 3 frames must be 3");
    assert_eq!(state.render_count, 3, "Total renders across 3 frames must be 3");
}
