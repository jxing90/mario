// Feature #6: Life, Death & Win — GameOverState
// Design Reference: docs/features/6-life-death-win.md §4, §8

use crate::entities::player::{Player, PlayerConfig};
use crate::states::{GameState, LifeState, PlayingState};

pub struct GameOverState {
    pub coins: u32,
    pub blink_phase: f32,
    pub screen_w: f32,
    pub screen_h: f32,
}

impl GameOverState {
    pub fn new(coins: u32) -> Self {
        Self { coins, blink_phase: 0.0, screen_w: 0.0, screen_h: 0.0 }
    }

    pub fn update(&mut self, dt: f32) -> Option<GameState> {
        self.blink_phase += dt;
        if self.screen_w > 0.0
            && macroquad::input::is_key_pressed(macroquad::input::KeyCode::Space)
        {
            let mut player = Player::new(PlayerConfig::default());
            player.pos.y = 584.0;
            player.on_ground = true;
            let life_state = LifeState::new();
            let mut playing = PlayingState::new(player, life_state);
            playing.screen_w = self.screen_w;
            playing.screen_h = self.screen_h;
            return Some(GameState::Playing(Box::new(playing)));
        }
        None
    }

    pub fn render(&mut self, _alpha: f32) {
        if self.screen_w <= 0.0 { return; }
        use macroquad::prelude::*;
        draw_rectangle(0.0, 0.0, self.screen_w, self.screen_h, Color::new(0.0, 0.0, 0.0, 0.65));
        let cx = self.screen_w / 2.0;
        draw_text("GAME OVER", cx - 90.0, self.screen_h * 0.40, 36.0, WHITE);
        let coin_text = format!("Coins: {:03}", self.coins);
        draw_text(&coin_text, cx - 50.0, self.screen_h * 0.50, 24.0, WHITE);
        if (self.blink_phase * 2.0) as u32 % 2 == 0 {
            draw_text("Press Space to Restart", cx - 120.0, self.screen_h * 0.60, 18.0, WHITE);
        }
    }
}
