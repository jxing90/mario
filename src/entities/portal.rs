// Feature: Portal — warp doors with vortex animation
// Each portal has an id and dest_id. Player presses UP while overlapping
// to activate, then warps to the destination portal.

use crate::level::{AABB, KeyColor, Vec2};

/// A warp portal entity — closed by default, opens on UP input, warps player.
pub struct Portal {
    pub id: u32,
    pub dest_id: u32,
    pub pos: Vec2,
    pub open: bool,
    pub open_progress: f32,
    pub vortex_angle: f32,
    pub warp_cooldown: f32,
    pub open_timer: f32,
    /// If true, portal requires a matching-color key to open.
    pub locked: bool,
    /// Which key color unlocks this portal (None if not locked).
    pub key_color: Option<KeyColor>,
    /// If true, portal is permanently destroyed and can never open.
    pub destroyed: bool,
}

impl Portal {
    pub const W: f32 = 32.0;
    pub const H: f32 = 48.0;

    pub fn new(id: u32, dest_id: u32, pos: Vec2) -> Self {
        Self {
            id, dest_id, pos,
            open: false,
            open_progress: 0.0,
            vortex_angle: 0.0,
            warp_cooldown: 0.0,
            open_timer: 0.0,
            locked: false,
            key_color: None,
            destroyed: false,
        }
    }

    pub fn new_with(id: u32, dest_id: u32, pos: Vec2, locked: bool, key_color: Option<KeyColor>, destroyed: bool) -> Self {
        Self {
            id, dest_id, pos,
            open: false,
            open_progress: 0.0,
            vortex_angle: 0.0,
            warp_cooldown: 0.0,
            open_timer: 0.0,
            locked,
            key_color,
            destroyed,
        }
    }

    pub fn collider(&self) -> AABB {
        AABB {
            x: self.pos.x - Self::W / 2.0,
            y: self.pos.y - Self::H,
            w: Self::W,
            h: Self::H,
        }
    }

    pub fn warp_destination(&self) -> Vec2 {
        Vec2 {
            x: self.pos.x,
            y: self.pos.y - Self::H + 16.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.open && self.open_progress < 1.0 {
            self.open_progress = (self.open_progress + dt * 4.0).min(1.0);
        } else if !self.open && self.open_progress > 0.0 {
            self.open_progress = (self.open_progress - dt * 4.0).max(0.0);
        }
        if self.open_progress >= 1.0 && self.open {
            self.open_timer += dt;
            if self.open_timer >= 2.0 {
                self.open = false;
            }
        } else {
            self.open_timer = 0.0;
        }
        if self.open_progress > 0.0 {
            self.vortex_angle += dt * 4.0;
        }
        if self.warp_cooldown > 0.0 {
            self.warp_cooldown = (self.warp_cooldown - dt).max(0.0);
        }
    }

    /// Draw: single-panel door hinged on left.  When open, a dense layered
    /// spiral vortex fills the entire doorway edge-to-edge — no overflow.
    pub fn draw(
        &self, sx: f32, sy: f32,
        ws: &impl Fn(f32, f32) -> (f32, f32),
        _frame_count: u64,
    ) {
        use macroquad::color::Color;
        use macroquad::shapes::{draw_circle, draw_circle_lines, draw_ellipse, draw_line, draw_rectangle, draw_rectangle_lines};

        let (px, py) = ws(self.pos.x - Self::W / 2.0, self.pos.y - Self::H);
        let pw = Self::W * sx;
        let ph = Self::H * sy;
        let open_t = self.open_progress;
        let ft = 3.0 * sx.min(sy);

        // ── 0. Destroyed portal: broken frame + debris, no door/vortex/handle/chain ──
        if self.destroyed {
            let dc = Color::new(0.06, 0.05, 0.10, 1.0);
            // Dark void inside doorway
            draw_rectangle(px, py, pw, ph, Color::new(0.02, 0.01, 0.04, 1.0));
            // Broken frame fragments — gaps on each side
            // Left frame: top piece + bottom piece with gap
            draw_rectangle(px - ft, py - ft, ft, ph * 0.40, dc);
            draw_rectangle(px - ft, py + ph * 0.65, ft, ph * 0.35 + ft, dc);
            // Right frame: top piece + bottom piece with gap
            draw_rectangle(px + pw, py - ft, ft, ph * 0.55, dc);
            draw_rectangle(px + pw, py + ph * 0.80, ft, ph * 0.20 + ft, dc);
            // Top frame: broken
            draw_rectangle(px - ft, py - ft, pw * 0.30 + ft, ft, dc);
            draw_rectangle(px + pw * 0.55, py - ft, pw * 0.45 + ft, ft, dc);
            // Bottom frame: mostly intact but cracked
            draw_rectangle(px - ft, py + ph, pw * 0.70 + ft, ft, dc);
            draw_rectangle(px + pw * 0.85, py + ph, pw * 0.15 + ft, ft, dc);
            // Crack lines across the doorway
            let crack_c = Color::new(0.18, 0.14, 0.28, 0.7);
            draw_line(px + pw * 0.1, py + ph * 0.15, px + pw * 0.5, py + ph * 0.45, 1.2, crack_c);
            draw_line(px + pw * 0.5, py + ph * 0.45, px + pw * 0.35, py + ph * 0.75, 1.0, crack_c);
            draw_line(px + pw * 0.7, py + ph * 0.2, px + pw * 0.85, py + ph * 0.55, 1.0, crack_c);
            draw_line(px + pw * 0.85, py + ph * 0.55, px + pw * 0.6, py + ph * 0.9, 1.2, crack_c);
            // Debris rubble at bottom
            let rubble_c = Color::new(0.12, 0.10, 0.18, 0.75);
            for j in 0..5 {
                let rx = px + pw * (0.15 + j as f32 * 0.16);
                let ry = py + ph * (0.85 + (j % 3) as f32 * 0.04);
                let rs = 2.5 * sx.min(sy) * (0.6 + (j % 3) as f32 * 0.25);
                draw_rectangle(rx, ry, rs * 1.6, rs, rubble_c);
            }
            // Inner border — broken line segments
            let ib_c = Color::new(0.10, 0.08, 0.18, 0.6);
            draw_line(px + 2.0, py + 2.0, px + pw * 0.4, py + 2.0, 1.0, ib_c);
            draw_line(px + pw * 0.6, py + 2.0, px + pw - 2.0, py + 2.0, 1.0, ib_c);
            draw_line(px + 2.0, py + 2.0, px + 2.0, py + ph * 0.45, 1.0, ib_c);
            draw_line(px + pw - 2.0, py + 2.0, px + pw - 2.0, py + ph * 0.55, 1.0, ib_c);
            draw_line(px + 2.0, py + ph - 2.0, px + pw * 0.35, py + ph - 2.0, 1.0, ib_c);
            draw_line(px + pw * 0.5, py + ph - 2.0, px + pw - 2.0, py + ph - 2.0, 1.0, ib_c);
            return;
        }

        // ── 1. Dark abyss inside the doorway ──
        if open_t > 0.01 {
            draw_rectangle(px, py, pw, ph, Color::new(0.01, 0.00, 0.04, 1.0));
        }

        // ── 2. Dense vortex — fills the entire door, no spill ──
        if open_t > 0.02 {
            let cx = px + pw / 2.0;
            let cy = py + ph / 2.0;
            // Rings scale from center to the door edges (minus frame inset).
            let rx_max = pw / 2.0 - ft;
            let ry_max = ph / 2.0 - ft;
            let base_angle = self.vortex_angle;
            let ring_count = 16;

            for i in 0..ring_count {
                let t = i as f32 / (ring_count - 1) as f32;
                let rx = rx_max * (0.04 + t * 0.96);
                // Inner rings more circular, outer rings fill the door proportion
                let ry_ratio = 0.50 + t * 0.50;
                let ry = ry_max * (0.04 + t * 0.96) * ry_ratio;
                let alpha = 0.04 + t * 0.56;
                let phase = base_angle + t * 5.0;
                let (rc, gc, bc) = if i % 2 == 0 {
                    // Dark → mid purple
                    (0.25 + t * 0.40, 0.04 + t * 0.08, 0.45 + t * 0.45)
                } else {
                    // Mid → bright purple
                    (0.50 + t * 0.40, 0.10 + t * 0.15, 0.60 + t * 0.38)
                };
                draw_ellipse(cx, cy, rx, ry, phase,
                    Color::new(rc, gc, bc, alpha));
            }

            // Bright core
            let core_r = rx_max.min(ry_max) * 0.07;
            draw_circle(cx, cy, core_r, Color::new(1.0, 1.0, 1.0, 0.70));
            draw_circle(cx, cy, core_r * 0.35, Color::new(1.0, 1.0, 1.0, 0.90));
        }

        // ── 3. Door frame (thin border, within door perimeter) ──
        // Tint with key color when locked; neutral dark blue-purple when unlocked.
        let base_frame = if self.locked {
            if let Some(kc) = self.key_color {
                let k = crate::entities::key::key_color_rgba(kc);
                Color::new(
                    k.r * 0.6 + 0.12, k.g * 0.6 + 0.04, k.b * 0.6 + 0.20, 1.0,
                )
            } else {
                Color::new(0.30, 0.18, 0.50, 1.0)
            }
        } else {
            Color::new(0.22, 0.24, 0.42, 1.0) // unlocked: dark blue-purple
        };
        let frame_color = if open_t > 0.98 {
            Color::new(
                (base_frame.r * 1.5).min(1.0),
                (base_frame.g * 1.5).min(1.0),
                (base_frame.b * 1.5).min(1.0),
                1.0,
            )
        } else {
            base_frame
        };
        draw_rectangle(px - ft, py - ft, ft, ph + ft * 2.0, frame_color);
        draw_rectangle(px + pw, py - ft, ft, ph + ft * 2.0, frame_color);
        draw_rectangle(px - ft, py - ft, pw + ft * 2.0, ft, frame_color);
        draw_rectangle(px - ft, py + ph, pw + ft * 2.0, ft, frame_color);

        // ── 4. Door panel (hinged left, slides rightward as it opens) ──
        let panel_w = pw * (1.0 - open_t);
        if panel_w > 0.5 {
            let panel_fill = if self.locked && self.key_color.is_some() {
                let k = crate::entities::key::key_color_rgba(self.key_color.unwrap());
                Color::new(k.r * 0.35 + 0.05, k.g * 0.35 + 0.02, k.b * 0.35 + 0.10, 1.0)
            } else if self.locked {
                Color::new(0.15, 0.12, 0.25, 1.0)
            } else {
                Color::new(0.10, 0.12, 0.20, 1.0) // unlocked: dark inner
            };
            let panel_line = if self.locked && self.key_color.is_some() {
                let k = crate::entities::key::key_color_rgba(self.key_color.unwrap());
                Color::new(k.r * 0.6 + 0.15, k.g * 0.6 + 0.05, k.b * 0.6 + 0.25, 1.0)
            } else if self.locked {
                Color::new(0.45, 0.30, 0.75, 1.0)
            } else {
                Color::new(0.32, 0.35, 0.55, 1.0) // unlocked: outline
            };
            draw_rectangle(px, py, panel_w, ph, panel_fill);
            draw_rectangle_lines(px, py, panel_w, ph, 1.5, panel_line);
        }

        // Handle on right edge
        if panel_w > 6.0 * sx && open_t < 0.70 {
            let handle_x = px + panel_w - 5.0 * sx;
            let handle_y = py + ph * 0.48;
            draw_circle(handle_x, handle_y, 2.5 * sx.min(sy),
                Color::new(0.7, 0.6, 0.2, 1.0));
            draw_circle(handle_x, handle_y, 1.2 * sx.min(sy),
                Color::new(0.9, 0.8, 0.3, 1.0));
        }

        // ── 4.5 Lock: X-cross chains + center padlock ──
        if self.locked && !self.destroyed && panel_w > 0.5 && open_t < 0.70 {
            let cx = px + panel_w / 2.0;
            let cy = py + ph / 2.0;
            let inset = 4.0 * sx.min(sy);
            let link_r = 1.1 * sx.min(sy);
            let link_n = 10;
            let bright = Color::new(0.70, 0.70, 0.70, 0.95);
            let dark = Color::new(0.38, 0.38, 0.38, 0.95);

            // TL→BR chain
            let (x1, y1) = (px + inset, py + inset);
            let (x2, y2) = (px + panel_w - inset, py + ph - inset);
            for j in 0..link_n {
                let t = (j as f32 + 0.5) / link_n as f32;
                draw_circle(x1 + (x2 - x1) * t, y1 + (y2 - y1) * t, link_r,
                    if j % 2 == 0 { bright } else { dark });
            }
            // TR→BL chain
            let (x3, y3) = (px + panel_w - inset, py + inset);
            let (x4, y4) = (px + inset, py + ph - inset);
            for j in 0..link_n {
                let t = (j as f32 + 0.5) / link_n as f32;
                draw_circle(x3 + (x4 - x3) * t, y3 + (y4 - y3) * t, link_r,
                    if j % 2 == 1 { bright } else { dark });
            }
            // Padlock body at center
            let lw = 5.0 * sx;
            let lh = 6.5 * sy;
            draw_rectangle(cx - lw / 2.0, cy - lh / 2.0, lw, lh,
                Color::new(0.68, 0.56, 0.15, 0.95));
            draw_rectangle_lines(cx - lw / 2.0, cy - lh / 2.0, lw, lh, 1.0,
                Color::new(0.82, 0.72, 0.28, 0.9));
            // Keyhole
            draw_circle(cx, cy + 0.5 * sy, 1.0 * sx.min(sy),
                Color::new(0.12, 0.10, 0.04, 0.9));
            draw_rectangle(cx - 0.4 * sx, cy + 0.3 * sy, 0.8 * sx, 2.0 * sy,
                Color::new(0.12, 0.10, 0.04, 0.9));
            // Shackle arc
            draw_circle_lines(cx, cy - lh / 2.0 - 1.5 * sy, 2.5 * sx.min(sy), 1.3,
                Color::new(0.78, 0.68, 0.22, 0.95));
        }

        // ── 5. Inner decorative border ──
        draw_rectangle_lines(px, py, pw, ph, 1.5,
            Color::new(0.18, 0.12, 0.35, 0.9));
    }
}
