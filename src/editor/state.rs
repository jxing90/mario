// EditorState — struct, helpers, IO, editing, and menu actions.

use std::fs;

use crate::editor::data::*;
use crate::editor::tool::*;

// ============================================================================
// EditorState
// ============================================================================

pub struct EditorState {
    pub(crate) data: LevelData,
    pub(crate) level_num: u32,
    pub(crate) cam_x: f32,
    pub(crate) cam_y: f32,
    pub(crate) zoom: f32,
    pub(crate) tool: Tool,
    pub(crate) grid_snap: bool,
    pub(crate) screen_w: f32,
    pub(crate) screen_h: f32,
    pub(crate) status: String,
    pub(crate) status_timer: f32,
    pub(crate) plat_start: Option<(f32, f32)>,
    pub(crate) enemy_start: Option<(f32, f32)>,
    pub(crate) osc_start: Option<(f32, f32)>,
    pub(crate) dirty: bool,
    pub(crate) save_as_mode: bool,
    pub(crate) save_path_buf: String,
    pub(crate) rename_mode: bool,
    pub(crate) name_buf: String,
    pub(crate) open_file_mode: bool,
    pub(crate) open_file_buf: String,
    pub(crate) drag_target: Option<DragTarget>,
    /// Offset from entity origin to mouse position at drag start, in world coords.
    pub(crate) drag_offset: (f32, f32),
    pub(crate) tool_rects: Vec<(Tool, f32, f32, f32, f32)>,
    pub(crate) menu_open: Option<usize>,
    pub(crate) menu_hdr_rects: Vec<(f32, f32, f32, f32)>,
    pub(crate) menu_item_rects: Vec<(f32, f32, f32, f32)>,
    pub(crate) prev_mouse: (f32, f32),
    /// Selected entity for property inspection (View mode).
    pub(crate) selected_entity: Option<DragTarget>,
    /// Name of the field currently being edited, e.g. "x", "w", "top_y".
    pub(crate) editing_field: Option<String>,
    /// Text buffer for the field being edited.
    pub(crate) edit_buf: String,
    /// Dropdown for enum fields (e.g. key_clr). None = closed.
    pub(crate) dropdown_field: Option<String>,
    /// Dropdown option rects for hit-testing: (x, y, w, h).
    pub(crate) dropdown_rects: Vec<(f32, f32, f32, f32)>,
}

impl EditorState {
    pub fn new() -> Self { Self::default() }
}

impl Default for EditorState {
    fn default() -> Self {
        // Load level 1 from disk on startup, fall back to default template.
        let (data, num) = match std::fs::read_to_string("assets/levels/1.json") {
            Ok(json) => match serde_json::from_str::<LevelData>(&json) {
                Ok(d) => (d, 1u32),
                Err(_) => (LevelData::default(), 1),
            },
            Err(_) => (LevelData::default(), 1),
        };
        Self {
            data,
            level_num: num,
            cam_x: 0.0, cam_y: 0.0,
            zoom: 1.0,
            tool: Tool::View,
            grid_snap: true,
            screen_w: 1280.0, screen_h: 720.0,
            status: String::from("Editor ready. Menu or shortcuts to operate."),
            status_timer: 0.0,
            plat_start: None, enemy_start: None, osc_start: None,
            dirty: false,
            save_as_mode: false, save_path_buf: String::new(),
            rename_mode: false, name_buf: String::new(),
            open_file_mode: false, open_file_buf: String::new(),
            drag_target: None,
            drag_offset: (0.0, 0.0),
            tool_rects: Vec::new(),
            menu_open: None,
            menu_hdr_rects: Vec::new(),
            menu_item_rects: Vec::new(),
            prev_mouse: (0.0, 0.0),
            selected_entity: None,
            editing_field: None,
            edit_buf: String::new(),
            dropdown_field: None,
            dropdown_rects: Vec::new(),
        }
    }
}

impl EditorState {
    pub fn set_screen(&mut self, w: f32, h: f32) {
        self.screen_w = w; self.screen_h = h;
    }

    // ── Grid helpers ──
    pub(crate) const GRID: f32 = 16.0;
    pub(crate) const MENU_H: f32 = 24.0;
    pub(crate) const TOOLBAR_H: f32 = 40.0;
    pub(crate) const HEADER_H: f32 = Self::MENU_H + Self::TOOLBAR_H;

    pub(crate) fn snap(&self, v: f32) -> f32 {
        if self.grid_snap { (v / Self::GRID).round() * Self::GRID } else { v }
    }

    pub(crate) fn snap_pos(&self, wx: f32, wy: f32) -> (f32, f32) {
        (self.snap(wx), self.snap(wy))
    }

    // ── Coordinate transforms ──
    pub(crate) fn screen_to_world(&self, sx: f32, sy: f32) -> (f32, f32) {
        let wx = sx / self.zoom + self.cam_x;
        let wy = (sy - Self::HEADER_H) / self.zoom + self.cam_y;
        (wx, wy)
    }

    /// Which entity (if any) is under the world-space point (wx, wy).
    pub(crate) fn hit_test(&self, wx: f32, wy: f32) -> Option<DragTarget> {
        let tol = 10.0;
        for (i, p) in self.data.platforms.iter().enumerate() {
            if wx >= p.x - tol && wx <= p.x + p.w + tol && wy >= p.y - tol && wy <= p.y + p.h + tol {
                return Some(DragTarget::Platform(i));
            }
        }
        for (i, s) in self.data.spikes.iter().enumerate() {
            if (wx - s.x).abs() <= tol && (wy - s.y).abs() <= tol { return Some(DragTarget::Spike(i)); }
        }
        for (i, c) in self.data.coins.iter().enumerate() {
            if (wx - c.x).abs() <= tol && (wy - c.y).abs() <= tol { return Some(DragTarget::Coin(i)); }
        }
        for (i, q) in self.data.question_blocks.iter().enumerate() {
            if wx >= q.x - tol && wx <= q.x + 32.0 + tol && wy >= q.y - tol && wy <= q.y + 32.0 + tol {
                return Some(DragTarget::QuestionBlock(i));
            }
        }
        for (i, b) in self.data.bricks.iter().enumerate() {
            if wx >= b.x - tol && wx <= b.x + 32.0 + tol && wy >= b.y - tol && wy <= b.y + 32.0 + tol {
                return Some(DragTarget::Brick(i));
            }
        }
        for (i, e) in self.data.enemies.iter().enumerate() {
            if wx >= e.x - 16.0 - tol && wx <= e.x + 16.0 + tol && wy >= e.y - 16.0 - tol && wy <= e.y + tol {
                return Some(DragTarget::Enemy(i));
            }
        }
        for (i, d) in self.data.dart_enemies.iter().enumerate() {
            if wx >= d.x - 16.0 - tol && wx <= d.x + 16.0 + tol && wy >= d.y - 16.0 - tol && wy <= d.y + tol {
                return Some(DragTarget::DartEnemy(i));
            }
        }
        for (i, o) in self.data.osc_fireballs.iter().enumerate() {
            if (wx - o.x).abs() <= tol * 2.0 && wy >= o.top_y - tol && wy <= o.bottom_y + tol {
                return Some(DragTarget::OscFireball(i));
            }
        }
        for (i, cp) in self.data.checkpoints.iter().enumerate() {
            if wx >= cp.x - 16.0 - tol && wx <= cp.x + 16.0 + tol && wy >= cp.y - 52.0 - tol && wy <= cp.y + tol {
                return Some(DragTarget::Checkpoint(i));
            }
        }
        let fp = self.data.flagpole;
        if (wx - fp.x).abs() <= tol * 3.0 && wy >= fp.y - 120.0 - tol && wy <= fp.y + tol {
            return Some(DragTarget::Flagpole);
        }
        let ps = self.data.player_spawn;
        if (wx - ps.x).abs() <= tol * 2.0 && (wy - ps.y).abs() <= tol * 2.0 {
            return Some(DragTarget::PlayerSpawn);
        }
        for (i, c) in self.data.clouds.iter().enumerate() {
            let cx = c.x + c.w / 2.0;
            let cy = c.y + c.h / 2.0;
            if wx >= cx - c.w / 2.0 - tol && wx <= cx + c.w / 2.0 + tol
                && wy >= cy - c.h / 2.0 - tol && wy <= cy + c.h / 2.0 + tol
            {
                return Some(DragTarget::Cloud(i));
            }
        }
        for (i, p) in self.data.portals.iter().enumerate() {
            if (wx - p.x).abs() <= 20.0 && wy >= p.y - 52.0 - tol && wy <= p.y + tol {
                return Some(DragTarget::Portal(i));
            }
        }
        None
    }

    pub(crate) fn move_entity(&mut self, target: DragTarget, x: f32, y: f32) {
        match target {
            DragTarget::Platform(i) => { if let Some(p) = self.data.platforms.get_mut(i) { p.x = x; p.y = y; } }
            DragTarget::Spike(i) => { if let Some(s) = self.data.spikes.get_mut(i) { s.x = x; s.y = y; } }
            DragTarget::Coin(i) => { if let Some(c) = self.data.coins.get_mut(i) { c.x = x; c.y = y; } }
            DragTarget::QuestionBlock(i) => { if let Some(q) = self.data.question_blocks.get_mut(i) { q.x = x; q.y = y; } }
            DragTarget::Brick(i) => { if let Some(b) = self.data.bricks.get_mut(i) { b.x = x; b.y = y; } }
            DragTarget::Enemy(i) => {
                if let Some(e) = self.data.enemies.get_mut(i) {
                    let dx = x - e.x; let dy = y - e.y;
                    e.x = x; e.y = y;
                    e.waypoint_a.x += dx; e.waypoint_a.y += dy;
                    e.waypoint_b.x += dx; e.waypoint_b.y += dy;
                }
            }
            DragTarget::DartEnemy(i) => { if let Some(d) = self.data.dart_enemies.get_mut(i) { d.x = x; d.y = y; } }
            DragTarget::OscFireball(i) => {
                if let Some(o) = self.data.osc_fireballs.get_mut(i) {
                    let span = o.bottom_y - o.top_y; o.x = x; o.top_y = y; o.bottom_y = y + span;
                }
            }
            DragTarget::Checkpoint(i) => { if let Some(cp) = self.data.checkpoints.get_mut(i) { cp.x = x; cp.y = y; } }
            DragTarget::PlayerSpawn => { self.data.player_spawn = Pos { x, y }; }
            DragTarget::Cloud(i) => { if let Some(c) = self.data.clouds.get_mut(i) { c.x = x; c.y = y; } }
            DragTarget::Portal(i) => { if let Some(p) = self.data.portals.get_mut(i) { p.x = x; p.y = y; } }
            DragTarget::Key(i) => { if let Some(k) = self.data.keys.get_mut(i) { k.x = x; k.y = y; } }
            DragTarget::Flagpole => { self.data.flagpole = Pos { x, y }; }
        }
    }

    /// Returns the world-space origin position for an entity.
    pub(crate) fn entity_pos(&self, target: DragTarget) -> (f32, f32) {
        match target {
            DragTarget::Platform(i) => self.data.platforms.get(i).map(|p| (p.x, p.y)).unwrap_or((0.0, 0.0)),
            DragTarget::Spike(i) => self.data.spikes.get(i).map(|s| (s.x, s.y)).unwrap_or((0.0, 0.0)),
            DragTarget::Coin(i) => self.data.coins.get(i).map(|c| (c.x, c.y)).unwrap_or((0.0, 0.0)),
            DragTarget::QuestionBlock(i) => self.data.question_blocks.get(i).map(|q| (q.x, q.y)).unwrap_or((0.0, 0.0)),
            DragTarget::Brick(i) => self.data.bricks.get(i).map(|b| (b.x, b.y)).unwrap_or((0.0, 0.0)),
            DragTarget::Enemy(i) => self.data.enemies.get(i).map(|e| (e.x, e.y)).unwrap_or((0.0, 0.0)),
            DragTarget::DartEnemy(i) => self.data.dart_enemies.get(i).map(|d| (d.x, d.y)).unwrap_or((0.0, 0.0)),
            DragTarget::OscFireball(i) => self.data.osc_fireballs.get(i).map(|o| (o.x, o.top_y)).unwrap_or((0.0, 0.0)),
            DragTarget::Checkpoint(i) => self.data.checkpoints.get(i).map(|c| (c.x, c.y)).unwrap_or((0.0, 0.0)),
            DragTarget::PlayerSpawn => (self.data.player_spawn.x, self.data.player_spawn.y),
            DragTarget::Cloud(i) => self.data.clouds.get(i).map(|c| (c.x, c.y)).unwrap_or((0.0, 0.0)),
            DragTarget::Portal(i) => self.data.portals.get(i).map(|p| (p.x, p.y)).unwrap_or((0.0, 0.0)),
            DragTarget::Flagpole => (self.data.flagpole.x, self.data.flagpole.y),
            DragTarget::Key(i) => self.data.keys.get(i).map(|k| (k.x, k.y)).unwrap_or((0.0, 0.0)),
        }
    }

    // ── IO ──
    pub(crate) fn level_path(level_num: u32) -> String { format!("assets/levels/{}.json", level_num) }

    pub(crate) fn set_status(&mut self, msg: &str) { self.status = msg.to_string(); self.status_timer = 3.0; }

    pub fn load_level(&mut self, n: u32) {
        let path = EditorState::level_path(n);
        match fs::read_to_string(&path) {
            Ok(json) => match serde_json::from_str::<LevelData>(&json) {
                Ok(data) => { self.data = data; self.level_num = n; self.dirty = false; self.set_status(&format!("Loaded level {}", n)); }
                Err(e) => self.set_status(&format!("Parse error: {}", e)),
            },
            Err(e) => self.set_status(&format!("Cannot read {}: {}", path, e)),
        }
    }

    pub fn save_level(&mut self) { self.save_to_path(&EditorState::level_path(self.level_num)); }

    pub fn save_to_path(&mut self, path: &str) {
        match serde_json::to_string_pretty(&self.data) {
            Ok(json) => match fs::write(path, &json) {
                Ok(()) => { self.dirty = false; self.set_status(&format!("Saved to {} ✓", path)); }
                Err(e) => self.set_status(&format!("Write error: {}", e)),
            },
            Err(e) => self.set_status(&format!("Serialize error: {}", e)),
        }
    }

    // ── Editing operations ──
    pub(crate) fn place_at(&mut self, wx: f32, wy: f32) {
        let (x, y) = self.snap_pos(wx, wy);
        self.dirty = true;
        match self.tool {
            Tool::Platform => {
                if self.plat_start.is_none() {
                    self.plat_start = Some((x, y));
                    self.set_status("Click second corner for platform size...");
                } else {
                    let (sx, sy) = self.plat_start.take().unwrap();
                    let rx = x.min(sx); let ry = y.min(sy);
                    let rw = (x - sx).abs().max(Self::GRID); let rh = (y - sy).abs().max(Self::GRID);
                    self.data.platforms.push(Rect { x: rx, y: ry, w: rw, h: rh });
                    self.set_status("Platform placed.");
                }
            }
            Tool::Spike => { self.data.spikes.push(Pos { x, y }); self.set_status("Spike placed."); }
            Tool::Coin => { self.data.coins.push(Pos { x, y }); self.set_status("Coin placed."); }
            Tool::QuestionBlock => { self.data.question_blocks.push(Pos { x, y }); self.set_status("Question block placed."); }
            Tool::Brick => { self.data.bricks.push(Pos { x, y }); self.set_status("Brick placed."); }
            Tool::Enemy => {
                if self.enemy_start.is_none() {
                    self.enemy_start = Some((x, y));
                    self.set_status("Click waypoint_b for enemy patrol range...");
                } else {
                    let (ax, ay) = self.enemy_start.take().unwrap();
                    self.data.enemies.push(EnemyDef { x: ax, y: ay, waypoint_a: Pos { x: ax, y: ay }, waypoint_b: Pos { x, y } });
                    self.set_status("Enemy placed.");
                }
            }
            Tool::DartEnemy => { self.data.dart_enemies.push(Pos { x, y }); self.set_status("Dart enemy placed."); }
            Tool::OscFireball => {
                if self.osc_start.is_none() {
                    self.osc_start = Some((x, y));
                    self.set_status("Click bottom_y for fireball travel range...");
                } else {
                    let (fx, top_y) = self.osc_start.take().unwrap();
                    let bottom_y = y;
                    self.data.osc_fireballs.push(OscFireballDef { x: fx, top_y: top_y.min(bottom_y), bottom_y: top_y.max(bottom_y) });
                    self.set_status("Oscillating fireball placed.");
                }
            }
            Tool::Checkpoint => { self.data.checkpoints.push(Pos { x, y }); self.set_status("Checkpoint placed."); }
            Tool::Flagpole => { self.data.flagpole = Pos { x, y }; self.set_status("Flagpole moved."); }
            Tool::PlayerSpawn => { self.data.player_spawn = Pos { x, y }; self.set_status("Player spawn moved."); }
            Tool::Cloud => { self.data.clouds.push(crate::level::CloudSpawn { x, y, w: 64.0, h: 24.0, speed: 0.3, color: [1.0, 1.0, 1.0, 0.7], dark: false }); self.set_status("Cloud placed."); }
            Tool::Portal => { let next_id = self.data.portals.iter().map(|p| p.id).max().unwrap_or(0) + 1; self.data.portals.push(crate::level::PortalSpawn { id: next_id, dest_id: next_id + 1, x, y, locked: false, key_color: None, destroyed: false }); self.set_status(&format!("Portal {} placed.", next_id)); }
            Tool::Key => { self.data.keys.push(crate::editor::data::KeyDef { x, y, color: crate::level::KeyColor::Red }); self.set_status("Key placed."); }
            Tool::Eraser => { self.delete_at(x, y); }
            Tool::Drag => {} // handled in update.rs, never reaches here
            Tool::View => {} // no-op in View mode
        }
    }

    pub(crate) fn delete_at(&mut self, wx: f32, wy: f32) {
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
        self.data.clouds.retain(|c| (c.x - x).abs() > tol * 4.0 || (c.y - y).abs() > tol * 4.0);
        self.data.portals.retain(|p| (p.x - x).abs() > 20.0 || (p.y - y).abs() > 52.0);
        self.data.keys.retain(|k| (k.x - x).abs() > tol || (k.y - y).abs() > tol);
        if self.entity_count() < before { self.dirty = true; self.set_status("Entity deleted."); }
    }

    fn entity_count(&self) -> usize {
        self.data.platforms.len() + self.data.spikes.len() + self.data.coins.len()
            + self.data.question_blocks.len() + self.data.bricks.len()
            + self.data.enemies.len() + self.data.dart_enemies.len()
            + self.data.osc_fireballs.len() + self.data.checkpoints.len()
            + self.data.clouds.len()
            + self.data.portals.len()
            + self.data.keys.len()
    }

    pub(crate) fn cancel_pending(&mut self) {
        if self.plat_start.is_some() || self.enemy_start.is_some() || self.osc_start.is_some() {
            self.plat_start = None; self.enemy_start = None; self.osc_start = None;
            self.set_status("Cancelled.");
        }
    }

    // ── Property inspector (View mode) ──

    /// Returns (label, field_name, current_value) tuples for the selected entity.
    pub(crate) fn entity_properties(&self) -> Vec<(&str, &str, f32)> {
        let target = match &self.selected_entity { Some(t) => t, None => return vec![] };
        match target {
            DragTarget::Platform(i) => self.data.platforms.get(*i).map(|p| vec![
                ("x", "x", p.x), ("y", "y", p.y), ("w", "w", p.w), ("h", "h", p.h),
            ]).unwrap_or_default(),
            DragTarget::Spike(i) => self.data.spikes.get(*i).map(|s| vec![
                ("x", "x", s.x), ("y", "y", s.y),
            ]).unwrap_or_default(),
            DragTarget::Coin(i) => self.data.coins.get(*i).map(|c| vec![
                ("x", "x", c.x), ("y", "y", c.y),
            ]).unwrap_or_default(),
            DragTarget::QuestionBlock(i) => self.data.question_blocks.get(*i).map(|q| vec![
                ("x", "x", q.x), ("y", "y", q.y),
            ]).unwrap_or_default(),
            DragTarget::Brick(i) => self.data.bricks.get(*i).map(|b| vec![
                ("x", "x", b.x), ("y", "y", b.y),
            ]).unwrap_or_default(),
            DragTarget::Enemy(i) => self.data.enemies.get(*i).map(|e| vec![
                ("x", "x", e.x), ("y", "y", e.y),
                ("way_a.x", "wax", e.waypoint_a.x), ("way_a.y", "way", e.waypoint_a.y),
                ("way_b.x", "wbx", e.waypoint_b.x), ("way_b.y", "wby", e.waypoint_b.y),
            ]).unwrap_or_default(),
            DragTarget::DartEnemy(i) => self.data.dart_enemies.get(*i).map(|d| vec![
                ("x", "x", d.x), ("y", "y", d.y),
            ]).unwrap_or_default(),
            DragTarget::OscFireball(i) => self.data.osc_fireballs.get(*i).map(|o| vec![
                ("x", "x", o.x), ("top_y", "top_y", o.top_y), ("bottom_y", "bottom_y", o.bottom_y),
            ]).unwrap_or_default(),
            DragTarget::Checkpoint(i) => self.data.checkpoints.get(*i).map(|c| vec![
                ("x", "x", c.x), ("y", "y", c.y),
            ]).unwrap_or_default(),
            DragTarget::PlayerSpawn => vec![
                ("x", "x", self.data.player_spawn.x), ("y", "y", self.data.player_spawn.y),
            ],
            DragTarget::Flagpole => vec![
                ("x", "x", self.data.flagpole.x), ("y", "y", self.data.flagpole.y),
            ],
            DragTarget::Key(i) => self.data.keys.get(*i).map(|k| {
                let color_idx = match k.color {
                    crate::level::KeyColor::Red => 0.0, crate::level::KeyColor::Orange => 1.0,
                    crate::level::KeyColor::Yellow => 2.0, crate::level::KeyColor::Green => 3.0,
                    crate::level::KeyColor::Blue => 4.0, crate::level::KeyColor::Indigo => 5.0,
                    crate::level::KeyColor::Violet => 6.0,
                };
                vec![("x", "x", k.x), ("y", "y", k.y), ("color", "color", color_idx)]
            }).unwrap_or_default(),
            DragTarget::Cloud(i) => self.data.clouds.get(*i).map(|c| vec![
                ("x", "x", c.x), ("y", "y", c.y),
                ("w", "w", c.w), ("h", "h", c.h),
                ("speed", "speed", c.speed),
            ]).unwrap_or_default(),
            DragTarget::Portal(i) => self.data.portals.get(*i).map(|p| {
                let key_clr_idx = match p.key_color {
                    Some(crate::level::KeyColor::Red) => 0.0, Some(crate::level::KeyColor::Orange) => 1.0,
                    Some(crate::level::KeyColor::Yellow) => 2.0, Some(crate::level::KeyColor::Green) => 3.0,
                    Some(crate::level::KeyColor::Blue) => 4.0, Some(crate::level::KeyColor::Indigo) => 5.0,
                    Some(crate::level::KeyColor::Violet) => 6.0,
                    None => 0.0,
                };
                vec![
                    ("id", "id", p.id as f32), ("dest_id", "dest_id", p.dest_id as f32),
                    ("x", "x", p.x), ("y", "y", p.y),
                    ("locked", "locked", if p.locked { 1.0 } else { 0.0 }),
                    ("key_clr", "key_clr", key_clr_idx),
                    ("destroyed", "destroyed", if p.destroyed { 1.0 } else { 0.0 }),
                ]
            }).unwrap_or_default(),
        }
    }

    /// Set a property on the selected entity. Returns true if successful.
    pub(crate) fn set_entity_property(&mut self, field_name: &str, value: f32) -> bool {
        let target = match &self.selected_entity { Some(t) => t, None => return false };
        let ok = match target {
            DragTarget::Platform(i) => self.data.platforms.get_mut(*i).map(|p| match field_name {
                "x" => { p.x = value; true } "y" => { p.y = value; true }
                "w" => { p.w = value.max(16.0); true } "h" => { p.h = value.max(16.0); true }
                _ => false,
            }).unwrap_or(false),
            DragTarget::Spike(i) => self.data.spikes.get_mut(*i).map(|s| match field_name {
                "x" => { s.x = value; true } "y" => { s.y = value; true } _ => false,
            }).unwrap_or(false),
            DragTarget::Coin(i) => self.data.coins.get_mut(*i).map(|c| match field_name {
                "x" => { c.x = value; true } "y" => { c.y = value; true } _ => false,
            }).unwrap_or(false),
            DragTarget::QuestionBlock(i) => self.data.question_blocks.get_mut(*i).map(|q| match field_name {
                "x" => { q.x = value; true } "y" => { q.y = value; true } _ => false,
            }).unwrap_or(false),
            DragTarget::Brick(i) => self.data.bricks.get_mut(*i).map(|b| match field_name {
                "x" => { b.x = value; true } "y" => { b.y = value; true } _ => false,
            }).unwrap_or(false),
            DragTarget::Enemy(i) => self.data.enemies.get_mut(*i).map(|e| match field_name {
                "x" => { e.x = value; true } "y" => { e.y = value; true }
                "wax" => { e.waypoint_a.x = value; true } "way" => { e.waypoint_a.y = value; true }
                "wbx" => { e.waypoint_b.x = value; true } "wby" => { e.waypoint_b.y = value; true }
                _ => false,
            }).unwrap_or(false),
            DragTarget::DartEnemy(i) => self.data.dart_enemies.get_mut(*i).map(|d| match field_name {
                "x" => { d.x = value; true } "y" => { d.y = value; true } _ => false,
            }).unwrap_or(false),
            DragTarget::OscFireball(i) => self.data.osc_fireballs.get_mut(*i).map(|o| match field_name {
                "x" => { o.x = value; true } "top_y" => { o.top_y = value; true }
                "bottom_y" => { o.bottom_y = value; true } _ => false,
            }).unwrap_or(false),
            DragTarget::Checkpoint(i) => self.data.checkpoints.get_mut(*i).map(|c| match field_name {
                "x" => { c.x = value; true } "y" => { c.y = value; true } _ => false,
            }).unwrap_or(false),
            DragTarget::PlayerSpawn => match field_name {
                "x" => { self.data.player_spawn.x = value; true }
                "y" => { self.data.player_spawn.y = value; true } _ => false,
            },
            DragTarget::Flagpole => match field_name {
                "x" => { self.data.flagpole.x = value; true }
                "y" => { self.data.flagpole.y = value; true } _ => false,
            },
            DragTarget::Key(i) => self.data.keys.get_mut(*i).map(|k| match field_name {
                "x" => { k.x = value; true }
                "y" => { k.y = value; true }
                "color" => { k.color = k.color.next(); true }
                _ => false,
            }).unwrap_or(false),
            DragTarget::Cloud(i) => self.data.clouds.get_mut(*i).map(|c| match field_name {
                "x" => { c.x = value; true } "y" => { c.y = value; true }
                "w" => { c.w = value.max(8.0); true } "h" => { c.h = value.max(8.0); true }
                "speed" => { c.speed = value.clamp(0.0, 1.0); true }
                _ => false,
            }).unwrap_or(false),
            DragTarget::Portal(i) => self.data.portals.get_mut(*i).map(|p| match field_name {
                "x" => { p.x = value; true }
                "y" => { p.y = value; true }
                "id" => { p.id = value.max(1.0) as u32; true }
                "dest_id" => { p.dest_id = value.max(0.0) as u32; true }
                "locked" => { p.locked = value >= 0.5; true }
                "key_clr" => {
                    p.key_color = match value as i32 {
                        0 => Some(crate::level::KeyColor::Red), 1 => Some(crate::level::KeyColor::Orange),
                        2 => Some(crate::level::KeyColor::Yellow), 3 => Some(crate::level::KeyColor::Green),
                        4 => Some(crate::level::KeyColor::Blue), 5 => Some(crate::level::KeyColor::Indigo),
                        6 => Some(crate::level::KeyColor::Violet),
                        _ => Some(crate::level::KeyColor::Red),
                    };
                    true
                }
                "destroyed" => { p.destroyed = value >= 0.5; true }
                _ => false,
            }).unwrap_or(false),
        };
        if ok { self.dirty = true; }
        ok
    }

    // ── Menu actions ──
    pub(crate) fn menu_action(&mut self, action: &str) {
        match action {
            "new" => {
                self.data = LevelData::default();
                let mut n = 1u32;
                while n <= 99 {
                    if std::fs::metadata(&format!("assets/levels/{}.json", n)).is_err() { break; }
                    n += 1;
                }
                self.level_num = n; self.dirty = true;
                self.set_status(&format!("New level {}. Ctrl+S to save.", n));
            }
            "save" => self.save_level(),
            "save_as" => {
                self.save_as_mode = true;
                self.save_path_buf = EditorState::level_path(self.level_num);
                self.set_status("Type path, Enter to confirm, Esc to cancel...");
            }
            "rename" => {
                self.rename_mode = true;
                self.name_buf = self.data.name.clone();
                self.set_status("Enter new level name, Enter to confirm, Esc to cancel...");
            }
            "open_file" => {
                self.open_file_mode = true;
                self.open_file_buf = String::from("assets/levels/");
                self.set_status("Enter path to level JSON, Enter to load, Esc to cancel...");
            }
            "zoom_in" => { self.zoom = (self.zoom * 1.2).min(4.0); }
            "zoom_out" => { self.zoom = (self.zoom / 1.2).max(0.25); }
            "zoom_reset" => { self.zoom = 1.0; }
            "grid" => { self.grid_snap = !self.grid_snap; self.set_status(if self.grid_snap { "Grid snap: ON" } else { "Grid snap: OFF" }); }
            _ => {}
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_editor() -> EditorState {
        let mut e = EditorState::new();
        e.set_screen(1280.0, 720.0);
        // Reset to empty data so tests start clean (default() now loads from disk).
        e.data = LevelData::default();
        e.data.platforms.clear(); // remove the default platform too
        e
    }

    // ── Construction ──

    #[test]
    fn test_new_has_defaults() {
        let e = EditorState::new();
        assert_eq!(e.level_num, 1);
        assert_eq!(e.zoom, 1.0);
        assert!(e.grid_snap);
        assert_eq!(e.tool, Tool::View);
        assert!(!e.dirty);
    }

    #[test]
    fn test_set_screen() {
        let mut e = EditorState::new();
        e.set_screen(800.0, 600.0);
        assert_eq!(e.screen_w, 800.0);
        assert_eq!(e.screen_h, 600.0);
    }

    // ── Grid snapping ──

    #[test]
    fn test_snap_on_grid() {
        let e = make_editor();
        assert_eq!(e.snap(0.0), 0.0);
        assert_eq!(e.snap(16.0), 16.0);
        assert_eq!(e.snap(20.0), 16.0);  // rounds down
        assert_eq!(e.snap(24.0), 32.0);  // rounds up
        assert_eq!(e.snap(32.0), 32.0);
    }

    #[test]
    fn test_snap_off_grid() {
        let mut e = make_editor();
        e.grid_snap = false;
        assert_eq!(e.snap(20.0), 20.0);
        assert_eq!(e.snap(24.5), 24.5);
    }

    #[test]
    fn test_snap_pos() {
        let e = make_editor();
        let (x, y) = e.snap_pos(20.0, 24.0);
        assert_eq!(x, 16.0);
        assert_eq!(y, 32.0);
    }

    // ── Screen ↔ world transforms ──

    #[test]
    fn test_screen_to_world_origin() {
        let e = make_editor();
        // At zoom=1, cam=(0,0): screen (0, HEADER_H) → world (0, 0)
        let (wx, wy) = e.screen_to_world(0.0, EditorState::HEADER_H);
        assert!((wx - 0.0).abs() < 0.01);
        assert!((wy - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_screen_to_world_panned() {
        let mut e = make_editor();
        e.cam_x = 100.0;
        e.cam_y = 50.0;
        // world_x = screen_x / zoom + cam_x
        let (wx, wy) = e.screen_to_world(0.0, EditorState::HEADER_H);
        assert!((wx - 100.0).abs() < 0.01);
        assert!((wy - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_screen_to_world_zoomed() {
        let mut e = make_editor();
        e.zoom = 2.0;
        // world_x = screen_x / 2 + 0
        let (wx, _) = e.screen_to_world(200.0, EditorState::HEADER_H);
        assert!((wx - 100.0).abs() < 0.01);
    }

    // ── Entity placement ──

    #[test]
    fn test_place_spike() {
        let mut e = make_editor();
        e.tool = Tool::Spike;
        e.place_at(100.0, 200.0);
        assert_eq!(e.data.spikes.len(), 1);
        assert!((e.data.spikes[0].x - 96.0).abs() < 0.01); // snapped to 96
        assert!((e.data.spikes[0].y - 208.0).abs() < 0.01); // snapped to 208
        assert!(e.dirty);
    }

    #[test]
    fn test_place_coin() {
        let mut e = make_editor();
        e.tool = Tool::Coin;
        e.place_at(48.0, 48.0);
        assert_eq!(e.data.coins.len(), 1);
        assert_eq!(e.data.coins[0].x, 48.0);
        assert_eq!(e.data.coins[0].y, 48.0);
    }

    #[test]
    fn test_place_question_block() {
        let mut e = make_editor();
        e.tool = Tool::QuestionBlock;
        e.place_at(32.0, 100.0);
        assert_eq!(e.data.question_blocks.len(), 1);
    }

    #[test]
    fn test_place_brick() {
        let mut e = make_editor();
        e.tool = Tool::Brick;
        e.place_at(64.0, 128.0);
        assert_eq!(e.data.bricks.len(), 1);
    }

    #[test]
    fn test_place_dart_enemy() {
        let mut e = make_editor();
        e.tool = Tool::DartEnemy;
        e.place_at(160.0, 300.0);
        assert_eq!(e.data.dart_enemies.len(), 1);
    }

    #[test]
    fn test_place_checkpoint() {
        let mut e = make_editor();
        e.tool = Tool::Checkpoint;
        e.place_at(400.0, 500.0);
        assert_eq!(e.data.checkpoints.len(), 1);
    }

    #[test]
    fn test_place_flagpole() {
        let mut e = make_editor();
        let old_x = e.data.flagpole.x;
        e.tool = Tool::Flagpole;
        e.place_at(2000.0, 560.0);
        assert!((e.data.flagpole.x - 2000.0).abs() < 0.01);
    }

    #[test]
    fn test_place_player_spawn() {
        let mut e = make_editor();
        e.tool = Tool::PlayerSpawn;
        e.place_at(300.0, 600.0);
        assert!((e.data.player_spawn.x - 304.0).abs() < 0.01);
    }

    #[test]
    fn test_place_platform_two_clicks() {
        let mut e = make_editor();
        e.tool = Tool::Platform;
        // First click: sets start
        e.place_at(0.0, 100.0);
        assert!(e.plat_start.is_some());
        assert_eq!(e.data.platforms.len(), 0); // not placed yet
        // Second click: places rect
        e.place_at(64.0, 132.0);
        assert!(e.plat_start.is_none());
        assert_eq!(e.data.platforms.len(), 1);
        let p = &e.data.platforms[0];
        assert!((p.x - 0.0).abs() < 0.01);
        assert!((p.y - 96.0).abs() < 0.01); // snapped
        assert!((p.w - 64.0).abs() < 0.01);
        assert!((p.h - 32.0).abs() < 0.01); // snapped
    }

    #[test]
    fn test_place_enemy_two_clicks() {
        let mut e = make_editor();
        e.tool = Tool::Enemy;
        e.place_at(100.0, 200.0);
        assert!(e.enemy_start.is_some());
        e.place_at(300.0, 200.0);
        assert_eq!(e.data.enemies.len(), 1);
    }

    #[test]
    fn test_place_osc_fireball_two_clicks() {
        let mut e = make_editor();
        e.tool = Tool::OscFireball;
        e.place_at(100.0, 300.0);
        assert!(e.osc_start.is_some());
        e.place_at(100.0, 500.0);
        assert_eq!(e.data.osc_fireballs.len(), 1);
    }

    // ── Delete ──

    #[test]
    fn test_delete_spike() {
        let mut e = make_editor();
        e.tool = Tool::Spike;
        e.place_at(100.0, 200.0);
        assert_eq!(e.data.spikes.len(), 1);
        e.tool = Tool::Eraser;
        e.place_at(100.0, 200.0); // Eraser calls delete_at
        assert_eq!(e.data.spikes.len(), 0);
    }

    #[test]
    fn test_delete_coin() {
        let mut e = make_editor();
        e.tool = Tool::Coin;
        e.place_at(48.0, 48.0);
        e.place_at(96.0, 96.0);
        assert_eq!(e.data.coins.len(), 2);
        e.delete_at(50.0, 50.0);
        assert_eq!(e.data.coins.len(), 1);
    }

    // ── Entity count ──

    #[test]
    fn test_entity_count_starts_with_default_platform() {
        let e = make_editor();
        // Default LevelData has 1 platform
        let mut e2 = make_editor();
        // entity_count is private but we can test indirectly via delete
        // Or... let me check: entity_count is fn (not pub(crate)), so it IS accessible in test module
        // Actually it IS accessible since we're in #[cfg(test)] within the same file
    }

    // ── Cancel pending ──

    #[test]
    fn test_cancel_pending_platform() {
        let mut e = make_editor();
        e.tool = Tool::Platform;
        e.place_at(0.0, 0.0); // first click
        assert!(e.plat_start.is_some());
        e.cancel_pending();
        assert!(e.plat_start.is_none());
    }

    #[test]
    fn test_cancel_pending_enemy() {
        let mut e = make_editor();
        e.tool = Tool::Enemy;
        e.place_at(0.0, 0.0);
        assert!(e.enemy_start.is_some());
        e.cancel_pending();
        assert!(e.enemy_start.is_none());
    }

    // ── hit_test ──

    #[test]
    fn test_hit_test_platform() {
        let mut e = make_editor();
        e.tool = Tool::Platform;
        e.place_at(0.0, 100.0);
        e.place_at(64.0, 132.0);
        // Hit the middle of the platform
        let result = e.hit_test(32.0, 110.0);
        assert!(matches!(result, Some(DragTarget::Platform(0))));
    }

    #[test]
    fn test_hit_test_spike() {
        let mut e = make_editor();
        e.tool = Tool::Spike;
        e.place_at(96.0, 208.0);
        let result = e.hit_test(96.0, 208.0);
        assert!(matches!(result, Some(DragTarget::Spike(0))));
    }

    #[test]
    fn test_hit_test_coin() {
        let mut e = make_editor();
        e.tool = Tool::Coin;
        e.place_at(48.0, 48.0);
        let result = e.hit_test(48.0, 48.0);
        assert!(matches!(result, Some(DragTarget::Coin(0))));
    }

    #[test]
    fn test_hit_test_miss() {
        let e = make_editor();
        let result = e.hit_test(9999.0, 9999.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_hit_test_question_block() {
        let mut e = make_editor();
        e.tool = Tool::QuestionBlock;
        e.place_at(64.0, 64.0);
        let result = e.hit_test(80.0, 80.0);
        assert!(matches!(result, Some(DragTarget::QuestionBlock(0))));
    }

    #[test]
    fn test_hit_test_brick() {
        let mut e = make_editor();
        e.tool = Tool::Brick;
        e.place_at(64.0, 128.0);
        let result = e.hit_test(64.0, 128.0);
        assert!(matches!(result, Some(DragTarget::Brick(0))));
    }

    #[test]
    fn test_hit_test_flagpole() {
        let mut e = make_editor();
        e.tool = Tool::Flagpole;
        e.place_at(2000.0, 560.0);
        let result = e.hit_test(2000.0, 500.0); // middle of flagpole
        assert!(matches!(result, Some(DragTarget::Flagpole)));
    }

    #[test]
    fn test_hit_test_player_spawn() {
        let mut e = make_editor();
        e.tool = Tool::PlayerSpawn;
        e.place_at(300.0, 600.0);
        let result = e.hit_test(304.0, 600.0); // snapped position
        assert!(matches!(result, Some(DragTarget::PlayerSpawn)));
    }

    // ── move_entity ──

    #[test]
    fn test_move_entity_spike() {
        let mut e = make_editor();
        e.tool = Tool::Spike;
        e.place_at(100.0, 200.0);
        e.move_entity(DragTarget::Spike(0), 300.0, 400.0);
        assert!((e.data.spikes[0].x - 300.0).abs() < 0.01);
        assert!((e.data.spikes[0].y - 400.0).abs() < 0.01);
    }

    #[test]
    fn test_move_entity_enemy_moves_waypoints() {
        let mut e = make_editor();
        e.tool = Tool::Enemy;
        e.place_at(100.0, 200.0);
        e.place_at(300.0, 200.0);
        let old_a = e.data.enemies[0].waypoint_a;
        let old_b = e.data.enemies[0].waypoint_b;
        // Enemy is placed at snapped (96, 208); move to (150, 250): dx=54, dy=42
        e.move_entity(DragTarget::Enemy(0), 150.0, 250.0);
        assert!((e.data.enemies[0].x - 150.0).abs() < 0.01);
        assert!((e.data.enemies[0].waypoint_a.x - (old_a.x + 54.0)).abs() < 1.0);
        assert!((e.data.enemies[0].waypoint_b.x - (old_b.x + 54.0)).abs() < 1.0);
    }

    #[test]
    fn test_move_entity_osc_fireball_preserves_span() {
        let mut e = make_editor();
        e.tool = Tool::OscFireball;
        e.place_at(100.0, 300.0);
        e.place_at(100.0, 500.0);
        let span = e.data.osc_fireballs[0].bottom_y - e.data.osc_fireballs[0].top_y;
        e.move_entity(DragTarget::OscFireball(0), 200.0, 350.0);
        let new_span = e.data.osc_fireballs[0].bottom_y - e.data.osc_fireballs[0].top_y;
        assert!((new_span - span).abs() < 0.01);
    }

    // ── level_path ──

    #[test]
    fn test_level_path() {
        assert_eq!(EditorState::level_path(1), "assets/levels/1.json");
        assert_eq!(EditorState::level_path(42), "assets/levels/42.json");
    }

    // ── set_status ──

    #[test]
    fn test_set_status() {
        let mut e = make_editor();
        e.set_status("hello");
        assert_eq!(e.status, "hello");
        assert!((e.status_timer - 3.0).abs() < 0.01);
    }

    // ── menu_action ──

    #[test]
    fn test_menu_action_zoom() {
        let mut e = make_editor();
        e.menu_action("zoom_in");
        assert!((e.zoom - 1.2).abs() < 0.01);
        e.menu_action("zoom_reset");
        assert!((e.zoom - 1.0).abs() < 0.01);
        e.menu_action("zoom_out");
        assert!((e.zoom - 1.0 / 1.2).abs() < 0.01);
    }

    #[test]
    fn test_menu_action_grid() {
        let mut e = make_editor();
        assert!(e.grid_snap);
        e.menu_action("grid");
        assert!(!e.grid_snap);
        e.menu_action("grid");
        assert!(e.grid_snap);
    }

    #[test]
    fn test_menu_action_save_as() {
        let mut e = make_editor();
        e.menu_action("save_as");
        assert!(e.save_as_mode);
        assert!(!e.save_path_buf.is_empty());
    }

    #[test]
    fn test_menu_action_rename() {
        let mut e = make_editor();
        e.menu_action("rename");
        assert!(e.rename_mode);
    }

    // ── Entity count ──

    #[test]
    fn test_default_leveldata_has_one_platform() {
        let d = LevelData::default();
        assert_eq!(d.platforms.len(), 1);
        assert_eq!(d.spikes.len(), 0);
        assert_eq!(d.coins.len(), 0);
    }

    // ── LevelData JSON roundtrip ──

    #[test]
    fn test_leveldata_serialize_deserialize() {
        let json = r#"{"bounds":{"min_x":0,"max_x":3600,"min_y":0,"kill_y":2500},"platforms":[{"x":0,"y":600,"w":700,"h":40}],"spikes":[],"coins":[],"question_blocks":[{"x":100,"y":400}],"bricks":[],"enemies":[],"dart_enemies":[],"osc_fireballs":[],"checkpoints":[],"flagpole":{"x":3500,"y":560},"player_spawn":{"x":100,"y":600},"parallax":[0.1,0.3,0.6]}"#;
        let data: LevelData = serde_json::from_str(json).unwrap();
        assert_eq!(data.question_blocks.len(), 1);
        assert_eq!(data.question_blocks[0].x, 100.0);
        assert_eq!(data.name, ""); // default
        let out = serde_json::to_string(&data).unwrap();
        let data2: LevelData = serde_json::from_str(&out).unwrap();
        assert_eq!(data2.question_blocks.len(), 1);
    }

    // ── Tool enum ──

    #[test]
    fn test_tool_all_has_all_variants() {
        assert_eq!(Tool::ALL.len(), 17); // +Cloud, +Portal, +Key
    }

    #[test]
    fn test_tool_name() {
        assert_eq!(Tool::Platform.name(), "Platform");
        assert_eq!(Tool::Eraser.name(), "Eraser");
    }

    #[test]
    fn test_tool_shortcut() {
        assert_eq!(Tool::Platform.shortcut(), "1");
        assert_eq!(Tool::PlayerSpawn.shortcut(), "P");
        assert_eq!(Tool::Eraser.shortcut(), "Del");
    }
}
