pub mod editor;
pub mod editor_window;

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
            app.add_plugin(DefaultInspectorConfigPlugin);
        }
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugin(EguiPlugin);
        }

        let (window_entity, always_active) = match self.window {
            WindowRef::Primary => {
                let entity = app
                    .world
                    .query_filtered::<Entity, With<PrimaryWindow>>()
                    .single(&app.world);
                (entity, false)
            }
            WindowRef::Entity(entity) => (entity, true),
        };

        app.insert_resource(Editor::new(window_entity, always_active))
            .init_resource::<EditorInternalState>()
            .add_event::<EditorEvent>()
            .configure_set(EditorSet::UI.in_base_set(CoreSet::PostUpdate))
            .add_system(
                Editor::system
                    .in_set(EditorSet::UI)
                    .before(TransformSystem::TransformPropagate)
                    .before(CameraUpdateSystem)
                    .after(EguiSet::ProcessOutput),
            );
    }
}
