use bevy::{
    core::CoreSystem,
    input::InputSystem,
    pbr::wireframe::WireframeConfig,
    prelude::*,
    render::{options::WgpuOptions, render_resource::WgpuFeatures},
};
use bevy_editor_pls_core::{editor_window::EditorWindow, Editor};
use bevy_inspector_egui::{bevy_egui, egui::Grid, Inspectable};

pub struct DebugSettingsWindowState {
    pub pause_time: bool,
    pub wireframes: bool,
    pub highlight_selected: bool,
}

impl Default for DebugSettingsWindowState {
    fn default() -> Self {
        Self {
            pause_time: false,
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

            ui.label("Pause time");
            ui.checkbox(&mut state.pause_time, "");
            ui.end_row();

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

    fn app_setup(app: &mut App) {
        app.init_resource::<EditorTime>()
            .init_resource::<StashedTime>()
            .add_system_to_stage(
                CoreStage::First,
                pause_time.exclusive_system().after(CoreSystem::Time),
            );

        use_editor_time_for_egui(app);
    }
}

fn use_editor_time_for_egui(app: &mut App) {
    // this doesn't guarantee that the time will by only changed for the `ProcessInput` system but I think it'll work well enough.
    app.add_system_to_stage(
        CoreStage::PreUpdate,
        use_editor_time
            .after(InputSystem)
            .before(bevy_egui::EguiSystem::ProcessInput),
    );
    app.add_system_to_stage(
        CoreStage::PreUpdate,
        unuse_editor_time
            .after(bevy_egui::EguiSystem::ProcessInput)
            .before(bevy_egui::EguiSystem::BeginFrame),
    );
}

#[derive(Default)]
pub struct EditorTime(pub Time);

fn pause_time(
    mut previously_paused_time: Local<bool>,

    editor: Res<Editor>,
    mut time: ResMut<Time>,
    mut editor_time: ResMut<EditorTime>,
) {
    editor_time.0.update();

    let pause_time = editor
        .window_state::<DebugSettingsWindow>()
        .unwrap()
        .pause_time;

    match (*previously_paused_time, pause_time) {
        // paused
        (_, true) => {
            *time = Time::default();
        }
        // just unpaused
        (true, false) => {
            *time = editor_time.0.clone();
        }
        // running
        (false, false) => {}
    }

    *previously_paused_time = pause_time;
}

#[derive(Default)]
struct StashedTime(Time);

// time > stashed_time
// editor_time -> time
fn use_editor_time(
    mut time: ResMut<Time>,
    editor_time: Res<EditorTime>,
    mut stashed_time: ResMut<StashedTime>,
) {
    let previous_time = std::mem::replace(&mut *time, editor_time.0.clone());
    stashed_time.0 = previous_time;
}

// stashed_time -> time
fn unuse_editor_time(mut time: ResMut<Time>, mut stashed_time: ResMut<StashedTime>) {
    *time = std::mem::take(&mut stashed_time.0);
}
