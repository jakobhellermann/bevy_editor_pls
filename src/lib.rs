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
//! use bevy::asset::AssetPath;
//! use bevy_editor_pls::{EditorPlugin, EditorSettings};
//!
//! #[derive(Clone, Eq, PartialEq, Hash, Debug)]
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
//!
//!     settings.on_file_drop(&["gltf", "glb"], |path, world| {
//!         let asset_path = AssetPath::new_ref(path, Some("Scene0".into()));
//!         let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
//!         let scene_handle = asset_server.load(asset_path);
//!
//!         let mut spawner = world.get_resource_mut::<SceneSpawner>().unwrap();
//!         spawner.spawn(scene_handle);
//!     });
//!
//!     settings
//! }
//!
//! fn main() {
//!     App::new()
//!         .insert_resource(editor_settings())
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(EditorPlugin)
//!         .insert_resource(State::new(AppState::MainMenu))
//!         .run();
//! }
//! ```
//!
//! [example]: https://github.com/jakobhellermann/bevy-editor-pls/blob/main/examples/main.rs

mod action;
mod drag_and_drop;
mod editor_settings;
pub mod extensions;
mod plugin;
mod second_window_plugin;
mod systems;
mod ui;
mod utils;

pub use bevy_fly_camera;
//pub use bevy_input_actionmap;
pub use bevy_mod_picking;

pub use action::EditorAction;
pub use editor_settings::EditorSettings;
pub use plugin::EditorPlugin;
pub use second_window_plugin::EditorPluginSecondWindow;

// use bevy::prelude::*;

/*
use bevy_input_actionmap::InputMap;
/// Sets up the default keybindings for the editor.
///
/// * `Ctrl + F`: toggle the fly camera
/// * `Ctrl + W`: toggle the world inspector
/// * `Ctrl + P`: toggle the performance panel
/// * `Ctrl + Esc`: toggle whether the editor UI should be displayed
pub fn setup_default_keybindings(mut input: ResMut<InputMap<EditorAction>>) {
    input.bind(EditorAction::ToggleFlycam, vec![KeyCode::LControl, KeyCode::F]);
    input.bind(EditorAction::TogglePerformancePanel, vec![KeyCode::LControl, KeyCode::P]);
    input.bind(EditorAction::ToggleWorldInspector, vec![KeyCode::LControl, KeyCode::W]);
    input.bind(EditorAction::ToggleEditorUi, vec![KeyCode::LControl, KeyCode::Escape]);
}
*/
