use bevy::{
    diagnostic::{Diagnostic, Diagnostics, FrameTimeDiagnosticsPlugin},
    pbr::wireframe::WireframeConfig,
    prelude::{App, Res, ResMut},
    render::{options::WgpuOptions, render_resource::WgpuFeatures},
};
use bevy_editor_pls_core::{editor_window::EditorWindow, Editor};
use bevy_inspector_egui::{bevy_egui::EguiContext, egui, egui::Grid, Inspectable};

#[derive(Default)]
pub struct DebugSettingsWindowState {
    pub wireframes: bool,
    pub performance_ui: bool,
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

            ui.label("Performace window");
            if state
                .performance_ui
                .ui(ui, Default::default(), &mut inspect_cx)
            {}
            ui.end_row();
        });
    }

    fn app_setup(app: &mut App) {
        app.add_system(performance_panel_ui);
    }
}

fn performance_panel_ui(
    mut editor: ResMut<Editor>,
    egui_context: Res<EguiContext>,
    diagnostics: Res<Diagnostics>,
) {
    let state = editor.window_state::<DebugSettingsWindow>().unwrap();
    if !state.performance_ui {
        return;
    }

    let diagnostics = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .and_then(Diagnostic::average)
        .zip(
            diagnostics
                .get(FrameTimeDiagnosticsPlugin::FRAME_TIME)
                .and_then(Diagnostic::average),
        );

    let mut open = true;

    egui::Window::new("Performance")
        .open(&mut open)
        .resizable(false)
        .show(egui_context.ctx(), |ui| {
            let (fps, frame_time) = match diagnostics {
                Some(value) => value,
                None => {
                    ui.label(
                        "Add the `FrameTimeDiagnosticsPlugin` to see the performance editor panel.",
                    );
                    return;
                }
            };

            egui::Grid::new("frame time diagnostics").show(ui, |ui| {
                ui.label("FPS");
                ui.label(format!("{:.2}", fps));
                ui.end_row();
                ui.label("Frame Time");
                ui.label(format!("{:.4}", frame_time));
                ui.end_row();
            });
        });

    if !open {
        editor
            .window_state_mut::<DebugSettingsWindow>()
            .unwrap()
            .performance_ui = false;
    }
}
