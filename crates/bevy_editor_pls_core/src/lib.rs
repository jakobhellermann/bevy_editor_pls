mod drag_and_drop;
mod editor;
pub mod editor_window;

use std::any::TypeId;

use editor::EditorInternalState;
pub use editor::{Editor, EditorEvent, EditorPlugin, EditorState};

use bevy::prelude::App;
use editor_window::EditorWindow;

pub trait AddEditorWindow {
    fn add_editor_window<W: EditorWindow>(&mut self) -> &mut Self;
    fn set_default_panels<LFT: EditorWindow, BTTM: EditorWindow, RGHT: EditorWindow>(
        &mut self,
    ) -> &mut Self;
}

const MSG:&str = "Editor resource missing. Make sure to add the `EditorPlugin` before calling `app.add_editor_window`.";
impl AddEditorWindow for App {
    fn add_editor_window<W: EditorWindow>(&mut self) -> &mut Self {
        let mut editor = self.world.get_resource_mut::<Editor>().expect(MSG);
        editor.add_window::<W>();
        W::app_setup(self);
        self
    }
    fn set_default_panels<LFT: EditorWindow, BTTM: EditorWindow, RGHT: EditorWindow>(
        &mut self,
    ) -> &mut Self {
        if let Some(mut internal_settings) = self.world.get_resource_mut::<EditorInternalState>() {
            internal_settings.left_panel = Some(TypeId::of::<LFT>());
            internal_settings.bottom_panel = Some(TypeId::of::<BTTM>());
            internal_settings.right_panel = Some(TypeId::of::<RGHT>());
        } else {
            let state = EditorInternalState::new::<LFT, BTTM, RGHT>();
            self.world.insert_resource(state);
        }
        self
    }
}
