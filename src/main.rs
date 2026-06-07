// Mario 2D Platformer Demo — Binary entry point
// Rust edition 2024 + Macroquad 0.4
//
// Wires the library components into a runnable game: initializes the Macroquad
// window, creates the Playing state, and drives the fixed-timestep game loop.

use mario_platformer::engine::{GameLoop, WindowConfig};
use mario_platformer::states::{GameState, LevelSelectState};

fn window_conf() -> macroquad::window::Conf {
    macroquad::window::Conf {
        window_title: "Mario 2D Platformer Demo".to_owned(),
        window_width: 1280,
        window_height: 720,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Load CJK font for Chinese hint text in-game
    let cjk = macroquad::text::load_ttf_font("C:/Windows/Fonts/msyh.ttc").await.ok();
    mario_platformer::init_cjk_font(cjk);

    let mut select = LevelSelectState::new();
    select.screen_w = macroquad::window::screen_width();
    select.screen_h = macroquad::window::screen_height();
    let mut game_state = GameState::LevelSelect(select);

    let config = WindowConfig {
        width: 1280,
        height: 720,
        fullscreen: false,
    };

    let mut game_loop = match GameLoop::new(config) {
        Ok(gl) => gl,
        Err(_) => {
            eprintln!("Failed to initialize game loop — unsupported resolution.");
            return;
        }
    };

    game_loop.run(&mut game_state).await;
}
