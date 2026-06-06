// Level Editor — visual map editor for Mario 2D Platformer levels.
// Standalone binary: `cargo run --bin editor`
//
// Features:
//   - Grid-snapped placement of all entity types
//   - Pan (arrow keys) / Zoom (+/- / mouse wheel)
//   - Save (Ctrl+S) / Load (F1-F4)
//   - Delete entities (right-click or Del)
//   - Tool palette with keyboard shortcuts

use std::fs;

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

// ============================================================================
// Data model (mirrors assets/levels/{n}.json)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Bounds {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    kill_y: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Pos {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EnemyDef {
    x: f32,
    y: f32,
    waypoint_a: Pos,
    waypoint_b: Pos,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OscFireballDef {
    x: f32,
    top_y: f32,
    bottom_y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LevelData {
    bounds: Bounds,
    #[serde(default)]
    platforms: Vec<Rect>,
    #[serde(default)]
    spikes: Vec<Pos>,
    #[serde(default)]
    coins: Vec<Pos>,
    #[serde(default)]
    question_blocks: Vec<Pos>,
    #[serde(default)]
    bricks: Vec<Pos>,
    #[serde(default)]
    enemies: Vec<EnemyDef>,
    #[serde(default)]
    dart_enemies: Vec<Pos>,
    #[serde(default)]
    osc_fireballs: Vec<OscFireballDef>,
    #[serde(default)]
    checkpoints: Vec<Pos>,
    flagpole: Pos,
    #[serde(default = "default_parallax")]
    parallax: Vec<f32>,
}

fn default_parallax() -> Vec<f32> {
    vec![0.1, 0.3, 0.6]
}

impl Default for LevelData {
    fn default() -> Self {
        Self {
            bounds: Bounds { min_x: 0.0, max_x: 3600.0, min_y: 0.0, kill_y: 2500.0 },
            platforms: vec![
                Rect { x: 0.0, y: 600.0, w: 700.0, h: 40.0 },
            ],
            spikes: vec![],
            coins: vec![],
            question_blocks: vec![],
            bricks: vec![],
            enemies: vec![],
            dart_enemies: vec![],
            osc_fireballs: vec![],
            checkpoints: vec![],
            flagpole: Pos { x: 3500.0, y: 560.0 },
            parallax: default_parallax(),
        }
    }
}

// ============================================================================
// Editor tools
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tool {
    Platform,
    Spike,
    Coin,
    QuestionBlock,
    Brick,
    Enemy,
    DartEnemy,
    OscFireball,
    Checkpoint,
    Flagpole,
    Eraser,
}

impl Tool {
    const ALL: &[Tool] = &[
        Tool::Platform,
        Tool::Spike,
        Tool::Coin,
        Tool::QuestionBlock,
        Tool::Brick,
        Tool::Enemy,
        Tool::DartEnemy,
        Tool::OscFireball,
        Tool::Checkpoint,
        Tool::Flagpole,
        Tool::Eraser,
    ];

    fn name(&self) -> &str {
        match self {
            Tool::Platform => "Platform",
            Tool::Spike => "Spike",
            Tool::Coin => "Coin",
            Tool::QuestionBlock => "Q-Block",
            Tool::Brick => "Brick",
            Tool::Enemy => "Enemy",
            Tool::DartEnemy => "DartEnemy",
            Tool::OscFireball => "OscFire",
            Tool::Checkpoint => "Checkpt",
            Tool::Flagpole => "Flagpole",
            Tool::Eraser => "Eraser",
        }
    }

    fn shortcut(&self) -> &str {
        match self {
            Tool::Platform => "1",
            Tool::Spike => "2",
            Tool::Coin => "3",
            Tool::QuestionBlock => "4",
            Tool::Brick => "5",
            Tool::Enemy => "6",
            Tool::DartEnemy => "7",
            Tool::OscFireball => "8",
            Tool::Checkpoint => "9",
            Tool::Flagpole => "0",
            Tool::Eraser => "Del",
        }
    }
}

// ============================================================================
// EditorState
// ============================================================================

pub struct EditorState {
    /// The level currently being edited.
    data: LevelData,
    /// Current level number (1-4).
    level_num: u32,
    /// Camera pan offset in world coordinates.
    cam_x: f32,
    cam_y: f32,
    /// Zoom factor (1.0 = 100%).
    zoom: f32,
    /// Active tool.
    tool: Tool,
    /// Grid snap enabled.
    grid_snap: bool,
    /// Screen dimensions.
    screen_w: f32,
    screen_h: f32,
    /// Status message shown at the bottom.
    status: String,
    status_timer: f32,
    /// Platform drawing mode: first corner placed.
    plat_start: Option<(f32, f32)>,
    /// Enemy drawing: first point (waypoint_a) placed.
    enemy_start: Option<(f32, f32)>,
    /// Osc fireball drawing: first point (top_y) placed.
    osc_start: Option<(f32, f32)>,
    /// Unsaved changes flag.
    dirty: bool,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            data: LevelData::default(),
            level_num: 1,
            cam_x: 0.0,
            cam_y: 0.0,
            zoom: 1.0,
            tool: Tool::Platform,
            grid_snap: true,
            screen_w: 1280.0,
            screen_h: 720.0,
            status: String::from("Editor ready. Ctrl+S to save, F1-F4 to load."),
            status_timer: 0.0,
            plat_start: None,
            enemy_start: None,
            osc_start: None,
            dirty: false,
        }
    }

    pub fn set_screen(&mut self, w: f32, h: f32) {
        self.screen_w = w;
        self.screen_h = h;
    }

    // ── Grid helpers ──

    const GRID: f32 = 16.0;

    fn snap(&self, v: f32) -> f32 {
        if self.grid_snap {
            (v / Self::GRID).round() * Self::GRID
        } else {
            v
        }
    }

    fn snap_pos(&self, wx: f32, wy: f32) -> (f32, f32) {
        (self.snap(wx), self.snap(wy))
    }

    // ── Coordinate transforms ──

    fn screen_to_world(&self, sx: f32, sy: f32) -> (f32, f32) {
        let wx = sx / self.zoom + self.cam_x;
        let wy = sy / self.zoom + self.cam_y;
        (wx, wy)
    }

    // ── IO ──

    fn level_path(level_num: u32) -> String {
        format!("assets/levels/{}.json", level_num)
    }

    fn set_status(&mut self, msg: &str) {
        self.status = msg.to_string();
        self.status_timer = 3.0;
    }

    pub fn load_level(&mut self, n: u32) {
        let path = Self::level_path(n);
        match fs::read_to_string(&path) {
            Ok(json) => match serde_json::from_str::<LevelData>(&json) {
                Ok(data) => {
                    self.data = data;
                    self.level_num = n;
                    self.dirty = false;
                    self.set_status(&format!("Loaded level {}", n));
                }
                Err(e) => self.set_status(&format!("Parse error: {}", e)),
            },
            Err(e) => self.set_status(&format!("Cannot read {}: {}", path, e)),
        }
    }

    pub fn save_level(&mut self) {
        let path = Self::level_path(self.level_num);
        match serde_json::to_string_pretty(&self.data) {
            Ok(json) => match fs::write(&path, json) {
                Ok(()) => {
                    self.dirty = false;
                    self.set_status(&format!("Saved level {} ✓", self.level_num));
                }
                Err(e) => self.set_status(&format!("Write error: {}", e)),
            },
            Err(e) => self.set_status(&format!("Serialize error: {}", e)),
        }
    }

    // ── Editing operations ──

    fn place_at(&mut self, wx: f32, wy: f32) {
        let (x, y) = self.snap_pos(wx, wy);
        self.dirty = true;

        match self.tool {
            Tool::Platform => {
                if self.plat_start.is_none() {
                    self.plat_start = Some((x, y));
                    self.set_status("Click second corner for platform size...");
                } else {
                    let (sx, sy) = self.plat_start.take().unwrap();
                    let rx = x.min(sx);
                    let ry = y.min(sy);
                    let rw = (x - sx).abs().max(Self::GRID);
                    let rh = (y - sy).abs().max(Self::GRID);
                    self.data.platforms.push(Rect { x: rx, y: ry, w: rw, h: rh });
                    self.set_status("Platform placed.");
                }
            }
            Tool::Spike => {
                self.data.spikes.push(Pos { x, y });
                self.set_status("Spike placed.");
            }
            Tool::Coin => {
                self.data.coins.push(Pos { x, y });
                self.set_status("Coin placed.");
            }
            Tool::QuestionBlock => {
                self.data.question_blocks.push(Pos { x, y });
                self.set_status("Question block placed.");
            }
            Tool::Brick => {
                self.data.bricks.push(Pos { x, y });
                self.set_status("Brick placed.");
            }
            Tool::Enemy => {
                if self.enemy_start.is_none() {
                    self.enemy_start = Some((x, y));
                    self.set_status("Click waypoint_b for enemy patrol range...");
                } else {
                    let (ax, ay) = self.enemy_start.take().unwrap();
                    let way_a = Pos { x: ax, y: ay };
                    let way_b = Pos { x, y };
                    self.data.enemies.push(EnemyDef {
                        x: ax,
                        y: ay,
                        waypoint_a: way_a,
                        waypoint_b: way_b,
                    });
                    self.set_status("Enemy placed.");
                }
            }
            Tool::DartEnemy => {
                self.data.dart_enemies.push(Pos { x, y });
                self.set_status("Dart enemy placed.");
            }
            Tool::OscFireball => {
                if self.osc_start.is_none() {
                    self.osc_start = Some((x, y));
                    self.set_status("Click bottom_y for fireball travel range...");
                } else {
                    let (fx, top_y) = self.osc_start.take().unwrap();
                    let bottom_y = y;
                    self.data.osc_fireballs.push(OscFireballDef {
                        x: fx,
                        top_y: top_y.min(bottom_y),
                        bottom_y: top_y.max(bottom_y),
                    });
                    self.set_status("Oscillating fireball placed.");
                }
            }
            Tool::Checkpoint => {
                self.data.checkpoints.push(Pos { x, y });
                self.set_status("Checkpoint placed.");
            }
            Tool::Flagpole => {
                self.data.flagpole = Pos { x, y };
                self.set_status("Flagpole moved.");
            }
            Tool::Eraser => {
                self.delete_at(x, y);
            }
        }
    }

    fn delete_at(&mut self, wx: f32, wy: f32) {
        let (x, y) = self.snap_pos(wx, wy);
        let tol = Self::GRID;

        let before = self.entity_count();

        self.data.platforms.retain(|p| !(x >= p.x - tol && x <= p.x + p.w + tol && y >= p.y - tol && y <= p.y + p.h + tol));
        self.data.spikes.retain(|p| (p.x - x).abs() > tol || (p.y - y).abs() > tol);
        self.data.coins.retain(|p| (p.x - x).abs() > tol || (p.y - y).abs() > tol);
        self.data.question_blocks.retain(|p| (p.x - x).abs() > tol || (p.y - y).abs() > tol);
        self.data.bricks.retain(|p| (p.x - x).abs() > tol || (p.y - y).abs() > tol);
        self.data.enemies.retain(|e| (e.x - x).abs() > tol || (e.y - y).abs() > tol);
        self.data.dart_enemies.retain(|p| (p.x - x).abs() > tol || (p.y - y).abs() > tol);
        self.data.osc_fireballs.retain(|o| (o.x - x).abs() > tol || (o.top_y - y).abs() > tol);
        self.data.checkpoints.retain(|p| (p.x - x).abs() > tol || (p.y - y).abs() > tol);

        let after = self.entity_count();
        if after < before {
            self.dirty = true;
            self.set_status("Entity deleted.");
        }
    }

    fn entity_count(&self) -> usize {
        self.data.platforms.len()
            + self.data.spikes.len()
            + self.data.coins.len()
            + self.data.question_blocks.len()
            + self.data.bricks.len()
            + self.data.enemies.len()
            + self.data.dart_enemies.len()
            + self.data.osc_fireballs.len()
            + self.data.checkpoints.len()
    }

    fn cancel_pending(&mut self) {
        if self.plat_start.is_some() || self.enemy_start.is_some() || self.osc_start.is_some() {
            self.plat_start = None;
            self.enemy_start = None;
            self.osc_start = None;
            self.set_status("Cancelled.");
        }
    }

    // ── Update ──

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
        if is_key_down(KeyCode::LeftControl) && is_key_pressed(KeyCode::S) {
            self.save_level();
        }
        if is_key_pressed(KeyCode::F1) { self.load_level(1); }
        if is_key_pressed(KeyCode::F2) { self.load_level(2); }
        if is_key_pressed(KeyCode::F3) { self.load_level(3); }
        if is_key_pressed(KeyCode::F4) { self.load_level(4); }

        // Cancel pending operation
        if is_key_pressed(KeyCode::Escape) {
            self.cancel_pending();
        }

        // --- Mouse ---
        let (mx, my) = mouse_position();

        // Place on left click (only when mouse is in canvas area, not toolbar)
        if is_mouse_button_pressed(MouseButton::Left) && my > 40.0 {
            let (wx, wy) = self.screen_to_world(mx, my);
            self.place_at(wx, wy);
        }

        // Delete on right click
        if is_mouse_button_pressed(MouseButton::Right) && my > 40.0 {
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

    // ── Render ──

    pub fn render(&self) {
        clear_background(Color::new(0.15, 0.15, 0.22, 1.0));

        let sw = self.screen_w;
        let sh = self.screen_h;
        if sw <= 0.0 || sh <= 0.0 { return; }

        // ── World-to-screen transform ──
        let ws = |wx: f32, wy: f32| -> (f32, f32) {
            ((wx - self.cam_x) * self.zoom, (wy - self.cam_y) * self.zoom + 40.0)
        };

        // ── Grid ──
        let grid_color = Color::new(0.25, 0.25, 0.35, 0.5);
        let grid_step = Self::GRID * self.zoom;
        if grid_step > 4.0 {
            let start_x = (self.cam_x / Self::GRID).floor() * Self::GRID;
            let start_y = (self.cam_y / Self::GRID).floor() * Self::GRID;
            let end_x = self.cam_x + sw / self.zoom;
            let end_y = self.cam_y + (sh - 40.0) / self.zoom;
            let mut gx = start_x;
            while gx <= end_x {
                let (sx, sy1) = ws(gx, start_y);
                let (_, sy2) = ws(gx, end_y);
                draw_line(sx, sy1, sx, sy2, 1.0, grid_color);
                gx += Self::GRID;
            }
            let mut gy = start_y;
            while gy <= end_y {
                let (sx1, sy) = ws(start_x, gy);
                let (sx2, _) = ws(end_x, gy);
                draw_line(sx1, sy, sx2, sy, 1.0, grid_color);
                gy += Self::GRID;
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

        // ── Toolbar (top 40px) ──
        draw_rectangle(0.0, 0.0, sw, 40.0, Color::new(0.1, 0.1, 0.18, 1.0));
        let mut tx = 8.0;
        for &tool in Tool::ALL {
            let name = tool.name();
            let sc = tool.shortcut();
            let label = format!("[{}] {}", sc, name);
            let fw = 12.0 * label.len() as f32 * 0.55;
            let bg = if self.tool == tool {
                Color::new(0.3, 0.5, 0.9, 0.9)
            } else {
                Color::new(0.2, 0.2, 0.3, 0.7)
            };
            draw_rectangle(tx - 2.0, 4.0, fw + 6.0, 32.0, bg);
            draw_text(&label, tx, 28.0, 14.0, WHITE);
            tx += fw + 10.0;
        }

        // Level indicator
        let lvl_text = format!("Level: {} {}", self.level_num, if self.dirty { "*" } else { "" });
        draw_text(&lvl_text, sw - 140.0, 28.0, 16.0, if self.dirty { YELLOW } else { WHITE });
        draw_text("ESC:cancel", sw - 280.0, 28.0, 12.0, GRAY);
        draw_text("G:grid", sw - 360.0, 28.0, 12.0, if self.grid_snap { WHITE } else { GRAY });

        // ── Status bar (bottom) ──
        if self.status_timer > 0.0 {
            draw_rectangle(0.0, sh - 28.0, sw, 28.0, Color::new(0.0, 0.0, 0.0, 0.8));
            draw_text(&self.status, 8.0, sh - 6.0, 16.0, Color::new(0.7, 1.0, 0.7, 1.0));
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
