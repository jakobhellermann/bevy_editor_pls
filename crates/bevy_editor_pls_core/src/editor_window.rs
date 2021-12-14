use bevy::prelude::{App, World};
use bevy::utils::HashMap;
use bevy_inspector_egui::egui;
use std::any::{Any, TypeId};

use crate::editor::EditorWindowState;

pub trait EditorWindow: 'static {
    type State: Default + Any + Send + Sync;

    const NAME: &'static str;

    fn ui(world: &mut World, cx: EditorWindowContext, ui: &mut egui::Ui);

    fn app_setup(app: &mut App) {
        let _ = app;
    }
}

pub struct EditorWindowContext<'a> {
    pub(crate) window_states: &'a mut HashMap<TypeId, EditorWindowState>,
}
impl EditorWindowContext<'_> {
    pub fn state_mut<W: EditorWindow>(&mut self) -> Option<&mut W::State> {
        self.window_states
            .get_mut(&TypeId::of::<W>())
            .and_then(|s| s.downcast_mut::<W::State>())
    }
    pub fn state<W: EditorWindow>(&self) -> Option<&W::State> {
        self.window_states
            .get(&TypeId::of::<W>())
            .and_then(|s| s.downcast_ref::<W::State>())
    }
}
