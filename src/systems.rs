use bevy::{
    prelude::*,
    render::{camera::Camera, render_graph::base::camera},
};
use bevy_fly_camera::FlyCamera;
use bevy_mod_picking::{PickableBundle, PickableMesh, PickingCamera, PickingCameraBundle};

use crate::{plugin::EditorState, utils, EditorSettings};

pub fn maintain_inspected_entities(
    editor_settings: ResMut<EditorSettings>,
    mut editor_state: ResMut<EditorState>,
    query: Query<(Entity, &Interaction), Changed<Interaction>>,
) {
    if !editor_settings.click_to_inspect {
        return;
    }

    let entity = query
        .iter()
        .filter(|(_, interaction)| matches!(interaction, Interaction::Clicked))
        .map(|(entity, _)| entity)
        .next();

    if let Some(entity) = entity {
        if editor_state.currently_inspected == Some(entity) {
            editor_state.currently_inspected = None;
        } else {
            editor_state.currently_inspected = Some(entity);
        }
    }
}

// auto systems

pub fn make_everything_pickable(
    editor_settings: Res<EditorSettings>,
    mut commands: Commands,
    mut query: Query<Entity, (With<Draw>, Without<PickableMesh>, Without<Node>)>,
) {
    if !editor_settings.auto_pickable || !editor_settings.click_to_inspect {
        return;
    }

    for entity in query.iter_mut() {
        commands.insert_bundle(entity, PickableBundle::default());
    }
}
pub fn make_camera_picksource(
    editor_settings: Res<EditorSettings>,
    mut commands: Commands,
    mut query: Query<(Entity, &Camera), Without<PickingCamera>>,
) {
    if !editor_settings.auto_pickable || !editor_settings.click_to_inspect {
        return;
    }

    for (entity, cam) in query.iter_mut() {
        if cam.name.as_ref().map_or(false, |name| name == camera::CAMERA_3D) {
            commands.insert_bundle(entity, PickingCameraBundle::default());
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
            commands.insert(
                entity,
                FlyCamera {
                    enabled: editor_settings.fly_camera,
                    ..Default::default()
                },
            );
        }
    }
}

pub fn esc_cursor_grab(keys: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    if keys.just_pressed(KeyCode::Escape) {
        utils::toggle_grab_cursor(window);
    }
}
