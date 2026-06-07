use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Bounds {
    pub(crate) min_x: f32,
    pub(crate) max_x: f32,
    pub(crate) min_y: f32,
    pub(crate) kill_y: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct Rect {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) w: f32,
    pub(crate) h: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct Pos {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EnemyDef {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) waypoint_a: Pos,
    pub(crate) waypoint_b: Pos,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OscFireballDef {
    pub(crate) x: f32,
    pub(crate) top_y: f32,
    pub(crate) bottom_y: f32,
}

/// A key placed in the level editor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct KeyDef {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) color: crate::level::KeyColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LevelData {
    #[serde(default)]
    pub(crate) name: String,
    pub(crate) bounds: Bounds,
    #[serde(default)]
    pub(crate) platforms: Vec<Rect>,
    #[serde(default)]
    pub(crate) spikes: Vec<Pos>,
    #[serde(default)]
    pub(crate) coins: Vec<Pos>,
    #[serde(default)]
    pub(crate) question_blocks: Vec<Pos>,
    #[serde(default)]
    pub(crate) bricks: Vec<Pos>,
    #[serde(default)]
    pub(crate) enemies: Vec<EnemyDef>,
    #[serde(default)]
    pub(crate) dart_enemies: Vec<Pos>,
    #[serde(default)]
    pub(crate) osc_fireballs: Vec<OscFireballDef>,
    #[serde(default)]
    pub(crate) checkpoints: Vec<Pos>,
    pub(crate) flagpole: Pos,
    #[serde(default = "default_player_spawn_editor")]
    pub(crate) player_spawn: Pos,
    #[serde(default = "default_parallax")]
    pub(crate) parallax: Vec<f32>,
    #[serde(default)]
    pub(crate) theme: crate::level::ThemeColors,
    #[serde(default)]
    pub(crate) clouds: Vec<crate::level::CloudSpawn>,
    #[serde(default)]
    pub(crate) portals: Vec<crate::level::PortalSpawn>,
    #[serde(default)]
    pub(crate) keys: Vec<KeyDef>,
}

fn default_parallax() -> Vec<f32> {
    vec![0.1, 0.3, 0.6]
}

fn default_player_spawn_editor() -> Pos {
    Pos { x: 100.0, y: 600.0 }
}

impl Default for LevelData {
    fn default() -> Self {
        Self {
            name: String::new(),
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
            player_spawn: Pos { x: 100.0, y: 600.0 },
            parallax: default_parallax(),
            theme: crate::level::ThemeColors::default(),
            clouds: vec![],
            portals: vec![],
            keys: vec![],
        }
    }
}
