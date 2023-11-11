mod console;
mod logs;

use bevy::prelude::{App, World};
use bevy_editor_pls_core::editor_window::{EditorWindow, EditorWindowContext};
use bevy_inspector_egui::egui::Ui;
use console::*;

pub use logs::*;

pub fn setup(app: &mut App) {
    let logger = Logs::default();

    app.insert_resource(logger.clone());

    log::set_max_level(log::LevelFilter::Trace);

    if let Err(e) = log::set_boxed_logger(Box::from(logger)) {
        println!("Error setting boxed logger: {e:?}");
    }
}

pub struct ConsoleLogState {
    pub(crate) filter: log::Level,
}

impl Default for ConsoleLogState {
    fn default() -> Self {
        Self {
            filter: log::Level::Trace,
        }
    }
}

pub struct ConsoleLogWindow;
impl EditorWindow for ConsoleLogWindow {
    type State = ConsoleLogState;
    const NAME: &'static str = "Console";

    fn ui(world: &mut World, mut ctx: EditorWindowContext, ui: &mut Ui) {
        let state_mut = ctx.state_mut::<ConsoleLogWindow>();
        let state = state_mut.unwrap();

        let logs_res = world.resource::<Logs>();

        draw_console_logs(ui, &mut state.filter, logs_res);
    }
}
