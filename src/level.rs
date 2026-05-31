// Feature #2: Level & Background — Level, platforms, bounds, geometry primitives
// Design Reference: docs/features/2-level-background.md §4, §6, §8

use crate::parallax::ParallaxLayer;

// ============================================================================
// Geometry Primitives
// ============================================================================

/// 2D vector with x, y components — used for positions, offsets, and velocities.
#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

/// Axis-aligned bounding box. (x, y) is the top-left corner; (w, h) is width/height.
/// Both w and h must be >= 0.0 for valid geometry.
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl AABB {
    /// Returns true if `self` and `other` overlap on both axes.
    /// Boundary contact (e.g., self.max_x == other.min_x) counts as overlap
    /// (inclusive-edge semantics).
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

/// A terrain tile returned by `Level::query_terrain`.
/// `Empty` signals no terrain; `Platform(AABB)` is a solid block;
/// `Spike(AABB)` is a hazard zone (collision logic in Feature #5).
#[derive(Debug, Clone)]
pub enum Tile {
    Empty,
    Platform(AABB),
    Spike(AABB),
}

/// A static platform in the level. Holds its collision geometry (AABB).
#[derive(Debug, Clone)]
pub struct Platform {
    pub aabb: AABB,
}

// ============================================================================
// Level Bounds
// ============================================================================

/// Defines the playable area and kill plane for the level.
/// `kill_y` is the Y coordinate below which the player dies (pit fall).
#[derive(Debug, Clone, Copy)]
pub struct LevelBounds {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub kill_y: f32,
}

// ============================================================================
// Level
// ============================================================================

/// Holds the complete static level data: platforms, bounds, and parallax layers.
/// Constructed once at game start via `Level::new()` and passed by immutable reference.
pub struct Level {
    platforms: Vec<Platform>,
    bounds: LevelBounds,
    parallax_layers: Vec<ParallaxLayer>,
}

impl Default for Level {
    fn default() -> Self {
        Self::new()
    }
}

impl Level {
    /// Creates a new level with hardcoded platform geometry, bounds, and 3-layer parallax.
    ///
    /// Postconditions:
    /// - `platforms.len() >= 1` with all platforms having positive, finite dimensions
    /// - `bounds` defines a valid rectangular area with `kill_y` below the visible region
    /// - `parallax_layers.len() == 3` with speed factors 0.1, 0.3, 0.6 (far to near)
    pub fn new() -> Self {
        let platforms = vec![
            Platform {
                aabb: AABB {
                    x: 0.0,
                    y: 600.0,
                    w: 2000.0,
                    h: 40.0,
                },
            },
            Platform {
                aabb: AABB {
                    x: 400.0,
                    y: 450.0,
                    w: 200.0,
                    h: 30.0,
                },
            },
            Platform {
                aabb: AABB {
                    x: 1000.0,
                    y: 300.0,
                    w: 200.0,
                    h: 30.0,
                },
            },
        ];

        let bounds = LevelBounds {
            min_x: 0.0,
            max_x: 2000.0,
            min_y: 0.0,
            kill_y: 2500.0,
        };

        let parallax_layers = vec![
            ParallaxLayer::new(0.1),
            ParallaxLayer::new(0.3),
            ParallaxLayer::new(0.6),
        ];

        Level {
            platforms,
            bounds,
            parallax_layers,
        }
    }

    /// Returns a reference to all platforms in the level.
    pub fn platforms(&self) -> &[Platform] {
        &self.platforms
    }

    /// Queries terrain tiles that overlap with the given AABB.
    /// Returns `Vec<Tile>` with `Tile::Platform(aabb)` for each overlapping platform.
    /// Returns an empty Vec if no platforms overlap.
    /// Result order matches platform storage order. Does not panic on out-of-bounds AABBs.
    pub fn query_terrain(&self, aabb: &AABB) -> Vec<Tile> {
        self.platforms
            .iter()
            .filter(|p| p.aabb.intersects(aabb))
            .map(|p| Tile::Platform(p.aabb))
            .collect()
    }

    /// Returns the level bounds (pure function — always returns the same values).
    pub fn bounds(&self) -> LevelBounds {
        self.bounds
    }

    /// Returns a reference to the three parallax background layers.
    pub fn parallax_layers(&self) -> &[ParallaxLayer] {
        &self.parallax_layers
    }
}
