#![allow(clippy::type_complexity)]
//! Adds a menu bar to the app which has the following features:
//! - you can enable the **World Inspector**
//! - you can also enable **click to select**.
//!   For that to work, you need to tag your camera with [PickingCameraBundle](bevy_mod_picking::PickingCameraBundle) and your meshes with [PickableBundle](bevy_mod_picking::PickableBundle), see the [example] for a full demo.
//! - switch to app states you have registered using [`EditorSettings::add_state`]
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy::app::AppExit;
//! use bevy_editor_pls::{EditorPlugin, EditorSettings};
//!
//! #[derive(Clone)]
//! pub enum AppState {
//!     MainMenu,
//!     Game,
//! }
//!
//! fn editor_settings() -> EditorSettings {
//!     let mut settings = EditorSettings::default();
//!     settings.auto_pickable = true;
//!
//!     settings.add_event("Quit", || AppExit);
//!
//!     settings.add_state("Main menu", AppState::MainMenu);
//!     settings.add_state("Game", AppState::Game);
//!     settings
//! }
//!
//! fn main() {
//!     App::build()
//!         .insert_resource(editor_settings())
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(EditorPlugin)
//!         .insert_resource(State::new(AppState::MainMenu))
//!         .run();
//! }
//! ```
//!
//! [example]: https://github.com/jakobhellermann/bevy-editor-pls/blob/main/examples/main.rs

mod editor_settings;
pub mod extensions;
mod plugin;
mod systems;
mod ui;
mod utils;

pub use bevy_fly_camera;
pub use bevy_mod_picking;

pub use editor_settings::EditorSettings;
pub use plugin::EditorPlugin;
