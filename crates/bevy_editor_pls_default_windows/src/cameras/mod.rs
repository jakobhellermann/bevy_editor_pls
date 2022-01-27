mod editor_cam_controls;
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
use editor_cam_render::EDITOR_CAMERA_FLYCAM;

use crate::hierarchy::HideInEditor;

use self::persistent_active_cameras::PersistentActiveCameras;

#[derive(Component)]
pub struct EditorCamera;

pub struct CameraWindow;

enum EditorCamKind {
    Flycam,
}

impl Default for EditorCamKind {
    fn default() -> Self {
        EditorCamKind::Flycam
    }
}

#[derive(Default)]
pub struct CameraWindowState {
    editor_cam: EditorCamKind,
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

        app.add_plugin(editor_cam_controls::FlycamPlugin)
            .add_system(set_editor_cam_active.before(editor_cam_controls::CameraSystem::Movement))
            .add_system_to_stage(CoreStage::PreUpdate, toggle_editor_cam)
            .add_startup_system(spawn_editor_cam);

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

fn spawn_editor_cam(mut commands: Commands) {
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            camera: Camera {
                name: Some(EDITOR_CAMERA_FLYCAM.to_string()),
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 2.0, 5.0),
            ..Default::default()
        })
        .insert(editor_cam_controls::Flycam::default())
        .insert(crate::hierarchy::picking::EditorRayCastSource::new())
        .insert(EditorCamera)
        .insert(HideInEditor)
        .insert(Name::new("Editor Flycam 3d"));
}

fn set_editor_cam_active(
    editor_state: Res<EditorState>,
    mut editor_cam: Query<&mut editor_cam_controls::Flycam>,
) {
    let mut editor_cam = match editor_cam.get_single_mut() {
        Ok(cam) => cam,
        Err(_) => return,
    };

    let enabled = editor_state.active && !editor_state.listening_for_text;
    editor_cam.enabled = enabled;
}

fn toggle_editor_cam(
    mut editor_events: EventReader<EditorEvent>,
    mut editor: ResMut<Editor>,
    mut active_cameras_raw: ResMut<ActiveCameras>,
    mut active_cameras: ResMut<PersistentActiveCameras>,

    mut flycam: Query<(&mut Transform, &mut editor_cam_controls::Flycam)>,
    cameras: Query<&Transform, (With<Camera>, Without<editor_cam_controls::Flycam>)>,
) {
    for event in editor_events.iter() {
        if let EditorEvent::Toggle { now_active } = *event {
            let camera_state = editor.window_state_mut::<CameraWindow>().unwrap();

            let cam3d_transform = active_cameras_raw
                .get(CameraPlugin::CAMERA_3D)
                .and_then(|cam| cameras.get(cam.entity?).ok());

            if now_active {
                active_cameras.disable_all(&mut active_cameras_raw);
                active_cameras_raw.add(EDITOR_CAMERA_FLYCAM);
            } else {
                active_cameras_raw.remove(EDITOR_CAMERA_FLYCAM);
                active_cameras.enable_all(&mut active_cameras_raw);
            }

            match camera_state.editor_cam {
                EditorCamKind::Flycam => {
                    let (mut cam_transform, mut cam) = flycam.single_mut();
                    cam.enabled = now_active;

                    if !cam.was_initially_positioned {
                        if let Some(cam3d_transform) = cam3d_transform {
                            *cam_transform = cam3d_transform.clone();
                            let (yaw, pitch, _) = cam3d_transform.rotation.to_euler(EulerRot::YXZ);
                            cam.yaw = yaw.to_degrees();
                            cam.pitch = pitch.to_degrees();
                        }
                        cam.was_initially_positioned = true;
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
