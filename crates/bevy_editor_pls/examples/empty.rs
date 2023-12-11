use bevy::prelude::*;
use bevy::log::LogPlugin;
use bevy_editor_pls::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build().disable::<LogPlugin>())
        .add_plugins(EditorPlugin::new())
        .run();
}
