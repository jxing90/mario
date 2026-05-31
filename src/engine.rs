// Fixed-timestep game loop (FR-018)
//
// Implements the Engine Core feature: Macroquad window with virtual render target,
// 60fps fixed-timestep game loop with accumulator (max 5 catch-up steps),
// and the state-machine update → render pipeline coordinated each frame.
//
// IAPI-011 Provider: GameLoop::apply_display(w, h, fullscreen) responds to
// Display Config (Feature #10) resolution/fullscreen change requests.

use crate::state::StateMachine;

// ============================================================================
// Constants (§6 Implementation Summary)
// ============================================================================

/// Fixed timestep duration: 1/60 second.
const DT: f32 = 1.0 / 60.0;

/// Maximum catch-up simulation steps per frame to prevent spiral of death.
const MAX_STEPS: u32 = 5;

/// Supported display resolutions (FR-018, IAPI-011).
const SUPPORTED_RESOLUTIONS: [(u32, u32); 3] = [
    (1280, 720),
    (1920, 1080),
    (2560, 1440),
];

// ============================================================================
// WindowConfig (§8 Data Model)
// ============================================================================

/// Display configuration for the game window.
#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
}

// ============================================================================
// WindowError (§4 Interface Contract — Raises)
// ============================================================================

/// Errors that may occur during window creation or configuration.
#[derive(Debug, Clone, PartialEq)]
pub enum WindowError {
    /// Macroquad window creation failed.
    InitFailed,
    /// The requested resolution is not in the supported set.
    UnsupportedResolution,
}

// ============================================================================
// GameWindow (§8 Data Model)
// ============================================================================

/// Encapsulates the game window and its current display configuration.
///
/// In the full implementation, this holds a Macroquad `RenderTarget` for the
/// 480×270 virtual canvas with nearest-neighbor filtering.
pub struct GameWindow {
    pub config: WindowConfig,
}

impl GameWindow {
    /// Create a new game window with the given configuration.
    pub fn new(config: WindowConfig) -> Result<Self, WindowError> {
        if !is_supported_resolution(config.width, config.height) {
            return Err(WindowError::UnsupportedResolution);
        }
        Ok(Self { config })
    }
}

// ============================================================================
// GameLoop (§4 Interface Contract, §6 Implementation Summary)
// ============================================================================

/// Fixed-timestep game loop that drives the state machine each frame.
///
/// # Fields (§8 Data Model)
/// - `accumulator`: accumulated real time not yet consumed by simulation steps.
/// - `dt`: the fixed timestep duration (1/60 s). Immutable after construction.
/// - `max_steps`: maximum catch-up simulation steps per frame (5).
/// - `window`: the game window holding display configuration.
pub struct GameLoop {
    pub accumulator: f32,
    pub dt: f32,
    pub max_steps: u32,
    pub window: GameWindow,
}

impl GameLoop {
    /// Create a new game loop with the given window configuration.
    ///
    /// # Preconditions (§4)
    /// - `config.width` and `config.height` must be in the supported resolution set.
    ///
    /// # Postconditions (§4)
    /// - `accumulator = 0.0`
    /// - `dt = 1.0 / 60.0`
    /// - `max_steps = 5`
    /// - Returns `Ok(GameLoop)` on success.
    ///
    /// # Raises (§4)
    /// - `WindowError::UnsupportedResolution` if resolution not in supported set.
    /// - `WindowError::InitFailed` if window creation fails.
    pub fn new(config: WindowConfig) -> Result<Self, WindowError> {
        let window = GameWindow::new(config)?;
        Ok(Self {
            accumulator: 0.0,
            dt: DT,
            max_steps: MAX_STEPS,
            window,
        })
    }

    /// Process a single frame: accumulate `frame_time`, run up to `max_steps`
    /// fixed-timestep updates, then render once.
    ///
    /// This is the per-frame unit of `run()`, exposed for deterministic testing.
    ///
    /// # Algorithm (§6 flow chart)
    /// 1. `accumulator += frame_time`
    /// 2. Clamp: if `accumulator > dt * (max_steps + 1)`, set to ceiling.
    /// 3. While `accumulator >= dt` AND `steps < max_steps`:
    ///    - `state.update(dt)`
    ///    - `accumulator -= dt`
    ///    - `steps += 1`
    /// 4. `alpha = accumulator / dt`
    /// 5. `state.render(alpha)`
    pub fn tick<S: StateMachine + ?Sized>(&mut self, frame_time: f32, state: &mut S) {
        // Step 1: Accumulate real elapsed time.
        self.accumulator += frame_time;

        // Step 2: Clamp accumulator to prevent spiral of death.
        // Threshold = dt * (max_steps + 1) — one extra dt of buffer prevents
        // boundary jitter on spike recovery (§6 Design Rationale).
        let clamp_ceiling = self.dt * ((self.max_steps + 1) as f32);
        if self.accumulator > clamp_ceiling {
            self.accumulator = clamp_ceiling;
        }

        // Step 3: Consume accumulator in fixed-dt steps.
        let mut steps: u32 = 0;
        while self.accumulator >= self.dt && steps < self.max_steps {
            state.update(self.dt);
            self.accumulator -= self.dt;
            steps += 1;
        }

        // Step 4–5: Compute interpolation alpha and render.
        let alpha = self.accumulator / self.dt;
        state.render(alpha);
    }

    /// Apply display configuration changes (IAPI-011).
    ///
    /// # Preconditions (§4)
    /// - `w, h` are the requested resolution; `fullscreen` is the fullscreen flag.
    ///
    /// # Postconditions (§4)
    /// - If `(w, h)` is in the supported resolution set:
    ///   `WindowConfig` is updated to reflect the new settings.
    /// - If `(w, h)` is NOT supported: this is a silent no-op (config unchanged).
    ///
    /// # Raises (§4)
    /// - No error is raised; unsupported resolutions are silently ignored.
    pub fn apply_display(&mut self, w: u32, h: u32, fullscreen: bool) {
        if is_supported_resolution(w, h) {
            self.window.config.width = w;
            self.window.config.height = h;
            self.window.config.fullscreen = fullscreen;
        }
    }

    /// Run the game loop until the window is closed.
    ///
    /// This is the infinite-loop wrapper around [`tick`]. Each iteration:
    /// measures the real frame time via Macroquad, calls [`tick`] to drive
    /// the state machine, then awaits the next frame.
    ///
    /// # Returns (§4)
    /// Divergent (`!`) — the loop exits only when the window is closed,
    /// at which point the async context terminates.
    ///
    /// [`tick`]: GameLoop::tick
    pub async fn run(&mut self, state: &mut dyn StateMachine) {
        loop {
            let frame_time = macroquad::time::get_frame_time();
            self.tick(frame_time, state);
            macroquad::window::next_frame().await;
        }
    }
}

// ============================================================================
// Private helpers
// ============================================================================

/// Returns true if (w, h) is one of the supported display resolutions.
fn is_supported_resolution(w: u32, h: u32) -> bool {
    SUPPORTED_RESOLUTIONS.iter().any(|&(rw, rh)| rw == w && rh == h)
}
