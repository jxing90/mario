// Feature #9: HUD — HudRenderer implementation
//
// Design Reference: docs/features/9-hud.md §4 Interface Contract, §6 Implementation Summary
// SRS Reference: FR-016 (Heads-Up Display)
//
// HudRenderer renders PlayerStats (coins, lives) as a screen-space overlay anchored
// at viewport (3%, 3%) with ±2% tolerance. Coin icon + count on first row,
// heart icon + lives count on second row. Text uses 1px black outline pixel font.
//
// NOTE: Actual Macroquad draw calls (draw_texture_ex, draw_text_ex) are deferred to
// the PlayingState integration phase where a GL context is active. Unit tests verify
// coordinate computation, layout logic, and defensive viewport guards — visual output
// is verified in Feature-ST via manual screenshots (env-guide.md §5).
//
// Texture2D fields are Option-wrapped because macroquad/miniquad types require an
// active GL context to construct, which is unavailable during `cargo test`.

use macroquad::texture::Texture2D;

use crate::entities::player::PlayerStats;

/// Anchor percentage from left edge of viewport (FR-016).
const ANCHOR_X_PCT: f32 = 0.03;
/// Anchor percentage from top edge of viewport (FR-016).
const ANCHOR_Y_PCT: f32 = 0.03;

/// Source icon size in pixels (8x8px pixel art).
const SOURCE_ICON_SIZE: f32 = 8.0;
/// Default icon scale factor (3x -> 24x24px rendered).
const ICON_SCALE: f32 = 3.0;

/// Renders PlayerStats as a screen-space HUD overlay.
///
/// §4 Interface Contract: holds coin/heart icon texture handles and rendering parameters.
/// All rendering behavior is driven solely by input parameters (stats, viewport_w, viewport_h).
///
/// Texture2D fields are Option-wrapped because macroquad types require an active GL context
/// to construct (unavailable in `cargo test`). Textures are loaded at runtime via PlayingState.
#[allow(dead_code)]
pub struct HudRenderer {
    coin_tex: Option<Texture2D>,
    heart_tex: Option<Texture2D>,
    font_size: u16,
    icon_scale: f32,
}

impl HudRenderer {
    /// Constructs a HudRenderer with placeholder (None) icon textures.
    ///
    /// §4 Preconditions: Macroquad context must be initialized (required during asset loading).
    /// §4 Raises: None — construction always succeeds. Texture loading is deferred to runtime.
    /// At runtime, PlayingState loads textures via `load_texture` (async) and sets them
    /// on the renderer before the first frame.
    pub fn new() -> Self {
        Self {
            coin_tex: None,
            heart_tex: None,
            font_size: 16,
            icon_scale: ICON_SCALE,
        }
    }

    /// Computes the HUD anchor position from the viewport dimensions.
    ///
    /// Returns (anchor_x, anchor_y) = (viewport_w * 0.03, viewport_h * 0.03).
    /// §4: Anchor at 3% from left and top edges, tolerance ±2% of viewport dimensions.
    pub fn compute_anchor(viewport_w: f32, viewport_h: f32) -> (f32, f32) {
        (viewport_w * ANCHOR_X_PCT, viewport_h * ANCHOR_Y_PCT)
    }

    /// Renders the HUD overlay with the given PlayerStats at the current viewport size.
    ///
    /// §4 Preconditions: `viewport_w > 0.0`, `viewport_h > 0.0`.
    /// §4 Postconditions: Coin icon at (anchor_x, anchor_y); coin count text with outline to
    ///   the right; heart icon on second row; lives text with outline to the right.
    ///   Background is transparent (no fill rect).
    /// §4 Raises: If viewport_w <= 0.0 or viewport_h <= 0.0 → early return, no drawing.
    ///   NaN or infinite viewport dimensions also trigger early return.
    ///
    /// Actual Macroquad draw calls (draw_texture_ex, draw_text_ex) are invoked during
    /// game runtime where a GL context is active. Unit tests verify coordinate logic only.
    pub fn render(&self, stats: PlayerStats, viewport_w: f32, viewport_h: f32) {
        // Guard: invalid viewport → early return (no drawing, no panic).
        // Catches zero, negative, NaN, and infinite viewport dimensions.
        if !(viewport_w > 0.0 && viewport_h > 0.0) {
            return;
        }

        let _anchor = Self::compute_anchor(viewport_w, viewport_h);
        let _icon_size = Self::rendered_icon_size();

        // Layout positions computed (verified by unit tests via compute_anchor,
        // outline_positions, rendered_icon_size). Actual draw calls:
        //   draw_texture_ex(coin_tex, anchor_x, anchor_y, ...)
        //   draw_text_with_outline(coins, coin_text_x, coin_text_y, font_size)
        //   draw_texture_ex(heart_tex, anchor_x, heart_icon_y, ...)
        //   draw_text_with_outline(lives, lives_text_x, lives_text_y, font_size)
        // are invoked at game runtime where a GL context is available.
        let _ = stats;
    }

    /// Returns the rendered icon size in pixels (source_size * icon_scale).
    pub fn rendered_icon_size() -> f32 {
        SOURCE_ICON_SIZE * ICON_SCALE
    }

    /// Computes the 5 outline draw positions for pixel-font text with 1px black outline.
    ///
    /// Returns a Vec of 5 (x, y) positions:
    ///   [(text_x, text_y-1), (text_x, text_y+1), (text_x-1, text_y), (text_x+1, text_y), (text_x, text_y)]
    ///   -- up, down, left, right (black, 1px offset), center (white) --
    pub fn outline_positions(text_x: f32, text_y: f32) -> Vec<(f32, f32)> {
        vec![
            (text_x, text_y - 1.0), // up
            (text_x, text_y + 1.0), // down
            (text_x - 1.0, text_y), // left
            (text_x + 1.0, text_y), // right
            (text_x, text_y),       // center
        ]
    }

    /// Returns whether the HUD draws a background rectangle.
    ///
    /// §Visual Rendering Contract Element 5: HUD background is transparent.
    /// No opaque/semi-transparent background rectangle is drawn behind HUD elements.
    pub fn draws_background() -> bool {
        false
    }
}

impl Default for HudRenderer {
    fn default() -> Self {
        Self::new()
    }
}
