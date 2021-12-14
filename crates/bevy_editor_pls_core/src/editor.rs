use std::any::{Any, TypeId};

use bevy::{prelude::*, utils::HashMap};
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSettings};
use bevy_inspector_egui::{InspectableRegistry, WorldInspectorParams};
use indexmap::IndexMap;

use crate::editor_window::{EditorWindow, EditorWindowContext};

pub struct EditorPlugin;
impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        if !app.world.contains_resource::<EguiSettings>() {
            app.add_plugin(EguiPlugin);
        }
        if !app.world.contains_resource::<WorldInspectorParams>() {
            app.world
                .get_resource_or_insert_with(WorldInspectorParams::default);
            app.world
                .get_resource_or_insert_with(InspectableRegistry::default);
        }

        app.init_resource::<Editor>()
            .init_resource::<EditorState>()
            .add_system_to_stage(CoreStage::PostUpdate, Editor::system.exclusive_system());
    }
}

pub struct EditorState {
    pub active: bool,
}
impl Default for EditorState {
    fn default() -> Self {
        Self { active: true }
    }
}

#[derive(Default)]
pub struct Editor {
    windows: IndexMap<TypeId, EditorWindowData>,
    window_states: HashMap<TypeId, EditorWindowState>,
}

pub(crate) type UiFn =
    Box<dyn Fn(&mut World, EditorWindowContext, &mut egui::Ui) + Send + Sync + 'static>;
pub(crate) type EditorWindowState = Box<dyn Any + Send + Sync>;

struct EditorWindowData {
    name: &'static str,
    ui_fn: UiFn,
}

struct EditorInternalState {
    left_panel: Option<TypeId>,
    right_panel: Option<TypeId>,
    bottom_panel: Option<TypeId>,
}

fn ui_fn<W: EditorWindow>(world: &mut World, cx: EditorWindowContext, ui: &mut egui::Ui) {
    W::ui(world, cx, ui);
}

impl Editor {
    pub fn add_window<W: EditorWindow>(&mut self) {
        let type_id = std::any::TypeId::of::<W>();
        let ui_fn = Box::new(ui_fn::<W>);
        let data = EditorWindowData {
            ui_fn,
            name: W::NAME,
        };
        if self.windows.insert(type_id, data).is_some() {
            panic!(
                "window of type {} already inserted",
                std::any::type_name::<W>()
            );
        }
        self.window_states
            .insert(type_id, Box::new(W::State::default()));
    }

    fn system(world: &mut World) {
        if !world.contains_resource::<EditorInternalState>() {
            let editor = world.get_resource::<Editor>().unwrap();
            let mut windows = editor.windows.keys().copied();
            let state = EditorInternalState {
                left_panel: windows.next(),
                right_panel: windows.next(),
                bottom_panel: windows.next(),
            };
            world.insert_resource(state);
        }

        let ctx = world.get_resource::<EguiContext>().unwrap().ctx().clone();
        world.resource_scope(|world, mut editor: Mut<Editor>| {
            world.resource_scope(|world, mut editor_state: Mut<EditorState>| {
                world.resource_scope(
                    |world, mut editor_internal_state: Mut<EditorInternalState>| {
                        editor.editor_ui(
                            world,
                            &ctx,
                            &mut editor_state,
                            &mut editor_internal_state,
                        );
                    },
                );
            });
        });
    }

    fn editor_ui(
        &mut self,
        world: &mut World,
        ctx: &egui::CtxRef,
        state: &mut EditorState,
        internal_state: &mut EditorInternalState,
    ) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            if play_pause_button(state.active, ui).clicked() {
                state.active = !state.active;
            }
        });

        if !state.active {
            return;
        }
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .show(ctx, |ui| {
                self.editor_window(world, &mut internal_state.left_panel, ui);
            });

        egui::SidePanel::right("right_panel")
            .resizable(true)
            .show(ctx, |ui| {
                self.editor_window(world, &mut internal_state.right_panel, ui);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                egui::TopBottomPanel::bottom("bottom_panel")
                    .resizable(true)
                    .default_height(100.0)
                    .frame(
                        egui::Frame::none()
                            .fill(ui.style().visuals.window_fill())
                            .stroke(ui.style().visuals.window_stroke()),
                    )
                    .show_inside(ui, |ui| {
                        self.editor_window(world, &mut internal_state.bottom_panel, ui);
                    });
            });
    }

    fn editor_window(
        &mut self,
        world: &mut World,
        selected: &mut Option<TypeId>,
        ui: &mut egui::Ui,
    ) {
        let selected_text = selected
            .clone()
            .map_or_else(|| "Select a window", |id| self.windows[&id].name);
        egui::ComboBox::from_id_source("panel select")
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                for (id, window) in &self.windows {
                    if ui.selectable_label(false, window.name).clicked() {
                        *selected = Some(*id);
                    }
                }
                if ui.selectable_label(false, "None").clicked() {
                    *selected = None;
                }
            });

        if let Some(selected) = selected {
            let cx = EditorWindowContext {
                window_states: &mut self.window_states,
            };
            let ui_fn = &self.windows.get_mut(selected).unwrap().ui_fn;
            ui_fn(world, cx, ui);
        }

        ui.allocate_space(ui.available_size());
    }
}

fn play_pause_button(active: bool, ui: &mut egui::Ui) -> egui::Response {
    let icon = match active {
        true => "▶",
        false => "⏸",
    };
    ui.add(egui::Button::new(icon).frame(false))
}
