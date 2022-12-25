pub mod editor;
pub mod editor_window;

pub use editor::{Editor, EditorEvent, EditorPlugin, EditorState};

use bevy::prelude::App;
use editor_window::EditorWindow;

pub use egui_dock;

pub trait AddEditorWindow {
    fn add_editor_window<W: EditorWindow>(&mut self) -> &mut Self;
}

impl AddEditorWindow for App {
    fn add_editor_window<W: EditorWindow>(&mut self) -> &mut Self {
        let mut editor = self.world.get_resource_mut::<Editor>().expect("Editor resource missing. Make sure to add the `EditorPlugin` before calling `app.add_editor_window`.");
        editor.add_window::<W>();
        W::app_setup(self);
        self
    }
}
