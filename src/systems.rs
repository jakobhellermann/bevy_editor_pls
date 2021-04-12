use bevy::{
    ecs::system::QuerySingleError,
    prelude::*,
    render::{camera::Camera, render_graph::base::camera},
};
use bevy_fly_camera::FlyCamera;
use bevy_mod_picking::{PickableBundle, PickableMesh, PickingCamera, PickingCameraBundle};
use bevy_orbit_controls::OrbitCamera;

use crate::{plugin::EditorState, utils, EditorSettings};

fn should_inspect_entity(input: &Input<KeyCode>) -> bool {
    input.pressed(KeyCode::LControl)
}

fn should_select_orbit_target(input: &Input<KeyCode>) -> bool {
    input.pressed(KeyCode::LAlt)
}

pub fn maintain_inspected_entities(
    mut commands: Commands,
    editor_settings: ResMut<EditorSettings>,
    mut editor_state: ResMut<EditorState>,
    query: Query<(Entity, &GlobalTransform, &Interaction), Changed<Interaction>>,
    mut orbit_camera: Query<&mut OrbitCamera>,
    cameras: Query<(Entity, &Camera)>,
    input: Res<Input<KeyCode>>,
) {
    if !editor_settings.click_to_inspect && !editor_settings.orbit_camera {
        return;
    }

    let result = query
        .iter()
        .filter(|(_, _, interaction)| matches!(interaction, Interaction::Clicked))
        .map(|(entity, transform, _)| (entity, transform))
        .next();

    let (entity, transform) = match result {
        Some(result) => result,
        _ => return,
    };

    if editor_settings.click_to_inspect && should_inspect_entity(&input) {
        if editor_state.currently_inspected == Some(entity) {
            editor_state.currently_inspected = None;
        } else {
            editor_state.currently_inspected = Some(entity);
        }
    }

    if editor_settings.orbit_camera && should_select_orbit_target(&input) {
        match orbit_camera.single_mut() {
            Err(QuerySingleError::NoEntities(_)) => {
                let (cam_entity, _) = cameras
                    .iter()
                    .find(|(_, cam)| cam.name.as_ref().map_or(false, |name| name == camera::CAMERA_3D))
                    .unwrap();

                commands
                    .entity(cam_entity)
                    .insert(OrbitCamera::new(10.0, transform.translation));
            }
            Err(QuerySingleError::MultipleEntities(_)) => panic!("There are multiple orbit cameras"),
            Ok(mut cam) => {
                cam.center = transform.translation;
            }
        };
    }
}

// auto systems

pub fn make_everything_pickable(
    editor_settings: Res<EditorSettings>,
    mut commands: Commands,
    mut query: Query<Entity, (With<Draw>, Without<PickableMesh>, Without<Node>)>,
) {
    if !editor_settings.auto_pickable {
        return;
    }

    for entity in query.iter_mut() {
        commands.entity(entity).insert_bundle(PickableBundle::default());
    }
}
pub fn make_camera_picksource(
    editor_settings: Res<EditorSettings>,
    mut commands: Commands,
    mut query: Query<(Entity, &Camera), Without<PickingCamera>>,
) {
    if !editor_settings.auto_pickable {
        return;
    }

    for (entity, cam) in query.iter_mut() {
        if cam.name.as_ref().map_or(false, |name| name == camera::CAMERA_3D) {
            commands.entity(entity).insert_bundle(PickingCameraBundle::default());
        }
    }
}

pub fn make_cam_flycam(
    editor_settings: Res<EditorSettings>,
    mut commands: Commands,
    mut query: Query<(Entity, &Camera), Without<FlyCamera>>,
) {
    if !editor_settings.auto_flycam {
        return;
    }

    for (entity, cam) in query.iter_mut() {
        if cam.name.as_ref().map_or(false, |name| name == camera::CAMERA_3D) {
            commands.entity(entity).insert(FlyCamera {
                enabled: editor_settings.fly_camera,
                ..Default::default()
            });
        }
    }
}

pub fn esc_cursor_grab(keys: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    if keys.just_pressed(KeyCode::Escape) {
        utils::toggle_grab_cursor(window);
    }
}
