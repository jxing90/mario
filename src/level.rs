// Feature #2: Level & Background — Level geometry, bounds, and entity spawn data.
// All level data is read from assets/level.json at compile time via include_str!().

use crate::entities::hazard::Spike;
use crate::parallax::ParallaxLayer;
use serde::Deserialize;

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

#[derive(Deserialize)]
struct JsonLevel {
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
    enemies: Vec<JsonEnemy>,
    #[serde(default)]
    checkpoints: Vec<JsonPos>,
    flagpole: JsonPos,
    #[serde(default = "default_parallax")]
    parallax: Vec<f32>,
}

#[derive(Deserialize)]
struct JsonBounds { min_x: f32, max_x: f32, min_y: f32, kill_y: f32 }

#[derive(Deserialize)]
struct JsonRect { x: f32, y: f32, w: f32, h: f32 }

#[derive(Deserialize)]
struct JsonPos { x: f32, y: f32 }

#[derive(Deserialize)]
struct JsonEnemy { x: f32, y: f32, waypoint_a: JsonPos, waypoint_b: JsonPos }

fn default_parallax() -> Vec<f32> { vec![0.1, 0.3, 0.6] }

// ============================================================================
// Public entity spawn data (read by PlayingState)
// ============================================================================

#[derive(Debug, Clone)]
pub struct CoinSpawn { pub pos: Vec2 }

#[derive(Debug, Clone)]
pub struct QuestionBlockSpawn { pub pos: Vec2 }

#[derive(Debug, Clone)]
pub struct EnemySpawn { pub pos: Vec2, pub waypoint_a: Vec2, pub waypoint_b: Vec2 }

#[derive(Debug, Clone)]
pub struct CheckpointSpawn { pub pos: Vec2 }

#[derive(Debug, Clone)]
pub struct FlagpoleSpawn { pub pos: Vec2 }

// ============================================================================
// Level
// ============================================================================

/// Complete static level data loaded from assets/level.json at compile time.
pub struct Level {
    platforms: Vec<Platform>,
    spikes: Vec<Spike>,
    bounds: LevelBounds,
    parallax_layers: Vec<ParallaxLayer>,
    pub coin_spawns: Vec<CoinSpawn>,
    pub block_spawns: Vec<QuestionBlockSpawn>,
    pub enemy_spawns: Vec<EnemySpawn>,
    pub checkpoint_spawns: Vec<CheckpointSpawn>,
    pub flagpole_spawn: FlagpoleSpawn,
}

impl Default for Level {
    fn default() -> Self { Self::new() }
}

impl Level {
    /// Loads the level from the embedded assets/level.json.
    pub fn new() -> Self {
        let json = include_str!("../assets/level.json");
        let data: JsonLevel = serde_json::from_str(json)
            .expect("Failed to parse assets/level.json");

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

        let enemy_spawns: Vec<EnemySpawn> = data.enemies.iter().map(|e| EnemySpawn {
            pos: Vec2 { x: e.x, y: e.y },
            waypoint_a: Vec2 { x: e.waypoint_a.x, y: e.waypoint_a.y },
            waypoint_b: Vec2 { x: e.waypoint_b.x, y: e.waypoint_b.y },
        }).collect();

        let checkpoint_spawns: Vec<CheckpointSpawn> = data.checkpoints.iter()
            .map(|c| CheckpointSpawn { pos: Vec2 { x: c.x, y: c.y } }).collect();

        let flagpole_spawn = FlagpoleSpawn {
            pos: Vec2 { x: data.flagpole.x, y: data.flagpole.y },
        };

        Level { platforms, spikes, bounds, parallax_layers,
            coin_spawns, block_spawns, enemy_spawns, checkpoint_spawns, flagpole_spawn }
    }

    pub fn platforms(&self) -> &[Platform] { &self.platforms }

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
