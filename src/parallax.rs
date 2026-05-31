// Feature #2: Level & Background — ParallaxLayer
// Design Reference: docs/features/2-level-background.md §4, §6

use crate::level::Vec2;

#[derive(Debug, Clone)]
pub struct ParallaxLayer {
    pub speed: f32,
    pub scroll_offset: Vec2,
}

impl ParallaxLayer {
    /// Creates a new parallax layer with the given speed factor.
    /// scroll_offset is initialized to (0.0, 0.0).
    pub fn new(speed: f32) -> Self {
        ParallaxLayer {
            speed,
            scroll_offset: Vec2 { x: 0.0, y: 0.0 },
        }
    }

    /// Updates the scroll offset based on the camera's world offset.
    /// Horizontal scroll = camera_offset.x * speed; vertical scroll stays at 0.0.
    pub fn update_scroll(&mut self, camera_offset: Vec2) {
        self.scroll_offset.x = camera_offset.x * self.speed;
        self.scroll_offset.y = 0.0;
    }
}
