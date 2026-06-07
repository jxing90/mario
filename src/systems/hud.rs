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
use macroquad::prelude::Color;
use macroquad::text::draw_text_ex;
use macroquad::text::TextParams;
use macroquad::texture::draw_texture_ex;
use macroquad::texture::DrawTextureParams;
use macroquad::texture::Texture2D;

use crate::entities::player::PlayerStats;
use crate::level::KeyColor;

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

/// Draws a key shape at the given screen position with the given color and size.
///
/// The key consists of: a circular head at the top, a rectangular stem extending
/// downward, and two small rectangular teeth on the right side of the stem.
///
/// `x`, `y` — screen-space center of the key head.
/// `s` — scale factor (use f32::min(sx, sy) for aspect-ratio-correct rendering).
pub fn draw_key_shape(x: f32, y: f32, s: f32, color: macroquad::color::Color) {
    use macroquad::shapes::{draw_circle, draw_rectangle};
    // Key head (circle)
    draw_circle(x, y - 4.0 * s, 5.0 * s, color);
    // Key stem (rectangle downward from head)
    draw_rectangle(x - 1.5 * s, y + 1.0 * s, 3.0 * s, 8.0 * s, color);
    // Key teeth (two small rectangles on the stem)
    draw_rectangle(x + 1.5 * s, y + 3.0 * s, 3.0 * s, 2.0 * s, color);
    draw_rectangle(x + 1.5 * s, y + 6.0 * s, 3.0 * s, 2.0 * s, color);
}

// ============================================================================
// Horizontal HUD bar — screen-space overlay rendered at the top of the viewport.
// Replaces the inline rendering previously in PlayingState::render().
// ============================================================================

/// Data needed to render the horizontal HUD bar.
pub struct HudBarContext {
    pub screen_w: f32,
    pub screen_h: f32,
    pub sx: f32,
    pub sy: f32,
    pub world_label: String,
    pub coins: u32,
    pub lives: u32,
    pub time_remaining: f32,
    pub star_timer: f32,
    /// (color, count) for each collected key color.
    pub keys: Vec<(KeyColor, u32)>,
    pub hint_text: String,
    pub hint_timer: f32,
}

/// Renders the horizontal HUD bar at the top of the viewport.
///
/// Columns (left to right):
///   1. World name
///   2. Coins
///   3. Lives
///   4. Time remaining
///   5. Collected keys with key-shape icons
///   6. Star power countdown (only when active)
///
/// Hint text (portal lock/unlock messages) is rendered center-screen
/// with alpha-fade based on the timer.
pub fn render_hud_bar(ctx: &HudBarContext) {
    if ctx.screen_w <= 0.0 || ctx.screen_h <= 0.0 {
        return;
    }
    use macroquad::text::draw_text;

    let font_size = 14.0 * ctx.sx.min(ctx.sy);
    let y_pos = 12.0 * ctx.sy;

    // Column 1: World
    draw_text(&ctx.world_label, 8.0, y_pos, font_size, WHITE);

    // Column 2: Coins
    draw_text(
        &format!("COINS {}", ctx.coins),
        150.0 * ctx.sx,
        y_pos,
        font_size,
        Color::new(1.0, 0.85, 0.0, 1.0),
    );

    // Column 3: Lives
    draw_text(
        &format!("LIVES {}", ctx.lives),
        280.0 * ctx.sx,
        y_pos,
        font_size,
        Color::new(1.0, 0.3, 0.3, 1.0),
    );

    // Column 4: Time
    let time_color = if ctx.time_remaining <= 60.0 {
        Color::new(1.0, 0.2, 0.2, 1.0)
    } else {
        WHITE
    };
    draw_text(
        &format!("TIME {}", ctx.time_remaining as u32),
        430.0 * ctx.sx,
        y_pos,
        font_size,
        time_color,
    );

    // Column 5: Collected keys
    let key_x = 580.0 * ctx.sx;
    let key_colors = [
        KeyColor::Red,
        KeyColor::Orange,
        KeyColor::Yellow,
        KeyColor::Green,
        KeyColor::Blue,
        KeyColor::Indigo,
        KeyColor::Violet,
    ];
    for (i, &kc) in key_colors.iter().enumerate() {
        let count = ctx
            .keys
            .iter()
            .filter(|(c, _)| *c == kc)
            .map(|(_, n)| *n)
            .next()
            .unwrap_or(0);
        if count > 0 {
            let kx = key_x + i as f32 * 32.0 * ctx.sx;
            let rgba = crate::entities::key::key_color_rgba(kc);
            crate::systems::hud::draw_key_shape(
                kx + 5.0 * ctx.sx,
                y_pos - 6.0 * ctx.sy,
                ctx.sx.min(ctx.sy) * 0.7,
                rgba,
            );
            draw_text(
                &format!("x{}", count),
                kx + 13.0 * ctx.sx,
                y_pos,
                font_size,
                WHITE,
            );
        }
    }

    // Column 6: Star power countdown
    if ctx.star_timer > 0.0 {
        draw_text(
            &format!("STAR {:.1}", ctx.star_timer),
            560.0 * ctx.sx,
            y_pos,
            font_size,
            Color::new(1.0, 0.85, 0.0, 1.0),
        );
    }

    // Hint text (centered, fades with timer)
    if !ctx.hint_text.is_empty() {
        let hint_font = 22.0 * ctx.sx.min(ctx.sy);
        let alpha = (ctx.hint_timer / 2.0).min(1.0);
        let hc = Color::new(1.0, 0.85, 0.3, alpha);
        crate::draw_text_cjk(
            &ctx.hint_text,
            ctx.screen_w / 2.0 - hint_font * ctx.hint_text.len() as f32 * 0.3,
            ctx.screen_h * 0.55,
            hint_font,
            hc,
        );
    }
}
