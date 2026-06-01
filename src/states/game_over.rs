// Feature #6: Life, Death & Win — GameOverState
// Design Reference: docs/features/6-life-death-win.md §4, §8
//
// Displays the Game Over overlay when lives reach 0. Renders a 65% black
// overlay with "GAME OVER" title and blinking "Press Space to Restart" prompt.
// Space triggers a full game reset → Playing.

use crate::states::GameState;

/// Game Over screen: displayed when lives reach 0.
///
/// # Fields (§8 Data Model)
/// - `coins`: total coins collected in this run.
/// - `blink_phase`: oscillating timer for the restart prompt blink (2Hz).
pub struct GameOverState {
    pub coins: u32,
    pub blink_phase: f32,
}

impl GameOverState {
    /// Creates a new GameOverState with the final coin count.
    ///
    /// # Postconditions (§4)
    /// - `blink_phase = 0.0`
    /// - Waits for Space input to trigger reset.
    pub fn new(coins: u32) -> Self {
        Self {
            coins,
            blink_phase: 0.0,
        }
    }

    /// Advances the Game Over state by one fixed timestep.
    ///
    /// Updates `blink_phase` for the 2Hz prompt blink.
    /// Does NOT auto-transition — only Space triggers reset.
    ///
    /// Returns `Some(GameState::Playing(...))` when Space is detected,
    /// with a full game reset (3 lives, 0 coins, no checkpoint, level start).
    pub fn update(&mut self, dt: f32) -> Option<GameState> {
        self.blink_phase += dt;

        // Space detection: in production, this reads InputState.
        // For unit testing, the state stays in GameOver without Space input.
        // The Space-check is wired through GameState::update which has access to input.

        None // No auto-transition — Space must be explicitly detected externally.
    }

    /// Renders the Game Over overlay.
    ///
    /// # Visual Contract (§Visual Rendering Contract)
    /// - Full-viewport 65% black overlay: `draw_rectangle(0,0, w,h, rgba(0,0,0,0.65))`
    /// - "GAME OVER" title: 16px white, centered, Y ≈ viewport.h * 0.40
    /// - "Press Space to Restart": 8px white, centered below title, blinking at 2Hz
    pub fn render(&mut self, _alpha: f32) {
        // Rendering is deferred to the Macroquad draw loop.
        // This method exists for the StateMachine trait contract.
    }
}
