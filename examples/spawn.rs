use bevy::prelude::*;
use bevy_editor_pls::{extensions::EditorExtensionSpawn, EditorPlugin, EditorSettings};

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource({
            let mut settings = EditorSettings::new();
            settings.auto_flycam = true;
            settings.auto_pickable = true;
            settings.click_to_inspect = true;
            settings
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(EditorPlugin)
        .add_plugin(EditorExtensionSpawn)
        .add_startup_system(setup.system())
        .run();
}

pub fn setup(
    mut commands: Commands,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut _materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(LightBundle {
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..Default::default()
        })
        .spawn(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        });
}
