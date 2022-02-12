use bevy::{
    pbr::wireframe::WireframeConfig,
    render::{options::WgpuOptions, render_resource::WgpuFeatures},
};
use bevy_editor_pls_core::editor_window::EditorWindow;
use bevy_inspector_egui::{egui::Grid, Inspectable};

pub struct DebugSettingsWindowState {
    pub wireframes: bool,
    pub highlight_selected: bool,
}

impl Default for DebugSettingsWindowState {
    fn default() -> Self {
        Self {
            wireframes: false,
            highlight_selected: true,
        }
    }
}

pub struct DebugSettingsWindow;
impl EditorWindow for DebugSettingsWindow {
    type State = DebugSettingsWindowState;
    const NAME: &'static str = "Debug settings";

    fn ui(
        world: &mut bevy::prelude::World,
        mut cx: bevy_editor_pls_core::editor_window::EditorWindowContext,
        ui: &mut bevy_inspector_egui::egui::Ui,
    ) {
        let state = cx.state_mut::<DebugSettingsWindow>().unwrap();

        Grid::new("debug settings").show(ui, |ui| {
            let mut inspect_cx = bevy_inspector_egui::Context::new_shared(None);

            let wireframe_enabled = world
                .get_resource::<WgpuOptions>()
                .map_or(false, |options| {
                    options.features.contains(WgpuFeatures::POLYGON_MODE_LINE)
                });

            if wireframe_enabled {
                ui.label("Wireframes");
            } else {
                ui.label("Wireframes (enable POLYGON_MODE_LINE feature)");
            }
            ui.scope(|ui| {
                ui.set_enabled(wireframe_enabled);
                if state.wireframes.ui(ui, Default::default(), &mut inspect_cx) {
                    world
                        .get_resource_or_insert_with(WireframeConfig::default)
                        .global = state.wireframes;
                }
            });
            ui.end_row();

            if !wireframe_enabled {
                state.highlight_selected = false;
            }

            ui.label("Highlight selected entity");
            ui.scope(|ui| {
                ui.set_enabled(wireframe_enabled);
                ui.checkbox(&mut state.highlight_selected, "");
            });
            ui.end_row();
        });
    }
}
