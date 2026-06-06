// Level Select — choose a level (1-8) before starting the game
//
// Shows 8 level tiles in a 2-row × 4-column grid. Player navigates
// with arrow keys and confirms with Enter or Space.

use crate::entities::player::{Player, PlayerConfig};
use crate::states::{GameState, LifeState, PlayingState};

const MAX_LEVELS: usize = 8;

/// Level selection state shown before the game begins.
pub struct LevelSelectState {
    /// Currently highlighted level index (0-7 maps to levels 1-8).
    pub selected: usize,
    /// Screen dimensions for layout.
    pub screen_w: f32,
    pub screen_h: f32,
    /// Previous-frame key states for edge detection.
    prev_left: bool,
    prev_right: bool,
    prev_up: bool,
    prev_down: bool,
    prev_confirm: bool,
}

impl LevelSelectState {
    pub fn new() -> Self {
        Self {
            selected: 0,
            screen_w: 0.0,
            screen_h: 0.0,
            prev_left: false,
            prev_right: false,
            prev_up: false,
            prev_down: false,
            prev_confirm: false,
        }
    }

    pub fn update(&mut self, dt: f32) -> Option<GameState> {
        let _ = dt;
        if self.screen_w <= 0.0 {
            return None;
        }

        let left = macroquad::input::is_key_down(macroquad::input::KeyCode::Left);
        let right = macroquad::input::is_key_down(macroquad::input::KeyCode::Right);
        let up = macroquad::input::is_key_down(macroquad::input::KeyCode::Up);
        let down = macroquad::input::is_key_down(macroquad::input::KeyCode::Down);
        let confirm = macroquad::input::is_key_down(macroquad::input::KeyCode::Enter)
            || macroquad::input::is_key_down(macroquad::input::KeyCode::Space);

        // Edge-detect navigation (4-direction, wrap within 2×4 grid)
        if left && !self.prev_left {
            if self.selected % 4 == 0 {
                self.selected = (self.selected + 3).min(MAX_LEVELS - 1); // wrap to rightmost
            } else {
                self.selected = self.selected.saturating_sub(1);
            }
        }
        if right && !self.prev_right {
            if self.selected % 4 == 3 || self.selected == MAX_LEVELS - 1 {
                self.selected = self.selected - self.selected % 4; // wrap to leftmost
            } else {
                self.selected = (self.selected + 1).min(MAX_LEVELS - 1);
            }
        }
        if up && !self.prev_up {
            if self.selected >= 4 {
                self.selected -= 4;
            } else {
                self.selected += 4; // wrap to bottom row
                if self.selected >= MAX_LEVELS { self.selected -= 4; }
            }
        }
        if down && !self.prev_down {
            let candidate = self.selected + 4;
            if candidate < MAX_LEVELS {
                self.selected = candidate;
            } else {
                self.selected %= 4; // wrap to top row
            }
        }

        self.prev_left = left;
        self.prev_right = right;
        self.prev_up = up;
        self.prev_down = down;

        let confirm_just = confirm && !self.prev_confirm;
        self.prev_confirm = confirm;
        if confirm_just {
            let level_num = (self.selected + 1) as u32;
            let level = crate::level::Level::load(level_num);
            let spawn = level.player_spawn.pos;
            let mut player = Player::new(PlayerConfig::default());
            player.pos.x = spawn.x;
            player.pos.y = spawn.y;
            player.on_ground = true;
            let life_state = LifeState::new();
            let playing = PlayingState::with_level(player, life_state, level_num);
            return Some(GameState::Playing(Box::new(playing)));
        }

        None
    }

    pub fn render(&mut self, _alpha: f32) {
        use macroquad::prelude::*;

        let sw = self.screen_w;
        let sh = self.screen_h;
        if sw <= 0.0 || sh <= 0.0 {
            return;
        }

        clear_background(Color::new(0.05, 0.08, 0.25, 1.0));

        // Title
        let title = "SELECT LEVEL";
        let title_size = 40.0;
        let title_w = title_size * title.len() as f32 * 0.3;
        draw_text(title, sw / 2.0 - title_w, sh * 0.06, title_size, Color::new(1.0, 0.9, 0.1, 1.0));

        // Layout: 2 rows × 4 columns
        let tile_w = 150.0;
        let tile_h = 170.0;
        let gap_x = 20.0;
        let gap_y = 16.0;
        let cols = 4;
        let total_w = cols as f32 * tile_w + (cols - 1) as f32 * gap_x;
        let start_x = (sw - total_w) / 2.0;
        let rows_start_y = sh * 0.18;

        let level_names = [
            "PLAINS", "CAVERN", "SKY", "CASTLE",
            "CUSTOM 5", "CUSTOM 6", "CUSTOM 7", "CUSTOM 8",
        ];
        let level_colors = [
            Color::new(0.30, 0.65, 0.20, 1.0), // green
            Color::new(0.30, 0.25, 0.40, 1.0), // purple
            Color::new(0.40, 0.60, 0.90, 1.0), // blue
            Color::new(0.55, 0.15, 0.15, 1.0), // red
            Color::new(0.20, 0.50, 0.50, 1.0), // teal
            Color::new(0.60, 0.40, 0.15, 1.0), // orange
            Color::new(0.25, 0.25, 0.25, 1.0), // dark
            Color::new(0.50, 0.50, 0.15, 1.0), // olive
        ];

        for i in 0..MAX_LEVELS {
            let row = i / 4;
            let col = i % 4;
            let x = start_x + col as f32 * (tile_w + gap_x);
            let y = rows_start_y + row as f32 * (tile_h + gap_y);
            let is_selected = i == self.selected;

            let bg = if is_selected {
                Color::new(1.0, 0.85, 0.15, 1.0)
            } else {
                Color::new(0.15, 0.15, 0.35, 1.0)
            };
            draw_rectangle(x, y, tile_w, tile_h, bg);

            // Color swatch
            let m = 6.0;
            draw_rectangle(x + m, y + m, tile_w - m * 2.0, tile_h * 0.50, level_colors[i]);

            // Number
            let num_text = format!("{}", i + 1);
            let num_size = 48.0;
            draw_text(
                &num_text,
                x + tile_w / 2.0 - 14.0,
                y + 55.0,
                num_size,
                if is_selected { Color::new(0.1, 0.1, 0.1, 1.0) } else { WHITE },
            );

            // Name
            let name_size = 16.0;
            let name_w = name_size * level_names[i].len() as f32 * 0.32;
            draw_text(
                level_names[i],
                x + tile_w / 2.0 - name_w,
                y + tile_h - 30.0,
                name_size,
                if is_selected { Color::new(0.1, 0.1, 0.1, 1.0) } else { Color::new(0.7, 0.7, 0.8, 1.0) },
            );
        }

        let hint = "ARROWS: Navigate    ENTER/SPACE: Start";
        let hint_size = 16.0;
        let hint_w = hint_size * hint.len() as f32 * 0.28;
        draw_text(hint, sw / 2.0 - hint_w, sh * 0.92, hint_size, Color::new(0.6, 0.6, 0.7, 1.0));
    }
}
