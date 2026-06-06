// Level Editor — standalone binary for Mario 2D Platformer.
// Run: `cargo run --bin editor`
//
// Provides a visual editor for creating and modifying level JSON files.
// See src/editor/mod.rs for the editor implementation.

use mario_platformer::editor::EditorState;
use mario_platformer::editor::render::init_cjk_font;
use macroquad::prelude::*;

fn window_conf() -> macroquad::window::Conf {
    macroquad::window::Conf {
        window_title: "Mario Level Editor".to_owned(),
        window_width: 1400,
        window_height: 900,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Try loading a CJK-capable font for Chinese tooltips.
    // On Windows, try Microsoft YaHei; fall back gracefully.
    let cjk = macroquad::text::load_ttf_font("C:/Windows/Fonts/msyh.ttc").await.ok();
    init_cjk_font(cjk);

    let mut editor = EditorState::new();

    loop {
        editor.set_screen(screen_width(), screen_height());
        editor.update();
        editor.render();
        next_frame().await;
    }
}
