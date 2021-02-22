use bevy::{app::AppExit, prelude::*};
use bevy_editor_pls::{EditorPlugin, EditorSettings};
use bevy_mod_picking::{PickableBundle, PickingCameraBundle};

#[derive(Clone, Hash)]
pub enum AppState {
    MainMenu,
    Overworld,
}

struct SaveEvent;

fn editor_settings() -> EditorSettings {
    let mut settings = EditorSettings::default();
    settings.add_event("Save", || SaveEvent);
    settings.add_event("Quit", || AppExit);

    settings.add_state("Main menu", AppState::MainMenu);
    settings.add_state("Overworld", AppState::Overworld);

    settings
}

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(editor_settings())
        .add_event::<SaveEvent>()
        .add_plugins(DefaultPlugins)
        .add_plugin(EditorPlugin)
        // states
        .insert_resource(State::new(AppState::MainMenu))
        .add_stage_before(CoreStage::Update, "app update", {
            let mut state = StateStage::<AppState>::default();
            state.on_state_enter(AppState::MainMenu, (|| println!("Main menu")).system());
            state.on_state_enter(AppState::Overworld, (|| println!("Overworld")).system());
            state
        })
        // systems
        .add_startup_system(setup.system())
        .add_system(save.system())
        .run();
}

fn save(mut events: EventReader<SaveEvent>) {
    for _ in events.iter() {
        println!("Saving...");
    }
}

fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..Default::default()
        })
        .with_bundle(PickableBundle::default())
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .with_bundle(PickableBundle::default())
        .spawn(LightBundle {
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..Default::default()
        })
        .spawn(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0)
                .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        })
        .with_bundle(PickingCameraBundle::default());
}
