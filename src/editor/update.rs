use macroquad::prelude::*;
use crate::editor::state::EditorState;
use crate::editor::tool::Tool;
use crate::editor::data::LevelData;

impl EditorState {
    pub fn update(&mut self) {
        // Status timer
        if self.status_timer > 0.0 {
            self.status_timer -= get_frame_time();
        }

        // --- Keyboard shortcuts ---

        // Tool selection: 1-0
        if is_key_pressed(KeyCode::Key1) { self.tool = Tool::Platform; }
        if is_key_pressed(KeyCode::Key2) { self.tool = Tool::Spike; }
        if is_key_pressed(KeyCode::Key3) { self.tool = Tool::Coin; }
        if is_key_pressed(KeyCode::Key4) { self.tool = Tool::QuestionBlock; }
        if is_key_pressed(KeyCode::Key5) { self.tool = Tool::Brick; }
        if is_key_pressed(KeyCode::Key6) { self.tool = Tool::Enemy; }
        if is_key_pressed(KeyCode::Key7) { self.tool = Tool::DartEnemy; }
        if is_key_pressed(KeyCode::Key8) { self.tool = Tool::OscFireball; }
        if is_key_pressed(KeyCode::Key9) { self.tool = Tool::Checkpoint; }
        if is_key_pressed(KeyCode::Key0) { self.tool = Tool::Flagpole; }
        if is_key_pressed(KeyCode::P) {
            self.tool = Tool::PlayerSpawn;
        }
        if is_key_pressed(KeyCode::Delete) || is_key_pressed(KeyCode::Backspace) {
            self.tool = Tool::Eraser;
        }

        // Pan: arrow keys
        let pan_speed = 400.0 * get_frame_time() / self.zoom;
        if is_key_down(KeyCode::Left)  { self.cam_x -= pan_speed; }
        if is_key_down(KeyCode::Right) { self.cam_x += pan_speed; }
        if is_key_down(KeyCode::Up)    { self.cam_y -= pan_speed; }
        if is_key_down(KeyCode::Down)  { self.cam_y += pan_speed; }

        // Zoom: +/- or mouse wheel
        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
            self.zoom = (self.zoom * 1.2).min(4.0);
        }
        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
            self.zoom = (self.zoom / 1.2).max(0.25);
        }
        // Mouse wheel zoom (when Ctrl held or just wheel)
        let wheel = mouse_wheel();
        if wheel.1 != 0.0 {
            if wheel.1 > 0.0 { self.zoom = (self.zoom * 1.15).min(4.0); }
            else { self.zoom = (self.zoom / 1.15).max(0.25); }
        }

        // Grid toggle
        if is_key_pressed(KeyCode::G) {
            self.grid_snap = !self.grid_snap;
            self.set_status(if self.grid_snap { "Grid snap: ON" } else { "Grid snap: OFF" });
        }

        // Save / Load
        if is_key_down(KeyCode::LeftControl) && is_key_down(KeyCode::LeftShift) && is_key_pressed(KeyCode::S) {
            self.save_as_mode = true;
            self.save_path_buf = EditorState::level_path(self.level_num);
            self.set_status("Type path, Enter to confirm, Esc to cancel...");
        }
        if is_key_down(KeyCode::LeftControl) && !is_key_down(KeyCode::LeftShift) && is_key_pressed(KeyCode::S) {
            self.save_level();
        }
        if is_key_pressed(KeyCode::F1) { self.load_level(1); }
        if is_key_pressed(KeyCode::F2) { self.load_level(2); }
        if is_key_pressed(KeyCode::F3) { self.load_level(3); }
        if is_key_pressed(KeyCode::F4) { self.load_level(4); }
        if is_key_pressed(KeyCode::F5) { self.load_level(5); }
        if is_key_pressed(KeyCode::F6) { self.load_level(6); }
        if is_key_pressed(KeyCode::F7) { self.load_level(7); }
        if is_key_pressed(KeyCode::F8) { self.load_level(8); }

        // Ctrl+N: new blank level
        if is_key_down(KeyCode::LeftControl) && is_key_pressed(KeyCode::N) {
            self.data = LevelData::default();
            // Auto-increment to next available level number
            let mut n = 1u32;
            while n <= 99 {
                let path = format!("assets/levels/{}.json", n);
                if std::fs::metadata(&path).is_err() {
                    break;
                }
                n += 1;
            }
            self.level_num = n;
            self.dirty = true;
            self.set_status(&format!("New level {}. Ctrl+S to save.", n));
        }

        // Save-as text input mode
        if self.save_as_mode {
            // Type characters
            while let Some(c) = get_char_pressed() {
                if c.is_ascii_graphic() || c == '.' || c == '/' || c == '\\' || c == '_' || c == '-' {
                    self.save_path_buf.push(c);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                self.save_path_buf.pop();
            }
            if is_key_pressed(KeyCode::Enter) {
                let path = self.save_path_buf.clone();
                self.save_to_path(&path);
                self.save_as_mode = false;
                self.save_path_buf.clear();
            }
            if is_key_pressed(KeyCode::Escape) {
                self.save_as_mode = false;
                self.save_path_buf.clear();
                self.set_status("Save cancelled.");
            }
            return; // Block other input while typing path
        }

        // Cancel pending operation
        if is_key_pressed(KeyCode::Escape) {
            self.cancel_pending();
        }

        // --- Mouse ---
        let (mx, my) = mouse_position();

        // ── Drag: move entity while mouse is held ──
        if let Some(target) = self.drag_target {
            let (wx, wy) = self.screen_to_world(mx, my);
            let (sx, sy) = self.snap_pos(wx, wy);
            if is_mouse_button_pressed(MouseButton::Right) || is_key_pressed(KeyCode::Escape) {
                self.drag_target = None;
                self.set_status("Drag cancelled.");
            } else if is_mouse_button_released(MouseButton::Left) {
                self.move_entity(target, sx, sy);
                self.dirty = true;
                self.drag_target = None;
                self.set_status("Entity moved.");
            } else if is_mouse_button_down(MouseButton::Left) {
                self.move_entity(target, sx, sy);
            } else {
                self.drag_target = None;
                self.dirty = true;
                self.set_status("Entity moved.");
            }
        } else {
            // Place on left click (only when mouse is in canvas area)
            if is_mouse_button_pressed(MouseButton::Left) && my > EditorState::HEADER_H {
                let (wx, wy) = self.screen_to_world(mx, my);
                if let Some(target) = self.hit_test(wx, wy) {
                    self.drag_target = Some(target);
                    self.set_status("Drag to move. Right-click or Esc to cancel.");
                } else {
                    self.place_at(wx, wy);
                }
            }
        }

        // Delete on right click
        if is_mouse_button_pressed(MouseButton::Right) && my > EditorState::HEADER_H {
            let (wx, wy) = self.screen_to_world(mx, my);
            self.delete_at(wx, wy);
        }

        // Pan with middle mouse drag
        if is_mouse_button_down(MouseButton::Middle) {
            let dx = mouse_delta_position();
            self.cam_x -= dx.x / self.zoom;
            self.cam_y -= dx.y / self.zoom;
        }

    }
}
