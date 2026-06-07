// Feature #2: Level & Background — Level geometry, bounds, and entity spawn data.
// All level data is read from assets/level.json at compile time via include_str!().

use crate::entities::hazard::Spike;
use crate::parallax::ParallaxLayer;
use serde::{Deserialize, Serialize};

// ============================================================================
// Geometry Primitives
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub struct Vec2 { pub x: f32, pub y: f32 }

#[derive(Debug, Clone, Copy)]
pub struct AABB { pub x: f32, pub y: f32, pub w: f32, pub h: f32 }

impl AABB {
    pub fn intersects(&self, other: &AABB) -> bool {
        self.x <= other.x + other.w
            && self.x + self.w >= other.x
            && self.y <= other.y + other.h
            && self.y + self.h >= other.y
    }
}

// ============================================================================
// Tile & Platform
// ============================================================================

#[derive(Debug, Clone)]
pub enum Tile {
    Empty,
    Platform(AABB),
    Spike(AABB),
}

/// Key colors for locked portals. Serialized as lowercase strings in JSON.
/// 7 rainbow colors: Red → Orange → Yellow → Green → Blue → Indigo → Violet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum KeyColor {
    #[default]
    #[serde(rename = "red")]
    Red,
    #[serde(rename = "orange")]
    Orange,
    #[serde(rename = "yellow")]
    Yellow,
    #[serde(rename = "green")]
    Green,
    #[serde(rename = "blue")]
    Blue,
    #[serde(rename = "indigo")]
    Indigo,
    #[serde(rename = "violet")]
    Violet,
}

impl KeyColor {
    /// Cycle to the next color. Returns the next color in rainbow order.
    pub fn next(self) -> Self {
        use KeyColor::*;
        match self {
            Red    => Orange,
            Orange => Yellow,
            Yellow => Green,
            Green  => Blue,
            Blue   => Indigo,
            Indigo => Violet,
            Violet => Red,
        }
    }

    /// Human-readable name in Chinese.
    pub fn chinese_name(self) -> &'static str {
        match self {
            KeyColor::Red    => "红色",
            KeyColor::Orange => "橙色",
            KeyColor::Yellow => "黄色",
            KeyColor::Green  => "绿色",
            KeyColor::Blue   => "蓝色",
            KeyColor::Indigo => "靛色",
            KeyColor::Violet => "紫色",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            KeyColor::Red    => "Red",
            KeyColor::Orange => "Orange",
            KeyColor::Yellow => "Yellow",
            KeyColor::Green  => "Green",
            KeyColor::Blue   => "Blue",
            KeyColor::Indigo => "Indigo",
            KeyColor::Violet => "Violet",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Platform { pub aabb: AABB }

// ============================================================================
// Level Bounds
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub struct LevelBounds { pub min_x: f32, pub max_x: f32, pub min_y: f32, pub kill_y: f32 }

// ============================================================================
// JSON data shapes (serde)
// ============================================================================

/// Background theme colors stored per-level in JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub bg:       [f32; 4],  // sky clear color
    pub ground:   [f32; 4],  // ground-level platforms
    pub platform: [f32; 4],  // elevated platforms
    pub spike:    [f32; 4],  // spike tint
}

impl Default for ThemeColors {
    fn default() -> Self {
        ThemeColors {
            bg:       [0.35, 0.65, 0.95, 1.0],
            ground:   [0.40, 0.75, 0.30, 1.0],
            platform: [0.55, 0.35, 0.15, 1.0],
            spike:    [0.90, 0.20, 0.10, 1.0],
        }
    }
}

/// A decorative background cloud with parallax scrolling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSpawn {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub speed: f32,        // parallax factor (0.0 = static, 1.0 = match camera)
    #[serde(default = "default_cloud_color")]
    pub color: [f32; 4],   // RGBA
    /// If true, render as a dark storm-cloud instead of white fluffy.
    #[serde(default)]
    pub dark: bool,
}

fn default_cloud_color() -> [f32; 4] { [1.0, 1.0, 1.0, 0.7] }

#[derive(Deserialize)]
struct JsonLevel {
    #[serde(default)]
    name: String,
    bounds: JsonBounds,
    #[serde(default)]
    platforms: Vec<JsonRect>,
    #[serde(default)]
    spikes: Vec<JsonPos>,
    #[serde(default)]
    coins: Vec<JsonPos>,
    #[serde(default)]
    question_blocks: Vec<JsonPos>,
    #[serde(default)]
    bricks: Vec<JsonPos>,
    #[serde(default)]
    enemies: Vec<JsonEnemy>,
    #[serde(default)]
    dart_enemies: Vec<JsonDartEnemy>,
    #[serde(default)]
    osc_fireballs: Vec<JsonOscFireball>,
    #[serde(default)]
    checkpoints: Vec<JsonPos>,
    flagpole: JsonPos,
    #[serde(default = "default_player_spawn")]
    player_spawn: JsonPos,
    #[serde(default = "default_parallax")]
    parallax: Vec<f32>,
    #[serde(default)]
    theme: ThemeColors,
    #[serde(default)]
    clouds: Vec<CloudSpawn>,
    #[serde(default)]
    portals: Vec<PortalSpawn>,
    #[serde(default)]
    keys: Vec<KeySpawn>,
}

/// A warp portal — closed by default, opens on UP input, warps to dest_id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalSpawn {
    pub id: u32,
    pub dest_id: u32,
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub key_color: Option<KeyColor>,
    #[serde(default)]
    pub destroyed: bool,
}

/// A collectible key that unlocks matching-color locked portals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeySpawn {
    pub x: f32,
    pub y: f32,
    pub color: KeyColor,
}

#[derive(Deserialize)]
struct JsonBounds { min_x: f32, max_x: f32, min_y: f32, kill_y: f32 }

#[derive(Deserialize)]
struct JsonRect { x: f32, y: f32, w: f32, h: f32 }

#[derive(Deserialize)]
struct JsonPos { x: f32, y: f32 }

#[derive(Deserialize)]
struct JsonEnemy { x: f32, y: f32, waypoint_a: JsonPos, waypoint_b: JsonPos }

#[derive(Deserialize)]
struct JsonDartEnemy { x: f32, y: f32 }

#[derive(Deserialize)]
struct JsonOscFireball { x: f32, top_y: f32, bottom_y: f32 }

fn default_parallax() -> Vec<f32> { vec![0.1, 0.3, 0.6] }

fn default_player_spawn() -> JsonPos {
    JsonPos { x: 100.0, y: 600.0 }
}

// ============================================================================
// Public entity spawn data (read by PlayingState)
// ============================================================================

#[derive(Debug, Clone)]
pub struct CoinSpawn { pub pos: Vec2 }

#[derive(Debug, Clone)]
pub struct QuestionBlockSpawn { pub pos: Vec2 }

#[derive(Debug, Clone)]
pub struct BrickSpawn { pub pos: Vec2 }

#[derive(Debug, Clone)]
pub struct EnemySpawn { pub pos: Vec2, pub waypoint_a: Vec2, pub waypoint_b: Vec2 }

#[derive(Debug, Clone, Copy)]
pub struct DartEnemySpawn { pub x: f32, pub y: f32 }

#[derive(Debug, Clone, Copy)]
pub struct OscFireballSpawn { pub x: f32, pub top_y: f32, pub bottom_y: f32 }

#[derive(Debug, Clone)]
pub struct CheckpointSpawn { pub pos: Vec2 }

#[derive(Debug, Clone)]
pub struct FlagpoleSpawn { pub pos: Vec2 }

#[derive(Debug, Clone, Copy)]
pub struct PlayerSpawn { pub pos: Vec2 }

// ============================================================================
// Level
// ============================================================================

/// Complete static level data loaded from assets/level.json at compile time.
pub struct Level {
    pub name: String,
    platforms: Vec<Platform>,
    spikes: Vec<Spike>,
    bounds: LevelBounds,
    parallax_layers: Vec<ParallaxLayer>,
    pub coin_spawns: Vec<CoinSpawn>,
    pub block_spawns: Vec<QuestionBlockSpawn>,
    pub brick_spawns: Vec<BrickSpawn>,
    pub enemy_spawns: Vec<EnemySpawn>,
    pub dart_enemy_spawns: Vec<DartEnemySpawn>,
    pub osc_fireball_spawns: Vec<OscFireballSpawn>,
    pub checkpoint_spawns: Vec<CheckpointSpawn>,
    pub flagpole_spawn: FlagpoleSpawn,
    pub player_spawn: PlayerSpawn,
    pub theme: ThemeColors,
    pub cloud_spawns: Vec<CloudSpawn>,
    pub portal_spawns: Vec<PortalSpawn>,
    pub key_spawns: Vec<KeySpawn>,
}

impl Default for Level {
    fn default() -> Self { Self::load(1) }
}

impl Level {
    /// Loads a level from the embedded assets/levels/{n}.json.
    /// Panics if the file is missing or invalid JSON.
    pub fn new() -> Self { Self::load(1) }

    /// Load level n (1-4). Falls back to level 1 for invalid numbers.
    /// Tries filesystem first (so editor modifications are live), then
    /// falls back to the embedded JSON (compile-time include_str!).
    pub fn load(n: u32) -> Self {
        let path = format!("assets/levels/{}.json", n);
        let json_str: String = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| {
                // Fallback: use compile-time embedded JSON
                let embedded: &str = match n {
                    1 => include_str!("../assets/levels/1.json"),
                    2 => include_str!("../assets/levels/2.json"),
                    3 => include_str!("../assets/levels/3.json"),
                    4 => include_str!("../assets/levels/4.json"),
                    _ => include_str!("../assets/levels/1.json"),
                };
                embedded.to_string()
            });
        let data: JsonLevel = serde_json::from_str(&json_str)
            .unwrap_or_else(|e| panic!("Failed to parse level {n}: {e}"));

        let platforms: Vec<Platform> = data.platforms.iter().map(|p| Platform {
            aabb: AABB { x: p.x, y: p.y, w: p.w, h: p.h },
        }).collect();

        let spikes: Vec<Spike> = data.spikes.iter().map(|s| {
            Spike::new(Vec2 { x: s.x, y: s.y })
        }).collect();

        let bounds = LevelBounds {
            min_x: data.bounds.min_x, max_x: data.bounds.max_x,
            min_y: data.bounds.min_y, kill_y: data.bounds.kill_y,
        };

        let parallax_layers: Vec<ParallaxLayer> = data.parallax.iter()
            .map(|&s| ParallaxLayer::new(s)).collect();

        let coin_spawns: Vec<CoinSpawn> = data.coins.iter()
            .map(|c| CoinSpawn { pos: Vec2 { x: c.x, y: c.y } }).collect();

        let block_spawns: Vec<QuestionBlockSpawn> = data.question_blocks.iter()
            .map(|b| QuestionBlockSpawn { pos: Vec2 { x: b.x, y: b.y } }).collect();

        let brick_spawns: Vec<BrickSpawn> = data.bricks.iter()
            .map(|b| BrickSpawn { pos: Vec2 { x: b.x, y: b.y } }).collect();

        let enemy_spawns: Vec<EnemySpawn> = data.enemies.iter().map(|e| EnemySpawn {
            pos: Vec2 { x: e.x, y: e.y },
            waypoint_a: Vec2 { x: e.waypoint_a.x, y: e.waypoint_a.y },
            waypoint_b: Vec2 { x: e.waypoint_b.x, y: e.waypoint_b.y },
        }).collect();

        let dart_enemy_spawns: Vec<DartEnemySpawn> = data.dart_enemies.iter()
            .map(|d| DartEnemySpawn { x: d.x, y: d.y }).collect();

        let osc_fireball_spawns: Vec<OscFireballSpawn> = data.osc_fireballs.iter()
            .map(|o| OscFireballSpawn { x: o.x, top_y: o.top_y, bottom_y: o.bottom_y }).collect();

        let checkpoint_spawns: Vec<CheckpointSpawn> = data.checkpoints.iter()
            .map(|c| CheckpointSpawn { pos: Vec2 { x: c.x, y: c.y } }).collect();

        let flagpole_spawn = FlagpoleSpawn {
            pos: Vec2 { x: data.flagpole.x, y: data.flagpole.y },
        };

        let player_spawn = PlayerSpawn {
            pos: Vec2 { x: data.player_spawn.x, y: data.player_spawn.y },
        };

        let key_spawns: Vec<KeySpawn> = data.keys;

        let name = if data.name.is_empty() {
            format!("Level {}", n)
        } else {
            data.name.clone()
        };

        Level { name, platforms, spikes, bounds, parallax_layers,
            coin_spawns, block_spawns, brick_spawns, enemy_spawns,
            dart_enemy_spawns, osc_fireball_spawns,
            checkpoint_spawns, flagpole_spawn, player_spawn,
            theme: data.theme,
            cloud_spawns: data.clouds,
            portal_spawns: data.portals,
            key_spawns, }
    }

    pub fn platforms(&self) -> &[Platform] { &self.platforms }

    pub fn spikes(&self) -> &[Spike] { &self.spikes }

    pub fn query_terrain(&self, aabb: &AABB) -> Vec<Tile> {
        let mut tiles: Vec<Tile> = self.platforms.iter()
            .filter(|p| p.aabb.intersects(aabb))
            .map(|p| Tile::Platform(p.aabb))
            .collect();
        for spike in &self.spikes {
            let sa = spike.collider();
            if sa.intersects(aabb) { tiles.push(Tile::Spike(sa)); }
        }
        tiles
    }

    pub fn bounds(&self) -> LevelBounds { self.bounds }

    pub fn parallax_layers(&self) -> &[ParallaxLayer] { &self.parallax_layers }
}

/// Scan `assets/levels/` for all `{n}.json` files, return sorted level numbers.
pub fn list_levels() -> Vec<u32> {
    let mut nums = Vec::new();
    let dir = std::path::Path::new("assets/levels");
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(rest) = name.strip_suffix(".json") {
                if let Ok(n) = rest.parse::<u32>() {
                    nums.push(n);
                }
            }
        }
    }
    nums.sort_unstable();
    nums
}
