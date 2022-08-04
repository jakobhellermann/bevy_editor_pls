pub mod camera_2d_panzoom;
pub mod camera_3d_free;
pub mod camera_3d_panorbit;
mod editor_cam_render;

use bevy::{
    prelude::*,
    render::primitives::Aabb,
};
use bevy::utils::HashSet;
use bevy_editor_pls_core::{
    editor_window::{EditorWindow, EditorWindowContext},
    Editor, EditorEvent, EditorState,
};
use bevy_inspector_egui::egui;

use crate::hierarchy::{HideInEditor, HierarchyWindow};

use self::camera_3d_panorbit::PanOrbitCamera;

// Present on all editor cameras
#[derive(Component)]
pub struct EditorCamera;

// Present only one the one currently active camera
#[derive(Component)]
pub struct ActiveEditorCamera;

// Marker component for the 3d free camera
#[derive(Component)]
struct EditorCamera3dFree;

// Marker component for the 3d pan+orbit
#[derive(Component)]
struct EditorCamera3dPanOrbit;

// Marker component for the 2d pan+zoom camera
#[derive(Component)]
struct EditorCamera2dPanZoom;

pub struct CameraWindow;

#[derive(Clone, Copy, PartialEq)]
pub enum EditorCamKind {
    D2PanZoom,
    D3Free,
    D3PanOrbit,
}

impl EditorCamKind {
    fn name(self) -> &'static str {
        match self {
            EditorCamKind::D2PanZoom => "2D (Pan/Zoom)",
            EditorCamKind::D3Free => "3D (Free)",
            EditorCamKind::D3PanOrbit => "3D (Pan/Orbit)",
        }
    }

    fn all() -> [EditorCamKind; 3] {
        [
            EditorCamKind::D2PanZoom,
            EditorCamKind::D3Free,
            EditorCamKind::D3PanOrbit,
        ]
    }
}

impl Default for EditorCamKind {
    fn default() -> Self {
        EditorCamKind::D3PanOrbit
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

    fn ui(world: &mut World, _cx: EditorWindowContext, ui: &mut egui::Ui) {
        cameras_ui(ui, world);
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
        app.init_resource::<PreviouslyActiveCameras>();

        app.add_plugin(camera_2d_panzoom::PanCamPlugin)
            .add_plugin(camera_3d_free::FlycamPlugin)
            .add_plugin(camera_3d_panorbit::PanOrbitCameraPlugin)
            .add_system(
                set_editor_cam_active
                    .before(camera_3d_panorbit::CameraSystem::Movement)
                    .before(camera_3d_free::CameraSystem::Movement)
                    .before(camera_2d_panzoom::CameraSystem::Movement),
            )
            .add_system_to_stage(CoreStage::PreUpdate, toggle_editor_cam)
            .add_system_to_stage(CoreStage::PreUpdate, focus_selected)
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
    let previously_active = previously_active_iter.next();

    assert!(
        previously_active_iter.next().is_none(),
        "there should be only one `ActiveEditorCamera`"
    );

    if let Some(previously_active) = previously_active {
        world
            .entity_mut(previously_active)
            .remove::<ActiveEditorCamera>();
    }

    let entity = match editor_cam {
        EditorCamKind::D2PanZoom => {
            let mut state = world.query_filtered::<Entity, With<EditorCamera2dPanZoom>>();
            state.iter(world).next().unwrap()
        }
        EditorCamKind::D3Free => {
            let mut state = world.query_filtered::<Entity, With<EditorCamera3dFree>>();
            state.iter(world).next().unwrap()
        }
        EditorCamKind::D3PanOrbit => {
            let mut state = world.query_filtered::<Entity, With<EditorCamera3dPanOrbit>>();
            state.iter(world).next().unwrap()
        }
    };
    world.entity_mut(entity).insert(ActiveEditorCamera);
}

fn cameras_ui(ui: &mut egui::Ui, world: &mut World) {
    // let cameras = active_cameras.all_sorted();
    // let mut query: QueryState<&Camera> = world.query();
    // for camera in query.iter(world) {
    //
    // }

    let prev_cams = world.resource::<PreviouslyActiveCameras>();

    ui.label("Cameras");
    for cam in prev_cams.0.iter() {
        ui.horizontal(|ui| {
            // let active = curr_active.or(prev_active);

            /*let text = egui::RichText::new("👁").heading();
            let show_hide_button = egui::Button::new(text).frame(false);
            if ui.add(show_hide_button).clicked() {
                toggle_cam_visibility = Some((camera.to_string(), active));
            }*/

            // if active.is_none() {
            //     ui.set_enabled(false);
            // }

            ui.label(format!("{}: {:?}", "Camera", cam));
        });
    }
}

fn spawn_editor_cameras(mut commands: Commands) {
    #[derive(Component, Default)]
    struct Ec2d;
    #[derive(Component, Default)]
    struct Ec3d;

    commands
        .spawn_bundle(Camera3dBundle {
            camera: Camera::default(),
            transform: Transform::from_xyz(0.0, 2.0, 5.0),
            ..Camera3dBundle::default()
        })
        .insert(Ec3d)
        .insert(camera_3d_free::FlycamControls::default())
        .insert(crate::hierarchy::picking::EditorRayCastSource::new())
        .insert(EditorCamera)
        .insert(EditorCamera3dFree)
        .insert(HideInEditor)
        .insert(Name::new("Editor Camera 3D Free"));

    commands
        .spawn_bundle(Camera3dBundle {
            camera: Camera::default(),
            transform: Transform::from_xyz(0.0, 2.0, 5.0),
            ..Camera3dBundle::default()
        })
        .insert(Ec3d)
        .insert(PanOrbitCamera::default())
        .insert(crate::hierarchy::picking::EditorRayCastSource::new())
        .insert(EditorCamera)
        .insert(EditorCamera3dPanOrbit)
        .insert(HideInEditor)
        .insert(Name::new("Editor Camera 3D Pan/Orbit"));

    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(Ec2d)
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
    mut editor_cam_3d_panorbit: Query<&mut camera_3d_panorbit::PanOrbitCamera>,
    mut editor_cam_2d_panzoom: Query<&mut camera_2d_panzoom::PanCamControls>,
) {
    let editor_cam = editor.window_state::<CameraWindow>().unwrap().editor_cam;

    let mut editor_cam_3d_free = editor_cam_3d_free.single_mut();
    let mut editor_cam_3d_panorbit = editor_cam_3d_panorbit.single_mut();
    let mut editor_cam_2d_panzoom = editor_cam_2d_panzoom.single_mut();

    editor_cam_3d_free.enable_movement = matches!(editor_cam, EditorCamKind::D3Free)
        && editor_state.active
        && !editor_state.listening_for_text;
    editor_cam_3d_free.enable_look = matches!(editor_cam, EditorCamKind::D3Free)
        && editor_state.active
        && editor_state.viewport_interaction_active();
    editor_cam_3d_panorbit.enabled = matches!(editor_cam, EditorCamKind::D3PanOrbit)
        && editor_state.active
        && editor_state.viewport_interaction_active();
    editor_cam_2d_panzoom.enabled = matches!(editor_cam, EditorCamKind::D2PanZoom)
        && editor_state.active
        && editor_state.viewport_interaction_active();
}

#[derive(Component, Default)]
struct PreviouslyActiveCameras(HashSet<Entity>);

fn toggle_editor_cam(
    mut editor_events: EventReader<EditorEvent>,
    mut prev_active_cams: ResMut<PreviouslyActiveCameras>,
    mut cam_query: Query<(Entity, &mut Camera)>,
) {
    for event in editor_events.iter() {
        if let EditorEvent::Toggle { now_active } = *event {
            if now_active {
                // Add all currently active cameras
                for (e, mut cam) in cam_query.iter_mut().filter(|(_e, c)| c.is_active) {
                    prev_active_cams.0.insert(e);
                    cam.is_active = false;
                }

                // if let Some(cam) = prev_active_cam_3d.0 {
                //     if let Ok(camera) = cam_query.get_mut(cam) {
                //         camera.is_active = false;
                //     }
                // }
                // if let Some(cam) = prev_active_cam_2d.0 {
                //     if let Ok(camera) = cam_query.get_mut(cam) {
                //         camera.is_active = false;
                //     }
                // }
            } else {
                for cam in prev_active_cams.0.iter() {
                    if let Ok((_e, mut camera)) = cam_query.get_mut(*cam) {
                        camera.is_active = true;
                    }
                }
                prev_active_cams.0.clear();
                // if let Some(prev_active) = prev_active_cam_3d.0 {
                //     active_cam_3d.set(prev_active);
                //     if let Ok(camera) = cam_query.get_mut(prev_active) {
                //         camera.is_active = true;
                //     }
                // }
                // if let Some(prev_active) = prev_active_cam_2d.0 {
                //     active_cam_3d.set(prev_active);
                //     if let Ok(camera) = cam_query.get_mut(prev_active) {
                //         camera.is_active = true;
                //     }
                // }
            }
        }
    }
}

fn focus_selected(
    mut editor_events: EventReader<EditorEvent>,
    mut active_cam: Query<
        (
            &mut Transform,
            Option<&mut PanOrbitCamera>,
            Option<&mut OrthographicProjection>,
        ),
        With<ActiveEditorCamera>,
    >,
    selected_query: Query<
        (&GlobalTransform, Option<&Aabb>, Option<&Sprite>),
        Without<ActiveEditorCamera>,
    >,
    editor: Res<Editor>,
    windows: Res<Windows>,
) {
    for event in editor_events.iter() {
        match *event {
            EditorEvent::FocusSelected => (),
            _ => continue,
        }

        let hierarchy = editor.window_state::<HierarchyWindow>().unwrap();
        if hierarchy.selected.is_empty() {
            info!("Coudldn't focus on selection because selection is empty");
            return;
        }

        let (bounds_min, bounds_max) = hierarchy
            .selected
            .iter()
            .filter_map(|selected_e| {
                selected_query
                    .get(selected_e)
                    .map(|(&tf, bounds, sprite)| {
                        let default_value = (tf.translation(), tf.translation());
                        let sprite_size = sprite
                            .map(|s| s.custom_size.unwrap_or(Vec2::ONE))
                            .map_or(default_value, |sprite_size| {
                                (
                                    tf * Vec3::from((sprite_size * -0.5, 0.0)),
                                    tf * Vec3::from((sprite_size * 0.5, 0.0)),
                                )
                            });

                        bounds.map_or(sprite_size, |bounds| {
                            (tf * Vec3::from(bounds.min()), tf * Vec3::from(bounds.max()))
                        })
                    })
                    .ok()
            })
            .fold(
                (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)),
                |(acc_min, acc_max), (min, max)| (acc_min.min(min), acc_max.max(max)),
            );

        const RADIUS_MULTIPLIER: f32 = 2.0;

        let bounds_size = bounds_max - bounds_min;
        let focus_loc = bounds_min + bounds_size * 0.5;
        let radius = if bounds_size.max_element() > f32::EPSILON {
            bounds_size.length() * RADIUS_MULTIPLIER
        } else {
            RADIUS_MULTIPLIER
        };

        let (mut camera_tf, pan_orbit_cam, ortho) = active_cam.single_mut();

        if let Some(mut ortho) = ortho {
            camera_tf.translation.x = focus_loc.x;
            camera_tf.translation.y = focus_loc.y;

            if let Some(w) = windows.get_primary() {
                ortho.scale = radius / w.width().min(w.height()).max(1.0);
            }
        } else {
            camera_tf.translation = focus_loc + camera_tf.rotation.mul_vec3(Vec3::Z) * radius;
        }

        if let Some(mut pan_orbit_cam) = pan_orbit_cam {
            pan_orbit_cam.focus = focus_loc;
            pan_orbit_cam.radius = radius;
        }

        let len = hierarchy.selected.len();
        let noun = if len == 1 { "entity" } else { "entities" };
        info!("Focused on {} {}", len, noun);
    }
}

fn initial_camera_setup(
    mut has_decided_initial_cam: Local<bool>,
    mut was_positioned_3d: Local<bool>,
    mut was_positioned_2d: Local<bool>,

    mut commands: Commands,
    mut editor: ResMut<Editor>,

    mut cameras: ParamSet<(
        // 2d pan/zoom
        Query<(Entity, &mut Transform), With<EditorCamera2dPanZoom>>,
        // 3d free
        Query<
            (Entity, &mut Transform, &mut camera_3d_free::FlycamControls),
            With<EditorCamera3dFree>,
        >,
        // 3d pan/orbit
        Query<
            (
                Entity,
                &mut Transform,
                &mut camera_3d_panorbit::PanOrbitCamera,
            ),
            With<EditorCamera3dPanOrbit>,
        >,
        Query<&Transform, With<Camera2d>>,
        Query<&Transform, With<Camera3d>>,
    )>,
) {
    let cam2d = cameras.p3().get_single().ok().cloned();
    let cam3d = cameras.p4().get_single().ok().cloned();

    if !*has_decided_initial_cam {
        let camera_state = editor.window_state_mut::<CameraWindow>().unwrap();

        match (cam2d.is_some(), cam3d.is_some()) {
            (true, false) => {
                camera_state.editor_cam = EditorCamKind::D2PanZoom;
                commands
                    .entity(cameras.p0().single().0)
                    .insert(ActiveEditorCamera);
                *has_decided_initial_cam = true;
            }
            (false, true) => {
                camera_state.editor_cam = EditorCamKind::D3PanOrbit;
                commands
                    .entity(cameras.p2().single().0)
                    .insert(ActiveEditorCamera);
                *has_decided_initial_cam = true;
            }
            (true, true) => {
                camera_state.editor_cam = EditorCamKind::D3PanOrbit;
                commands
                    .entity(cameras.p2().single().0)
                    .insert(ActiveEditorCamera);
                *has_decided_initial_cam = true;
            }
            (false, false) => return,
        }
    }

    if !*was_positioned_2d {
        if let Some(cam2d_transform) = cam2d {
            let mut query = cameras.p0();
            let (_, mut cam_transform) = query.single_mut();
            *cam_transform = cam2d_transform.clone();

            *was_positioned_2d = true;
        }
    }

    if !*was_positioned_3d {
        if let Some(cam3d_transform) = cam3d {
            {
                let mut query = cameras.p1();
                let (_, mut cam_transform, mut cam) = query.single_mut();
                *cam_transform = cam3d_transform.clone();
                let (yaw, pitch, _) = cam3d_transform.rotation.to_euler(EulerRot::YXZ);
                cam.yaw = yaw.to_degrees();
                cam.pitch = pitch.to_degrees();
            }

            {
                let mut query = cameras.p2();
                let (_, mut cam_transform, mut cam) = query.single_mut();
                cam.radius = cam3d_transform.translation.distance(cam.focus);
                *cam_transform = cam3d_transform.clone();
            }

            *was_positioned_3d = true;
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
