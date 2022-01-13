#[cfg(feature = "default_windows")]
pub mod controls;

use bevy::prelude::Plugin;
pub use bevy_editor_pls_core::*;

pub use egui;

#[cfg(feature = "default_windows")]
pub use bevy_editor_pls_default_windows as default_windows;

pub mod prelude {
    pub use crate::{AddEditorWindow, EditorPlugin};
}

pub struct EditorPlugin;
impl Plugin for EditorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(bevy_editor_pls_core::EditorPlugin);

        #[cfg(feature = "default_windows")]
        {
            use bevy_editor_pls_default_windows::cameras::CameraWindow;
            use bevy_editor_pls_default_windows::debug_settings::DebugSettingsWindow;
            use bevy_editor_pls_default_windows::diagnostics::DiagnosticsWindow;
            use bevy_editor_pls_default_windows::hierarchy::HierarchyWindow;
            use bevy_editor_pls_default_windows::inspector::InspectorWindow;
            use bevy_editor_pls_default_windows::scenes::SceneWindow;

            app.add_editor_window::<HierarchyWindow>();
            app.add_editor_window::<InspectorWindow>();
            app.add_editor_window::<DebugSettingsWindow>();
            app.add_editor_window::<DiagnosticsWindow>();
            app.add_editor_window::<CameraWindow>();
            app.add_editor_window::<SceneWindow>();

            app.add_plugin(bevy::pbr::wireframe::WireframePlugin);

            app.init_resource::<controls::EditorControls>()
                .add_system(controls::editor_controls_system);
        }
    }
}
