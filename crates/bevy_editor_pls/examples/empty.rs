use bevy::prelude::*;
use bevy_editor_pls::prelude::*;
use bevy_editor_pls_default_windows::hierarchy::picking::EditorRayCastSource;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EditorPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(EditorRayCastSource::new());
}
