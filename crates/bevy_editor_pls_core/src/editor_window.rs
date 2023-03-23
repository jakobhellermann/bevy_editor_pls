use bevy::prelude::{App, World};
use bevy::utils::HashMap;
use bevy_inspector_egui::egui;
use std::any::{Any, TypeId};

use crate::editor::EditorWindowState;

pub trait EditorWindow: 'static {
    type State: Default + Any + Send + Sync;

    const NAME: &'static str;
    const DEFAULT_SIZE: (f32, f32) = (0.0, 0.0);

    fn ui(world: &mut World, cx: EditorWindowContext, ui: &mut egui::Ui);
    fn menu_ui(world: &mut World, mut cx: EditorWindowContext, ui: &mut egui::Ui) {
        let _ = world;

        if ui.button(Self::NAME).clicked() {
            cx.open_floating_window::<Self>();
            ui.close_menu();
        }
    }
    fn viewport_toolbar_ui(world: &mut World, cx: EditorWindowContext, ui: &mut egui::Ui) {
        let _ = (world, cx, ui);
    }
    fn viewport_ui(world: &mut World, cx: EditorWindowContext, ui: &mut egui::Ui) {
        let _ = (world, cx, ui);
    }

    fn app_setup(app: &mut App) {
        let _ = app;
    }
}

pub struct EditorWindowContext<'a> {
    pub(crate) window_states: &'a mut HashMap<TypeId, EditorWindowState>,
    pub(crate) internal_state: &'a mut crate::editor::EditorInternalState,
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

    pub fn state_mut_many<const N: usize>(
        &mut self,
        ids: [&TypeId; N],
    ) -> [&mut (dyn Any + Send + Sync + 'static); N] {
        self.window_states
            .get_many_mut(ids)
            .unwrap()
            .map(|val| &mut **val)
    }
    pub fn state_mut_triplet<W1: EditorWindow, W2: EditorWindow, W3: EditorWindow>(
        &mut self,
    ) -> Option<(&mut W1::State, &mut W2::State, &mut W3::State)> {
        let [a, b, c] = self.window_states.get_many_mut([
            &TypeId::of::<W1>(),
            &TypeId::of::<W2>(),
            &TypeId::of::<W3>(),
        ])?;

        let a = a.downcast_mut::<W1::State>()?;
        let b = b.downcast_mut::<W2::State>()?;
        let c = c.downcast_mut::<W3::State>()?;
        Some((a, b, c))
    }

    pub fn state_mut_pair<W1: EditorWindow, W2: EditorWindow>(
        &mut self,
    ) -> Option<(&mut W1::State, &mut W2::State)> {
        assert_ne!(TypeId::of::<W1>(), TypeId::of::<W2>());

        let [a, b] = self
            .window_states
            .get_many_mut([&TypeId::of::<W1>(), &TypeId::of::<W2>()])?;

        let a = a.downcast_mut::<W1::State>()?;
        let b = b.downcast_mut::<W2::State>()?;
        Some((a, b))
    }

    pub fn open_floating_window<W: ?Sized + EditorWindow>(&mut self) {
        let floating_window_id = self.internal_state.next_floating_window_id();
        let window_id = std::any::TypeId::of::<W>();
        self.internal_state
            .floating_windows
            .push(crate::editor::FloatingWindow {
                window: window_id,
                id: floating_window_id,
                initial_position: None,
            });
    }
}
