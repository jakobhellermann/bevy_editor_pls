use bevy::prelude::*;
use bevy::render::wireframe::WireframeConfig;

use bevy_fly_camera::FlyCameraPlugin;
use bevy_pancam::PanCamPlugin;
// use bevy_input_actionmap::ActionPlugin;
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_mod_picking::{InteractablePickingPlugin, PickingPlugin, PickingPluginState, PickingSystem};

use crate::{drag_and_drop, systems, ui, EditorSettings};

/// See the [crate-level docs](index.html) for usage
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        // bevy-inspector-egui
        app.world.get_resource_or_insert_with(|| WorldInspectorParams {
            enabled: false,
            despawnable_entities: true,
            ..Default::default()
        });
        app.add_plugin(WorldInspectorPlugin::new());

        // bevy_mod_picking
        if !app.world.contains_resource::<PickingPluginState>() {
            app.add_plugin(PickingPlugin).add_plugin(InteractablePickingPlugin);
        };

        // bevy_mod_flycamera
        app.add_plugin(FlyCameraPlugin);

        // bevy_pancam
        app.add_plugin(PanCamPlugin);

        // bevy_orbit_controls
        app.add_plugin(bevy_orbit_controls::OrbitCameraPlugin);

        // bevy_input_actionmap
        // app.add_plugin(ActionPlugin::<EditorAction>::default());

        // resources
        app.init_resource::<EditorState>().add_event::<ui::EditorMenuEvent>();

        let editor_settings = app.world.get_resource_or_insert_with(EditorSettings::default);
        let show_wireframes = editor_settings.show_wireframes;

        let add_gizmo_plugin =
            editor_settings.add_gizmo_plugin || editor_settings.auto_gizmo_target || editor_settings.auto_gizmo_camera;

        if add_gizmo_plugin {
            app.add_plugin(bevy_transform_gizmo::TransformGizmoPlugin);
        }

        if app.world.contains_resource::<WireframeConfig>() {
            app.world.get_resource_mut::<WireframeConfig>().unwrap().global = show_wireframes;
        }

        // systems
        app.add_system(ui::menu_system.exclusive_system());
        app.add_system(ui::currently_inspected_system.exclusive_system());
        app.add_system(ui::handle_menu_event.system());
        app.add_system(ui::performance_panel.system());

        app.add_system(drag_and_drop::drag_and_drop_system.exclusive_system());

        // auto add systems
        app.add_system(systems::make_everything_pickable.system());
        app.add_system(systems::make_camera_picksource.system());
        app.add_system(systems::make_cam_flycam.system());
        app.add_system(systems::make_cam_pancam.system());

        app.add_system_to_stage(
            CoreStage::PostUpdate,
            systems::maintain_inspected_entities.system().after(PickingSystem::Focus),
        );

        // app.add_system(crate::action::action_system.system());
    }
}

pub struct EditorState {
    pub currently_inspected: Option<Entity>,
}

impl Default for EditorState {
    fn default() -> Self {
        EditorState {
            currently_inspected: None,
        }
    }
}
