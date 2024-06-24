/// Editor systems, events and resources
pub mod editor;
/// Trait definition for new editor windows
pub mod editor_window;

use std::marker::PhantomData;

use bevy::prelude::*;
use bevy::render::camera::CameraUpdateSystem;
use bevy::transform::TransformSystem;
use bevy::window::{PrimaryWindow, WindowRef};
use bevy_inspector_egui::{
    bevy_egui::{EguiPlugin, EguiSet},
    DefaultInspectorConfigPlugin,
};
use editor::EditorInternalState;
use editor_window::EditorWindow;

pub use editor::{Editor, EditorEvent};

/// Re-export of [`egui_dock`]
pub use egui_dock;

/// Extension trait for [`App`] to add a new editor window type
pub trait AddEditorWindow {
    fn add_editor_window<W: EditorWindow>(&mut self) -> &mut Self;
}

struct WindowSetupPlugin<W>(PhantomData<fn() -> W>);
impl<W: EditorWindow> Plugin for WindowSetupPlugin<W> {
    fn build(&self, app: &mut App) {
        W::app_setup(app);
    }

    fn finish(&self, app: &mut App) {
        W::app_finish(app);
    }
}

impl AddEditorWindow for App {
    fn add_editor_window<W: EditorWindow>(&mut self) -> &mut Self {
        let mut editor = self.world_mut().get_resource_mut::<Editor>().expect("Editor resource missing. Make sure to add the `EditorPlugin` before calling `app.add_editor_window`.");
        editor.add_window::<W>();

        self.add_plugins(WindowSetupPlugin::<W>(PhantomData));

        self
    }
}

#[derive(SystemSet, Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum EditorSet {
    /// In [`CoreSet::PostUpdate`]
    UI,
}

pub struct EditorPlugin {
    pub window: WindowRef,
}
impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<DefaultInspectorConfigPlugin>() {
            app.add_plugins(DefaultInspectorConfigPlugin);
        }
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }

        let (window_entity, always_active) = match self.window {
            WindowRef::Primary => {
                let entity = app
                    .world_mut()
                    .query_filtered::<Entity, With<PrimaryWindow>>()
                    .single(&app.world());
                (entity, false)
            }
            WindowRef::Entity(entity) => (entity, true),
        };

        app.insert_resource(Editor::new(window_entity, always_active))
            .init_resource::<EditorInternalState>()
            .add_event::<EditorEvent>()
            .configure_sets(PostUpdate, EditorSet::UI)
            .add_systems(
                Update,
                Editor::system
                    .in_set(EditorSet::UI)
                    .before(TransformSystem::TransformPropagate)
                    .before(CameraUpdateSystem)
                    .before(EguiSet::ProcessOutput),
            );
    }
}
