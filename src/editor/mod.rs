// Level Editor — visual map editor for Mario 2D Platformer levels.
// Standalone binary: `cargo run --bin editor`
//
// Module structure:
//   data.rs   — LevelData and all data types
//   tool.rs   — Tool enum, DragTarget, menu constants
//   state.rs  — EditorState struct + core methods (new, IO, editing, menu_action, hit_test, drag)
//   update.rs — EditorState::update() method
//   render.rs — EditorState::render() method

mod data;
mod tool;
pub(crate) mod state;
mod update;
pub mod render;

pub use state::EditorState;
