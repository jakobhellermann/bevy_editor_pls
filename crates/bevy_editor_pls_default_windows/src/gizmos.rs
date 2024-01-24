use bevy::{
    ecs::query::ReadOnlyWorldQuery,
    prelude::*,
    render::{camera::CameraProjection, view::RenderLayers},
};

use bevy_editor_pls_core::editor_window::{EditorWindow, EditorWindowContext};
use bevy_inspector_egui::{bevy_inspector::hierarchy::SelectedEntities, egui};
use egui_gizmo::GizmoMode;

use crate::{
    cameras::{ActiveEditorCamera, CameraWindow, EditorCamera, EDITOR_RENDER_LAYER},
    hierarchy::HierarchyWindow,
};

pub struct GizmoState {
    pub camera_gizmo_active: bool,
    pub gizmo_mode: GizmoMode,
}

impl Default for GizmoState {
    fn default() -> Self {
        Self {
            camera_gizmo_active: true,
            gizmo_mode: GizmoMode::Translate,
        }
    }
}

pub struct GizmoWindow;

impl EditorWindow for GizmoWindow {
    type State = GizmoState;

    const NAME: &'static str = "Gizmos";

    fn ui(_world: &mut World, _cx: EditorWindowContext, ui: &mut egui::Ui) {
        ui.label("Gizmos can currently not be configured");
    }

    fn viewport_toolbar_ui(world: &mut World, cx: EditorWindowContext, ui: &mut egui::Ui) {
        let gizmo_state = cx.state::<GizmoWindow>().unwrap();

        if gizmo_state.camera_gizmo_active {
            if let (Some(hierarchy_state), Some(_camera_state)) =
                (cx.state::<HierarchyWindow>(), cx.state::<CameraWindow>())
            {
                draw_gizmo(ui, world, &hierarchy_state.selected, gizmo_state.gizmo_mode);
            }
        }
    }

    fn app_setup(app: &mut App) {
        let mut materials = app.world.resource_mut::<Assets<StandardMaterial>>();
        let material_light = materials.add(StandardMaterial {
            base_color: Color::rgba_u8(222, 208, 103, 255),
            unlit: true,
            fog_enabled: false,
            alpha_mode: AlphaMode::Add,
            ..default()
        });
        let material_camera = materials.add(StandardMaterial {
            base_color: Color::rgb(1.0, 1.0, 1.0),
            unlit: true,
            fog_enabled: false,
            alpha_mode: AlphaMode::Multiply,
            ..default()
        });

        let mut meshes = app.world.resource_mut::<Assets<Mesh>>();
        let sphere = meshes.add(
            shape::UVSphere {
                radius: 0.3,
                sectors: 16,
                stacks: 16,
            }
            .into(),
        );

        app.world.insert_resource(GizmoMarkerConfig {
            point_light_mesh: sphere.clone(),
            point_light_material: material_light.clone(),
            directional_light_mesh: sphere.clone(),
            directional_light_material: material_light,
            camera_mesh: sphere,
            camera_material: material_camera,
        });

        app.add_systems(PostUpdate, add_gizmo_markers);
    }
}

#[derive(Resource)]
struct GizmoMarkerConfig {
    point_light_mesh: Handle<Mesh>,
    point_light_material: Handle<StandardMaterial>,
    directional_light_mesh: Handle<Mesh>,
    directional_light_material: Handle<StandardMaterial>,
    camera_mesh: Handle<Mesh>,
    camera_material: Handle<StandardMaterial>,
}

#[derive(Component)]
struct HasGizmoMarker;

type GizmoMarkerQuery<'w, 's, T, F = ()> =
    Query<'w, 's, Entity, (With<T>, Without<HasGizmoMarker>, F)>;

fn add_gizmo_markers(
    mut commands: Commands,
    gizmo_marker_meshes: Res<GizmoMarkerConfig>,

    point_lights: GizmoMarkerQuery<PointLight>,
    directional_lights: GizmoMarkerQuery<DirectionalLight>,
    cameras: GizmoMarkerQuery<Camera, Without<EditorCamera>>,
) {
    fn add<T: Component, F: ReadOnlyWorldQuery, B: Bundle>(
        commands: &mut Commands,
        query: GizmoMarkerQuery<T, F>,
        name: &'static str,
        f: impl Fn() -> B,
    ) {
        let render_layers = RenderLayers::layer(EDITOR_RENDER_LAYER);
        for entity in &query {
            commands
                .entity(entity)
                .insert(HasGizmoMarker)
                .with_children(|commands| {
                    commands.spawn((f(), render_layers, Name::new(name)));
                });
        }
    }

    add(&mut commands, point_lights, "PointLight Gizmo", || {
        PbrBundle {
            mesh: gizmo_marker_meshes.point_light_mesh.clone_weak(),
            material: gizmo_marker_meshes.point_light_material.clone_weak(),
            ..default()
        }
    });
    add(
        &mut commands,
        directional_lights,
        "DirectionalLight Gizmo",
        || PbrBundle {
            mesh: gizmo_marker_meshes.directional_light_mesh.clone_weak(),
            material: gizmo_marker_meshes.directional_light_material.clone_weak(),
            ..default()
        },
    );

    let render_layers = RenderLayers::layer(EDITOR_RENDER_LAYER);
    for entity in &cameras {
        commands
            .entity(entity)
            .insert((
                HasGizmoMarker,
                Visibility::Visible,
                InheritedVisibility::VISIBLE,
                ViewVisibility::default(),
            ))
            .with_children(|commands| {
                commands.spawn((
                    PbrBundle {
                        mesh: gizmo_marker_meshes.camera_mesh.clone_weak(),
                        material: gizmo_marker_meshes.camera_material.clone_weak(),
                        ..default()
                    },
                    render_layers,
                    Name::new("Camera Gizmo"),
                ));
            });
    }
}

fn draw_gizmo(
    ui: &mut egui::Ui,
    world: &mut World,
    selected_entities: &SelectedEntities,
    gizmo_mode: GizmoMode,
) {
    let Ok((cam_transform, projection)) = world
        .query_filtered::<(&GlobalTransform, &Projection), With<ActiveEditorCamera>>()
        .get_single(world)
    else {
        return;
    };
    let view_matrix = Mat4::from(cam_transform.affine().inverse());
    let projection_matrix = projection.get_projection_matrix();

    if selected_entities.len() != 1 {
        return;
    }

    for selected in selected_entities.iter() {
        let Some(transform) = world.get::<GlobalTransform>(selected) else {
            continue;
        };
        let model_matrix = transform.compute_matrix();

        let Some(result) = egui_gizmo::Gizmo::new(selected)
            .model_matrix(model_matrix.into())
            .view_matrix(view_matrix.into())
            .projection_matrix(projection_matrix.into())
            .orientation(egui_gizmo::GizmoOrientation::Local)
            .mode(gizmo_mode)
            .interact(ui)
        else {
            continue;
        };

        let mut transform = world.get_mut::<Transform>(selected).unwrap();
        *transform = Transform {
            translation: Vec3::from(<[f32; 3]>::from(result.translation)),
            rotation: Quat::from_array(<[f32; 4]>::from(result.rotation)),
            scale: Vec3::from(<[f32; 3]>::from(result.scale)),
        };
    }
}
