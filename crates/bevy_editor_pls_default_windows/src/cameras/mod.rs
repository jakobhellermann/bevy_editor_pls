pub mod camera_2d_panzoom;
pub mod camera_3d_free;
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

// Present only one the one currently active camera
#[derive(Component)]
pub struct ActiveEditorCamera;

// Marker component for the 3d free camera
#[derive(Component)]
struct EditorCamera3dFree;

// Marker component for the 2d pan+zoom camera
#[derive(Component)]
struct EditorCamera2dPanZoom;

pub struct CameraWindow;

#[derive(Clone, Copy, PartialEq)]
pub enum EditorCamKind {
    D2PanZoom,
    D3Free,
}

impl EditorCamKind {
    fn name(self) -> &'static str {
        match self {
            EditorCamKind::D2PanZoom => "2D (Pan/Zoom)",
            EditorCamKind::D3Free => "3D (Free)",
        }
    }

    fn all() -> [EditorCamKind; 2] {
        [EditorCamKind::D2PanZoom, EditorCamKind::D3Free]
    }
}

impl Default for EditorCamKind {
    fn default() -> Self {
        EditorCamKind::D3Free
    }
}

#[derive(Default)]
pub struct CameraWindowState {
    // make sure to keep the `ActiveEditorCamera` marker component in sync with this field
    editor_cam: EditorCamKind,
}

impl CameraWindowState {
    pub fn editor_cam(&self) -> EditorCamKind {
        self.editor_cam
    }
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

    fn viewport_toolbar_ui(world: &mut World, mut cx: EditorWindowContext, ui: &mut egui::Ui) {
        let state = cx.state_mut::<CameraWindow>().unwrap();
        ui.menu_button(state.editor_cam.name(), |ui| {
            for camera in EditorCamKind::all() {
                ui.horizontal(|ui| {
                    if ui.button(camera.name()).clicked() {
                        if state.editor_cam != camera {
                            set_active_editor_camera_marker(world, camera);
                        }

                        state.editor_cam = camera;

                        ui.close_menu();
                    }
                });
            }
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
            .add_system(initial_camera_setup)
            .add_startup_system_to_stage(StartupStage::PreStartup, spawn_editor_cameras);

        editor_cam_render::setup(app);

        #[cfg(feature = "viewport")]
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            set_main_pass_viewport.before(bevy::render::camera::UpdateCameraProjectionSystem),
        );
    }
}

fn set_active_editor_camera_marker(world: &mut World, editor_cam: EditorCamKind) {
    let mut previously_active = world.query_filtered::<Entity, With<ActiveEditorCamera>>();
    let mut previously_active_iter = previously_active.iter(world);
    let previously_active = previously_active_iter
        .next()
        .expect("there should be a camera with the `ActiveEditorCamera` component");
    assert!(
        previously_active_iter.next().is_none(),
        "there should be only one `ActiveEditorCamera`"
    );
    world
        .entity_mut(previously_active)
        .remove::<ActiveEditorCamera>();

    let entity = match editor_cam {
        EditorCamKind::D2PanZoom => {
            let mut state = world.query_filtered::<Entity, With<EditorCamera2dPanZoom>>();
            state.iter(world).next().unwrap()
        }
        EditorCamKind::D3Free => {
            let mut state = world.query_filtered::<Entity, With<EditorCamera3dFree>>();
            state.iter(world).next().unwrap()
        }
    };
    world.entity_mut(entity).insert(ActiveEditorCamera);
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
        .insert(camera_3d_free::FlycamControls::default())
        .insert(crate::hierarchy::picking::EditorRayCastSource::new())
        .insert(EditorCamera)
        .insert(EditorCamera3dFree)
        .insert(HideInEditor)
        .insert(Name::new("Editor Camera 3D Free"));

    commands
        .spawn_bundle(OrthographicCameraBundle {
            camera: Camera {
                name: Some(EDITOR_CAMERA_2D_PAN_ZOOM.to_string()),
                ..Default::default()
            },
            ..OrthographicCameraBundle::new_2d()
        })
        .insert(camera_2d_panzoom::PanCamControls::default())
        .insert(EditorCamera)
        .insert(EditorCamera2dPanZoom)
        .insert(HideInEditor)
        .insert(Name::new("Editor Camera 2D Pan/Zoom"));
}

fn set_editor_cam_active(
    editor: Res<Editor>,
    editor_state: Res<EditorState>,
    mut editor_cam_3d_free: Query<&mut camera_3d_free::FlycamControls>,
    mut editor_cam_2d_panzoom: Query<&mut camera_2d_panzoom::PanCamControls>,
) {
    let editor_cam = editor.window_state::<CameraWindow>().unwrap().editor_cam;

    let mut editor_cam_3d_free = editor_cam_3d_free.single_mut();
    let mut editor_cam_2d_panzoom = editor_cam_2d_panzoom.single_mut();

    editor_cam_3d_free.enable_movement = matches!(editor_cam, EditorCamKind::D3Free)
        && editor_state.active
        && !editor_state.listening_for_text;
    editor_cam_3d_free.enable_look = matches!(editor_cam, EditorCamKind::D3Free)
        && editor_state.active
        && !editor_state.pointer_used();
    editor_cam_2d_panzoom.enabled = matches!(editor_cam, EditorCamKind::D2PanZoom)
        && editor_state.active
        && !editor_state.pointer_used();
}

fn toggle_editor_cam(
    mut editor_events: EventReader<EditorEvent>,
    mut active_cameras_raw: ResMut<ActiveCameras>,
    mut active_cameras: ResMut<PersistentActiveCameras>,
) {
    for event in editor_events.iter() {
        if let EditorEvent::Toggle { now_active } = *event {
            if now_active {
                active_cameras.disable_all(&mut active_cameras_raw);
            } else {
                active_cameras.enable_all(&mut active_cameras_raw);
            }
        }
    }
}

fn initial_camera_setup(
    mut has_decided_initial_cam: Local<bool>,
    mut was_positioned_3d_free: Local<bool>,
    mut was_positioned_2d_panzoom: Local<bool>,

    mut commands: Commands,
    mut editor: ResMut<Editor>,

    mut query_camera_3d_free: Query<
        (Entity, &mut Transform, &mut camera_3d_free::FlycamControls),
        (With<EditorCamera3dFree>, Without<EditorCamera2dPanZoom>),
    >,
    mut query_camera_2d_pan_zoom: Query<
        (Entity, &mut Transform),
        (With<EditorCamera2dPanZoom>, Without<EditorCamera3dFree>),
    >,
    cameras: Query<
        (&Camera, &Transform),
        (
            With<Camera>,
            Without<EditorCamera3dFree>,
            Without<EditorCamera2dPanZoom>,
        ),
    >,
) {
    let mut cam2d = None;
    let mut cam3d = None;

    for (camera, transform) in cameras.iter() {
        if camera.name.as_deref() == Some(CameraPlugin::CAMERA_2D) {
            cam2d = Some(transform);
        }
        if camera.name.as_deref() == Some(CameraPlugin::CAMERA_3D) {
            cam3d = Some(transform);
        }
    }

    if !*has_decided_initial_cam {
        let camera_state = editor.window_state_mut::<CameraWindow>().unwrap();

        match (cam2d.is_some(), cam3d.is_some()) {
            (true, false) => {
                camera_state.editor_cam = EditorCamKind::D2PanZoom;
                commands
                    .entity(query_camera_3d_free.single().0)
                    .insert(ActiveEditorCamera);
                *has_decided_initial_cam = true;
            }
            (false, true) => {
                camera_state.editor_cam = EditorCamKind::D3Free;
                commands
                    .entity(query_camera_3d_free.single().0)
                    .insert(ActiveEditorCamera);
                *has_decided_initial_cam = true;
            }
            (true, true) => {
                camera_state.editor_cam = EditorCamKind::D3Free;
                commands
                    .entity(query_camera_3d_free.single().0)
                    .insert(ActiveEditorCamera);
                *has_decided_initial_cam = true;
            }
            (false, false) => return,
        }
    }

    if !*was_positioned_3d_free {
        let (_, mut cam_transform, mut cam) = query_camera_3d_free.single_mut();
        if let Some(cam3d_transform) = cam3d {
            *cam_transform = cam3d_transform.clone();
            let (yaw, pitch, _) = cam3d_transform.rotation.to_euler(EulerRot::YXZ);
            cam.yaw = yaw.to_degrees();
            cam.pitch = pitch.to_degrees();
            *was_positioned_3d_free = true;
        }
    }

    if !*was_positioned_2d_panzoom {
        let (_, mut cam_transform) = query_camera_2d_pan_zoom.single_mut();
        if let Some(cam2d_transform) = cam2d {
            *cam_transform = cam2d_transform.clone();
            *was_positioned_2d_panzoom = true;
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
