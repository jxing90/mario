// Mario 2D Platformer Demo — Library root
// Rust edition 2024 + Macroquad 0.4
//
// This lib.rs provides the public API surface for integration tests (tests/*.rs)
// and serves as the core library for the binary entry point (main.rs).

pub mod engine;
pub mod state;
pub mod input;
pub mod audio;
pub mod assets;
pub mod level;
pub mod parallax;

pub mod metrics;
pub mod states;
pub mod entities;
pub mod palette;
pub mod systems;
pub mod verification;
