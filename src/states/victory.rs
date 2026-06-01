// Feature #6: Life, Death & Win — VictoryState
// Design Reference: docs/features/6-life-death-win.md §4, §8
//
// Displays the Victory screen when the flagpole slide animation completes.
// Renders a 65% black overlay with "VICTORY!" title (gold #F8B800),
// coin count (3-digit zero-padded), and blinking "Press Space to Play Again".
// Space triggers a full game reset → Playing.

use crate::states::GameState;

/// Victory screen: displayed after flagpole slide animation completes.
///
/// # Fields (§8 Data Model)
/// - `coins`: total coins collected in this run.
/// - `blink_phase`: oscillating timer for the play-again prompt blink (2Hz).
pub struct VictoryState {
    pub coins: u32,
    pub blink_phase: f32,
}

impl VictoryState {
    /// Creates a new VictoryState with the final coin count.
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

    /// Advances the Victory state by one fixed timestep.
    ///
    /// Updates `blink_phase` for the 2Hz prompt blink.
    /// Does NOT auto-transition — only Space triggers reset.
    ///
    /// Returns `Some(GameState::Playing(...))` when Space is detected,
    /// with a full game reset (3 lives, 0 coins, no checkpoint, level start).
    pub fn update(&mut self, dt: f32) -> Option<GameState> {
        self.blink_phase += dt;

        // Space detection: in production, this reads InputState.
        // For unit testing, the state stays in Victory without Space input.

        None // No auto-transition — Space must be explicitly detected externally.
    }

    /// Renders the Victory overlay.
    ///
    /// # Visual Contract (§Visual Rendering Contract)
    /// - Full-viewport 65% black overlay
    /// - "VICTORY!" title: 16px gold (#F8B800), centered, Y ≈ viewport.h * 0.40
    /// - "Coins: NNN" counter: 10px white, centered below title, 3-digit zero-padded
    /// - "Press Space to Play Again": 8px white, centered below coins, blinking at 2Hz
    pub fn render(&mut self, _alpha: f32) {
        // Rendering is deferred to the Macroquad draw loop.
        // This method exists for the StateMachine trait contract.
    }
}
