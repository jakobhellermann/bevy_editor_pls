mod camera_2d_panzoom;
mod camera_3d_free;
mod editor_cam_render;

mod persistent_active_cameras;

use bevy::{
    prelude::*,
    render::camera::{ActiveCameras, CameraPlugin},
};
use bevy_editor_pls_core::{
    editor_window::{EditorWindow, EditorWindowContext},
    Editor, EditorEvent, EditorState,
};
use bevy_inspector_egui::egui;
use editor_cam_render::EDITOR_CAMERA_3D_FLYCAM;

use crate::hierarchy::HideInEditor;

use self::{
    editor_cam_render::EDITOR_CAMERA_2D_PAN_ZOOM,
    persistent_active_cameras::PersistentActiveCameras,
};

// Present on all editor cameras
#[derive(Component)]
pub struct EditorCamera;

// Present on only the one active editor camera
#[derive(Component)]
pub struct ActiveEditorCamera;

// Marker component for the 3d free camera
#[derive(Component)]
pub struct EditorCamera3dFree;

// Marker component for the 2d pan+zoom camera
#[derive(Component)]
pub struct EditorCamera2dPanZoom;

pub struct CameraWindow;

enum EditorCamKind {
    D2PanZoom,
    D3Free,
}

impl Default for EditorCamKind {
    fn default() -> Self {
        EditorCamKind::D3Free
    }
}

#[derive(Default)]
pub struct CameraWindowState {
    editor_cam: EditorCamKind,
    has_decided_initial_cam: bool,
}

impl EditorWindow for CameraWindow {
    type State = CameraWindowState;

    const NAME: &'static str = "Cameras";

    fn ui(world: &mut World, _cx: EditorWindowContext, ui: &mut bevy_inspector_egui::egui::Ui) {
        world.resource_scope(|world, active_cameras_raw: Mut<ActiveCameras>| {
            world.resource_scope(|_world, mut active_cameras: Mut<PersistentActiveCameras>| {
                active_cameras.update(&active_cameras_raw);
                cameras_ui(ui, &mut active_cameras, active_cameras_raw);
            });
        });
    }

    fn app_setup(app: &mut App) {
        app.init_resource::<PersistentActiveCameras>();

        app.add_plugin(camera_3d_free::FlycamPlugin)
            .add_plugin(camera_2d_panzoom::PanCamPlugin)
            .add_system(
                set_editor_cam_active
                    .before(camera_3d_free::CameraSystem::Movement)
                    .before(camera_2d_panzoom::CameraSystem::Movement),
            )
            .add_system_to_stage(CoreStage::PreUpdate, toggle_editor_cam)
            .add_startup_system(spawn_editor_cameras);

        editor_cam_render::setup(app);

        #[cfg(feature = "viewport")]
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            set_main_pass_viewport.before(bevy::render::camera::UpdateCameraProjectionSystem),
        );
    }
}

fn cameras_ui(
    ui: &mut egui::Ui,
    active_cameras: &mut PersistentActiveCameras,
    mut active_cameras_raw: Mut<ActiveCameras>,
) {
    let cameras = active_cameras.all_sorted();

    let mut toggle_cam_visibility = None;

    ui.label("Cameras");
    for (camera, active) in cameras {
        ui.horizontal(|ui| {
            let text = egui::RichText::new("👁").heading();
            let show_hide_button = egui::Button::new(text).frame(false);
            if ui.add(show_hide_button).clicked() {
                toggle_cam_visibility = Some((camera.to_string(), active));
            }

            if !active {
                ui.set_enabled(false);
            }

            ui.label(camera);
        });
    }

    if let Some((camera, previously_active)) = toggle_cam_visibility {
        active_cameras.set_active(camera, !previously_active, &mut active_cameras_raw);
    }
}

// Indicates that the camera should be moved to the position of the corresponding main app camera.
// Used to ensure that when switching to the editor for the first time, the camera will match
// the view of the in-game camera.
#[derive(Component)]
struct NeedsInitialPosition;

fn spawn_editor_cameras(mut commands: Commands) {
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            camera: Camera {
                name: Some(EDITOR_CAMERA_3D_FLYCAM.to_string()),
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 2.0, 5.0),
            ..Default::default()
        })
        .insert(camera_3d_free::Flycam::default())
        .insert(crate::hierarchy::picking::EditorRayCastSource::new())
        .insert(EditorCamera)
        .insert(EditorCamera3dFree)
        .insert(HideInEditor)
        .insert(NeedsInitialPosition)
        .insert(Name::new("Editor Camera 3D Free"));

    commands
        .spawn_bundle(OrthographicCameraBundle {
            camera: Camera {
                name: Some(EDITOR_CAMERA_2D_PAN_ZOOM.to_string()),
                ..Default::default()
            },
            ..OrthographicCameraBundle::new_2d()
        })
        .insert(camera_2d_panzoom::PanCam::default())
        .insert(EditorCamera)
        .insert(EditorCamera2dPanZoom)
        .insert(HideInEditor)
        .insert(NeedsInitialPosition)
        .insert(Name::new("Editor Camera 2D Pan/Zoom"));
}

fn set_editor_cam_active(
    editor_state: Res<EditorState>,
    mut editor_cam_3d_free: Query<&mut camera_3d_free::Flycam>,
    mut editor_cam_2d_panzoom: Query<&mut camera_2d_panzoom::PanCam>,
) {
    let mut editor_cam_3d_free = editor_cam_3d_free.single_mut();
    let mut editor_cam_2d_panzoom = editor_cam_2d_panzoom.single_mut();

    editor_cam_3d_free.enable_movement = editor_state.active && !editor_state.listening_for_text;
    editor_cam_3d_free.enable_look = editor_state.active && !editor_state.pointer_used();
    editor_cam_2d_panzoom.enabled = editor_state.active && !editor_state.pointer_used();
}

fn toggle_editor_cam(
    mut commands: Commands,
    mut editor_events: EventReader<EditorEvent>,
    mut editor: ResMut<Editor>,
    mut active_cameras_raw: ResMut<ActiveCameras>,
    mut active_cameras: ResMut<PersistentActiveCameras>,

    mut query_camera_3d_free: Query<
        (
            Entity,
            &mut Transform,
            &mut camera_3d_free::Flycam,
            Option<&NeedsInitialPosition>,
        ),
        (With<EditorCamera3dFree>, Without<EditorCamera2dPanZoom>),
    >,
    mut query_camera_2d_pan_zoom: Query<
        (
            Entity,
            &mut Transform,
            &mut camera_2d_panzoom::PanCam,
            Option<&NeedsInitialPosition>,
        ),
        (With<EditorCamera2dPanZoom>, Without<EditorCamera3dFree>),
    >,

    cameras: Query<
        &Transform,
        (
            With<Camera>,
            Without<EditorCamera3dFree>,
            Without<EditorCamera2dPanZoom>,
        ),
    >,
) {
    for event in editor_events.iter() {
        if let EditorEvent::Toggle { now_active } = *event {
            let camera_state = editor.window_state_mut::<CameraWindow>().unwrap();

            let cam2d_transform = active_cameras_raw
                .get(CameraPlugin::CAMERA_2D)
                .and_then(|cam| cameras.get(cam.entity?).ok());

            let cam3d_transform = active_cameras_raw
                .get(CameraPlugin::CAMERA_3D)
                .and_then(|cam| cameras.get(cam.entity?).ok());

            if now_active {
                active_cameras.disable_all(&mut active_cameras_raw);
            } else {
                active_cameras.enable_all(&mut active_cameras_raw);
            }

            if !camera_state.has_decided_initial_cam {
                match (cam2d_transform.is_some(), cam3d_transform.is_some()) {
                    (true, false) => {
                        camera_state.editor_cam = EditorCamKind::D2PanZoom;
                        let cam_2d = query_camera_2d_pan_zoom.single().0;
                        commands.entity(cam_2d).insert(ActiveEditorCamera);
                    }
                    (false, true) => {
                        camera_state.editor_cam = {
                            let cam_3d = query_camera_3d_free.single().0;
                            commands.entity(cam_3d).insert(ActiveEditorCamera);
                            EditorCamKind::D3Free
                        }
                    }
                    (false, false) | (true, true) => {}
                }

                camera_state.has_decided_initial_cam = true;
            }

            match camera_state.editor_cam {
                EditorCamKind::D3Free => {
                    let (entity, mut cam_transform, mut cam, needs_initial_position) =
                        query_camera_3d_free.single_mut();
                    cam.enable_movement = now_active;

                    if needs_initial_position.is_some() {
                        if let Some(cam3d_transform) = cam3d_transform {
                            *cam_transform = cam3d_transform.clone();
                            let (yaw, pitch, _) = cam3d_transform.rotation.to_euler(EulerRot::YXZ);
                            cam.yaw = yaw.to_degrees();
                            cam.pitch = pitch.to_degrees();
                        }
                        commands.entity(entity).remove::<NeedsInitialPosition>();
                    }
                }
                EditorCamKind::D2PanZoom => {
                    let (entity, mut cam_transform, mut cam, needs_initial_position) =
                        query_camera_2d_pan_zoom.single_mut();
                    cam.enabled = now_active;

                    if needs_initial_position.is_some() {
                        if let Some(cam2d_transform) = cam2d_transform {
                            *cam_transform = cam2d_transform.clone();
                        }
                        commands.entity(entity).remove::<NeedsInitialPosition>();
                    }
                }
            };
        }
    }
}

#[cfg(feature = "viewport")]
fn set_main_pass_viewport(
    editor_state: Res<bevy_editor_pls_core::EditorState>,
    egui_settings: Res<bevy_inspector_egui::bevy_egui::EguiSettings>,
    windows: Res<Windows>,
    mut cameras: Query<&mut Camera>,
) {
    if !editor_state.is_changed() {
        return;
    };

    let scale_factor = windows.get_primary().unwrap().scale_factor() * egui_settings.scale_factor;

    let viewport_pos = editor_state.viewport.left_top().to_vec2() * scale_factor as f32;
    let viewport_size = editor_state.viewport.size() * scale_factor as f32;

    cameras.iter_mut().for_each(|mut cam| {
        cam.viewport = editor_state.active.then(|| bevy::render::camera::Viewport {
            x: viewport_pos.x,
            y: viewport_pos.y,
            w: viewport_size.x.max(1.0),
            h: viewport_size.y.max(1.0),
            min_depth: 0.0,
            max_depth: 1.0,
            scaling_mode: bevy::render::camera::ViewportScalingMode::Pixels,
        });
    });
}
