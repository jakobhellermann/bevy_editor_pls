use bevy::prelude::*;
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
    meshes_query: Query<
        Entity,
        (
            Without<PickRaycastTarget>,
            Without<NoEditorPicking>,
            With<Handle<Mesh>>,
        ),
    >,
) {
    for entity in meshes_query.iter() {
        commands
            .entity(entity)
            .insert((PickableBundle::default(), PickRaycastTarget::default()));
    }
}
