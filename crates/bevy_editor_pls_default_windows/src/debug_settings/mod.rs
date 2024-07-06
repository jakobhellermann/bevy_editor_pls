pub mod debugdump;

use bevy::{
    pbr::wireframe::WireframeConfig,
    prelude::*,
    reflect::TypeRegistry,
    render::{render_resource::WgpuFeatures, renderer::RenderAdapter},
};
use bevy_editor_pls_core::editor_window::EditorWindow;
use bevy_inspector_egui::{
    egui::{self, Grid},
    reflect_inspector::ui_for_value,
};

pub struct DebugSettingsWindowState {
    pub pause_time: bool,
    pub wireframes: bool,
    pub highlight_selected: bool,

    open_debugdump_status: Option<DebugdumpError>,
}

enum DebugdumpError {
    DotNotFound,
    ScheduleNotFound,
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
        let type_registry = world.resource::<AppTypeRegistry>().clone();
        let type_registry = type_registry.read();

        let state = cx.state_mut::<DebugSettingsWindow>().unwrap();
        debug_ui(world, state, ui, &type_registry);
    }

    fn app_finish(app: &mut App) {
        debugdump::setup(app);
    }
}

fn debug_ui(
    world: &mut World,
    state: &mut DebugSettingsWindowState,
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
) {
    let available_size = ui.available_size();
    let horizontal = available_size.x > available_size.y;

    horizontal_if(ui, horizontal, |ui| {
        debug_ui_options(world, state, ui, type_registry);

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

fn debug_ui_options(
    world: &mut World,
    state: &mut DebugSettingsWindowState,
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
) {
    Grid::new("debug settings").show(ui, |ui| {
        ui.label("Pause time");

        let mut time = world.resource_mut::<Time<Virtual>>();

        if ui.checkbox(&mut state.pause_time, "").changed() {
            if state.pause_time {
                time.pause();
            } else {
                time.unpause();
            }
        }
        ui.end_row();
        ui.label("Game Speed");

        let mut speed = time.relative_speed_f64();
        if ui
            .add(
                egui::DragValue::new(&mut speed)
                    .range(0..=20)
                    .speed(0.1),
            )
            .changed()
        {
            time.set_relative_speed_f64(speed);
        }
        ui.end_row();

        let wireframe_enabled = world
            .get_resource::<RenderAdapter>()
            .map_or(false, |adapter| {
                adapter
                    .0
                    .features()
                    .contains(WgpuFeatures::POLYGON_MODE_LINE)
            });

        if wireframe_enabled {
            ui.label("Wireframes");
        } else {
            ui.label("Wireframes (enable POLYGON_MODE_LINE feature)");
        }
        ui.scope(|ui| {
            ui.set_enabled(wireframe_enabled);
            if ui_for_value(&mut state.wireframes, ui, type_registry) {
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
    let open_dot = |dot: &Option<String>, path: &str| -> Result<(), DebugdumpError> {
        let dot = dot.as_ref().ok_or(DebugdumpError::ScheduleNotFound)?;

        let format = "svg";
        let rendered = match debugdump::execute_dot(dot, format) {
            Ok(rendered) => rendered,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(DebugdumpError::DotNotFound)
            }
            Err(e) => return Err(DebugdumpError::IO(e)),
        };
        let path = std::env::temp_dir().join(path).with_extension(format);
        std::fs::write(&path, rendered).map_err(DebugdumpError::IO)?;
        opener::open(path).map_err(DebugdumpError::OpenError)?;
        Ok(())
    };

    ui.vertical(|ui| {
        if ui.button("Open `Update` schedule").clicked() {
            let schedule_graph = world.get_resource::<debugdump::DotGraphs>().unwrap();
            if let Err(e) = open_dot(&schedule_graph.update_schedule, "schedule_main") {
                state.open_debugdump_status = Some(e);
            }
        }
        if ui.button("Open `FixedUpdate` schedule").clicked() {
            let schedule_graph = world.get_resource::<debugdump::DotGraphs>().unwrap();
            if let Err(e) = open_dot(&schedule_graph.fixed_update_schedule, "schedule_fixed") {
                state.open_debugdump_status = Some(e);
            }
        }
        if ui.button("Open render extract schedule").clicked() {
            let schedule_graph = world.get_resource::<debugdump::DotGraphs>().unwrap();
            if let Err(e) = open_dot(
                &schedule_graph.render_extract_schedule,
                "schedule_render_extract",
            ) {
                state.open_debugdump_status = Some(e);
            }
        }
        if ui.button("Open render main schedule").clicked() {
            let schedule_graph = world.get_resource::<debugdump::DotGraphs>().unwrap();
            if let Err(e) = open_dot(&schedule_graph.render_main_schedule, "schedule_render_main") {
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
            DebugdumpError::ScheduleNotFound => {
                ui.label("Schedule does not exist");
                return;
            }
        };
        ui.label(egui::RichText::new(msg).color(egui::Color32::RED));
    }
}
