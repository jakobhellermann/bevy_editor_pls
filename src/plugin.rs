use bevy::prelude::*;
use bevy::render::wireframe::WireframeConfig;

use bevy_fly_camera::FlyCameraPlugin;
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_mod_picking::{InteractablePickingPlugin, PickingPlugin, PickingPluginState, PickingSystem};

use crate::{drag_and_drop, systems, ui, EditorSettings};

/// See the [crate-level docs](index.html) for usage
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // bevy-inspector-egui
        app.world_mut().get_resource_or_insert_with(|| WorldInspectorParams {
            enabled: false,
            despawnable_entities: true,
            ..Default::default()
        });
        app.add_plugin(WorldInspectorPlugin::new());

        // bevy_mod_picking
        if !app.world().contains_resource::<PickingPluginState>() {
            app.add_plugin(PickingPlugin).add_plugin(InteractablePickingPlugin);
        };

        // bevy_mod_flycamera
        app.add_plugin(FlyCameraPlugin);
        app.add_system(systems::esc_cursor_grab.system());

        // resources
        app.init_resource::<EditorState>().add_event::<ui::EditorMenuEvent>();

        let show_wireframes = app
            .world_mut()
            .get_resource_or_insert_with(EditorSettings::default)
            .show_wireframes;
        if app.world().contains_resource::<WireframeConfig>() {
            app.world_mut().get_resource_mut::<WireframeConfig>().unwrap().global = show_wireframes;
        }

        // systems
        app.add_system(ui::menu_system.exclusive_system());
        app.add_system(ui::currently_inspected_system.exclusive_system());
        app.add_system(ui::handle_menu_event.system());

        app.add_system(drag_and_drop::drag_and_drop_system.exclusive_system());

        // auto add systems
        app.add_system(systems::make_everything_pickable.system());
        app.add_system(systems::make_camera_picksource.system());
        app.add_system(systems::make_cam_flycam.system());

        app.add_system_to_stage(
            CoreStage::PostUpdate,
            systems::maintain_inspected_entities.system().after(PickingSystem::Focus),
        );
    }
}

#[derive(Default)]
pub struct EditorState {
    pub currently_inspected: Option<Entity>,
}
