use std::fs;
use macroquad::prelude::*;
use crate::editor::state::EditorState;
use crate::editor::tool::{Tool, MENU_BAR};
use crate::editor::data::LevelData;
use crate::level;

impl EditorState {
    pub fn update(&mut self) {
        // Status timer
        if self.status_timer > 0.0 {
            self.status_timer -= get_frame_time();
        }

        // --- Keyboard shortcuts ---
        let skip_keys = self.editing_field.is_some();
        if !skip_keys {

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
        if is_key_pressed(KeyCode::P) { self.tool = Tool::PlayerSpawn; }
        if is_key_pressed(KeyCode::C) { self.tool = Tool::Cloud; }
        if is_key_pressed(KeyCode::Delete) || is_key_pressed(KeyCode::Backspace) { self.tool = Tool::Eraser; }
        if is_key_pressed(KeyCode::D) { self.tool = Tool::Drag; }
        if is_key_pressed(KeyCode::V) { self.tool = Tool::View; }

        // Pan: arrow keys — nudge entity if selected, else pan canvas
        if self.selected_entity.is_some() {
            // Nudge selected entity by 1px
            let mut dx = 0.0f32;
            let mut dy = 0.0f32;
            if is_key_pressed(KeyCode::Left)  { dx -= 1.0; }
            if is_key_pressed(KeyCode::Right) { dx += 1.0; }
            if is_key_pressed(KeyCode::Up)    { dy -= 1.0; }
            if is_key_pressed(KeyCode::Down)  { dy += 1.0; }
            if dx != 0.0 || dy != 0.0 {
                let target = self.selected_entity.unwrap();
                let (ex, ey) = self.entity_pos(target);
                self.move_entity(target, ex + dx, ey + dy);
                self.dirty = true;
            }
        } else {
            let pan_speed = 400.0 * get_frame_time() / self.zoom;
            if is_key_down(KeyCode::Left)  { self.cam_x -= pan_speed; }
            if is_key_down(KeyCode::Right) { self.cam_x += pan_speed; }
            if is_key_down(KeyCode::Up)    { self.cam_y -= pan_speed; }
            if is_key_down(KeyCode::Down)  { self.cam_y += pan_speed; }
        }

        // Zoom: +/- or mouse wheel
        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) { self.zoom = (self.zoom * 1.2).min(4.0); }
        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) { self.zoom = (self.zoom / 1.2).max(0.25); }
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

        // Save / Load keyboard shortcuts
        if is_key_down(KeyCode::LeftControl) && is_key_down(KeyCode::LeftShift) && is_key_pressed(KeyCode::S) {
            self.save_as_mode = true;
            self.save_path_buf = EditorState::level_path(self.level_num);
            self.set_status("Type path, Enter to confirm, Esc to cancel...");
        }
        if is_key_down(KeyCode::LeftControl) && !is_key_down(KeyCode::LeftShift) && is_key_pressed(KeyCode::S) {
            self.save_level();
        }

        // F1-F12: load level by index from available
        let available = level::list_levels();
        let f_keys: [(KeyCode, usize); 12] = [
            (KeyCode::F1, 0), (KeyCode::F2, 1), (KeyCode::F3, 2), (KeyCode::F4, 3),
            (KeyCode::F5, 4), (KeyCode::F6, 5), (KeyCode::F7, 6), (KeyCode::F8, 7),
            (KeyCode::F9, 8), (KeyCode::F10, 9), (KeyCode::F11, 10), (KeyCode::F12, 11),
        ];
        for &(key, idx) in &f_keys {
            if is_key_pressed(key) && idx < available.len() { self.load_level(available[idx]); }
        }

        // Ctrl+N: new blank level
        if is_key_down(KeyCode::LeftControl) && is_key_pressed(KeyCode::N) {
            self.data = LevelData::default();
            let mut n = 1u32;
            while n <= 99 {
                if std::fs::metadata(&format!("assets/levels/{}.json", n)).is_err() { break; }
                n += 1;
            }
            self.level_num = n; self.dirty = true;
            self.set_status(&format!("New level {}. Ctrl+S to save.", n));
        }

        // Ctrl+R: rename level
        if is_key_down(KeyCode::LeftControl) && is_key_pressed(KeyCode::R) {
            self.rename_mode = true;
            self.name_buf = self.data.name.clone();
            self.set_status("Enter new level name, Enter to confirm, Esc to cancel...");
        }
        } // end if !skip_keys

        // Save-as text input mode
        if self.save_as_mode {
            while let Some(c) = get_char_pressed() {
                if c.is_ascii_graphic() || c == '.' || c == '/' || c == '\\' || c == '_' || c == '-' { self.save_path_buf.push(c); }
            }
            if is_key_pressed(KeyCode::Backspace) { self.save_path_buf.pop(); }
            if is_key_pressed(KeyCode::Enter) {
                let path = self.save_path_buf.clone();
                self.save_to_path(&path);
                self.save_as_mode = false; self.save_path_buf.clear();
            }
            if is_key_pressed(KeyCode::Escape) {
                self.save_as_mode = false; self.save_path_buf.clear();
                self.set_status("Save cancelled.");
            }
            return;
        }

        // Rename text input mode
        if self.rename_mode {
            while let Some(c) = get_char_pressed() {
                if c.is_ascii_graphic() || c == ' ' || c == '_' || c == '-' { self.name_buf.push(c); }
            }
            if is_key_pressed(KeyCode::Backspace) { self.name_buf.pop(); }
            if is_key_pressed(KeyCode::Enter) {
                self.data.name = self.name_buf.clone();
                self.dirty = true; self.rename_mode = false; self.name_buf.clear();
                self.set_status(&format!("Renamed to: {}", self.data.name));
            }
            if is_key_pressed(KeyCode::Escape) {
                self.rename_mode = false; self.name_buf.clear();
                self.set_status("Rename cancelled.");
            }
            return;
        }

        // Open File text input mode
        if self.open_file_mode {
            while let Some(c) = get_char_pressed() {
                if c.is_ascii_graphic() || c == '.' || c == '/' || c == '\\' || c == '_' || c == '-' || c == ':' { self.open_file_buf.push(c); }
            }
            if is_key_pressed(KeyCode::Backspace) { self.open_file_buf.pop(); }
            if is_key_pressed(KeyCode::Enter) {
                let path = self.open_file_buf.clone();
                self.open_file_mode = false; self.open_file_buf.clear();
                match fs::read_to_string(&path) {
                    Ok(json) => match serde_json::from_str::<LevelData>(&json) {
                        Ok(data) => {
                            self.data = data;
                            if let Some(stem) = std::path::Path::new(&path).file_stem() {
                                if let Ok(n) = stem.to_string_lossy().parse::<u32>() { self.level_num = n; }
                            }
                            self.dirty = false;
                            self.set_status(&format!("Loaded from {}", path));
                        }
                        Err(e) => self.set_status(&format!("Parse error: {}", e)),
                    },
                    Err(e) => self.set_status(&format!("Cannot read {}: {}", path, e)),
                }
            }
            if is_key_pressed(KeyCode::Escape) {
                self.open_file_mode = false; self.open_file_buf.clear();
                self.set_status("Open cancelled.");
            }
            return;
        }

        // ── Property editing mode (View mode) ──
        if self.editing_field.is_some() {
            while let Some(c) = get_char_pressed() {
                if c.is_ascii_digit() || c == '-' || c == '.' {
                    self.edit_buf.push(c);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                self.edit_buf.pop();
            }
            if is_key_pressed(KeyCode::Escape) {
                // Discard edit and go back to viewing
                self.editing_field = None;
                self.edit_buf.clear();
            }
            if is_key_pressed(KeyCode::Enter) {
                // Apply current edit
                if let Ok(val) = self.edit_buf.parse::<f32>() {
                    let field = self.editing_field.take().unwrap();
                    self.set_entity_property(&field, val);
                    self.set_status(&format!("Set {} = {}", field, val));
                } else {
                    self.editing_field = None;
                    self.edit_buf.clear();
                }
            }
        }

        // Cancel pending operation
        if is_key_pressed(KeyCode::Escape) {
            self.cancel_pending();
            if self.selected_entity.is_some() {
                self.selected_entity = None;
                self.editing_field = None;
                self.edit_buf.clear();
                self.set_status("Entity deselected.");
            }
        }

        // --- Mouse ---
        let (mx, my) = mouse_position();

        // Single left-click event — check all targets in priority:
        //   1. Menu bar / dropdown  2. Toolbar  3. Canvas (drag/place)
        if is_mouse_button_pressed(MouseButton::Left) {
            // 1. Menu bar / dropdown
            let in_menu_bar = my < EditorState::MENU_H;
            let menu_is_open = self.menu_open.is_some();
            let mut menu_handled = false;
            if in_menu_bar || menu_is_open {
                if in_menu_bar {
                    for (i, &(hx, hy, hw, hh)) in self.menu_hdr_rects.iter().enumerate() {
                        if mx >= hx && mx <= hx + hw && my >= hy && my <= hy + hh {
                            self.menu_open = if self.menu_open == Some(i) { None } else { Some(i) };
                            menu_handled = true;
                            break;
                        }
                    }
                }
                if !menu_handled && menu_is_open {
                    for (j, &(ix, iy, iw, ih)) in self.menu_item_rects.iter().enumerate() {
                        if mx >= ix && mx <= ix + iw && my >= iy && my <= iy + ih {
                            let menu_idx = self.menu_open.unwrap();
                            let action: String;
                            {
                                let items = MENU_BAR[menu_idx].1;
                                if menu_idx == 0 {
                                    let mut v: Vec<(String, String)> = Vec::new();
                                    for &(label, act) in items {
                                        if act == "open_header" {
                                            let available = level::list_levels();
                                            if available.is_empty() {
                                                v.push(("  (no levels)".into(), "".into()));
                                            } else {
                                                for (k, &_lv) in available.iter().enumerate() {
                                                    if k >= 12 { break; }
                                                    v.push((String::new(), format!("open:{}", k)));
                                                }
                                            }
                                        } else { v.push((label.into(), act.into())); }
                                    }
                                    action = v.get(j).map(|(_, a)| a.clone()).unwrap_or_default();
                                } else {
                                    action = items.get(j).map(|(_, a)| a.to_string()).unwrap_or_default();
                                }
                            }
                            self.menu_open = None;
                            if let Some(level_idx) = action.strip_prefix("open:") {
                                if let Ok(n) = level_idx.parse::<usize>() {
                                    let available = level::list_levels();
                                    if let Some(&lv) = available.get(n) { self.load_level(lv); }
                                }
                            } else if !action.is_empty() {
                                self.menu_action(&action);
                            }
                            menu_handled = true;
                            break;
                        }
                    }
                    if !menu_handled {
                        // Click outside dropdown — close it
                        self.menu_open = None;
                        menu_handled = true;
                    }
                }
                if menu_handled {
                    return;
                }
            }

            // 2. Toolbar
            if my >= EditorState::MENU_H && my <= EditorState::HEADER_H {
                for &(tool, rx, ry, rw, rh) in &self.tool_rects {
                    if mx >= rx && mx <= rx + rw && my >= ry && my <= ry + rh {
                        self.tool = tool;
                        self.cancel_pending();
                        return;
                    }
                }
            }

            // 2b. Properties panel (View mode, entity selected)
            if self.selected_entity.is_some() {
                let props = self.entity_properties();
                if !props.is_empty() {
                    let sw = self.screen_w;
                    let sh = self.screen_h;
                    let panel_w = 220.0;
                    let row_h = 22.0;
                    let title_row_h = 26.0;
                    let pad = 8.0;
                    let is_editing = self.editing_field.is_some();
                    let btn_extra = if is_editing { 4.0 + 22.0 } else { 0.0 };
                    let panel_h = pad * 2.0 + title_row_h + 2.0 + props.len() as f32 * row_h + btn_extra;
                    let panel_x = sw - panel_w - 12.0;
                    let panel_y = sh - panel_h - 12.0;
                    let sep_y = panel_y + pad + title_row_h;
                    if mx >= panel_x && mx <= panel_x + panel_w && my >= panel_y && my <= panel_y + panel_h {
                        // First check property rows
                        for (i, (_label, field_name, _value)) in props.iter().enumerate() {
                            let row_y = sep_y + 4.0 + i as f32 * row_h;
                            if my >= row_y && my <= row_y + row_h {
                                // Discard any previous edit, start editing this field
                                self.editing_field = Some(field_name.to_string());
                                self.edit_buf.clear();
                                let props2 = self.entity_properties();
                                if let Some((_, _, val)) = props2.get(i) {
                                    self.edit_buf = format!("{:.0}", val);
                                }
                                return;
                            }
                        }
                        // Then check Apply / Reset buttons (only when editing)
                        if is_editing {
                            let props_end = sep_y + 4.0 + props.len() as f32 * row_h;
                            let btn_sep_y = props_end + 2.0;
                            let btn_y = btn_sep_y + 4.0;
                            let btn_h = 18.0;
                            let half_w = (panel_w - 16.0) / 2.0;
                            let apply_x = panel_x + 6.0;
                            let reset_x = apply_x + half_w + 4.0;
                            if my >= btn_y && my <= btn_y + btn_h {
                                if mx >= apply_x && mx <= apply_x + half_w {
                                    // Apply
                                    if let Ok(val) = self.edit_buf.parse::<f32>() {
                                        let field = self.editing_field.take().unwrap();
                                        self.set_entity_property(&field, val);
                                        self.set_status(&format!("Set {} = {}", field, val));
                                    } else {
                                        self.editing_field = None;
                                        self.edit_buf.clear();
                                    }
                                    return;
                                } else if mx >= reset_x && mx <= reset_x + half_w {
                                    // Reset
                                    self.editing_field = None;
                                    self.edit_buf.clear();
                                    return;
                                }
                            }
                        }
                    }
                }
            }

            // 3. Canvas: drag, view, erase, or place
            if my > EditorState::HEADER_H {
                let (wx, wy) = self.screen_to_world(mx, my);
                if self.tool == Tool::Drag {
                    // Drag mode: only move existing entities, never place new ones
                    if let Some(target) = self.hit_test(wx, wy) {
                        self.drag_target = Some(target);
                        // Store offset from entity origin to snapped mouse pos.
                        // Using snap here (not raw) matches the drag update below,
                        // preventing the entity from jumping on initial click.
                        let (ex, ey) = self.entity_pos(target);
                        let (sx, sy) = self.snap_pos(wx, wy);
                        self.drag_offset = (ex - sx, ey - sy);
                        // Also select for property inspection
                        self.selected_entity = Some(target);
                        self.editing_field = None;
                        self.edit_buf.clear();
                        self.set_status("Drag to move. Right-click or Esc to cancel.");
                    } else {
                        // Clicked empty space — deselect
                        self.selected_entity = None;
                        self.editing_field = None;
                        self.edit_buf.clear();
                    }
                } else if self.tool == Tool::View {
                    // View mode: select entity for property inspection
                    if let Some(target) = self.hit_test(wx, wy) {
                        self.selected_entity = Some(target);
                        self.editing_field = None;
                        self.edit_buf.clear();
                        self.set_status("Entity selected. Click property value to edit, Esc to deselect.");
                    } else {
                        self.selected_entity = None;
                        self.editing_field = None;
                        self.edit_buf.clear();
                    }
                } else if self.tool == Tool::Eraser {
                    self.delete_at(wx, wy);
                } else {
                    self.place_at(wx, wy);
                }
            }
        }

        if let Some(target) = self.drag_target {
            let (wx, wy) = self.screen_to_world(mx, my);
            let (sx, sy) = self.snap_pos(wx, wy);
            // Apply stored offset so entity keeps its grab point
            let (ox, oy) = self.drag_offset;
            let fx = sx + ox;
            let fy = sy + oy;
            if is_mouse_button_pressed(MouseButton::Right) || is_key_pressed(KeyCode::Escape) {
                self.drag_target = None; self.set_status("Drag cancelled.");
            } else if is_mouse_button_released(MouseButton::Left) {
                self.move_entity(target, fx, fy);
                self.dirty = true; self.drag_target = None; self.set_status("Entity moved.");
            } else if is_mouse_button_down(MouseButton::Left) {
                self.move_entity(target, fx, fy);
            } else {
                self.move_entity(target, fx, fy);
                self.drag_target = None; self.dirty = true; self.set_status("Entity moved.");
            }
        }

        // Delete on right click
        if is_mouse_button_pressed(MouseButton::Right) && my > EditorState::HEADER_H {
            let (wx, wy) = self.screen_to_world(mx, my);
            self.delete_at(wx, wy);
        }

        // Pan with middle mouse drag (always available)
        if is_mouse_button_down(MouseButton::Middle) {
            let dx = mouse_delta_position();
            self.cam_x -= dx.x / self.zoom;
            self.cam_y -= dx.y / self.zoom;
        }

        // View mode: left-drag to pan canvas (manual delta from prev_mouse)
        if self.tool == Tool::View && is_mouse_button_down(MouseButton::Left) && my > EditorState::HEADER_H {
            let (pmx, pmy) = self.prev_mouse;
            if pmx != 0.0 || pmy != 0.0 {
                self.cam_x -= (mx - pmx) / self.zoom;
                self.cam_y -= (my - pmy) / self.zoom;
            }
        }
        self.prev_mouse = (mx, my);
    }
}
