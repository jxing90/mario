// Level Select — dynamically scans assets/levels/*.json, no upper limit.
//
// Shows a scrollable 4-column grid. Navigate with arrow keys,
// auto-scrolls to keep the selected tile visible.

use crate::entities::player::{Player, PlayerConfig};
use crate::states::{GameState, LifeState, PlayingState};

const COLS: usize = 4;
const VISIBLE_ROWS: usize = 2;

/// Level selection state shown before the game begins.
pub struct LevelSelectState {
    /// Sorted list of available level numbers (from filesystem).
    levels: Vec<u32>,
    /// Cached level names (parallel to `levels`).
    level_names: Vec<String>,
    /// Index into `levels` of the currently highlighted tile.
    selected: usize,
    /// Top visible row (in tile-grid row units, 0-based).
    scroll_row: usize,
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
        let levels = crate::level::list_levels();
        let level_names: Vec<String> = levels.iter().map(|&n| {
            let level = crate::level::Level::load(n);
            level.name
        }).collect();
        // Default select level 1 if available, else first.
        let sel = levels.iter().position(|&n| n == 1).unwrap_or(0);
        Self {
            levels,
            level_names,
            selected: sel,
            scroll_row: 0,
            screen_w: 0.0,
            screen_h: 0.0,
            prev_left: false,
            prev_right: false,
            prev_up: false,
            prev_down: false,
            prev_confirm: false,
        }
    }

    /// Re-scan levels (useful after editor creates/deletes files).
    pub fn refresh_levels(&mut self) {
        self.levels = crate::level::list_levels();
        self.level_names = self.levels.iter().map(|&n| {
            crate::level::Level::load(n).name
        }).collect();
        if self.selected >= self.levels.len() && !self.levels.is_empty() {
            self.selected = self.levels.len() - 1;
        }
        // Clamp scroll
        let max_scroll = self.max_scroll();
        if self.scroll_row > max_scroll {
            self.scroll_row = max_scroll;
        }
    }

    fn max_scroll(&self) -> usize {
        let total_rows = (self.levels.len() + COLS - 1) / COLS;
        if total_rows <= VISIBLE_ROWS { 0 } else { total_rows - VISIBLE_ROWS }
    }

    fn selected_row(&self) -> usize { self.selected / COLS }
    fn selected_col(&self) -> usize { self.selected % COLS }

    pub fn update(&mut self, dt: f32) -> Option<GameState> {
        let _ = dt;
        if self.screen_w <= 0.0 || self.levels.is_empty() {
            return None;
        }

        let left = macroquad::input::is_key_down(macroquad::input::KeyCode::Left);
        let right = macroquad::input::is_key_down(macroquad::input::KeyCode::Right);
        let up = macroquad::input::is_key_down(macroquad::input::KeyCode::Up);
        let down = macroquad::input::is_key_down(macroquad::input::KeyCode::Down);
        let confirm = macroquad::input::is_key_down(macroquad::input::KeyCode::Enter)
            || macroquad::input::is_key_down(macroquad::input::KeyCode::Space);

        let total = self.levels.len();
        let total_rows = (total + COLS - 1) / COLS;

        // Edge-detect left
        if left && !self.prev_left {
            let row = self.selected_row();
            let col = self.selected_col();
            if col == 0 {
                // Wrap to last column of this row
                let last_col = if row == total_rows - 1 {
                    // Last row may be partial
                    (total - 1) % COLS
                } else {
                    COLS - 1
                };
                self.selected = row * COLS + last_col;
            } else {
                self.selected = self.selected.saturating_sub(1);
            }
        }
        // Edge-detect right
        if right && !self.prev_right {
            let row = self.selected_row();
            let col = self.selected_col();
            let last_in_row = if row == total_rows - 1 {
                (total - 1) % COLS
            } else {
                COLS - 1
            };
            if col >= last_in_row || col == COLS - 1 {
                self.selected = row * COLS; // wrap to first
            } else {
                self.selected = (self.selected + 1).min(total - 1);
            }
        }
        // Edge-detect up
        if up && !self.prev_up {
            if self.selected_row() == 0 {
                // Wrap to bottom row
                let bottom_row = total_rows - 1;
                let col = self.selected_col();
                let max_col = if bottom_row == total_rows - 1 {
                    (total - 1) % COLS
                } else {
                    COLS - 1
                };
                let c = col.min(max_col);
                self.selected = (bottom_row * COLS + c).min(total - 1);
            } else {
                let row = self.selected_row();
                let col = self.selected_col();
                let new_row = row - 1;
                let max_col = if new_row == total_rows - 1 {
                    (total - 1) % COLS
                } else {
                    COLS - 1
                };
                let c = col.min(max_col);
                self.selected = (new_row * COLS + c).min(total - 1);
            }
        }
        // Edge-detect down
        if down && !self.prev_down {
            let row = self.selected_row();
            if row == total_rows - 1 {
                let col = self.selected_col();
                let max_col_top = if total_rows > 1 {
                    // first row is always full unless total < COLS
                    if total <= COLS { total - 1 } else { COLS - 1 }
                } else {
                    total - 1
                };
                self.selected = col.min(max_col_top);
            } else {
                let new_row = row + 1;
                let col = self.selected_col();
                let max_col = if new_row == total_rows - 1 {
                    (total - 1) % COLS
                } else {
                    COLS - 1
                };
                let c = col.min(max_col);
                self.selected = (new_row * COLS + c).min(total - 1);
            }
        }

        // Auto-scroll
        let sel_row = self.selected_row();
        if sel_row < self.scroll_row {
            self.scroll_row = sel_row;
        } else if sel_row >= self.scroll_row + VISIBLE_ROWS {
            self.scroll_row = sel_row - VISIBLE_ROWS + 1;
        }
        if self.scroll_row > self.max_scroll() {
            self.scroll_row = self.max_scroll();
        }

        self.prev_left = left;
        self.prev_right = right;
        self.prev_up = up;
        self.prev_down = down;

        let confirm_just = confirm && !self.prev_confirm;
        self.prev_confirm = confirm;
        if confirm_just {
            let level_num = self.levels[self.selected];
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

        if self.levels.is_empty() {
            let msg = "No levels found in assets/levels/";
            let msg_size = 20.0;
            let msg_w = msg_size * msg.len() as f32 * 0.28;
            draw_text(msg, sw / 2.0 - msg_w, sh / 2.0, msg_size, Color::new(0.8, 0.3, 0.3, 1.0));
            return;
        }

        // Layout
        let tile_w = 150.0;
        let tile_h = 170.0;
        let gap_x = 20.0;
        let gap_y = 16.0;
        let total_w = COLS as f32 * tile_w + (COLS - 1) as f32 * gap_x;
        let start_x = (sw - total_w) / 2.0;
        let rows_start_y = sh * 0.18;

        let total = self.levels.len();
        let total_rows = (total + COLS - 1) / COLS;

        for vi in 0..VISIBLE_ROWS {
            let grid_row = self.scroll_row + vi;
            if grid_row >= total_rows {
                break;
            }
            for col in 0..COLS {
                let idx = grid_row * COLS + col;
                if idx >= total {
                    break;
                }
                let level_num = self.levels[idx];
                let is_selected = idx == self.selected;

                let x = start_x + col as f32 * (tile_w + gap_x);
                let y = rows_start_y + vi as f32 * (tile_h + gap_y);

                let bg = if is_selected {
                    Color::new(1.0, 0.85, 0.15, 1.0)
                } else {
                    Color::new(0.15, 0.15, 0.35, 1.0)
                };
                draw_rectangle(x, y, tile_w, tile_h, bg);

                // Color swatch: cycle through a palette based on level number
                let palette = [
                    Color::new(0.30, 0.65, 0.20, 1.0), // green
                    Color::new(0.30, 0.25, 0.40, 1.0), // purple
                    Color::new(0.40, 0.60, 0.90, 1.0), // blue
                    Color::new(0.55, 0.15, 0.15, 1.0), // red
                    Color::new(0.20, 0.50, 0.50, 1.0), // teal
                    Color::new(0.60, 0.40, 0.15, 1.0), // orange
                    Color::new(0.25, 0.25, 0.25, 1.0), // dark
                    Color::new(0.50, 0.50, 0.15, 1.0), // olive
                ];
                let swatch = palette[idx % palette.len()];
                let m = 6.0;
                draw_rectangle(x + m, y + m, tile_w - m * 2.0, tile_h * 0.50, swatch);

                // Number
                let num_text = format!("{}", level_num);
                let num_size = 48.0;
                draw_text(
                    &num_text,
                    x + tile_w / 2.0 - 14.0,
                    y + 55.0,
                    num_size,
                    if is_selected {
                        Color::new(0.1, 0.1, 0.1, 1.0)
                    } else {
                        WHITE
                    },
                );

                // Name: from cached level name (from JSON or auto-generated)
                let name = &self.level_names[idx];
                let name_size = 14.0;
                let name_w = name_size * name.len() as f32 * 0.32;
                draw_text(
                    &name,
                    x + tile_w / 2.0 - name_w,
                    y + tile_h - 30.0,
                    name_size,
                    if is_selected {
                        Color::new(0.1, 0.1, 0.1, 1.0)
                    } else {
                        Color::new(0.7, 0.7, 0.8, 1.0)
                    },
                );
            }
        }

        // Scrollbar (right edge)
        if total_rows > VISIBLE_ROWS {
            let bar_x = start_x + total_w + 20.0;
            let bar_w = 6.0;
            let bar_area_top = rows_start_y;
            let bar_area_h = VISIBLE_ROWS as f32 * (tile_h + gap_y);
            let thumb_h = VISIBLE_ROWS as f32 / total_rows as f32 * bar_area_h;
            let thumb_y = bar_area_top
                + self.scroll_row as f32 / total_rows as f32 * bar_area_h;
            draw_rectangle(bar_x, bar_area_top, bar_w, bar_area_h, Color::new(0.1, 0.1, 0.2, 1.0));
            draw_rectangle(bar_x, thumb_y, bar_w, thumb_h, Color::new(0.5, 0.5, 0.7, 1.0));
        }

        // Level count
        let count_text = format!("{}/{} levels", self.selected + 1, total);
        let count_size = 16.0;
        draw_text(
            &count_text,
            20.0,
            sh - 30.0,
            count_size,
            Color::new(0.5, 0.5, 0.6, 1.0),
        );

        let hint = "ARROWS: Navigate    ENTER/SPACE: Start";
        let hint_size = 16.0;
        let hint_w = hint_size * hint.len() as f32 * 0.28;
        draw_text(
            hint,
            sw / 2.0 - hint_w,
            sh * 0.92,
            hint_size,
            Color::new(0.6, 0.6, 0.7, 1.0),
        );
    }
}
