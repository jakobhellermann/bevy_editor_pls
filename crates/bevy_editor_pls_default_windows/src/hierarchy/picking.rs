use bevy::{prelude::*, render::render_resource::PrimitiveTopology};
use bevy_mod_picking::prelude::{PickRaycastTarget, PickableBundle};

pub struct EditorPickingSet;

/// Prevents the entity from being selectable in the editor window.
#[derive(Component)]
pub struct NoEditorPicking;

pub fn setup(app: &mut App) {
    app.add_plugins(bevy_mod_picking::plugins::DefaultPickingPlugins)
        .add_system(auto_add_editor_picking_set);
}

fn auto_add_editor_picking_set(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    meshes_query: Query<
        (Entity, &Handle<Mesh>),
        (Without<PickRaycastTarget>, Without<NoEditorPicking>),
    >,
) {
    for (entity, handle) in meshes_query.iter() {
        if let Some(mesh) = meshes.get(handle) {
            if let PrimitiveTopology::TriangleList = mesh.primitive_topology() {
                commands
                    .entity(entity)
                    .insert((PickableBundle::default(), PickRaycastTarget::default()));
            }
        }
    }
}
