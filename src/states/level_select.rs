// Level Select — choose a level (1-4) before starting the game
//
// Shows a menu with 4 level tiles. Player navigates with Left/Right arrows
// and confirms with Enter or Space. Esc returns to title (not implemented yet).

use crate::entities::player::{Player, PlayerConfig};
use crate::states::{GameState, LifeState, PlayingState};

/// Level selection state shown before the game begins.
pub struct LevelSelectState {
    /// Currently highlighted level index (0-3 maps to levels 1-4).
    pub selected: usize,
    /// Screen dimensions for layout.
    pub screen_w: f32,
    pub screen_h: f32,
    /// Previous-frame key states for edge detection.
    prev_left: bool,
    prev_right: bool,
    prev_confirm: bool,
}

impl LevelSelectState {
    /// Creates a new LevelSelectState with the first level highlighted.
    pub fn new() -> Self {
        Self {
            selected: 0,
            screen_w: 0.0,
            screen_h: 0.0,
            prev_left: false,
            prev_right: false,
            prev_confirm: false,
        }
    }

    /// Updates the level select menu. Returns Some(GameState::Playing(...)) when
    /// the player confirms a level, or None to stay on this screen.
    pub fn update(&mut self, dt: f32) -> Option<GameState> {
        let _ = dt; // unused — menu is static

        // Skip input if screen not yet sized
        if self.screen_w <= 0.0 {
            return None;
        }

        // Read keyboard
        let left = macroquad::input::is_key_down(macroquad::input::KeyCode::Left);
        let right = macroquad::input::is_key_down(macroquad::input::KeyCode::Right);
        let confirm = macroquad::input::is_key_down(macroquad::input::KeyCode::Enter)
            || macroquad::input::is_key_down(macroquad::input::KeyCode::Space);

        // Edge detect: navigate
        if left && !self.prev_left {
            self.selected = self.selected.saturating_sub(1);
        }
        if right && !self.prev_right {
            if self.selected < 3 {
                self.selected += 1;
            }
        }
        self.prev_left = left;
        self.prev_right = right;

        // Edge detect: confirm
        let confirm_just = confirm && !self.prev_confirm;
        self.prev_confirm = confirm;
        if confirm_just {
            let level_num = (self.selected + 1) as u32;
            let mut player = Player::new(PlayerConfig::default());
            player.pos.y = 600.0; // feet on ground
            player.on_ground = true;
            let life_state = LifeState::new();
            let playing = PlayingState::with_level(player, life_state, level_num);
            return Some(GameState::Playing(Box::new(playing)));
        }

        None
    }

    /// Renders the level select screen with 4 level tiles in a row.
    pub fn render(&mut self, _alpha: f32) {
        use macroquad::prelude::*;

        let sw = self.screen_w;
        let sh = self.screen_h;
        if sw <= 0.0 || sh <= 0.0 {
            return;
        }

        // Background: dark blue gradient-like
        clear_background(Color::new(0.05, 0.08, 0.25, 1.0));

        // Title
        let title = "SELECT LEVEL";
        let title_size = 48.0;
        let title_w = title_size * title.len() as f32 * 0.35; // rough estimate
        draw_text(
            title,
            sw / 2.0 - title_w,
            sh * 0.15,
            title_size,
            Color::new(1.0, 0.9, 0.1, 1.0),
        );

        // Level tile layout
        let tile_w = 160.0;
        let tile_h = 200.0;
        let gap = 24.0;
        let total_w = 4.0 * tile_w + 3.0 * gap;
        let start_x = (sw - total_w) / 2.0;
        let tile_y = sh * 0.35;

        let level_names = ["PLAINS", "CAVERN", "SKY", "CASTLE"];
        let level_colors = [
            Color::new(0.30, 0.65, 0.20, 1.0), // green
            Color::new(0.30, 0.25, 0.40, 1.0), // purple-gray
            Color::new(0.40, 0.60, 0.90, 1.0), // blue
            Color::new(0.55, 0.15, 0.15, 1.0), // red-dark
        ];

        for i in 0..4 {
            let x = start_x + i as f32 * (tile_w + gap);
            let is_selected = i == self.selected;

            // Tile background
            let bg = if is_selected {
                Color::new(1.0, 0.85, 0.15, 1.0) // gold highlight
            } else {
                Color::new(0.15, 0.15, 0.35, 1.0) // dark tile
            };
            draw_rectangle(x, tile_y, tile_w, tile_h, bg);

            // Inner color swatch
            let swatch_margin = 8.0;
            draw_rectangle(
                x + swatch_margin,
                tile_y + swatch_margin,
                tile_w - swatch_margin * 2.0,
                tile_h * 0.55,
                level_colors[i],
            );

            // Level number
            let num_text = format!("{}", i + 1);
            let num_size = 56.0;
            draw_text(
                &num_text,
                x + tile_w / 2.0 - 16.0,
                tile_y + 70.0,
                num_size,
                if is_selected { Color::new(0.1, 0.1, 0.1, 1.0) } else { WHITE },
            );

            // Level name
            let name_size = 18.0;
            let name_w = name_size * level_names[i].len() as f32 * 0.35;
            draw_text(
                level_names[i],
                x + tile_w / 2.0 - name_w,
                tile_y + tile_h - 36.0,
                name_size,
                if is_selected { Color::new(0.1, 0.1, 0.1, 1.0) } else { Color::new(0.7, 0.7, 0.8, 1.0) },
            );
        }

        // Instructions
        let hint = "ARROWS: Select    ENTER/SPACE: Start";
        let hint_size = 18.0;
        let hint_w = hint_size * hint.len() as f32 * 0.3;
        draw_text(
            hint,
            sw / 2.0 - hint_w,
            sh * 0.85,
            hint_size,
            Color::new(0.6, 0.6, 0.7, 1.0),
        );
    }
}
