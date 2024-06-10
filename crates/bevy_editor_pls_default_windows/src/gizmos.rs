use bevy::{
    ecs::{query::QueryFilter, system::RunSystemOnce},
    prelude::*,
    render::{camera::CameraProjection, view::RenderLayers},
};

use bevy_editor_pls_core::editor_window::{EditorWindow, EditorWindowContext};
use bevy_inspector_egui::{bevy_inspector::hierarchy::SelectedEntities, egui};
use transform_gizmo_bevy::{EnumSet, GizmoMode};
use transform_gizmo_bevy::GizmoTarget;

use crate::{
    cameras::{ActiveEditorCamera, CameraWindow, EditorCamera, EDITOR_RENDER_LAYER},
    hierarchy::HierarchyWindow,
};

pub struct GizmoState {
    pub camera_gizmo_active: bool,
		/// TODO: Take these settings into account
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
                    trace!("Hydrating a gizmo on entity {:?} because it is selected", entity);
                    // TODO: Maybe change the exact gizmo target instance instead of using default? should this load from some config?
                    commands.entity(entity).insert(GizmoTarget::default());
                }
            }

            /// Takes all entities that should have their [GizmoTarget] removed because they are no longer selected.
            fn deconstruct_gizmos(
                mut commands: Commands,
                entities: Query<Entity, (With<GizmoTarget>, Without<EntityShouldShowGizmo>)>,
            ) {
                for entity in entities.iter() {
                    commands.entity(entity).remove::<GizmoTarget>();
                    debug!(
                        "Removing GizmoTarget from entity {:?} because it has lost focus",
                        entity
                    );
                }
            }

            if let Some(hierarchy_state) = cx.state::<HierarchyWindow>() {
                // here should assign the `EntityShouldShowGizmo` component, which is later synced
                // with the actual gizmo ui system

                world.run_system_once(reset_gizmos_selected_state);

                let selected_entities = hierarchy_state.selected.iter();
                for entity in selected_entities {
                    world.entity_mut(entity).insert(EntityShouldShowGizmo);
                }

                world.run_system_once(hydrate_gizmos);
                world.run_system_once(deconstruct_gizmos);
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
        let sphere = meshes.add(Sphere { radius: 0.3 });

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

// fn draw_gizmo(
//     ui: &mut egui::Ui,
//     world: &mut World,
//     selected_entities: &SelectedEntities,
//     gizmo_mode: GizmoMode,
// ) {
// 		for entity in selected_entities.iter() {
// 			world.entity_mut(entity).insert(transform_gizmo_bevy::GizmoTarget::default());
// 			info!("Inserted GizmoTarget to entity: {:?}", entity);
// 		}
//     // let Ok((cam_transform, projection)) = world
//     //     .query_filtered::<(&GlobalTransform, &Projection), With<ActiveEditorCamera>>()
//     //     .get_single(world)
//     // else {
//     //     return;
//     // };
//     // let view_matrix = Mat4::from(cam_transform.affine().inverse());
//     // let projection_matrix = projection.get_projection_matrix();

//     // if selected_entities.len() != 1 {
//     //     return;
//     // }

//     // for selected in selected_entities.iter() {
//     //     // let Some(global_transform) = world.get::<GlobalTransform>(selected) else {
//     //     //     continue;
//     //     // };
//     //     // let model_matrix = global_transform.compute_matrix();

//     //     // let Some(result) = transform_gizmo_bevy::Gizmo::new(selected)
//     //     //     .model_matrix(model_matrix.into())
//     //     //     .view_matrix(view_matrix.into())
//     //     //     .projection_matrix(projection_matrix.into())
//     //     //     .orientation(transform_gizmo_bevy::GizmoOrientation::Local)
//     //     //     .mode(gizmo_mode)
//     //     //     .interact(ui)
//     //     // else {
//     //     //     continue;
//     //     // };

//     //     // let global_affine = global_transform.affine();

//     //     // let mut transform = world.get_mut::<Transform>(selected).unwrap();

//     //     // let parent_affine = global_affine * transform.compute_affine().inverse();
//     //     // let inverse_parent_transform = GlobalTransform::from(parent_affine.inverse());

//     //     // let global_transform = Transform {
//     //     //     translation: result.translation.into(),
//     //     //     rotation: result.rotation.into(),
//     //     //     scale: result.scale.into(),
//     //     // };

//     //     // *transform = (inverse_parent_transform * global_transform).into();
//     // }
// }
