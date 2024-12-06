use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy_editor_pls::EditorPlugin::new())
        .add_systems(Startup, (setup_2d, setup_3d))
        .run();
}

fn setup_2d(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: 1,
            ..Default::default()
        },
    ));
    commands.spawn(Sprite {
        color: Color::srgb(0.25, 0.25, 0.75),
        custom_size: Some(Vec2::new(50.0, 50.0)),
        ..Default::default()
    });
}

fn setup_3d(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::new(0.25, 0.25)).mesh())),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Cuboid::from_size(Vec3::ONE)))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    commands.spawn((
        PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    commands.spawn(Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y));
}
