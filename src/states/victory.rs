// Feature #6: Life, Death & Win — VictoryState
use crate::entities::player::{Player, PlayerConfig, PlayerState};
use crate::states::{GameState, LifeState, PlayingState};

pub struct VictoryState {
    pub coins: u32,
    pub lives: u32,
    pub player_state: PlayerState,
    pub level: u32,
    pub blink_phase: f32,
    pub screen_w: f32,
    pub screen_h: f32,
}

impl VictoryState {
    /// Creates a victory screen that carries over player progress to the next level.
    /// `coins`, `lives`, and `player_state` are preserved from the completed level.
    pub fn new(coins: u32, lives: u32, player_state: PlayerState, level: u32) -> Self {
        Self {
            coins, lives, player_state, level,
            blink_phase: 0.0, screen_w: 0.0, screen_h: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) -> Option<GameState> {
        self.blink_phase += dt;
        if self.screen_w > 0.0
            && macroquad::input::is_key_pressed(macroquad::input::KeyCode::Space)
        {
            let next_level = if self.level >= 4 { 1 } else { self.level + 1 };
            return Some(self.make_playing(next_level, self.screen_w, self.screen_h));
        }
        None
    }

    fn make_playing(&self, level: u32, sw: f32, sh: f32) -> GameState {
        let mut player = Player::new(PlayerConfig::default());
        player.pos.y = 600.0;
        player.on_ground = true;
        // Carry over player progress from previous level
        player.coins = self.coins;
        player.lives = self.lives;
        player.state = self.player_state;
        let life_state = LifeState::new();
        let mut playing = PlayingState::with_level(player, life_state, level);
        playing.screen_w = sw;
        playing.screen_h = sh;
        GameState::Playing(Box::new(playing))
    }

    pub fn render(&mut self, _alpha: f32) {
        if self.screen_w <= 0.0 { return; }
        use macroquad::prelude::*;
        draw_rectangle(0.0, 0.0, self.screen_w, self.screen_h, Color::new(0.0, 0.0, 0.0, 0.65));
        let cx = self.screen_w / 2.0;

        if self.level >= 4 {
            draw_text("ALL LEVELS CLEAR!", cx - 150.0, self.screen_h * 0.33, 32.0, GOLD);
        } else {
            draw_text("VICTORY!", cx - 80.0, self.screen_h * 0.33, 40.0, Color::new(0.97, 0.72, 0.0, 1.0));
        }

        draw_text(&format!("World 1-{} Complete", self.level), cx - 120.0, self.screen_h * 0.43, 22.0, WHITE);
        draw_text(&format!("Coins: {:03}", self.coins), cx - 50.0, self.screen_h * 0.51, 24.0, WHITE);
        draw_text(&format!("Lives: {}", self.lives), cx - 40.0, self.screen_h * 0.57, 20.0, WHITE);

        if (self.blink_phase * 2.0) as u32 % 2 == 0 {
            let msg = if self.level >= 4 { "Press Space to Play Again" }
                       else { "Press Space for Next Level" };
            draw_text(msg, cx - 130.0, self.screen_h * 0.66, 18.0, WHITE);
        }
    }
}
