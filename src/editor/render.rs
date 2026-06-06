use macroquad::prelude::*;
use crate::editor::state::EditorState;
use crate::editor::tool::Tool;

impl EditorState {
    pub fn render(&mut self) {
        clear_background(Color::new(0.15, 0.15, 0.22, 1.0));

        let sw = self.screen_w;
        let sh = self.screen_h;
        if sw <= 0.0 || sh <= 0.0 { return; }

        // ── World-to-screen transform ──
        let cam_x = self.cam_x;
        let cam_y = self.cam_y;
        let zoom = self.zoom;
        let ws = |wx: f32, wy: f32| -> (f32, f32) {
            ((wx - cam_x) * zoom, (wy - cam_y) * zoom + 40.0)
        };

        // ── Grid ──
        let grid_color = Color::new(0.25, 0.25, 0.35, 0.5);
        let grid_step = EditorState::GRID * self.zoom;
        if grid_step > 4.0 {
            let start_x = (self.cam_x / EditorState::GRID).floor() * EditorState::GRID;
            let start_y = (self.cam_y / EditorState::GRID).floor() * EditorState::GRID;
            let end_x = self.cam_x + sw / self.zoom;
            let end_y = self.cam_y + (sh - 40.0) / self.zoom;
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
            let c = if p.y >= 590.0 {
                Color::new(0.25, 0.55, 0.15, 0.8)  // ground
            } else {
                Color::new(0.45, 0.28, 0.10, 0.8)  // platform
            };
            draw_rectangle(px, py, p.w * self.zoom, p.h * self.zoom, c);
        }

        // ── Spikes ──
        for s in &self.data.spikes {
            let (sx, sy) = ws(s.x, s.y);
            let hw = 8.0 * self.zoom;
            let hh = 4.0 * self.zoom;
            draw_triangle(
                Vec2::new(sx, sy + hh),
                Vec2::new(sx - hw, sy - hh),
                Vec2::new(sx + hw, sy - hh),
                Color::new(0.9, 0.2, 0.1, 0.9),
            );
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
            // Waypoint lines
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
            let (ox, oy) = ws(o.x, o.top_y);
            let (_, by) = ws(o.x, o.bottom_y);
            draw_circle(ox, oy, 6.0 * self.zoom, Color::new(1.0, 0.5, 0.0, 0.8));
            draw_circle(ox, by, 6.0 * self.zoom, Color::new(1.0, 0.3, 0.0, 0.5));
            draw_line(ox, oy, ox, by, 1.0, Color::new(1.0, 0.4, 0.0, 0.4));
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
            // Mario silhouette: red hat + blue overalls
            let sz = 12.0 * self.zoom;
            draw_rectangle(sx - sz * 0.5, sy - sz * 2.0, sz, sz * 0.3, RED);
            draw_rectangle(sx - sz * 0.5, sy - sz * 1.7, sz, sz * 0.4, Color::new(1.0, 0.75, 0.55, 1.0));
            draw_rectangle(sx - sz * 0.5, sy - sz * 1.3, sz, sz * 0.5, Color::new(0.1, 0.3, 0.9, 1.0));
            draw_text("START", sx + sz * 0.7, sy - sz * 1.2, 10.0 * self.zoom, YELLOW);
        }

        // ── Pending placement preview ──
        if let Some((sx, sy)) = self.plat_start {
            let (mx, my) = mouse_position();
            if my > 40.0 {
                let (wx, wy) = self.screen_to_world(mx, my);
                let (gx, gy) = self.snap_pos(wx, wy);
                let rx = gx.min(sx);
                let ry = gy.min(sy);
                let rw = (gx - sx).abs();
                let rh = (gy - sy).abs();
                let (prx, pry) = ws(rx, ry);
                draw_rectangle_lines(prx, pry, rw * self.zoom, rh * self.zoom, 2.0, YELLOW);
            }
        }
        if let Some((sx, sy)) = self.enemy_start {
            let (mx, my) = mouse_position();
            if my > 40.0 {
                let (wx, wy) = self.screen_to_world(mx, my);
                let (gx, gy) = self.snap_pos(wx, wy);
                let (p1x, p1y) = ws(sx, sy);
                let (p2x, p2y) = ws(gx, gy);
                draw_line(p1x, p1y, p2x, p2y, 2.0, YELLOW);
            }
        }
        if let Some((_sx, sy)) = self.osc_start {
            let (mx, my) = mouse_position();
            if my > 40.0 {
                let (wx, wy) = self.screen_to_world(mx, my);
                let (gx, _) = self.snap_pos(wx, wy);
                let (p1x, p1y) = ws(gx, sy);
                let (p2x, p2y) = ws(gx, wy);
                draw_line(p1x, p1y, p2x, p2y, 2.0, Color::new(1.0, 0.6, 0.0, 0.8));
            }
        }

        // ── Toolbar (top 40px, clickable) ──
        draw_rectangle(0.0, 0.0, sw, 40.0, Color::new(0.1, 0.1, 0.18, 1.0));
        let mut tx = 8.0;
        self.tool_rects.clear();
        for &tool in Tool::ALL {
            let name = tool.name();
            let sc = tool.shortcut();
            let label = format!("[{}] {}", sc, name);
            let fw = 12.0 * label.len() as f32 * 0.55;
            let bw = fw + 6.0;
            let bh = 32.0;
            let (mx, my) = mouse_position();
            let hover = mx >= tx - 2.0 && mx <= tx + bw && my >= 4.0 && my <= 36.0;
            let bg = if self.tool == tool {
                Color::new(0.3, 0.5, 0.9, 0.9)
            } else if hover {
                Color::new(0.3, 0.35, 0.5, 0.8)
            } else {
                Color::new(0.2, 0.2, 0.3, 0.7)
            };
            draw_rectangle(tx - 2.0, 4.0, bw, bh, bg);
            draw_text(&label, tx, 28.0, 14.0, WHITE);
            self.tool_rects.push((tool, tx - 2.0, 4.0, bw, bh));
            tx += bw + 4.0;
        }

        // Toolbar separator line
        draw_line(0.0, 40.0, sw, 40.0, 2.0, Color::new(0.3, 0.3, 0.5, 0.8));

        // Toolbar click detection
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            for &(tool, rx, ry, rw, rh) in &self.tool_rects {
                if mx >= rx && mx <= rx + rw && my >= ry && my <= ry + rh {
                    self.tool = tool;
                    self.cancel_pending();
                    break;
                }
            }
        }

        // Level indicator & info
        let lvl_text = format!("Level: {} {}", self.level_num, if self.dirty { "*" } else { "" });
        draw_text(&lvl_text, sw - 140.0, 28.0, 16.0, if self.dirty { YELLOW } else { WHITE });
        draw_text("ESC:cancel", sw - 280.0, 28.0, 12.0, GRAY);
        draw_text("G:grid", sw - 360.0, 28.0, 12.0, if self.grid_snap { WHITE } else { GRAY });

        // ── Status bar (bottom) ──
        if self.status_timer > 0.0 {
            draw_rectangle(0.0, sh - 28.0, sw, 28.0, Color::new(0.0, 0.0, 0.0, 0.8));
            draw_text(&self.status, 8.0, sh - 6.0, 16.0, Color::new(0.7, 1.0, 0.7, 1.0));
        }

        // ── Save-As dialog ──
        if self.save_as_mode {
            let dialog_w = 500.0;
            let dialog_h = 80.0;
            let dx = (sw - dialog_w) / 2.0;
            let dy = sh - 140.0;
            draw_rectangle(dx, dy, dialog_w, dialog_h, Color::new(0.05, 0.05, 0.15, 1.0));
            draw_rectangle_lines(dx, dy, dialog_w, dialog_h, 2.0, Color::new(0.3, 0.5, 0.9, 1.0));
            draw_text("Save As — Enter path, then press Enter:", dx + 8.0, dy + 20.0, 14.0, GRAY);
            draw_text(&self.save_path_buf, dx + 8.0, dy + 48.0, 18.0, WHITE);
            // Blinking cursor
            let cursor_x = dx + 8.0 + 10.0 * self.save_path_buf.len() as f32;
            draw_line(cursor_x, dy + 32.0, cursor_x, dy + 56.0, 2.0, Color::new(1.0, 1.0, 0.0, 0.8));
        }

        // ── Mouse cursor crosshair ──
        let (mx, my) = mouse_position();
        if my > 40.0 {
            let (gx, gy) = self.snap_pos(
                mx / self.zoom + self.cam_x,
                (my - 40.0) / self.zoom + self.cam_y,
            );
            let (csx, csy) = ws(gx, gy);
            draw_circle_lines(csx, csy, 4.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.6));

            // Show world coordinates
            let coord_text = format!("{:.0}, {:.0}", gx, gy);
            draw_text(&coord_text, mx + 12.0, my - 4.0, 12.0, Color::new(1.0, 1.0, 1.0, 0.8));
        }
    }
}
