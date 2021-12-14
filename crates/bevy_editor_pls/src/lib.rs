use bevy::prelude::Plugin;
pub use bevy_editor_pls_core::*;

pub mod prelude {
    pub use crate::{AddEditorWindow, EditorPlugin};
}

pub struct EditorPlugin;
impl Plugin for EditorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(bevy_editor_pls_core::EditorPlugin);

        #[cfg(feature = "default_windows")]
        {
            use bevy_editor_pls_default_windows::hierarchy::HierarchyWindow;
            use bevy_editor_pls_default_windows::inspector::InspectorWindow;

            let mut editor = app.world.get_resource_mut::<Editor>().unwrap();
            editor.add_window::<HierarchyWindow>();
            editor.add_window::<InspectorWindow>();
        }
    }
}
