use bevy::{
    app::AppExit,
    ecs::component::Component,
    prelude::*,
    render::wireframe::WireframePlugin,
    wgpu::{WgpuFeature, WgpuFeatures, WgpuOptions},
};
use bevy_editor_pls::{EditorPlugin, EditorSettings};

#[derive(Clone)]
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
        .add_plugin(WireframePlugin)
        .add_plugin(EditorPlugin)
        // states
        .insert_resource(State::new(AppState::Overworld))
        .add_stage_before(CoreStage::Update, "app update", {
            let mut state = StateStage::<AppState>::default();
            state.on_state_enter(AppState::Overworld, overworld::setup.system());
            state.on_state_enter(AppState::Hell, hell::setup.system());

            state.on_state_exit(AppState::Overworld, despawn_all::<overworld::StateCleanup>.system());
            state.on_state_exit(AppState::Hell, despawn_all::<hell::StateCleanup>.system());
            state
        })
        // systems
        .add_startup_system(setup.system())
        .add_system(save.system())
        .run();
}

fn despawn_all<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for e in query.iter() {
        commands.despawn_recursive(e);
    }
}

fn save(mut events: EventReader<SaveEvent>) {
    for _ in events.iter() {
        println!("Saving...");
    }
}

pub fn setup(mut commands: Commands) {
    commands.spawn(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

mod overworld {
    use bevy::prelude::*;

    pub struct StateCleanup;

    pub fn setup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        commands
            .insert_resource(ClearColor::default())
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
                material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
                ..Default::default()
            })
            .with(StateCleanup)
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..Default::default()
            })
            .with(StateCleanup)
            .spawn(LightBundle {
                transform: Transform::from_xyz(4.0, 8.0, 4.0),
                ..Default::default()
            })
            .with(StateCleanup);
    }
}

mod hell {
    use bevy::prelude::*;

    pub struct StateCleanup;

    pub fn setup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        commands
            .insert_resource(ClearColor(Color::rgb(0.01, 0.0, 0.008)))
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
                material: materials.add(Color::rgb(0.8, 0.1, 0.2).into()),
                ..Default::default()
            })
            .with(StateCleanup)
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::rgb(0.4, 0.2, 0.1).into()),
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..Default::default()
            })
            .with(StateCleanup)
            .spawn(LightBundle {
                transform: Transform::from_xyz(4.0, 8.0, 4.0),
                ..Default::default()
            })
            .with(StateCleanup);
    }
}
