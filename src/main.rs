// Mario 2D Platformer Demo — Binary entry point
// Rust edition 2024 + Macroquad 0.4
//
// Wires the library components into a runnable game: initializes the Macroquad
// window, creates the Playing state, and drives the fixed-timestep game loop.

use mario_platformer::engine::{GameLoop, WindowConfig};
use mario_platformer::entities::player::{Player, PlayerConfig};
use mario_platformer::states::{GameState, LifeState, PlayingState};

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
    // Spawn player on the ground (level ground platform is at y=600)
    let mut player = Player::new(PlayerConfig::default());
    player.pos.y = 584.0;
    player.on_ground = true;

    let life_state = LifeState::new();
    let playing = PlayingState::new(player, life_state);
    let mut game_state = GameState::Playing(Box::new(playing));

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
