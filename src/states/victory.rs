// Feature #6: Life, Death & Win — VictoryState
use crate::entities::player::{Player, PlayerConfig};
use crate::states::{GameState, LifeState, PlayingState};

pub struct VictoryState {
    pub coins: u32,
    pub level: u32,
    pub blink_phase: f32,
    pub screen_w: f32,
    pub screen_h: f32,
}

impl VictoryState {
    pub fn new(coins: u32, level: u32) -> Self {
        Self { coins, level, blink_phase: 0.0, screen_w: 0.0, screen_h: 0.0 }
    }

    pub fn update(&mut self, dt: f32) -> Option<GameState> {
        self.blink_phase += dt;
        if self.screen_w > 0.0
            && macroquad::input::is_key_pressed(macroquad::input::KeyCode::Space)
        {
            if self.level >= 4 {
                // All levels cleared — restart from level 1
                return Some(Self::make_playing(1, self.screen_w, self.screen_h));
            }
            return Some(Self::make_playing(self.level + 1, self.screen_w, self.screen_h));
        }
        None
    }

    fn make_playing(level: u32, sw: f32, sh: f32) -> GameState {
        let mut player = Player::new(PlayerConfig::default());
        player.pos.y = 584.0;
        player.on_ground = true;
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

        if (self.blink_phase * 2.0) as u32 % 2 == 0 {
            let msg = if self.level >= 4 { "Press Space to Play Again" }
                       else { "Press Space for Next Level" };
            draw_text(msg, cx - 130.0, self.screen_h * 0.61, 18.0, WHITE);
        }
    }
}
