use bevy::{
    asset::diagnostic::AssetCountDiagnosticsPlugin,
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_editor_pls::prelude::*;
use bevy_editor_pls_default_windows::hierarchy::picking::EditorRayCastSource;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EditorPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(EntityCountDiagnosticsPlugin)
        .add_plugin(AssetCountDiagnosticsPlugin::<StandardMaterial>::default())
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    // cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });
    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    // camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(EditorRayCastSource::new());
}
