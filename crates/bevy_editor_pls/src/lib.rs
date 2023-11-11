#![allow(clippy::needless_doctest_main)]

/// input settings for the editor UI
#[cfg(feature = "default_windows")]
pub mod controls;

use bevy::{
    prelude::{Entity, Plugin, Update},
    window::{MonitorSelection, Window, WindowPosition, WindowRef, WindowResolution},
};

pub use bevy_editor_pls_core::egui_dock;
#[doc(inline)]
pub use bevy_editor_pls_core::{editor, editor_window, AddEditorWindow};
pub use egui;

#[cfg(feature = "default_windows")]
#[doc(inline)]
pub use bevy_editor_pls_default_windows as default_windows;

/// Commonly used types and extension traits
pub mod prelude {
    pub use crate::{AddEditorWindow, EditorPlugin};
    #[cfg(feature = "default_windows")]
    pub use bevy_editor_pls_default_windows::scenes::NotInScene;
}

/// Where to show the editor
#[derive(Default)]
pub enum EditorWindowPlacement {
    /// On the primary window
    #[default]
    Primary,
    /// Spawn a new window for the editor
    New(Window),
    /// On an existing window
    Window(Entity),
}

/// Plugin adding various editor UI to the game executable.
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_editor_pls::EditorPlugin;
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(EditorPlugin::new())
///         .run();
/// }
/// ```
#[derive(Default)]
pub struct EditorPlugin {
    pub window: EditorWindowPlacement,
}

impl EditorPlugin {
    pub fn new() -> Self {
        EditorPlugin::default()
    }

    /// Start the editor in a new window. Use [`Window::default`] for creating a new window with default settings.
    pub fn in_new_window(mut self, window: Window) -> Self {
        self.window = EditorWindowPlacement::New(window);
        self
    }
    /// Start the editor on the second window ([`MonitorSelection::Index(1)`].
    pub fn on_second_monitor_fullscreen(self) -> Self {
        self.in_new_window(Window {
            // TODO: just use `mode: BorderlessFullscreen` https://github.com/bevyengine/bevy/pull/8178
            resolution: WindowResolution::new(1920.0, 1080.0),
            position: WindowPosition::Centered(MonitorSelection::Index(1)),
            decorations: false,
            ..Default::default()
        })
    }
}

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let window = match self.window {
            EditorWindowPlacement::New(ref window) => {
                let mut window = window.clone();
                if window.title == "Bevy App" {
                    window.title = "bevy_editor_pls".into();
                }
                let entity = app.world.spawn(window);
                WindowRef::Entity(entity.id())
            }
            EditorWindowPlacement::Window(entity) => WindowRef::Entity(entity),
            EditorWindowPlacement::Primary => WindowRef::Primary,
        };

        app.add_plugins(bevy_editor_pls_core::EditorPlugin { window });

        // if !app.is_plugin_added::<bevy_framepace::FramepacePlugin>() {
        //     app.add_plugins(bevy_framepace::FramepacePlugin);
        //     app.add_plugins(bevy_framepace::debug::DiagnosticsPlugin);
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
            use bevy_editor_pls_default_windows::console_log;

            console_log::setup(app);

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
            app.add_editor_window::<console_log::ConsoleLogWindow>();

            app.add_plugins(bevy::pbr::wireframe::WireframePlugin);

            app.insert_resource(controls::EditorControls::default_bindings())
                .add_systems(Update, controls::editor_controls_system);

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
                    std::any::TypeId::of::<console_log::ConsoleLogWindow>(),
                ],
            );
        }
    }
}
