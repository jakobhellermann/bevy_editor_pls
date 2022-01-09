use bevy::{
    ecs::system::QuerySingleError,
    prelude::*,
    render::camera::{Camera, CameraPlugin, OrthographicProjection},
};
use bevy_fly_camera::FlyCamera;
use bevy_mod_picking::{PickableBundle, PickableMesh, PickingCamera, PickingCameraBundle};
use bevy_orbit_controls::OrbitCamera;
use bevy_pancam::PanCam;

use crate::{plugin::EditorState, ui::EditorMenuEvent, EditorSettings};

fn should_inspect_entity(input: &Input<KeyCode>) -> bool {
    input.pressed(KeyCode::LControl)
}

fn should_select_orbit_target(input: &Input<KeyCode>) -> bool {
    input.pressed(KeyCode::LAlt)
}

pub(crate) fn maintain_inspected_entities(
    mut commands: Commands,
    mut editor_settings: ResMut<EditorSettings>,
    mut editor_state: ResMut<EditorState>,
    mut editor_events: EventWriter<EditorMenuEvent>,
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
        match orbit_camera.get_single_mut() {
            Err(QuerySingleError::NoEntities(_)) => {
                let (cam_entity, _) = cameras
                    .iter()
                    .find(|(_, cam)| cam.name.as_ref().map_or(false, |name| name == CameraPlugin::CAMERA_3D))
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

        if editor_settings.fly_camera {
            editor_settings.fly_camera = false;
            editor_events.send(EditorMenuEvent::EnableFlyCams(false));
        }
    }
}

// auto systems

pub fn make_everything_pickable(
    editor_settings: Res<EditorSettings>,
    mut commands: Commands,
    // TODO: replacement for With<Draw>
    mut query: Query<Entity, (Without<PickableMesh>, Without<Node>)>,
) {
    if !editor_settings.auto_pickable && !editor_settings.auto_gizmo_target {
        return;
    }

    query.iter_mut().for_each(|entity| {
        let mut entity = commands.entity(entity);
        if editor_settings.auto_pickable {
            entity.insert_bundle(PickableBundle::default());
        }
        #[cfg(feature = "transform_gizmo")]
        if editor_settings.auto_gizmo_target {
            entity.insert(bevy_transform_gizmo::GizmoTransformable);
        }
    });
}
pub fn make_camera_picksource(
    editor_settings: Res<EditorSettings>,
    mut commands: Commands,
    mut query: Query<(Entity, &Camera), Without<PickingCamera>>,
) {
    if !editor_settings.auto_pickable_camera && !editor_settings.auto_gizmo_camera {
        return;
    }

    for (entity, cam) in query.iter_mut() {
        if cam.name.as_ref().map_or(false, |name| name == CameraPlugin::CAMERA_3D) {
            let mut entity = commands.entity(entity);
            if editor_settings.auto_pickable {
                entity.insert_bundle(PickingCameraBundle::default());
            }
            #[cfg(feature = "transform_gizmo")]
            if editor_settings.auto_gizmo_camera {
                entity.insert(bevy_transform_gizmo::GizmoPickSource::new());
            }
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
        if cam.name.as_ref().map_or(false, |name| name == CameraPlugin::CAMERA_3D) {
            commands.entity(entity).insert(FlyCamera {
                enabled: editor_settings.fly_camera,
                sensitivity: 6.0,
                only_if_mouse_down: Some(MouseButton::Left),
                ..Default::default()
            });
        }
    }
}

pub fn make_cam_pancam(
    editor_settings: Res<EditorSettings>,
    mut commands: Commands,
    mut query: Query<(Entity, &Camera), (With<OrthographicProjection>, Without<PanCam>)>,
) {
    if !editor_settings.auto_pancam {
        return;
    }

    for (entity, cam) in query.iter_mut() {
        if cam.name.as_ref().map_or(false, |name| name == CameraPlugin::CAMERA_2D) {
            commands.entity(entity).insert(PanCam::default());
        }
    }
}
