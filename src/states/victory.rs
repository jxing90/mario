// Feature #6: Life, Death & Win — VictoryState
// Design Reference: docs/features/6-life-death-win.md §4, §8

use crate::entities::player::{Player, PlayerConfig};
use crate::states::{GameState, LifeState, PlayingState};

pub struct VictoryState {
    pub coins: u32,
    pub blink_phase: f32,
    pub screen_w: f32,
    pub screen_h: f32,
}

impl VictoryState {
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
        // 65% black overlay
        draw_rectangle(0.0, 0.0, self.screen_w, self.screen_h, Color::new(0.0, 0.0, 0.0, 0.65));
        let cx = self.screen_w / 2.0;
        let title_size = 40.0;
        draw_text("VICTORY!", cx - 80.0, self.screen_h * 0.35, title_size, Color::new(0.97, 0.72, 0.0, 1.0));
        let coin_text = format!("Coins: {:03}", self.coins);
        draw_text(&coin_text, cx - 50.0, self.screen_h * 0.48, 24.0, WHITE);
        // Blinking restart prompt at 2Hz
        if (self.blink_phase * 2.0) as u32 % 2 == 0 {
            draw_text("Press Space to Play Again", cx - 130.0, self.screen_h * 0.58, 18.0, WHITE);
        }
    }
}
