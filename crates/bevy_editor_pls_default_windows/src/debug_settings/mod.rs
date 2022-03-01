pub mod debugdump;
pub mod fake_time;

use bevy::{
    pbr::wireframe::WireframeConfig,
    prelude::*,
    render::{render_resource::WgpuFeatures, options::WgpuSettings},
};
use bevy_editor_pls_core::editor_window::EditorWindow;
use bevy_inspector_egui::{
    egui::{self, Grid},
    Inspectable,
};

pub struct DebugSettingsWindowState {
    pub pause_time: bool,
    pub wireframes: bool,
    pub highlight_selected: bool,

    open_debugdump_status: Option<DebugdumpError>,
}

enum DebugdumpError {
    DotNotFound,
    OpenError(opener::OpenError),
    IO(std::io::Error),
}

impl Default for DebugSettingsWindowState {
    fn default() -> Self {
        Self {
            pause_time: false,
            wireframes: false,
            highlight_selected: true,

            open_debugdump_status: None,
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
        ui: &mut egui::Ui,
    ) {
        let state = cx.state_mut::<DebugSettingsWindow>().unwrap();
        debug_ui(world, state, ui);
    }

    fn app_setup(app: &mut App) {
        fake_time::setup(app);
        debugdump::setup(app);
    }
}

fn debug_ui(world: &mut World, state: &mut DebugSettingsWindowState, ui: &mut egui::Ui) {
    let available_size = ui.available_size();
    let horizontal = available_size.x > available_size.y;

    horizontal_if(ui, horizontal, |ui| {
        debug_ui_options(world, state, ui);

        if !horizontal {
            ui.separator();
        }

        debug_ui_debugdump(world, state, ui);
    });
}

pub fn horizontal_if<R>(
    ui: &mut egui::Ui,
    horizontal: bool,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    if horizontal {
        ui.horizontal(add_contents).inner
    } else {
        add_contents(ui)
    }
}

fn debug_ui_options(world: &mut World, state: &mut DebugSettingsWindowState, ui: &mut egui::Ui) {
    Grid::new("debug settings").show(ui, |ui| {
        let mut inspect_cx = bevy_inspector_egui::Context::new_shared(None);

        ui.label("Pause time");
        ui.checkbox(&mut state.pause_time, "");
        ui.end_row();

        let wireframe_enabled = world
            .get_resource::<WgpuSettings>()
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

fn debug_ui_debugdump(world: &mut World, state: &mut DebugSettingsWindowState, ui: &mut egui::Ui) {
    let open_dot = |dot: &str, path: &str| -> Result<(), DebugdumpError> {
        let format = "svg";
        let rendered = match debugdump::execute_dot(dot, format) {
            Ok(rendered) => rendered,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(DebugdumpError::DotNotFound)
            }
            Err(e) => return Err(DebugdumpError::IO(e)),
        };
        let path = std::env::temp_dir().join(path).with_extension(format);
        std::fs::write(&path, &rendered).map_err(DebugdumpError::IO)?;
        opener::open(path).map_err(|e| DebugdumpError::OpenError(e))?;
        Ok(())
    };

    ui.vertical(|ui| {
        if ui.button("Open schedule").clicked() {
            let schedule_graph = world.get_resource::<debugdump::DotGraphs>().unwrap();
            if let Err(e) = open_dot(&schedule_graph.schedule_graph, "schedule_graph") {
                state.open_debugdump_status = Some(e);
            }
        }
        if ui.button("Open render app schedule").clicked() {
            let schedule_graph = world.get_resource::<debugdump::DotGraphs>().unwrap();
            if let Err(e) = open_dot(
                &schedule_graph.render_schedule_graph,
                "render_schedule_graph",
            ) {
                state.open_debugdump_status = Some(e);
            }
        }
        if ui.button("Open render graph").clicked() {
            let schedule_graph = world.get_resource::<debugdump::DotGraphs>().unwrap();
            if let Err(e) = open_dot(&schedule_graph.render_graph, "render_graph") {
                state.open_debugdump_status = Some(e);
            }
        }
    });

    if let Some(error) = &state.open_debugdump_status {
        let msg = match error {
            DebugdumpError::DotNotFound => {
                ui.vertical(|ui| {
                    ui.label("Could not generate svg.");
                    ui.label("Make sure to install the `dot` program from");
                    ui.hyperlink("https://graphviz.org/download/");
                    ui.label("and make it available in your PATH.");
                });
                return;
            }
            DebugdumpError::OpenError(e) => e.to_string(),
            DebugdumpError::IO(e) => e.to_string(),
        };
        ui.label(egui::RichText::new(msg).color(egui::Color32::RED));
    }
}
