use bevy::prelude::*;
use bevy_mod_raycast::{
    DebugCursorMesh, DefaultRaycastingPlugin as RaycastingPlugin, RayCastMethod, RaycastSystem,
};

pub struct EditorPickingSet;

pub type EditorRayCastSource = bevy_mod_raycast::RayCastSource<EditorPickingSet>;
pub type EditorRayCastMesh = bevy_mod_raycast::RayCastMesh<EditorPickingSet>;
pub type EditorRayCastState = bevy_mod_raycast::DefaultPluginState<EditorPickingSet>;

pub fn setup(app: &mut App) {
    app.add_plugin(RaycastingPlugin::<EditorPickingSet>::default())
        .insert_resource(EditorRayCastState::default())
        .add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::new()
                .with_system(update_raycast_with_cursor)
                .with_system(auto_add_editor_picking_set)
                .after(RaycastSystem::BuildRays),
        );
}

fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<&mut EditorRayCastSource>,
) {
    for mut pick_source in query.iter_mut() {
        if let Some(cursor_latest) = cursor.iter().last() {
            pick_source.cast_method = RayCastMethod::Screenspace(cursor_latest.position);
        }
    }
}

fn auto_add_editor_picking_set(
    mut commands: Commands,
    meshes: Query<
        Entity,
        (
            With<Handle<Mesh>>,
            Without<EditorRayCastMesh>,
            Without<DebugCursorMesh<EditorPickingSet>>,
        ),
    >,
) {
    for entity in meshes.iter() {
        commands.entity(entity).insert(EditorRayCastMesh::default());
    }
}
