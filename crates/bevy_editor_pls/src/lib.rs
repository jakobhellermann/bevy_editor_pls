#[cfg(feature = "default_windows")]
pub mod controls;

use bevy::{
    prelude::{Entity, Plugin},
    window::{MonitorSelection, Window, WindowPosition, WindowRef, WindowResolution},
};
pub use bevy_editor_pls_core::*;

// use bevy_framepace::FramepacePlugin;
pub use egui;

#[cfg(feature = "default_windows")]
pub use bevy_editor_pls_default_windows as default_windows;

pub mod prelude {
    pub use crate::{AddEditorWindow, EditorPlugin};
    #[cfg(feature = "default_windows")]
    pub use bevy_editor_pls_default_windows::scenes::NotInScene;
}

#[derive(Default)]
pub enum EditorWindow {
    New(Window),
    #[default]
    Primary,
    Window(Entity),
}

#[derive(Default)]
pub struct EditorPlugin {
    pub window: EditorWindow,
}

impl EditorPlugin {
    pub fn new() -> Self {
        EditorPlugin::default()
    }
    pub fn in_separate_window(mut self) -> Self {
        self.window = EditorWindow::New(Window::default());
        self
    }

    pub fn in_separate_window_fullscreen(mut self) -> Self {
        self.window = EditorWindow::New(Window {
            // TODO: just use `mode: BorderlessFullscreen` https://github.com/bevyengine/bevy/pull/8178
            resolution: WindowResolution::new(1920.0, 1080.0),
            position: WindowPosition::Centered(MonitorSelection::Index(1)),
            decorations: false,
            ..Default::default()
        });

        self
    }
}

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let window = match self.window {
            EditorWindow::New(ref window) => {
                let mut window = window.clone();
                if window.title == "Bevy App" {
                    window.title = "bevy_editor_pls".into();
                }
                let entity = app.world.spawn(window);
                WindowRef::Entity(entity.id())
            }
            EditorWindow::Window(entity) => WindowRef::Entity(entity),
            EditorWindow::Primary => WindowRef::Primary,
        };

        app.add_plugin(bevy_editor_pls_core::EditorPlugin { window });

        // if !app.is_plugin_added::<FramepacePlugin>() {
        // app.add_plugin(FramepacePlugin);
        // }

        #[cfg(feature = "default_windows")]
        {
            use bevy_editor_pls_default_windows::add::AddWindow;
            use bevy_editor_pls_default_windows::assets::AssetsWindow;
            use bevy_editor_pls_default_windows::cameras::CameraWindow;
            use bevy_editor_pls_default_windows::debug_settings::DebugSettingsWindow;
            use bevy_editor_pls_default_windows::diagnostics::DiagnosticsWindow;
            use bevy_editor_pls_default_windows::gizmos::GizmoWindow;
            use bevy_editor_pls_default_windows::hierarchy::HierarchyWindow;
            use bevy_editor_pls_default_windows::inspector::InspectorWindow;
            use bevy_editor_pls_default_windows::renderer::RendererWindow;
            use bevy_editor_pls_default_windows::resources::ResourcesWindow;
            use bevy_editor_pls_default_windows::scenes::SceneWindow;

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
            app.add_editor_window::<GizmoWindow>();
            app.add_editor_window::<controls::ControlsWindow>();

            app.add_plugin(bevy::pbr::wireframe::WireframePlugin);

            app.insert_resource(controls::EditorControls::default_bindings())
                .add_system(controls::editor_controls_system);

            let mut internal_state = app.world.resource_mut::<editor::EditorInternalState>();

            let [game, _inspector] =
                internal_state.split_right::<InspectorWindow>(egui_dock::NodeIndex::root(), 0.75);
            let [game, _hierarchy] = internal_state.split_left::<HierarchyWindow>(game, 0.2);
            let [_game, _bottom] = internal_state.split_many(
                game,
                0.8,
                egui_dock::Split::Below,
                &[
                    std::any::TypeId::of::<ResourcesWindow>(),
                    std::any::TypeId::of::<AssetsWindow>(),
                    std::any::TypeId::of::<DebugSettingsWindow>(),
                    std::any::TypeId::of::<DiagnosticsWindow>(),
                ],
            );
        }
    }
}
