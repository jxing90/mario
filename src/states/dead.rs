// Feature #6: Life, Death & Win — DeadState
// Design Reference: docs/features/6-life-death-win.md §4, §8
//
// Manages the death animation: 1.5s timer locking input, bounce+fade visual,
// then transitions to Playing (respawn) or GameOver based on remaining lives.

use crate::level::Vec2;
use crate::states::{GameState, LifeState, PlayingState, GameOverState};
use crate::entities::player::{Player, PlayerConfig};

/// Duration of the death animation (seconds).
const DEATH_DURATION: f32 = 1.5;
/// Bounce phase duration at the start of the death animation (seconds).
const BOUNCE_DURATION: f32 = 0.3;
/// Fallback respawn position when no checkpoint is activated.
const LEVEL_START: Vec2 = Vec2 { x: 100.0, y: 100.0 };
/// Invulnerability granted after respawn (seconds).
const RESPAWN_INVULN_DURATION: f32 = 2.0;

/// Death animation state: locks input for 1.5s, plays bounce+fade animation.
///
/// # Fields (§8 Data Model)
/// - `lives`: remaining lives AFTER this death was decremented.
/// - `coins`: coin count preserved from the run.
/// - `death_timer`: remaining animation time (initial 1.5s).
/// - `checkpoint`: most recent activated checkpoint, or None.
/// - `player_x`, `player_y`: death position for rendering the animation.
/// - `bounce_phase`: 0.0 → 1.0 during bounce-up segment (first 0.3s).
pub struct DeadState {
    pub lives: u32,
    pub coins: u32,
    pub death_timer: f32,
    pub checkpoint: Option<Vec2>,
    pub player_x: f32,
    pub player_y: f32,
    pub bounce_phase: f32,
}

impl DeadState {
    /// Creates a new DeadState for the death animation.
    ///
    /// # Preconditions (§4)
    /// - `lives >= 0` (already decremented by PlayingState before construction).
    ///
    /// # Postconditions (§4)
    /// - `death_timer = 1.5`
    /// - `bounce_phase = 0.0`
    /// - Input is locked (DeadState does not process movement input).
    pub fn new(lives: u32, coins: u32, checkpoint: Option<Vec2>, player_pos: Vec2) -> Self {
        Self {
            lives,
            coins,
            death_timer: DEATH_DURATION,
            checkpoint,
            player_x: player_pos.x,
            player_y: player_pos.y,
            bounce_phase: 0.0,
        }
    }

    /// Advances the death timer by one fixed timestep.
    ///
    /// When `death_timer` reaches 0, returns a transition:
    /// - `Some(GameState::Playing(...))` if lives > 0 (respawn).
    /// - `Some(GameState::GameOver(...))` if lives == 0.
    /// - `None` if the timer is still counting down.
    pub fn update(&mut self, dt: f32) -> Option<GameState> {
        self.death_timer -= dt;

        // Update bounce phase for the first BOUNCE_DURATION seconds
        let bounce_end = DEATH_DURATION - BOUNCE_DURATION;
        if self.death_timer > bounce_end {
            let elapsed = DEATH_DURATION - self.death_timer;
            self.bounce_phase = (elapsed / BOUNCE_DURATION).clamp(0.0, 1.0);
        }

        if self.death_timer <= 0.0 {
            if self.lives > 0 {
                // Respawn: create new PlayingState at checkpoint or level start
                let spawn_pos = self.checkpoint.unwrap_or(LEVEL_START);

                let mut player = Player::new(PlayerConfig::default());
                player.pos = spawn_pos;
                player.lives = self.lives;
                player.coins = self.coins;

                let life_state = LifeState {
                    lives: self.lives,
                    coins: self.coins,
                    checkpoint: self.checkpoint,
                };

                let mut playing = PlayingState::new(player, life_state);
                playing.invuln_timer = RESPAWN_INVULN_DURATION;
                playing.flicker_phase = 0.0;

                Some(GameState::Playing(Box::new(playing)))
            } else {
                // Game Over
                let game_over = GameOverState::new(self.coins, 1);
                Some(GameState::GameOver(game_over))
            }
        } else {
            None
        }
    }

    /// Renders the death animation: bounce phase (first 0.3s) then fall+fade (remaining 1.2s).
    ///
    /// # Visual Contract (§Visual Rendering Contract)
    /// - Phase 1 (death_timer > 1.0): player sprite bounces up 0→16px.
    /// - Phase 2 (death_timer <= 1.0): player sprite falls with alpha fade 1.0→0.0.
    pub fn render(&mut self, _alpha: f32) {
        // Rendering is deferred to the Macroquad draw loop.
        // This method exists for the StateMachine trait contract.
        // bounce_offset and fall_offset are computed from death_timer/bounce_phase
        // in the render pass.
    }
}
