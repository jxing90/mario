// Feature #6: Life, Death & Win — Game state machine and LifeState
// Design Reference: docs/features/6-life-death-win.md §4, §6, §8
//
// Defines the top-level GameState enum that delegates to individual state
// variants (Playing, Dead, GameOver, Victory, Paused, OptionsMenu).
// LifeState holds player progress data (lives, checkpoint, coins) shared
// across the death/respawn/win cycle.

pub mod playing;
pub mod dead;
pub mod game_over;
pub mod victory;
pub mod options;

use crate::state::StateMachine;
use crate::level::Vec2;

/// Tracks persistent player progress across lives.
///
/// # Fields (§8 Data Model)
/// - `lives`: remaining lives (initial 3, 0 = game over).
/// - `checkpoint`: most recent activated checkpoint position, or None.
/// - `coins`: coins collected in this run.
#[derive(Debug, Clone, Copy)]
pub struct LifeState {
    pub lives: u32,
    pub checkpoint: Option<Vec2>,
    pub coins: u32,
}

impl LifeState {
    /// Creates a new LifeState with initial game values.
    pub fn new() -> Self {
        Self {
            lives: 3,
            checkpoint: None,
            coins: 0,
        }
    }

    /// Resets all fields to initial values.
    ///
    /// # Postconditions (§4)
    /// - `lives = 3`, `checkpoint = None`, `coins = 0`
    pub fn reset(&mut self) {
        self.lives = 3;
        self.checkpoint = None;
        self.coins = 0;
    }
}

impl Default for LifeState {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export state structs so they are accessible as crate::states::PlayingState etc.
pub use playing::PlayingState;
pub use dead::DeadState;
pub use game_over::GameOverState;
pub use victory::VictoryState;

/// Top-level game state machine.
///
/// Delegates `update(dt)` and `render(alpha)` to the active variant.
/// State transitions are driven by each variant's `update` returning
/// `Some(GameState)` to signal the next state.
pub enum GameState {
    Playing(Box<PlayingState>),
    Dead(DeadState),
    GameOver(GameOverState),
    Victory(VictoryState),
    Paused,
    OptionsMenu,
}

impl GameState {
    /// Performs a full game reset, returning to Playing with fresh state.
    ///
    /// # Postconditions (§4)
    /// - New `LifeState` with 3 lives, 0 coins, no checkpoint.
    /// - New `Player` at level start position.
    /// - Level unchanged; switches to `Playing` state.
    pub fn full_reset(&mut self) {
        use crate::entities::player::{Player, PlayerConfig};

        let player = Player::new(PlayerConfig::default());
        let life_state = LifeState::new();

        let playing = PlayingState::new(player, life_state);
        *self = GameState::Playing(Box::new(playing));
    }
}

impl StateMachine for GameState {
    fn update(&mut self, dt: f32) {
        // Take ownership of self temporarily to allow transition
        let transition = match self {
            GameState::Playing(state) => {
                state.update(dt);
                None
            }
            GameState::Dead(state) => state.update(dt),
            GameState::GameOver(state) => state.update(dt),
            GameState::Victory(state) => state.update(dt),
            GameState::Paused | GameState::OptionsMenu => {
                // Placeholder — no-op for now
                None
            }
        };

        if let Some(next) = transition {
            *self = next;
        }
    }

    fn render(&mut self, alpha: f32) {
        match self {
            GameState::Playing(state) => state.render(alpha),
            GameState::Dead(state) => state.render(alpha),
            GameState::GameOver(state) => state.render(alpha),
            GameState::Victory(state) => state.render(alpha),
            GameState::Paused | GameState::OptionsMenu => {
                // Placeholder — no-op for now
            }
        }
    }
}
