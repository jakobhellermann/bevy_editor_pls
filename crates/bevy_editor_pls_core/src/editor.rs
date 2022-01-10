use std::any::{Any, TypeId};

use bevy::render::camera::UpdateCameraProjectionSystem;
use bevy::{prelude::*, render::camera::Viewport, utils::HashMap};
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
            .add_system_to_stage(
                CoreStage::PostUpdate,
                Editor::system.exclusive_system().at_start(),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                set_main_pass_viewport.before(UpdateCameraProjectionSystem),
            );
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
    viewport: EditorViewportSize,
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
                viewport: EditorViewportSize::default(),
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

                let position = ui.next_widget_position();
                let size = ui.available_size();

                if position != internal_state.viewport.position
                    || size != internal_state.viewport.size
                {
                    internal_state.viewport.position = position;
                    internal_state.viewport.size = size;
                }
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

    pub fn window_state_mut<W: EditorWindow>(&mut self) -> Option<&mut W::State> {
        self.window_states
            .get_mut(&TypeId::of::<W>())
            .and_then(|s| s.downcast_mut::<W::State>())
    }
    pub fn window_state<W: EditorWindow>(&self) -> Option<&W::State> {
        self.window_states
            .get(&TypeId::of::<W>())
            .and_then(|s| s.downcast_ref::<W::State>())
    }
}

fn play_pause_button(active: bool, ui: &mut egui::Ui) -> egui::Response {
    let icon = match active {
        true => "▶",
        false => "⏸",
    };
    ui.add(egui::Button::new(icon).frame(false))
}

#[derive(Clone, Default)]
struct EditorViewportSize {
    position: egui::Pos2,
    size: egui::Vec2,
}

fn set_main_pass_viewport(
    editor_state: Res<EditorState>,
    internal_state: Res<EditorInternalState>,
    egui_settings: Res<EguiSettings>,
    windows: Res<Windows>,
    mut cameras: Query<&mut Camera>,
) {
    fn vec2(egui: egui::Vec2) -> Vec2 {
        Vec2::new(egui.x, egui.y)
    }
    fn vec2_pos(egui: egui::Pos2) -> Vec2 {
        Vec2::new(egui.x, egui.y)
    }

    if !(internal_state.is_changed() || editor_state.is_changed()) {
        return;
    };

    let scale_factor = windows.get_primary().unwrap().scale_factor() * egui_settings.scale_factor;

    let viewport_pos = vec2_pos(internal_state.viewport.position) * scale_factor as f32;
    let viewport_size = vec2(internal_state.viewport.size) * scale_factor as f32;

    cameras.iter_mut().for_each(|mut cam| {
        cam.viewport = editor_state.active.then(|| Viewport {
            x: viewport_pos.x,
            y: viewport_pos.y,
            w: viewport_size.x,
            h: viewport_size.y,
            min_depth: 0.0,
            max_depth: 1.0,
            scaling_mode: bevy::render::camera::ViewportScalingMode::Pixels,
        });
    });
}
