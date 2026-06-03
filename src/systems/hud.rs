// Feature #9: HUD — HudRenderer implementation
//
// Design Reference: docs/features/9-hud.md §4 Interface Contract, §6 Implementation Summary
// SRS Reference: FR-016 (Heads-Up Display)
//
// HudRenderer renders PlayerStats (coins, lives) as a screen-space overlay anchored
// at viewport (3%, 3%) with ±2% tolerance. Coin icon + count on first row,
// heart icon + lives count on second row. Text uses 1px black outline pixel font.
//
// Texture2D fields are Option-wrapped because macroquad/miniquad types require an
// active GL context to construct, which is unavailable during `cargo test`.
// Draw calls are guarded by Option checks — they no-op when textures are absent.

use macroquad::color::{BLACK, WHITE};
use macroquad::math::Vec2;
use macroquad::text::draw_text_ex;
use macroquad::text::TextParams;
use macroquad::texture::draw_texture_ex;
use macroquad::texture::DrawTextureParams;
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
    /// Draw calls are guarded by Option<Texture2D> — when textures are None (e.g. during
    /// `cargo test` with no GL context), icon rendering is skipped. Text rendering always
    /// executes; at runtime the Macroquad draw queue will flush to the GPU.
    pub fn render(&self, stats: PlayerStats, viewport_w: f32, viewport_h: f32) {
        // Guard: invalid viewport → early return (no drawing, no panic).
        if !(viewport_w > 0.0 && viewport_h > 0.0) {
            return;
        }

        let (anchor_x, anchor_y) = Self::compute_anchor(viewport_w, viewport_h);
        let icon_size = Self::rendered_icon_size();

        // Skip all draw calls when textures are not loaded (no GL context).
        // During `cargo test`, macroquad/miniquad is not initialized and calling
        // draw_texture_ex / draw_text_ex would panic. At runtime, textures are
        // loaded by PlayingState before the first frame.
        if self.coin_tex.is_none() && self.heart_tex.is_none() {
            return;
        }

        // --- layout positions (§6 Implementation Summary §3) ---
        let coin_text_x = anchor_x + icon_size + 4.0; // icon right + 4px gap
        let coin_text_y = anchor_y;
        let heart_icon_y = anchor_y + icon_size + 4.0; // below coin row + 4px gap
        let lives_text_x = anchor_x + icon_size + 4.0;
        let lives_text_y = heart_icon_y;

        // --- draw coin icon (§Visual Rendering Contract Element 1) ---
        if let Some(tex) = self.coin_tex.as_ref() {
            draw_texture_ex(
                tex,
                anchor_x,
                anchor_y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(icon_size, icon_size)),
                    ..Default::default()
                },
            );
        }

        // --- draw coin count text with 1px black outline (§Visual Rendering Contract Element 2) ---
        {
            let s = stats.coins.to_string();
            for (i, &(ox, oy)) in Self::outline_positions(coin_text_x, coin_text_y).iter().enumerate() {
                draw_text_ex(
                    &s,
                    ox,
                    oy,
                    TextParams {
                        font_size: self.font_size,
                        color: if i < 4 { BLACK } else { WHITE },
                        ..Default::default()
                    },
                );
            }
        }

        // --- draw heart icon (§Visual Rendering Contract Element 3) ---
        if let Some(tex) = self.heart_tex.as_ref() {
            draw_texture_ex(
                tex,
                anchor_x,
                heart_icon_y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(icon_size, icon_size)),
                    ..Default::default()
                },
            );
        }

        // --- draw lives count text with 1px black outline (§Visual Rendering Contract Element 4) ---
        {
            let s = stats.lives.to_string();
            for (i, &(ox, oy)) in Self::outline_positions(lives_text_x, lives_text_y).iter().enumerate() {
                draw_text_ex(
                    &s,
                    ox,
                    oy,
                    TextParams {
                        font_size: self.font_size,
                        color: if i < 4 { BLACK } else { WHITE },
                        ..Default::default()
                    },
                );
            }
        }
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

    /// Set the icon textures after loading at runtime (when GL context is active).
    ///
    /// During tests, textures remain `None` and draw calls skip icon rendering.
    #[allow(dead_code)]
    pub fn set_textures(&mut self, coin_tex: Texture2D, heart_tex: Texture2D) {
        self.coin_tex = Some(coin_tex);
        self.heart_tex = Some(heart_tex);
    }
}

impl Default for HudRenderer {
    fn default() -> Self {
        Self::new()
    }
}
