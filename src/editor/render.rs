use macroquad::prelude::*;
use crate::editor::state::EditorState;
use crate::editor::tool::{Tool, MENU_BAR};

impl EditorState {
    pub fn render(&mut self) {
        clear_background(Color::new(0.15, 0.15, 0.22, 1.0));

        let sw = self.screen_w;
        let sh = self.screen_h;
        if sw <= 0.0 || sh <= 0.0 { return; }

        // ── Menu Bar (top 24px) ──
        self.menu_hdr_rects.clear();
        draw_rectangle(0.0, 0.0, sw, EditorState::MENU_H, Color::new(0.08, 0.08, 0.14, 1.0));
        let mut mx = 8.0f32;
        let font_size = 14.0;
        for (i, &(label, _items)) in MENU_BAR.iter().enumerate() {
            let tw = font_size * label.len() as f32 * 0.55;
            let bw = tw + 12.0;
            let bh = EditorState::MENU_H - 2.0;
            let is_open = self.menu_open == Some(i);
            let (cmx, cmy) = mouse_position();
            let hover = !is_open && cmx >= mx && cmx <= mx + bw && cmy >= 0.0 && cmy <= EditorState::MENU_H;
            let bg = if is_open { Color::new(0.2, 0.3, 0.5, 1.0) }
                else if hover { Color::new(0.18, 0.22, 0.35, 1.0) }
                else { Color::new(0.08, 0.08, 0.14, 0.0) };
            if is_open || hover { draw_rectangle(mx, 1.0, bw, bh, bg); }
            draw_text(label, mx + 6.0, EditorState::MENU_H - 6.0, font_size,
                if is_open { YELLOW } else { WHITE });
            self.menu_hdr_rects.push((mx, 0.0, bw, EditorState::MENU_H));
            mx += bw + 2.0;
        }

        // ── Toolbar (from MENU_H) ──
        draw_rectangle(0.0, EditorState::MENU_H, sw, EditorState::TOOLBAR_H, Color::new(0.1, 0.1, 0.18, 1.0));
        let mut tx = 8.0;
        self.tool_rects.clear();
        for &tool in Tool::ALL {
            let label = format!("[{}] {}", tool.shortcut(), tool.name());
            let fw = 12.0 * label.len() as f32 * 0.55;
            let bw = fw + 6.0;
            let bh = 32.0;
            let (cmx, cmy) = mouse_position();
            let hover = cmx >= tx - 2.0 && cmx <= tx + bw && cmy >= EditorState::MENU_H + 4.0 && cmy <= EditorState::MENU_H + 36.0;
            let bg = if self.tool == tool { Color::new(0.3, 0.5, 0.9, 0.9) }
                else if hover { Color::new(0.3, 0.35, 0.5, 0.8) }
                else { Color::new(0.2, 0.2, 0.3, 0.7) };
            draw_rectangle(tx - 2.0, EditorState::MENU_H + 4.0, bw, bh, bg);
            draw_text(&label, tx, EditorState::MENU_H + 28.0, 14.0, WHITE);
            self.tool_rects.push((tool, tx - 2.0, EditorState::MENU_H + 4.0, bw, bh));
            tx += bw + 4.0;
        }
        draw_line(0.0, EditorState::HEADER_H, sw, EditorState::HEADER_H, 2.0, Color::new(0.3, 0.3, 0.5, 0.8));

        // Level name & info
        let lvl_text = format!("Level: {} {}", self.level_num, if self.dirty { "*" } else { "" });
        draw_text(&lvl_text, sw - 140.0, EditorState::MENU_H + 28.0, 16.0, if self.dirty { YELLOW } else { WHITE });
        let name_display = if self.data.name.is_empty() { "(no name)" } else { &self.data.name };
        draw_text(name_display, sw - 350.0, EditorState::MENU_H + 28.0, 16.0, Color::new(0.6, 0.9, 1.0, 1.0));
        draw_text("Ctrl+R:rename", sw - 500.0, EditorState::MENU_H + 28.0, 12.0, GRAY);

        // ── World-to-screen transform ──
        let cam_x = self.cam_x;
        let cam_y = self.cam_y;
        let zoom = self.zoom;
        let ws = |wx: f32, wy: f32| -> (f32, f32) {
            ((wx - cam_x) * zoom, (wy - cam_y) * zoom + EditorState::HEADER_H)
        };

        // ── Grid ──
        let grid_color = Color::new(0.25, 0.25, 0.35, 0.5);
        let grid_step = EditorState::GRID * self.zoom;
        if grid_step > 4.0 {
            let start_x = (self.cam_x / EditorState::GRID).floor() * EditorState::GRID;
            let start_y = (self.cam_y / EditorState::GRID).floor() * EditorState::GRID;
            let end_x = self.cam_x + sw / self.zoom;
            let end_y = self.cam_y + (sh - EditorState::HEADER_H) / self.zoom;
            let mut gx = start_x;
            while gx <= end_x {
                let (sx, sy1) = ws(gx, start_y);
                let (_, sy2) = ws(gx, end_y);
                draw_line(sx, sy1, sx, sy2, 1.0, grid_color);
                gx += EditorState::GRID;
            }
            let mut gy = start_y;
            while gy <= end_y {
                let (sx1, sy) = ws(start_x, gy);
                let (sx2, _) = ws(end_x, gy);
                draw_line(sx1, sy, sx2, sy, 1.0, grid_color);
                gy += EditorState::GRID;
            }
        }

        // ── Level bounds ──
        {
            let (bx, by) = ws(self.data.bounds.min_x, self.data.bounds.min_y);
            let bw = (self.data.bounds.max_x - self.data.bounds.min_x) * self.zoom;
            let bh = (self.data.bounds.kill_y - self.data.bounds.min_y) * self.zoom;
            draw_rectangle_lines(bx, by, bw, bh, 2.0, Color::new(1.0, 0.3, 0.3, 0.6));
        }

        // ── Platforms ──
        for p in &self.data.platforms {
            let (px, py) = ws(p.x, p.y);
            let c = if p.y >= 590.0 { Color::new(0.25, 0.55, 0.15, 0.8) }
                else { Color::new(0.45, 0.28, 0.10, 0.8) };
            draw_rectangle(px, py, p.w * self.zoom, p.h * self.zoom, c);
        }

        // ── Spikes ──
        for s in &self.data.spikes {
            let (sx, sy) = ws(s.x, s.y);
            let hw = 8.0 * self.zoom;
            let hh = 4.0 * self.zoom;
            draw_triangle(Vec2::new(sx, sy + hh), Vec2::new(sx - hw, sy - hh), Vec2::new(sx + hw, sy - hh),
                Color::new(0.9, 0.2, 0.1, 0.9));
        }

        // ── Coins ──
        for c in &self.data.coins {
            let (cx, cy) = ws(c.x, c.y);
            draw_circle(cx, cy, 5.0 * self.zoom, Color::new(1.0, 0.85, 0.0, 0.9));
        }

        // ── Question blocks ──
        for q in &self.data.question_blocks {
            let (qx, qy) = ws(q.x, q.y);
            draw_rectangle(qx, qy, 32.0 * self.zoom, 32.0 * self.zoom, Color::new(1.0, 0.65, 0.0, 0.9));
            draw_text("?", qx + 8.0 * self.zoom, qy + 24.0 * self.zoom, 24.0 * self.zoom, WHITE);
        }

        // ── Bricks ──
        for b in &self.data.bricks {
            let (bx, by) = ws(b.x, b.y);
            draw_rectangle(bx, by, 32.0 * self.zoom, 32.0 * self.zoom, Color::new(0.55, 0.30, 0.12, 0.9));
        }

        // ── Enemies ──
        for e in &self.data.enemies {
            let (ex, ey) = ws(e.x, e.y - 16.0);
            draw_rectangle(ex, ey, 16.0 * self.zoom, 16.0 * self.zoom, BROWN);
            let (ax, ay) = ws(e.waypoint_a.x, e.waypoint_a.y);
            let (bx, by2) = ws(e.waypoint_b.x, e.waypoint_b.y);
            draw_line(ax, ay, bx, by2, 1.0, Color::new(1.0, 0.5, 0.0, 0.5));
        }

        // ── Dart enemies ──
        for d in &self.data.dart_enemies {
            let (dx, dy) = ws(d.x, d.y - 16.0);
            draw_rectangle(dx, dy, 16.0 * self.zoom, 16.0 * self.zoom, Color::new(0.3, 0.3, 0.35, 0.9));
            draw_circle(dx + 10.0 * self.zoom, dy + 5.0 * self.zoom, 3.0 * self.zoom, RED);
        }

        // ── Oscillating fireballs ──
        for o in &self.data.osc_fireballs {
            let (ox, oty) = ws(o.x, o.top_y);
            let (_, oby) = ws(o.x, o.bottom_y);
            draw_line(ox + 4.0 * self.zoom, oty, ox + 4.0 * self.zoom, oby, 1.0, Color::new(0.5, 0.3, 0.1, 0.5));
            draw_circle(ox + 4.0 * self.zoom, oty, 6.0 * self.zoom, Color::new(1.0, 0.5, 0.1, 0.9));
        }

        // ── Checkpoints ──
        for cp in &self.data.checkpoints {
            let (cpx, cpy) = ws(cp.x, cp.y);
            draw_line(cpx, cpy, cpx, cpy - 40.0 * self.zoom, 3.0 * self.zoom, GRAY);
            draw_rectangle(cpx + 3.0 * self.zoom, cpy - 40.0 * self.zoom, 16.0 * self.zoom, 12.0 * self.zoom, GREEN);
        }

        // ── Flagpole ──
        {
            let (fx, fy) = ws(self.data.flagpole.x, self.data.flagpole.y);
            draw_line(fx, fy, fx, fy - 120.0 * self.zoom, 6.0 * self.zoom, Color::new(0.6, 0.6, 0.6, 0.9));
            draw_rectangle(fx + 6.0 * self.zoom, fy - 120.0 * self.zoom, 24.0 * self.zoom, 18.0 * self.zoom, GREEN);
        }

        // ── Player spawn ──
        {
            let (sx, sy) = ws(self.data.player_spawn.x, self.data.player_spawn.y);
            let sz = 12.0 * self.zoom;
            draw_rectangle(sx - sz * 0.5, sy - sz * 2.0, sz, sz * 0.3, RED);
            draw_rectangle(sx - sz * 0.5, sy - sz * 1.7, sz, sz * 0.4, Color::new(1.0, 0.75, 0.55, 1.0));
            draw_rectangle(sx - sz * 0.5, sy - sz * 1.3, sz, sz * 0.5, Color::new(0.1, 0.3, 0.9, 1.0));
            draw_text("START", sx + sz * 0.7, sy - sz * 1.2, 10.0 * self.zoom, YELLOW);
        }

        // ── Clouds ──
        for c in &self.data.clouds {
            let (cx, cy) = ws(c.x, c.y);
            let cw = c.w * self.zoom;
            let ch = c.h * self.zoom;
            let alpha = c.color[3];
            let cc = if c.dark {
                Color::new(0.35, 0.35, 0.40, alpha)
            } else {
                Color::new(c.color[0], c.color[1], c.color[2], alpha)
            };
            draw_ellipse(cx + cw * 0.5, cy + ch * 0.3, cw, ch * 0.55, 0.0, cc);
            draw_ellipse(cx + cw * 0.15, cy + ch * 0.4, cw * 0.65, ch * 0.45, 0.0, cc);
            draw_ellipse(cx + cw * 0.6, cy + ch * 0.25, cw * 0.55, ch * 0.4, 0.0, cc);
        }

        // ── Pending placement preview ──
        if let Some((sx, sy)) = self.plat_start {
            let (mx2, my2) = mouse_position();
            if my2 > EditorState::HEADER_H {
                let (wx, wy) = self.screen_to_world(mx2, my2);
                let (gx, gy) = self.snap_pos(wx, wy);
                let rx = gx.min(sx); let ry = gy.min(sy);
                let rw = (gx - sx).abs(); let rh = (gy - sy).abs();
                let (prx, pry) = ws(rx, ry);
                draw_rectangle_lines(prx, pry, rw * self.zoom, rh * self.zoom, 2.0, YELLOW);
            }
        }
        if let Some((sx, sy)) = self.enemy_start {
            let (mx3, my3) = mouse_position();
            if my3 > EditorState::HEADER_H {
                let (wx, wy) = self.screen_to_world(mx3, my3);
                let (gx, gy) = self.snap_pos(wx, wy);
                let (p1x, p1y) = ws(sx, sy);
                let (p2x, p2y) = ws(gx, gy);
                draw_line(p1x, p1y, p2x, p2y, 2.0, YELLOW);
            }
        }
        if let Some((_sx, sy)) = self.osc_start {
            let (mx4, my4) = mouse_position();
            if my4 > EditorState::HEADER_H {
                let (wx, wy) = self.screen_to_world(mx4, my4);
                let (gx, _) = self.snap_pos(wx, wy);
                let (p1x, p1y) = ws(gx, sy);
                let (p2x, p2y) = ws(gx, wy);
                draw_line(p1x, p1y, p2x, p2y, 2.0, Color::new(1.0, 0.6, 0.0, 0.8));
            }
        }

        // ── Dropdown menu ──
        self.menu_item_rects.clear();
        if let Some(menu_idx) = self.menu_open {
            let items = MENU_BAR[menu_idx].1;
            let dx = self.menu_hdr_rects[menu_idx].0;
            let item_h = 22.0;
            let item_font = 14.0;

            let display_items: Vec<(String, String)> = if menu_idx == 0 {
                let mut v = Vec::new();
                for &(label, act) in items {
                    if act == "open_header" {
                        let available = crate::level::list_levels();
                        if available.is_empty() {
                            v.push(("  (no levels)".into(), "".into()));
                        } else {
                            for (k, &lv) in available.iter().enumerate() {
                                if k >= 12 { break; }
                                let name = &crate::level::Level::load(lv).name;
                                let short = if name.len() > 20 { &name[..20] } else { name };
                                v.push((format!("  {}: {}", lv, short), format!("open:{}", k)));
                            }
                        }
                    } else { v.push((label.into(), act.into())); }
                }
                v
            } else {
                items.iter().map(|(l, a)| (l.to_string(), a.to_string())).collect()
            };

            let max_w: f32 = display_items.iter().map(|(l, _)| item_font * l.len() as f32 * 0.6 + 40.0)
                .fold(0.0, f32::max).max(180.0);
            let total_h = display_items.len() as f32 * item_h;
            let dy = EditorState::MENU_H;
            draw_rectangle(dx, dy, max_w, total_h + 4.0, Color::new(0.12, 0.14, 0.25, 1.0));
            draw_rectangle_lines(dx, dy, max_w, total_h + 4.0, 1.0, Color::new(0.3, 0.35, 0.5, 1.0));

            for (j, (label, _action)) in display_items.iter().enumerate() {
                let iy = dy + 2.0 + j as f32 * item_h;
                let hover = {
                    let (cmx, cmy) = mouse_position();
                    cmx >= dx && cmx <= dx + max_w && cmy >= iy && cmy <= iy + item_h
                };
                if hover { draw_rectangle(dx, iy, max_w, item_h, Color::new(0.3, 0.5, 0.9, 0.6)); }
                draw_text(label, dx + 6.0, iy + item_h - 4.0, item_font, if hover { YELLOW } else { WHITE });
                self.menu_item_rects.push((dx, iy, max_w, item_h));
            }
        }

        // ── Status bar (bottom) ──
        if self.status_timer > 0.0 {
            draw_rectangle(0.0, sh - 28.0, sw, 28.0, Color::new(0.0, 0.0, 0.0, 0.8));
            draw_text(&self.status, 8.0, sh - 6.0, 16.0, Color::new(0.7, 1.0, 0.7, 1.0));
        }

        // ── Save-As dialog ──
        if self.save_as_mode {
            let dialog_w = 500.0; let dialog_h = 80.0;
            let dx = (sw - dialog_w) / 2.0; let dy = sh - 140.0;
            draw_rectangle(dx, dy, dialog_w, dialog_h, Color::new(0.05, 0.05, 0.15, 1.0));
            draw_rectangle_lines(dx, dy, dialog_w, dialog_h, 2.0, Color::new(0.3, 0.5, 0.9, 1.0));
            draw_text("Save As - Enter path, then press Enter:", dx + 8.0, dy + 20.0, 14.0, GRAY);
            draw_text(&self.save_path_buf, dx + 8.0, dy + 48.0, 18.0, WHITE);
            let cursor_x = dx + 8.0 + 10.0 * self.save_path_buf.len() as f32;
            draw_line(cursor_x, dy + 32.0, cursor_x, dy + 56.0, 2.0, Color::new(1.0, 1.0, 0.0, 0.8));
        }

        // ── Rename dialog ──
        if self.rename_mode {
            let dialog_w = 500.0; let dialog_h = 80.0;
            let dx = (sw - dialog_w) / 2.0; let dy = sh - 250.0;
            draw_rectangle(dx, dy, dialog_w, dialog_h, Color::new(0.05, 0.05, 0.15, 1.0));
            draw_rectangle_lines(dx, dy, dialog_w, dialog_h, 2.0, Color::new(0.9, 0.6, 0.1, 1.0));
            draw_text("Rename Level - Enter new name, then press Enter:", dx + 8.0, dy + 20.0, 14.0, GRAY);
            draw_text(&self.name_buf, dx + 8.0, dy + 48.0, 18.0, WHITE);
            let cursor_x = dx + 8.0 + 10.0 * self.name_buf.len() as f32;
            draw_line(cursor_x, dy + 32.0, cursor_x, dy + 56.0, 2.0, Color::new(1.0, 1.0, 0.0, 0.8));
        }

        // ── Open File dialog ──
        if self.open_file_mode {
            let dialog_w = 500.0; let dialog_h = 80.0;
            let dx = (sw - dialog_w) / 2.0; let dy = sh - 360.0;
            draw_rectangle(dx, dy, dialog_w, dialog_h, Color::new(0.05, 0.05, 0.15, 1.0));
            draw_rectangle_lines(dx, dy, dialog_w, dialog_h, 2.0, Color::new(0.2, 0.7, 0.3, 1.0));
            draw_text("Open File - Enter path to level JSON, then press Enter:", dx + 8.0, dy + 20.0, 14.0, GRAY);
            draw_text(&self.open_file_buf, dx + 8.0, dy + 48.0, 18.0, WHITE);
            let cursor_x = dx + 8.0 + 10.0 * self.open_file_buf.len() as f32;
            draw_line(cursor_x, dy + 32.0, cursor_x, dy + 56.0, 2.0, Color::new(1.0, 1.0, 0.0, 0.8));
        }

        // ── Mouse cursor crosshair ──
        let (mx5, my5) = mouse_position();
        if my5 > EditorState::HEADER_H {
            let (gx, gy) = self.snap_pos(
                mx5 / self.zoom + self.cam_x,
                (my5 - EditorState::HEADER_H) / self.zoom + self.cam_y,
            );
            let (csx, csy) = ws(gx, gy);
            draw_circle_lines(csx, csy, 4.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.6));
            let coord_text = format!("{:.0}, {:.0}", gx, gy);
            draw_text(&coord_text, mx5 + 12.0, my5 - 4.0, 12.0, Color::new(1.0, 1.0, 1.0, 0.8));
        }

        // ── Properties panel (View mode, entity selected) ──
        self.draw_properties_panel();
    }

    fn draw_properties_panel(&self) {
        let target = match &self.selected_entity { Some(t) => t, None => return };
        let props = self.entity_properties();
        if props.is_empty() { return; }

        let sw = self.screen_w;
        let sh = self.screen_h;
        let panel_w = 220.0;
        let row_h = 22.0;
        let title_row_h = 26.0;
        let pad = 8.0;
        let title_font = 18.0;
        let prop_font = 15.0;
        let btn_font = 14.0;
        // Extra button row when editing
        let is_editing = self.editing_field.is_some();
        let btn_row_h = if is_editing { 4.0 + 22.0 } else { 0.0 };
        let panel_h = pad * 2.0 + title_row_h + 2.0 + props.len() as f32 * row_h + btn_row_h;
        // Bottom-right corner
        let panel_x = sw - panel_w - 12.0;
        let panel_y = sh - panel_h - 12.0;

        // Semi-transparent background
        draw_rectangle(panel_x - 2.0, panel_y - 2.0, panel_w + 4.0, panel_h + 4.0, Color::new(0.0, 0.0, 0.0, 0.82));
        draw_rectangle_lines(panel_x - 2.0, panel_y - 2.0, panel_w + 4.0, panel_h + 4.0, 1.0, Color::new(0.5, 0.5, 0.5, 0.8));

        // Title: entity type
        let type_name = self.entity_type_name(target);
        draw_text(&type_name, panel_x + 4.0, panel_y + pad + title_font - 4.0, title_font, Color::new(1.0, 1.0, 0.5, 1.0));

        // Separator line
        let sep_y = panel_y + pad + title_row_h;
        draw_line(panel_x, sep_y, panel_x + panel_w, sep_y, 1.0, Color::new(0.4, 0.4, 0.4, 0.6));

        // Property rows
        for (i, (label, field_name, value)) in props.iter().enumerate() {
            let row_y = sep_y + 4.0 + i as f32 * row_h;
            let editing_this = self.editing_field.as_deref() == Some(*field_name);

            // Label
            let label_text = format!("{}:", label);
            let lw = measure_text(&label_text, None, 12, 1.0).width;
            draw_text(&label_text, panel_x + 6.0, row_y + prop_font, prop_font, Color::new(0.7, 0.7, 0.7, 1.0));

            // Tooltip on label hover
            let (cmx, cmy) = mouse_position();
            if cmx >= panel_x + 6.0 && cmx <= panel_x + 6.0 + lw && cmy >= row_y && cmy <= row_y + row_h {
                draw_property_tooltip(cmx, cmy, *field_name);
            }

            // Value (or edit buffer)
            let val_x = panel_x + panel_w - 6.0;
            let val_color = if editing_this { Color::new(1.0, 1.0, 0.3, 1.0) } else { Color::new(1.0, 1.0, 1.0, 1.0) };
            let val_text = if editing_this { format!("{}|", self.edit_buf) } else { format!("{:.0}", value) };
            let val_w = measure_text(&val_text, None, 12, 1.0).width;
            draw_text(&val_text, val_x - val_w, row_y + prop_font, prop_font, val_color);
        }

        // Apply / Reset buttons (only when editing)
        if is_editing {
            let props_end = sep_y + 4.0 + props.len() as f32 * row_h;
            let btn_sep_y = props_end + 2.0;
            draw_line(panel_x + 4.0, btn_sep_y, panel_x + panel_w - 4.0, btn_sep_y, 1.0, Color::new(0.3, 0.3, 0.3, 0.6));

            let btn_y = btn_sep_y + 4.0;
            let btn_h = 18.0;
            let half_w = (panel_w - 16.0) / 2.0;

            // Apply button (green)
            let apply_x = panel_x + 6.0;
            draw_rectangle(apply_x, btn_y, half_w, btn_h, Color::new(0.15, 0.55, 0.15, 0.9));
            let aw = measure_text("Apply", None, 12, 1.0).width;
            draw_text("Apply", apply_x + (half_w - aw) / 2.0, btn_y + btn_h - 4.0, btn_font, Color::new(0.8, 1.0, 0.8, 1.0));

            // Reset button (red)
            let reset_x = apply_x + half_w + 4.0;
            draw_rectangle(reset_x, btn_y, half_w, btn_h, Color::new(0.55, 0.15, 0.15, 0.9));
            let rw = measure_text("Reset", None, 12, 1.0).width;
            draw_text("Reset", reset_x + (half_w - rw) / 2.0, btn_y + btn_h - 4.0, btn_font, Color::new(1.0, 0.8, 0.8, 1.0));
        }
    }

    fn entity_type_name(&self, target: &crate::editor::tool::DragTarget) -> String {
        match target {
            crate::editor::tool::DragTarget::Platform(_) => "Platform".into(),
            crate::editor::tool::DragTarget::Spike(_) => "Spike".into(),
            crate::editor::tool::DragTarget::Coin(_) => "Coin".into(),
            crate::editor::tool::DragTarget::QuestionBlock(_) => "? Block".into(),
            crate::editor::tool::DragTarget::Brick(_) => "Brick".into(),
            crate::editor::tool::DragTarget::Enemy(_) => "Enemy".into(),
            crate::editor::tool::DragTarget::DartEnemy(_) => "Dart Enemy".into(),
            crate::editor::tool::DragTarget::OscFireball(_) => "Osc Fireball".into(),
            crate::editor::tool::DragTarget::Checkpoint(_) => "Checkpoint".into(),
            crate::editor::tool::DragTarget::PlayerSpawn => "Player Spawn".into(),
            crate::editor::tool::DragTarget::Flagpole => "Flagpole".into(),
            crate::editor::tool::DragTarget::Cloud(_) => "Cloud".into(),
        }
    }
}

fn draw_property_tooltip(mx: f32, my: f32, field_name: &str) {
    let tip = property_tooltip_text(field_name);
    if tip.is_empty() { return; }
    let font_size = 13.0;
    let tw = measure_text(tip, None, 12, 1.0).width + 12.0;
    let th = 20.0;
    let tx = (mx + 16.0).min(macroquad::prelude::screen_width() - tw - 10.0);
    let ty = my + 16.0;
    draw_rectangle(tx, ty, tw, th, Color::new(0.05, 0.05, 0.12, 0.92));
    draw_rectangle_lines(tx, ty, tw, th, 1.0, Color::new(0.4, 0.4, 0.5, 0.8));
    // Use CJK font if available, otherwise default font (will be garbled)
    if let Some(font) = cjk_font() {
        let params = macroquad::text::TextParams {
            font: Some(font),
            font_size: font_size as u16,
            color: Color::new(0.9, 0.9, 1.0, 1.0),
            ..Default::default()
        };
        macroquad::text::draw_text_ex(tip, tx + 6.0, ty + th - 4.0, params);
    } else {
        draw_text(tip, tx + 6.0, ty + th - 4.0, font_size, Color::new(0.9, 0.9, 1.0, 1.0));
    }
}

fn property_tooltip_text(field: &str) -> &'static str {
    match field {
        "x" => "水平坐标 (X轴位置)",
        "y" => "垂直坐标 (Y轴位置)",
        "w" => "宽度 (水平方向尺寸)",
        "h" => "高度 (垂直方向尺寸)",
        "top_y" => "顶部Y坐标 (震荡范围上限)",
        "bottom_y" => "底部Y坐标 (震荡范围下限)",
        "wax" => "巡逻点A的X坐标",
        "way" => "巡逻点A的Y坐标",
        "wbx" => "巡逻点B的X坐标",
        "wby" => "巡逻点B的Y坐标",
        _ => "",
    }
}

// ── CJK font for Chinese tooltips ──

use std::sync::OnceLock;
static CJK_FONT: OnceLock<Option<macroquad::text::Font>> = OnceLock::new();

/// Call once at startup to load a Chinese-capable font.
pub fn init_cjk_font(font: Option<macroquad::text::Font>) {
    CJK_FONT.set(font).ok();
}

fn cjk_font() -> Option<&'static macroquad::text::Font> {
    CJK_FONT.get().and_then(|f| f.as_ref())
}
