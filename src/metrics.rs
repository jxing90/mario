// Performance metrics module — Feature #11: 60fps Frame Rate (NFR-001)
//
// Tracks per-frame render times with min/max/avg/p99 statistics and a
// 60-second sliding window FPS counter. Called by GameLoop::tick() every frame.
//
// Design Reference: docs/features/11-60fps-frame-rate-nfr-001.md §4 / §6 / §8
// SRS Reference: NFR-001
// ATS Reference: docs/plans/2026-05-31-mario-platformer-ats.md §2.2, §3 (PERF)

/// Buffer capacity: 7200 frames = 120 seconds @ 60 fps (2× the 60s sliding window).
const BUFFER_CAPACITY: usize = 7200;

/// Floating-point tolerance for per-second window accumulator comparison.
///
/// After 60 frames at DT (≈ 16.667 ms each), the accumulated time may be
/// 0.9999.. instead of exactly 1.0 due to floating-point rounding in 1.0/60.0.
/// This tolerance allows the window to advance when within 0.1 ms of a full second.
const WINDOW_SECOND_TOLERANCE: f32 = 0.0001;

/// Aggregated frame statistics returned by FrameMetrics::stats().
///
/// §8 Data Model — FrameStats value object.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameStats {
    pub min: f32,
    pub max: f32,
    pub avg: f32,
    pub p99: f32,
    pub window_fps: f32,
    pub frame_count: u64,
}

/// Performance instrumentation: tracks per-frame render times with
/// min/max/avg/p99 statistics and a 60-second sliding window FPS counter.
///
/// # Fields (§8 Data Model)
/// - `frame_count`: total frames sampled since construction.
/// - `min_frame_time`: minimum frame time observed (initialized to `f32::MAX`).
/// - `max_frame_time`: maximum frame time observed (initialized to `0.0`).
/// - `sum_frame_time`: running sum of frame times for avg calculation.
/// - `frame_time_buffer`: ring buffer of recent frame times (capacity 7200).
/// - `window_ring`: ring buffer of per-second frame counts (capacity 60).
/// - `window_accumulator`: running accumulator for per-second bucketing.
/// - `window_frame_count`: frame count within the current in-progress second.
/// - `window_index`: next write position in `window_ring` (wraps at 60).
/// - `window_filled`: number of seconds written to `window_ring` (capped at 60).
pub struct FrameMetrics {
    pub frame_count: u64,
    pub min_frame_time: f32,
    pub max_frame_time: f32,
    sum_frame_time: f32,
    frame_time_buffer: Vec<f32>,
    window_ring: [u32; 60],
    window_accumulator: f32,
    window_frame_count: u32,
    window_index: usize,
    window_filled: usize,
}

impl FrameMetrics {
    /// Initialize a new FrameMetrics instance with zeroed counters.
    ///
    /// # Postconditions (§4)
    /// - `frame_count = 0`; `min_frame_time = f32::MAX`; `max_frame_time = 0.0`.
    /// - `window_ring` all zero; `window_frame_count = 0`; `window_accumulator = 0.0`.
    /// - `window_index = 0`; `window_filled = 0`; `frame_time_buffer` empty.
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            min_frame_time: f32::MAX,
            max_frame_time: 0.0,
            sum_frame_time: 0.0,
            frame_time_buffer: Vec::with_capacity(BUFFER_CAPACITY),
            window_ring: [0u32; 60],
            window_accumulator: 0.0,
            window_frame_count: 0,
            window_index: 0,
            window_filled: 0,
        }
    }

    /// Record a frame time sample. Called by GameLoop::tick() each frame.
    ///
    /// # Preconditions (§4)
    /// - `frame_time > 0.0` (normal frame); `frame_time = 0.0` allowed
    ///   (first frame or timer glitch — recorded but excludes from min/max/sum).
    ///
    /// # Postconditions (§4)
    /// - `frame_count += 1`.
    /// - `frame_time` appended to `frame_time_buffer` (FIFO eviction at 7200).
    /// - If `frame_time > 0.0`: updates `min_frame_time` / `max_frame_time`,
    ///   accumulates `sum_frame_time`, and advances per-second window logic.
    /// - If `frame_time = 0.0`: skips min/max/sum/window updates.
    pub fn sample(&mut self, frame_time: f32) {
        // Always increment frame_count and record the sample.
        self.frame_count += 1;

        // FIFO ring-buffer: evict oldest if at capacity.
        if self.frame_time_buffer.len() >= BUFFER_CAPACITY {
            self.frame_time_buffer.remove(0);
        }
        self.frame_time_buffer.push(frame_time);

        // Zero frame_time is a timer glitch — skip statistical updates
        // so it does not pollute min/max or inflate window frame counts.
        if frame_time > 0.0 {
            // Running min / max / sum.
            if frame_time < self.min_frame_time {
                self.min_frame_time = frame_time;
            }
            if frame_time > self.max_frame_time {
                self.max_frame_time = frame_time;
            }
            self.sum_frame_time += frame_time;

            // Per-second window bucketing.
            self.window_accumulator += frame_time;
            self.window_frame_count += 1;

            // Emit completed seconds to the window ring.
            self.advance_window_seconds();
        }
    }

    /// Advance the per-second window ring by emitting completed seconds.
    ///
    /// Handles multi-second frames via a while loop — e.g. a 1.5 s frame
    /// writes 1 frame to the current second and 0 frames to the next.
    /// Uses [`WINDOW_SECOND_TOLERANCE`] to compensate for f32 precision:
    /// after 60 frames at DT (≈ 16.667 ms), the accumulator may be 0.9999...
    /// instead of exactly 1.0 due to floating-point rounding in 1.0 / 60.0.
    fn advance_window_seconds(&mut self) {
        while self.window_accumulator >= 1.0 - WINDOW_SECOND_TOLERANCE {
            self.window_ring[self.window_index] = self.window_frame_count;
            self.window_index = (self.window_index + 1) % 60;
            if self.window_filled < 60 {
                self.window_filled += 1;
            }
            self.window_accumulator -= 1.0;
            self.window_frame_count = 0;
        }
    }

    /// Compute the 99th percentile frame time from recorded samples.
    ///
    /// Sorts a defensive copy of the frame time buffer and picks the element
    /// at index `floor(0.99 * len)`. Returns `0.0` when the buffer is empty.
    ///
    /// # Postconditions (§4)
    /// - Buffer empty → returns `0.0`.
    /// - Buffer non-empty → returns the P99 value (largest 1% of samples).
    pub fn p99(&self) -> f32 {
        let n = self.frame_time_buffer.len();
        if n == 0 {
            return 0.0;
        }
        let mut sorted = self.frame_time_buffer.clone();
        sorted.sort_by(|a, b| a.total_cmp(b));
        let idx = ((n as f64) * 0.99).floor() as usize;
        // idx is always < n because floor(0.99 * n) ≤ n-1 for n ≥ 1.
        sorted[idx]
    }

    /// Compute the average FPS over the 60-second sliding window.
    ///
    /// Sums all per-second frame counts in `window_ring` and divides by the
    /// number of filled seconds. If fewer than 60 seconds have been recorded,
    /// returns the average over the available seconds.
    ///
    /// # Postconditions (§4)
    /// - Window partially filled → average fps over filled seconds.
    /// - Window full (60s) → total frames / 60.0.
    pub fn window_fps(&self) -> f32 {
        if self.window_filled == 0 {
            return 0.0;
        }
        let total: u32 = self.window_ring.iter().sum();
        total as f32 / self.window_filled as f32
    }

    /// Aggregate all statistics into a `FrameStats` value object.
    ///
    /// # Postconditions (§4)
    /// - Returns `FrameStats { min, max, avg, p99, window_fps, frame_count }`.
    /// - `avg = sum_frame_time / frame_count` (0.0 when `frame_count == 0`).
    /// - `p99 = self.p99()`; `window_fps = self.window_fps()`.
    pub fn stats(&self) -> FrameStats {
        if self.frame_count == 0 {
            return FrameStats {
                min: 0.0,
                max: 0.0,
                avg: 0.0,
                p99: 0.0,
                window_fps: 0.0,
                frame_count: 0,
            };
        }
        let avg = self.sum_frame_time / self.frame_count as f32;
        FrameStats {
            min: self.min_frame_time,
            max: self.max_frame_time,
            avg,
            p99: self.p99(),
            window_fps: self.window_fps(),
            frame_count: self.frame_count,
        }
    }

    /// Print a formatted FPS report to stdout.
    ///
    /// # Postconditions (§4)
    /// - `frame_count == 0` → prints `[FPS] no frames sampled yet`.
    /// - `frame_count > 0` → prints frame-time statistics in ms and sliding-window FPS.
    pub fn log_report(&self) {
        if self.frame_count == 0 {
            println!("[FPS] no frames sampled yet");
            return;
        }
        let s = self.stats();
        println!(
            "[FPS] min={:.3}ms max={:.3}ms avg={:.3}ms p99={:.3}ms window_fps={:.1} frames={}",
            s.min * 1000.0,
            s.max * 1000.0,
            s.avg * 1000.0,
            s.p99 * 1000.0,
            s.window_fps,
            s.frame_count,
        );
    }
}

impl Default for FrameMetrics {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// §8 Data Model — core statistics exposed via stats(); frame_count, min, and
// max are pub for direct test assertion (A6, B5, P1–P3 integration tests).
// Unit tests live in tests/frame_rate_test.rs (integration test).
// ============================================================================
