use bevy::time::TimeSystem;
use bevy::{input::InputSystem, prelude::*};
use bevy_editor_pls_core::Editor;
use bevy_inspector_egui::bevy_egui;

use super::DebugSettingsWindow;

pub fn setup(app: &mut App) {
    app.init_resource::<EditorTime>()
        .init_resource::<StashedTime>()
        .add_system_to_stage(
            CoreStage::First,
            pause_time.after(TimeSystem),
        );

    use_editor_time_for_egui(app);
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

#[derive(Default, Resource)]
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

#[derive(Default, Resource)]
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
