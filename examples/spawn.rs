use bevy::prelude::*;
use bevy_editor_pls::{EditorPlugin, EditorSettings};
use bevy_mod_picking::{PickableBundle, PickingCameraBundle};

pub enum Event {
    Cube,
    Sphere,
}

fn editor_settings() -> EditorSettings {
    let mut settings = EditorSettings::default();
    settings.auto_flycam = true;

    settings.add_event("Spawn Cube", || Event::Cube);
    settings.add_event("Spawn Sphere", || Event::Sphere);
    settings
}

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(editor_settings())
        .add_plugins(DefaultPlugins)
        .add_plugin(EditorPlugin)
        .add_event::<Event>()
        .add_startup_system(setup.system())
        .add_system(spawn_events.system())
        .run();
}

fn spawn_events(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut events: EventReader<Event>,
) {
    for event in events.iter() {
        let mesh = match event {
            Event::Cube => Mesh::from(shape::Cube { size: 1.0 }),
            Event::Sphere => Mesh::from(shape::Icosphere {
                radius: 1.0,
                subdivisions: 16,
            }),
        };

        commands
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::WHITE.into()),
                ..Default::default()
            })
            .with_bundle(PickableBundle::default());
    }
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
        })
        .with_bundle(PickingCameraBundle::default());
}
