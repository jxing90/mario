// Mario 2D Platformer Demo — Library root
// Rust edition 2024 + Macroquad 0.4
//
// This lib.rs provides the public API surface for integration tests (tests/*.rs)
// and serves as the core library for the binary entry point (main.rs).

pub mod engine;
pub mod state;
pub mod input;
pub mod audio;
pub mod editor;
pub mod assets;
pub mod level;
pub mod parallax;

pub mod metrics;
pub mod states;
pub mod entities;
pub mod palette;
pub mod systems;
pub mod verification;

// ── Shared CJK font for Chinese text rendering ──

use std::sync::OnceLock;
static CJK_FONT: OnceLock<Option<macroquad::text::Font>> = OnceLock::new();

/// Call once at startup (in both editor and game main). 
/// On Windows, load from C:/Windows/Fonts/msyh.ttc; fall back gracefully.
pub fn init_cjk_font(font: Option<macroquad::text::Font>) {
    CJK_FONT.set(font).ok();
}

/// Get the loaded CJK font if available.
pub fn cjk_font() -> Option<&'static macroquad::text::Font> {
    CJK_FONT.get().and_then(|f| f.as_ref())
}

/// Draw text with CJK font fallback. Uses CJK font if available and text contains non-ASCII.
pub fn draw_text_cjk(text: &str, x: f32, y: f32, font_size: f32, color: macroquad::color::Color) {
    use macroquad::text::draw_text;
    if text.chars().any(|c| c as u32 > 0x7F) {
        if let Some(font) = cjk_font() {
            let params = macroquad::text::TextParams {
                font: Some(font),
                font_size: font_size as u16,
                color,
                ..Default::default()
            };
            macroquad::text::draw_text_ex(text, x, y, params);
            return;
        }
    }
    draw_text(text, x, y, font_size, color);
}
