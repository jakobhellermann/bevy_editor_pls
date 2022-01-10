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
    floating_windows: Vec<FloatingWindow>,
    viewport: EditorViewportSize,

    next_floating_window_id: u32,
}

#[derive(Clone, Copy)]
enum EditorPanel {
    Left,
    Right,
    Bottom,
}

struct FloatingWindow {
    window: TypeId,
    id: u32,
    original_panel: Option<EditorPanel>,
}

impl EditorInternalState {
    fn next_floating_window_id(&mut self) -> u32 {
        let id = self.next_floating_window_id;
        self.next_floating_window_id += 1;
        id
    }

    #[allow(unused)]
    fn active_editor(&self, panel: EditorPanel) -> Option<TypeId> {
        match panel {
            EditorPanel::Left => self.left_panel.clone(),
            EditorPanel::Right => self.right_panel.clone(),
            EditorPanel::Bottom => self.bottom_panel.clone(),
        }
    }
    fn active_editor_mut(&mut self, panel: EditorPanel) -> &mut Option<TypeId> {
        match panel {
            EditorPanel::Left => &mut self.left_panel,
            EditorPanel::Right => &mut self.right_panel,
            EditorPanel::Bottom => &mut self.bottom_panel,
        }
    }
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

impl Editor {
    fn system(world: &mut World) {
        if !world.contains_resource::<EditorInternalState>() {
            let editor = world.get_resource::<Editor>().unwrap();
            let mut windows = editor.windows.keys().copied();
            let state = EditorInternalState {
                left_panel: windows.next(),
                right_panel: windows.next(),
                bottom_panel: windows.next(),
                floating_windows: Vec::new(),
                next_floating_window_id: 0,
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
        editor_state: &mut EditorState,
        internal_state: &mut EditorInternalState,
    ) {
        self.editor_menu_bar(ctx, editor_state, internal_state);

        if !editor_state.active {
            self.editor_floating_windows(world, ctx, internal_state);
            return;
        }
        let res = egui::SidePanel::left("left_panel")
            .resizable(true)
            .show(ctx, |ui| {
                self.editor_window(world, &mut internal_state.left_panel, ui);
            });
        self.editor_window_context_menu(res.response, internal_state, EditorPanel::Left);

        let res = egui::SidePanel::right("right_panel")
            .resizable(true)
            .show(ctx, |ui| {
                self.editor_window(world, &mut internal_state.right_panel, ui);
            });
        self.editor_window_context_menu(res.response, internal_state, EditorPanel::Right);

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let res = egui::TopBottomPanel::bottom("bottom_panel")
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
                self.editor_window_context_menu(res.response, internal_state, EditorPanel::Bottom);

                let position = ui.next_widget_position();
                let size = ui.available_size();

                if position != internal_state.viewport.position
                    || size != internal_state.viewport.size
                {
                    internal_state.viewport.position = position;
                    internal_state.viewport.size = size;
                }
            });

        self.editor_floating_windows(world, ctx, internal_state);
    }

    fn editor_menu_bar(
        &mut self,
        ctx: &egui::CtxRef,
        editor_state: &mut EditorState,
        internal_state: &mut EditorInternalState,
    ) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if play_pause_button(editor_state.active, ui).clicked() {
                    editor_state.active = !editor_state.active;
                }

                ui.menu_button("Open window", |ui| {
                    for (&window_id, window) in self.windows.iter() {
                        if ui.button(window.name).clicked() {
                            let floating_window_id = internal_state.next_floating_window_id();
                            internal_state.floating_windows.push(FloatingWindow {
                                window: window_id,
                                id: floating_window_id,
                                original_panel: None,
                            });
                            ui.close_menu();
                        }
                    }
                });
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
            self.editor_window_inner(world, *selected, ui);
        }

        ui.allocate_space(ui.available_size());
    }

    fn editor_window_inner(&mut self, world: &mut World, selected: TypeId, ui: &mut egui::Ui) {
        let cx = EditorWindowContext {
            window_states: &mut self.window_states,
        };
        let ui_fn = &self.windows.get_mut(&selected).unwrap().ui_fn;
        ui_fn(world, cx, ui);
    }

    fn editor_window_context_menu(
        &mut self,
        response: egui::Response,
        internal_state: &mut EditorInternalState,
        panel: EditorPanel,
    ) {
        response.context_menu(|ui| {
            let window_is_set = internal_state.active_editor_mut(panel).is_some();

            if ui
                .add_enabled(window_is_set, egui::Button::new("Pop out"))
                .clicked()
            {
                let window = std::mem::take(internal_state.active_editor_mut(panel));
                if let Some(window) = window {
                    let id = internal_state.next_floating_window_id();
                    internal_state.floating_windows.push(FloatingWindow {
                        window,
                        id,
                        original_panel: Some(panel),
                    });
                }

                ui.close_menu();
            }
        });
    }

    fn editor_floating_windows(
        &mut self,
        world: &mut World,
        ctx: &egui::CtxRef,
        internal_state: &mut EditorInternalState,
    ) {
        let mut close_floating_windows = Vec::new();
        for (i, floating_window) in internal_state.floating_windows.iter().enumerate() {
            let id = egui::Id::new(floating_window.id);
            let title = self.windows[&floating_window.window].name;

            let mut open = true;
            egui::Window::new(title)
                .id(id)
                .open(&mut open)
                .resizable(true)
                .default_size((0.0, 0.0))
                .show(ctx, |ui| {
                    self.editor_window_inner(world, floating_window.window, ui);
                    ui.allocate_space(ui.available_size());
                });
            if !open {
                close_floating_windows.push(i);
            }
        }

        for &to_remove in close_floating_windows.iter().rev() {
            let floating_window = internal_state.floating_windows.swap_remove(to_remove);

            if let Some(original_panel) = floating_window.original_panel {
                internal_state
                    .active_editor_mut(original_panel)
                    .get_or_insert(floating_window.window);
            }
        }
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
