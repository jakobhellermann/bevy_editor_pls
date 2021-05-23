// use bevy::prelude::*;
// use bevy_input_actionmap::InputMap;
// use bevy_inspector_egui::WorldInspectorParams;

// use crate::{ui::EditorMenuEvent, EditorSettings};

#[derive(Hash, PartialEq, Eq, Clone)]
pub enum EditorAction {
    ToggleWorldInspector,
    ToggleClickToInspect,
    ToggleWireframes,
    ToggleFlycam,
    TogglePerformancePanel,
    ToggleEditorUi,
}

/*
pub(crate) fn action_system(
    input: Res<InputMap<EditorAction>>,
    mut settings: ResMut<EditorSettings>,
    mut editor_events: EventWriter<EditorMenuEvent>,
    mut world_inspector_params: ResMut<WorldInspectorParams>,
) {
    if input.just_active(EditorAction::ToggleClickToInspect) {
        settings.click_to_inspect = !settings.click_to_inspect;
    }
    if input.just_active(EditorAction::ToggleWireframes) {
        settings.show_wireframes = !settings.show_wireframes;
    }
    if input.just_active(EditorAction::ToggleFlycam) {
        settings.fly_camera = !settings.fly_camera;
        if settings.fly_camera {
            settings.orbit_camera = false;
        }
        editor_events.send(EditorMenuEvent::EnableFlyCams(settings.fly_camera));
    }
    if input.just_active(EditorAction::TogglePerformancePanel) {
        settings.performance_panel = !settings.performance_panel;
    }
    if input.just_active(EditorAction::ToggleWorldInspector) {
        world_inspector_params.enabled = !world_inspector_params.enabled;
    }
    if input.just_active(EditorAction::ToggleEditorUi) {
        settings.display_ui = !settings.display_ui;
    }
}
*/
