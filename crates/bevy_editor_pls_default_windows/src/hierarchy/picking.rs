use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy_mod_raycast::{
    DebugCursorMesh, DefaultRaycastingPlugin as RaycastingPlugin, RayCastMethod,
};

pub struct EditorPickingSet;

/// Prevent the entity from being added to the `EditorPickingSet` for entity selection.
#[derive(Component)]
pub struct IgnoreEditorRayCast;

pub type EditorRayCastSource = bevy_mod_raycast::RayCastSource<EditorPickingSet>;
pub type EditorRayCastMesh = bevy_mod_raycast::RayCastMesh<EditorPickingSet>;
pub type EditorRayCastState = bevy_mod_raycast::DefaultPluginState<EditorPickingSet>;
pub type EditorRayCastSystem = bevy_mod_raycast::RaycastSystem<EditorPickingSet>;

pub fn setup(app: &mut App) {
    app.add_plugin(RaycastingPlugin::<EditorPickingSet>::default())
        .insert_resource(EditorRayCastState::default())
        .add_system_set_to_stage(
            CoreStage::First,
            SystemSet::new()
                .with_system(update_raycast_with_cursor)
                .with_system(auto_add_editor_picking_set)
                .before(EditorRayCastSystem::BuildRays),
        );
}

fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<&mut EditorRayCastSource>,
) {
    let cursor_position = match cursor.iter().last() {
        Some(cursor_moved) => cursor_moved.position,
        None => return,
    };

    for mut pick_source in query.iter_mut() {
        pick_source.cast_method = RayCastMethod::Screenspace(cursor_position);
    }
}

fn auto_add_editor_picking_set(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    meshes_query: Query<
        (Entity, &Handle<Mesh>),
        (
            Without<EditorRayCastMesh>,
            Without<IgnoreEditorRayCast>,
            Without<DebugCursorMesh<EditorPickingSet>>,
        ),
    >,
) {
    for (entity, handle) in meshes_query.iter() {
        if let Some(mesh) = meshes.get(handle) {
            if matches!(mesh.primitive_topology(), PrimitiveTopology::TriangleList) {
                commands.entity(entity).insert(EditorRayCastMesh::default());
            }
        }
    }
}
