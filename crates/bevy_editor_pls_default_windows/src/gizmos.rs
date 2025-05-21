use bevy::{
    ecs::{query::QueryFilter, system::RunSystemOnce},
    prelude::*,
    render::view::RenderLayers,
};

use bevy_editor_pls_core::editor_window::{EditorWindow, EditorWindowContext};
use bevy_inspector_egui::egui;
use transform_gizmo_bevy::GizmoTarget;
use transform_gizmo_bevy::{EnumSet, GizmoMode};

use crate::{
    cameras::{EditorCamera, EDITOR_RENDER_LAYER},
    hierarchy::HierarchyWindow,
};

pub struct GizmoState {
    /// If [false], doesn't show any gizmos
    pub camera_gizmo_active: bool,
    /// Synced with the [transform_gizmo_bevy::GizmoOptions] resource
    pub gizmo_modes: EnumSet<GizmoMode>,
}

impl Default for GizmoState {
    fn default() -> Self {
        Self {
            camera_gizmo_active: true,
            gizmo_modes: GizmoMode::all_translate(),
        }
    }
}

pub struct GizmoWindow;

impl EditorWindow for GizmoWindow {
    type State = GizmoState;

    const NAME: &'static str = "Gizmos";

    fn ui(_world: &mut World, _cx: EditorWindowContext, ui: &mut egui::Ui) {
        ui.label("Gizmos can currently not be configured");
        // could definitely change some settings here in the future
    }

    /// Called every frame (hopefully), could this invariant (namely being called every frame) be documented,
    /// ideally in the [EditorWindow] trait?
    fn viewport_toolbar_ui(world: &mut World, cx: EditorWindowContext, _ui: &mut egui::Ui) {
        let gizmo_state = cx.state::<GizmoWindow>().unwrap();

        // syncs the [GizmoOptions] resource with the current state of the gizmo window
        let mut gizmo_options = world.resource_mut::<transform_gizmo_bevy::GizmoOptions>();
        gizmo_options.gizmo_modes = gizmo_state.gizmo_modes;

        if gizmo_state.camera_gizmo_active {
            /// Before [hydrate_gizmos] and [deconstruct_gizmos] are run, this system resets the state of all entities that have a [EntityShouldShowGizmo] component.
            /// Then, according to selection logic some entities are marked as focussed, and [hydrate_gizmos] and [deconstruct_gizmos] is run to sync the gizmo state with the selection state.
            fn reset_gizmos_selected_state(
                mut commands: Commands,
                entities: Query<Entity, With<EntityShouldShowGizmo>>,
            ) {
                for entity in entities.iter() {
                    commands.entity(entity).remove::<EntityShouldShowGizmo>();
                }
            }

            /// Takes all entities marked with [EntityShouldShowGizmo] and adds the [GizmoTarget] component to them.
            fn hydrate_gizmos(
                mut commands: Commands,
                entities: Query<Entity, (With<EntityShouldShowGizmo>, Without<GizmoTarget>)>,
            ) {
                for entity in entities.iter() {
                    if let Ok(mut entity) = commands.get_entity(entity) {
                        trace!(
                            "Hydrating a gizmo on entity {:?} because it is selected",
                            entity.id()
                        );
                        // implicitly assumes it is the only gizmo target in the world,
                        // otherwise setting [GizmoTarget].is_focussed may be necessary
                        entity.insert(GizmoTarget::default());
                    }
                }
            }

            /// Takes all entities that should have their [GizmoTarget] removed because they are no longer selected.
            fn deconstruct_gizmos(
                mut commands: Commands,
                entities: Query<Entity, (With<GizmoTarget>, Without<EntityShouldShowGizmo>)>,
            ) {
                for entity in entities.iter() {
                    if let Ok(mut entity) = commands.get_entity(entity) {
                        entity.remove::<GizmoTarget>();
                        debug!(
                            "Removing GizmoTarget from entity {:?} because it has lost focus",
                            entity.id()
                        );
                    }
                }
            }

            if let Some(hierarchy_state) = cx.state::<HierarchyWindow>() {
                // here should assign the `EntityShouldShowGizmo` component, which is later synced
                // with the actual gizmo ui system

                // todo: not ignore errors
                world.run_system_once(reset_gizmos_selected_state).ok();

                let selected_entities = hierarchy_state.selected.iter();
                for entity in selected_entities {
                    if let Ok(mut entity) = world.get_entity_mut(entity) {
                        entity.insert(EntityShouldShowGizmo);
                    }
                }

                world.run_system_once(hydrate_gizmos).ok();
                world.run_system_once(deconstruct_gizmos).ok();
            }
        }
    }

    fn app_setup(app: &mut App) {
        let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
        let material_light = materials.add(StandardMaterial {
            base_color: Color::srgba_u8(222, 208, 103, 255),
            unlit: true,
            fog_enabled: false,
            alpha_mode: AlphaMode::Add,
            ..default()
        });
        let material_camera = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 1.0, 1.0),
            unlit: true,
            fog_enabled: false,
            alpha_mode: AlphaMode::Multiply,
            ..default()
        });

        let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
        let sphere = meshes.add(Sphere { radius: 0.3 });

        app.world_mut().insert_resource(GizmoMarkerConfig {
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

/// can somebody document what this does? is it a duplicate of [EntityShouldShowGizmo]?
#[derive(Component)]
struct HasGizmoMarker;

/// When on an entity, this entity should be controllable using some sort of user gizmo.
/// Currently uses [transform_gizmo_bevy], and puts the [GizmoTarget] on the entity.
#[derive(Component)]
struct EntityShouldShowGizmo;

type GizmoMarkerQuery<'w, 's, T, F = ()> =
    Query<'w, 's, Entity, (With<T>, Without<HasGizmoMarker>, F)>;

fn add_gizmo_markers(
    mut commands: Commands,
    gizmo_marker_meshes: Res<GizmoMarkerConfig>,

    point_lights: GizmoMarkerQuery<PointLight>,
    directional_lights: GizmoMarkerQuery<DirectionalLight>,
    cameras: GizmoMarkerQuery<Camera, Without<EditorCamera>>,
) {
    fn add<T: Component, F: QueryFilter, B: Bundle>(
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
                    commands.spawn((f(), render_layers.clone(), Name::new(name)));
                });
        }
    }

    add(&mut commands, point_lights, "PointLight Gizmo", || {
        (
            Mesh3d(gizmo_marker_meshes.point_light_mesh.clone_weak()),
            MeshMaterial3d(gizmo_marker_meshes.point_light_material.clone_weak()),
        )
    });
    add(
        &mut commands,
        directional_lights,
        "DirectionalLight Gizmo",
        || {
            (
                Mesh3d(gizmo_marker_meshes.directional_light_mesh.clone_weak()),
                MeshMaterial3d(gizmo_marker_meshes.directional_light_material.clone_weak()),
            )
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
                    Mesh3d(gizmo_marker_meshes.camera_mesh.clone_weak()),
                    MeshMaterial3d(gizmo_marker_meshes.camera_material.clone_weak()),
                    render_layers.clone(),
                    Name::new("Camera Gizmo"),
                ));
            });
    }
}
