pub mod camera_2d_panzoom;
pub mod camera_3d_free;
pub mod camera_3d_panorbit;
use crate::scenes::NotInScene;

use bevy::render::camera::RenderTarget;
use bevy::render::view::RenderLayers;
use bevy::utils::HashSet;
use bevy::window::WindowRef;
use bevy::{prelude::*, render::primitives::Aabb};
use bevy_editor_pls_core::{
    editor_window::{EditorWindow, EditorWindowContext},
    Editor, EditorEvent,
};
use bevy_inspector_egui::egui;
use transform_gizmo_bevy::GizmoCamera;
// use bevy_mod_picking::prelude::PickRaycastSource;

use crate::hierarchy::{HideInEditor, HierarchyWindow};

use self::camera_3d_panorbit::PanOrbitCamera;

pub const EDITOR_RENDER_LAYER: usize = 19;

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

#[derive(Clone, Copy, PartialEq, Default)]
pub enum EditorCamKind {
    D2PanZoom,
    D3Free,
    #[default]
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

#[derive(Default)]
pub struct CameraWindowState {
    // make sure to keep the `ActiveEditorCamera` marker component in sync with this field
    editor_cam: EditorCamKind,
    pub show_ui: bool,
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
        ui.checkbox(&mut state.show_ui, "UI");
    }

    fn app_setup(app: &mut App) {
        app.init_resource::<PreviouslyActiveCameras>();

        app.add_plugins(camera_2d_panzoom::PanCamPlugin)
            .add_plugins(camera_3d_free::FlycamPlugin)
            .add_plugins(camera_3d_panorbit::PanOrbitCameraPlugin)
            .add_systems(
                Update,
                set_editor_cam_active
                    .before(camera_3d_panorbit::CameraSystem::EditorCam3dPanOrbit)
                    .before(camera_3d_free::CameraSystem::EditorCam3dFree)
                    .before(camera_2d_panzoom::CameraSystem::EditorCam2dPanZoom),
            )
            .add_systems(PreUpdate, toggle_editor_cam)
            .add_systems(PreUpdate, focus_selected)
            .add_systems(Update, initial_camera_setup);
        app.add_systems(PreStartup, spawn_editor_cameras);

        app.add_systems(
            PostUpdate,
            set_main_pass_viewport
                .after(bevy_editor_pls_core::EditorSet::UI)
                .before(bevy::render::camera::CameraUpdateSystem),
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

fn spawn_editor_cameras(mut commands: Commands, editor: Res<Editor>) {
    #[derive(Component, Default)]
    struct Ec2d;
    #[derive(Component, Default)]
    struct Ec3d;

    info!("Spawning editor cameras");

    let render_layers = RenderLayers::default().with(EDITOR_RENDER_LAYER);

    let editor_cam_priority = 100;

    let target = RenderTarget::Window(WindowRef::Entity(editor.window()));

    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: editor_cam_priority,
                is_active: false,
                target: target.clone(),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 2.0, 5.0),
            ..Camera3dBundle::default()
        },
        Ec3d,
        camera_3d_free::FlycamControls::default(),
        EditorCamera,
        EditorCamera3dFree,
        HideInEditor,
        Name::new("Editor Camera 3D Free"),
        NotInScene,
        GizmoCamera,
        render_layers.clone(),
    ));

    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                //  Prevent multiple cameras from having the same priority.
                order: editor_cam_priority + 1,
                target: target.clone(),
                is_active: false,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 2.0, 5.0),
            ..Camera3dBundle::default()
        },
        Ec3d,
        PanOrbitCamera::default(),
        EditorCamera,
        EditorCamera3dPanOrbit,
        HideInEditor,
        Name::new("Editor Camera 3D Pan/Orbit"),
        NotInScene,
        GizmoCamera,
        render_layers.clone(),
    ));

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                //  Prevent multiple cameras from having the same priority.
                order: editor_cam_priority + 2,
                target,
                is_active: false,
                ..default()
            },
            ..default()
        },
        Ec2d,
        camera_2d_panzoom::PanCamControls::default(),
        EditorCamera,
        EditorCamera2dPanZoom,
        HideInEditor,
        Name::new("Editor Camera 2D Pan/Zoom"),
        NotInScene,
        GizmoCamera,
        render_layers,
    ));
}

fn set_editor_cam_active(
    editor: Res<Editor>,

    mut editor_cameras: ParamSet<(
        Query<(&mut Camera, &mut camera_3d_free::FlycamControls)>,
        Query<(&mut Camera, &mut camera_3d_panorbit::PanOrbitCamera)>,
        Query<(&mut Camera, &mut camera_2d_panzoom::PanCamControls)>,
    )>,
    // mut ui_camera_settings: Query<&mut UiCameraConfig, With<EditorCamera>>,
) {
    let camera_window_state = &editor.window_state::<CameraWindow>().unwrap();
    let editor_cam = camera_window_state.editor_cam;

    // if editor.active() {
    // ui_camera_settings
    // .for_each_mut(|mut settings| settings.show_ui = camera_window_state.show_ui);
    // }

    {
        let mut q = editor_cameras.p0();
        let mut editor_cam_3d_free = q.single_mut();
        let active = matches!(editor_cam, EditorCamKind::D3Free) && editor.active();
        editor_cam_3d_free.0.is_active = active;
        editor_cam_3d_free.1.enable_movement = active && !editor.listening_for_text();
        editor_cam_3d_free.1.enable_look = active && editor.viewport_interaction_active();
    }
    {
        let mut q = editor_cameras.p1();
        let mut editor_cam_3d_panorbit = q.single_mut();
        let active = matches!(editor_cam, EditorCamKind::D3PanOrbit) && editor.active();
        editor_cam_3d_panorbit.0.is_active = active;
        editor_cam_3d_panorbit.1.enabled = active && editor.viewport_interaction_active();
    }
    {
        let mut q = editor_cameras.p2();
        let mut editor_cam_2d_panzoom = q.single_mut();
        let active = matches!(editor_cam, EditorCamKind::D2PanZoom) && editor.active();
        editor_cam_2d_panzoom.0.is_active = active;
        editor_cam_2d_panzoom.1.enabled = active && editor.viewport_interaction_active();
    }
}

#[derive(Resource, Default)]
struct PreviouslyActiveCameras(HashSet<Entity>);

fn toggle_editor_cam(
    editor: Res<Editor>,
    mut editor_events: EventReader<EditorEvent>,
    mut prev_active_cams: ResMut<PreviouslyActiveCameras>,
    mut cam_query: Query<(Entity, &mut Camera)>,
) {
    if editor.always_active() {
        //Prevent accumulation of irrelevant events
        editor_events.clear();
        return;
    }

    for event in editor_events.read() {
        if let EditorEvent::Toggle { now_active } = *event {
            if now_active {
                // Add all currently active cameras
                for (e, mut cam) in cam_query
                    .iter_mut()
                    //  Ignore non-Window render targets
                    .filter(|(_e, c)| matches!(c.target, RenderTarget::Window(_)))
                    .filter(|(_e, c)| c.is_active)
                {
                    prev_active_cams.0.insert(e);
                    cam.is_active = false;
                }
            } else {
                for cam in prev_active_cams.0.iter() {
                    if let Ok((_e, mut camera)) = cam_query.get_mut(*cam) {
                        camera.is_active = true;
                    }
                }
                prev_active_cams.0.clear();
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
    window: Query<&Window>,
) {
    let Ok(window) = window.get(editor.window()) else {
        //Prevent accumulation of irrelevant events
        editor_events.clear();
        return;
    };

    for event in editor_events.read() {
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

            ortho.scale = radius / window.width().min(window.height()).max(1.0);
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
        Query<&Transform, (With<Camera2d>, Without<EditorCamera>)>,
        Query<&Transform, (With<Camera3d>, Without<EditorCamera>)>,
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
            if !cam2d_transform.rotation.is_finite()
                || !cam2d_transform.translation.is_finite()
                || !cam2d_transform.scale.is_finite()
            {
                return;
            };

            let mut query = cameras.p0();
            let (_, mut cam_transform) = query.single_mut();
            *cam_transform = cam2d_transform;

            *was_positioned_2d = true;
        }
    }

    if !*was_positioned_3d {
        if let Some(cam3d_transform) = cam3d {
            if !cam3d_transform.rotation.is_finite()
                || !cam3d_transform.translation.is_finite()
                || !cam3d_transform.scale.is_finite()
            {
                return;
            };

            {
                let mut query = cameras.p1();
                let (_, mut cam_transform, mut cam) = query.single_mut();
                *cam_transform = cam3d_transform;
                let (yaw, pitch, _) = cam3d_transform.rotation.to_euler(EulerRot::YXZ);
                cam.yaw = yaw;
                cam.pitch = pitch;
            }

            {
                let mut query = cameras.p2();
                let (_, mut cam_transform, mut cam) = query.single_mut();
                cam.radius = cam3d_transform.translation.distance(cam.focus);
                *cam_transform = cam3d_transform;
            }

            *was_positioned_3d = true;
        }
    }
}

fn set_main_pass_viewport(
    editor: Res<Editor>,
    window: Query<(&bevy_inspector_egui::bevy_egui::EguiSettings, &Window)>,
    mut cameras: Query<&mut Camera, With<EditorCamera>>,
) {
    if !editor.is_changed() {
        return;
    };

    let Ok((egui_settings, window)) = window.get(editor.window()) else {
        return;
    };

    let viewport = editor.active().then(|| {
        let scale_factor = window.scale_factor() * egui_settings.scale_factor;

        let mut viewport_pos = editor.viewport().left_top().to_vec2() * scale_factor;
        let mut viewport_size = editor.viewport().size() * scale_factor;

        if !viewport_size.is_finite() {
            warn!("editor viewport size is infinite");
            viewport_pos = egui::Vec2::ZERO;
            viewport_size = egui::Vec2::new(640., 480.);
        }

        bevy::render::camera::Viewport {
            physical_position: UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32),
            physical_size: UVec2::new(
                (viewport_size.x as u32).max(1),
                (viewport_size.y as u32).max(1),
            ),
            depth: 0.0..1.0,
        }
    });

    cameras
        .iter_mut()
        .for_each(|mut cam| cam.viewport.clone_from(&viewport));
}
