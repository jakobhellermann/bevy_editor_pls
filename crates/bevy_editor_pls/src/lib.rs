#[cfg(feature = "default_windows")]
pub mod controls;

use std::any::TypeId;

use bevy::prelude::Plugin;
pub use bevy_editor_pls_core::*;

use bevy_editor_pls_core::{editor::EditorInternalState, egui_dock::NodeIndex};
use bevy_framepace::FramepacePlugin;
pub use egui;

#[cfg(feature = "default_windows")]
pub use bevy_editor_pls_default_windows as default_windows;

pub mod prelude {
    pub use crate::{AddEditorWindow, EditorPlugin};
    pub use bevy_editor_pls_default_windows::scenes::NotInScene;
}

pub struct EditorPlugin;
impl Plugin for EditorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(bevy_editor_pls_core::EditorPlugin);

        if !app.is_plugin_added::<FramepacePlugin>() {
            app.add_plugin(FramepacePlugin);
        }

        #[cfg(feature = "default_windows")]
        {
            use bevy_editor_pls_default_windows::add::AddWindow;
            use bevy_editor_pls_default_windows::assets::AssetsWindow;
            use bevy_editor_pls_default_windows::cameras::CameraWindow;
            use bevy_editor_pls_default_windows::debug_settings::DebugSettingsWindow;
            use bevy_editor_pls_default_windows::diagnostics::DiagnosticsWindow;
            use bevy_editor_pls_default_windows::hierarchy::HierarchyWindow;
            use bevy_editor_pls_default_windows::inspector::InspectorWindow;
            use bevy_editor_pls_default_windows::renderer::RendererWindow;
            use bevy_editor_pls_default_windows::resources::ResourcesWindow;
            use bevy_editor_pls_default_windows::scenes::SceneWindow;
            use bevy_editor_pls_default_windows::windows::WindowsWindow;

            app.add_editor_window::<HierarchyWindow>();
            app.add_editor_window::<AssetsWindow>();
            app.add_editor_window::<InspectorWindow>();
            app.add_editor_window::<DebugSettingsWindow>();
            app.add_editor_window::<AddWindow>();
            app.add_editor_window::<DiagnosticsWindow>();
            app.add_editor_window::<RendererWindow>();
            app.add_editor_window::<CameraWindow>();
            app.add_editor_window::<ResourcesWindow>();
            app.add_editor_window::<SceneWindow>();
            app.add_editor_window::<WindowsWindow>();

            app.add_editor_window::<controls::ControlsWindow>();

            app.add_plugin(bevy::pbr::wireframe::WireframePlugin);

            app.insert_resource(controls::EditorControls::default_bindings())
                .add_system(controls::editor_controls_system);

            let mut internal_state = app.world.resource_mut::<EditorInternalState>();

            let [game, _inspector] =
                internal_state.split_right::<InspectorWindow>(NodeIndex::root(), 0.75);
            let [game, _hierarchy] = internal_state.split_left::<HierarchyWindow>(game, 0.2);
            let [_game, _bottom] = internal_state.split_many(
                game,
                0.8,
                egui_dock::Split::Below,
                &[
                    TypeId::of::<ResourcesWindow>(),
                    TypeId::of::<AssetsWindow>(),
                    TypeId::of::<DebugSettingsWindow>(),
                    TypeId::of::<DiagnosticsWindow>(),
                ],
            );
        }
    }
}
