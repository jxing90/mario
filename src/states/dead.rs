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
/// - `current_level`: level number to respawn into (preserved from PlayingState).
/// - `death_timer`: remaining animation time (initial 1.5s).
/// - `checkpoint`: most recent activated checkpoint, or None.
/// - `player_x`, `player_y`: death position for rendering the animation.
/// - `bounce_phase`: 0.0 → 1.0 during bounce-up segment (first 0.3s).
/// - `screen_w`, `screen_h`: viewport dimensions (forwarded from PlayingState).
pub struct DeadState {
    pub lives: u32,
    pub coins: u32,
    pub current_level: u32,
    pub death_timer: f32,
    pub checkpoint: Option<Vec2>,
    pub player_x: f32,
    pub player_y: f32,
    pub bounce_phase: f32,
    pub screen_w: f32,
    pub screen_h: f32,
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
    pub fn new(
        lives: u32,
        coins: u32,
        current_level: u32,
        checkpoint: Option<Vec2>,
        player_pos: Vec2,
        screen_w: f32,
        screen_h: f32,
    ) -> Self {
        Self {
            lives,
            coins,
            current_level,
            death_timer: DEATH_DURATION,
            checkpoint,
            player_x: player_pos.x,
            player_y: player_pos.y,
            bounce_phase: 0.0,
            screen_w,
            screen_h,
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

                let mut playing = PlayingState::with_level(player, life_state, self.current_level);
                playing.invuln_timer = RESPAWN_INVULN_DURATION;
                playing.flicker_phase = 0.0;

                Some(GameState::Playing(Box::new(playing)))
            } else {
                // Game Over
                let game_over = GameOverState::new(self.coins, self.current_level);
                Some(GameState::GameOver(game_over))
            }
        } else {
            None
        }
    }

    /// Renders the death animation: bounce phase (first 0.3s) then fall+fade (remaining 1.2s).
    ///
    /// # Visual Contract (§Visual Rendering Contract)
    /// - Phase 1 (death_timer > 1.2): player sprite bounces up 0→24px.
    /// - Phase 2 (death_timer <= 1.2): player sprite falls with alpha fade 1.0→0.0.
    pub fn render(&mut self, _alpha: f32) {
        use macroquad::prelude::*;

        if self.screen_w <= 0.0 || self.screen_h <= 0.0 {
            return;
        }

        // Dark overlay
        draw_rectangle(0.0, 0.0, self.screen_w, self.screen_h, Color::new(0.0, 0.0, 0.0, 0.7));

        // Compute player's screen position
        // In the actual game, the player would be at camera coordinates.
        // For the death animation, we place the player at center-screen with bounce offset.
        let cx = self.screen_w / 2.0;
        let ground_y = self.screen_h * 0.75;

        // Bounce up during first 0.3s (phase 0→1), then fall
        let bounce_offset = if self.death_timer > DEATH_DURATION - BOUNCE_DURATION {
            // Bounce up: 0 → 24px
            self.bounce_phase * 24.0
        } else {
            // Fall back down
            24.0
        };

        let player_y = ground_y - bounce_offset;

        // Alpha fade: full opacity during bounce, fade to 0 during fall
        let alpha = if self.death_timer > DEATH_DURATION - BOUNCE_DURATION {
            1.0
        } else {
            let fall_elapsed = (DEATH_DURATION - BOUNCE_DURATION) - self.death_timer;
            let fall_duration = DEATH_DURATION - BOUNCE_DURATION;
            (1.0 - fall_elapsed / fall_duration).clamp(0.0, 1.0)
        };

        // Draw Mario sprite (simplified — red hat + blue overalls)
        let hat_color = Color::new(0.85, 0.15, 0.1, alpha);
        let skin_color = Color::new(1.0, 0.75, 0.55, alpha);
        let overall_color = Color::new(0.1, 0.3, 0.9, alpha);
        let shoe_color = Color::new(0.45, 0.25, 0.15, alpha);

        let pw = 24.0; // player width on screen
        let ph = 32.0; // player height

        let px = cx - pw / 2.0;
        let py = player_y - ph;

        // Hat
        draw_rectangle(px, py, pw, ph * 0.25, hat_color);
        // Face
        draw_rectangle(px, py + ph * 0.25, pw, ph * 0.2, skin_color);
        // Eye
        draw_rectangle(px + pw * 0.55, py + ph * 0.28, pw * 0.2, ph * 0.08, BLACK);
        // Overall
        draw_rectangle(px, py + ph * 0.45, pw, ph * 0.35, overall_color);
        // Shoes
        draw_rectangle(px + 2.0, py + ph * 0.8, pw * 0.35, ph * 0.2, shoe_color);
        draw_rectangle(px + pw * 0.55, py + ph * 0.8, pw * 0.35, ph * 0.2, shoe_color);

        // Remaining lives text
        if self.lives > 0 {
            let lives_text = format!("x{}", self.lives);
            draw_text(
                &lives_text,
                cx + pw / 2.0 + 12.0,
                player_y - ph / 2.0,
                24.0,
                Color::new(1.0, 0.3, 0.3, alpha),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// DeadState should store all fields passed to new().
    #[test]
    fn test_dead_state_new_stores_fields() {
        let pos = Vec2 { x: 300.0, y: 500.0 };
        let ds = DeadState::new(2, 10, 3, None, pos, 1280.0, 720.0);
        assert_eq!(ds.lives, 2);
        assert_eq!(ds.coins, 10);
        assert_eq!(ds.current_level, 3);
        assert!(ds.checkpoint.is_none());
        assert_eq!(ds.player_x, 300.0);
        assert_eq!(ds.player_y, 500.0);
        assert_eq!(ds.death_timer, 1.5);
        assert_eq!(ds.bounce_phase, 0.0);
        assert_eq!(ds.screen_w, 1280.0);
        assert_eq!(ds.screen_h, 720.0);
    }

    /// After 1.5s with lives > 0, DeadState should transition to Playing.
    #[test]
    fn test_dead_state_respawn_when_lives_remain() {
        let pos = Vec2 { x: 300.0, y: 500.0 };
        let mut ds = DeadState::new(2, 10, 2, Some(Vec2 { x: 400.0, y: 500.0 }), pos, 1280.0, 720.0);
        // Run enough frames to exhaust death timer (1.5s / (1/60) ≈ 90 frames)
        let dt = 1.0 / 60.0;
        for _ in 0..95 {
            let result = ds.update(dt);
            if result.is_some() {
                // Should only transition at the end
                match result.unwrap() {
                    GameState::Playing(p) => {
                        assert_eq!(p.player.lives, 2);
                        assert_eq!(p.player.coins, 10);
                        // Should respawn at checkpoint
                        assert!((p.player.pos.x - 400.0).abs() < 1.0);
                        assert!((p.player.pos.y - 500.0).abs() < 1.0);
                        return; // test passed
                    }
                    other => panic!("Expected Playing, got {:?}", std::mem::discriminant(&other)),
                }
            }
        }
        panic!("DeadState did not transition within 95 frames");
    }

    /// After 1.5s with lives == 0, DeadState should transition to GameOver.
    #[test]
    fn test_dead_state_game_over_when_no_lives() {
        let pos = Vec2 { x: 300.0, y: 500.0 };
        let mut ds = DeadState::new(0, 15, 4, None, pos, 1280.0, 720.0);
        let dt = 1.0 / 60.0;
        for _ in 0..95 {
            let result = ds.update(dt);
            if result.is_some() {
                match result.unwrap() {
                    GameState::GameOver(govr) => {
                        assert_eq!(govr.coins, 15);
                        return; // test passed
                    }
                    other => panic!("Expected GameOver, got {:?}", std::mem::discriminant(&other)),
                }
            }
        }
        panic!("DeadState did not transition within 95 frames");
    }

    /// Before timer expires, DeadState returns None (stay in death animation).
    #[test]
    fn test_dead_state_no_transition_before_timer_expires() {
        let pos = Vec2 { x: 300.0, y: 500.0 };
        let mut ds = DeadState::new(2, 10, 1, None, pos, 1280.0, 720.0);
        let dt = 1.0 / 60.0;
        // After 0.5s, should still be in death animation
        for _ in 0..30 {
            assert!(ds.update(dt).is_none(), "Should not transition before 1.5s");
        }
    }

    /// DeadState.render() with zero screen dimensions should early-return safely.
    #[test]
    fn test_dead_state_render_zero_screen_no_panic() {
        let pos = Vec2 { x: 300.0, y: 500.0 };
        let mut ds = DeadState::new(2, 10, 1, None, pos, 0.0, 0.0);
        ds.render(0.0); // Should early-return, no panic (no GL context needed for early return)
    }

    /// Bounce phase should progress during first 0.3s.
    #[test]
    fn test_dead_state_bounce_phase() {
        let pos = Vec2 { x: 300.0, y: 500.0 };
        let mut ds = DeadState::new(2, 10, 1, None, pos, 1280.0, 720.0);
        assert_eq!(ds.bounce_phase, 0.0);
        // After 0.15s (9 frames), bounce phase should be ~0.5
        let dt = 1.0 / 60.0;
        for _ in 0..9 {
            ds.update(dt);
        }
        assert!(ds.bounce_phase > 0.4 && ds.bounce_phase < 0.6,
            "bounce_phase should be ~0.5 after 0.15s, got {}", ds.bounce_phase);
        // After 0.3s, bounce phase should be ~1.0
        for _ in 0..9 {
            ds.update(dt);
        }
        assert!(ds.bounce_phase > 0.9,
            "bounce_phase should be ~1.0 after 0.3s, got {}", ds.bounce_phase);
    }
}
