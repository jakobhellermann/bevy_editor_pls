use bevy::prelude::*;
use bevy::log::LogPlugin;
use bevy_editor_pls::{controls, EditorPlugin};

fn main() {
    console_log::set_module_filter("controls=trace");

    App::new()
        .add_plugins(DefaultPlugins.build().disable::<LogPlugin>())
        .add_plugins(EditorPlugin::new())
        .insert_resource(editor_controls())
        .add_systems(Startup, (set_cam3d_controls, setup))
        .run();
}

fn editor_controls() -> controls::EditorControls {
    let mut editor_controls = controls::EditorControls::default_bindings();
    editor_controls.unbind(controls::Action::PlayPauseEditor);

    editor_controls.insert(
        controls::Action::PlayPauseEditor,
        controls::Binding {
            input: controls::UserInput::Single(controls::Button::Keyboard(KeyCode::Escape)),
            conditions: vec![controls::BindingCondition::ListeningForText(false)],
        },
    );

    editor_controls
}

fn set_cam3d_controls(
    mut query: Query<
        &mut bevy_editor_pls::default_windows::cameras::camera_3d_free::FlycamControls,
    >,
) {
    let mut controls = query.single_mut();
    controls.key_up = KeyCode::Q;
    controls.key_down = KeyCode::E;
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    log::info!("Start Setup Example");
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane::from_size(5.0))),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });
    // light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}
