use bevy::{
    app::AppExit,
    diagnostic::FrameTimeDiagnosticsPlugin,
    ecs::component::Component,
    prelude::*,
    render::wireframe::WireframePlugin,
    wgpu::{WgpuFeature, WgpuFeatures, WgpuOptions},
};
use bevy_editor_pls::{EditorPlugin, EditorSettings};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum AppState {
    Overworld,
    Hell,
}

struct SaveEvent;

fn editor_settings() -> EditorSettings {
    let mut settings = EditorSettings::default();
    settings.auto_pickable = true;
    settings.auto_flycam = true;

    settings.add_event("Save", || SaveEvent);
    settings.add_event("Quit", || AppExit);

    settings.add_state("Overworld", AppState::Overworld);
    settings.add_state("Hell", AppState::Hell);

    settings
}

fn main() {
    App::build()
        .insert_resource(WgpuOptions {
            features: WgpuFeatures {
                features: vec![WgpuFeature::NonFillPolygonMode],
            },
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(editor_settings())
        .add_event::<SaveEvent>()
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(WireframePlugin)
        .add_plugin(EditorPlugin)
        .add_startup_system(bevy_editor_pls::setup_default_keybindings.system())
        // states
        .add_state(AppState::Overworld)
        .add_system_set(SystemSet::on_enter(AppState::Overworld).with_system(overworld::setup.system()))
        .add_system_set(SystemSet::on_exit(AppState::Overworld).with_system(despawn_all::<overworld::StateCleanup>.system()))
        .add_system_set(SystemSet::on_enter(AppState::Hell).with_system(hell::setup.system()))
        .add_system_set(SystemSet::on_exit(AppState::Hell).with_system(despawn_all::<hell::StateCleanup>.system()))
        // systems
        .add_startup_system(setup.system())
        .add_system(save.system())
        .run();
}

fn despawn_all<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn save(mut events: EventReader<SaveEvent>) {
    for _ in events.iter() {
        println!("Saving...");
    }
}

pub fn setup(mut commands: Commands) {
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

mod overworld {
    use bevy::prelude::*;

    pub struct StateCleanup;

    pub fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
        commands.insert_resource(ClearColor::default());
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
                material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
                ..Default::default()
            })
            .insert(StateCleanup);
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..Default::default()
            })
            .insert(StateCleanup);
        commands
            .spawn_bundle(LightBundle {
                transform: Transform::from_xyz(4.0, 8.0, 4.0),
                ..Default::default()
            })
            .insert(StateCleanup);
    }
}

mod hell {
    use bevy::prelude::*;

    pub struct StateCleanup;

    pub fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
        commands.insert_resource(ClearColor(Color::rgb(0.01, 0.0, 0.008)));
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
                material: materials.add(Color::rgb(0.8, 0.1, 0.2).into()),
                ..Default::default()
            })
            .insert(StateCleanup);
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::rgb(0.4, 0.2, 0.1).into()),
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..Default::default()
            })
            .insert(StateCleanup);
        commands
            .spawn_bundle(LightBundle {
                transform: Transform::from_xyz(4.0, 8.0, 4.0),
                ..Default::default()
            })
            .insert(StateCleanup);
    }
}
