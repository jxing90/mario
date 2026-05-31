// Game state machine trait — implemented by each game state (Playing, Dead, GameOver, etc.)
// Defines the contract that GameLoop drives each frame: update(dt) then render(alpha).

/// Trait for game state modules driven by the fixed-timestep game loop.
///
/// Each frame, `GameLoop::tick` calls `update(dt)` zero to `max_steps` times
/// (once per consumed fixed timestep), followed by exactly one `render(alpha)`.
///
/// # Contract
/// - `dt` is always `1.0 / 60.0` (fixed timestep invariant).
/// - `alpha = accumulator / dt` is the interpolation factor in [0.0, max_steps+1).
///   `alpha == 0.0` means the simulation is exactly at a step boundary;
///   `alpha > 0.0` means the next simulation step is `alpha * dt` seconds ahead.
pub trait StateMachine {
    /// Advance the simulation by one fixed timestep.
    fn update(&mut self, dt: f32);

    /// Render the current state, interpolated by `alpha` between steps.
    fn render(&mut self, alpha: f32);
}
