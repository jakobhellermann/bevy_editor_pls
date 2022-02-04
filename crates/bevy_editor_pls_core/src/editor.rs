use std::any::{Any, TypeId};

use bevy::app::Events;
use bevy::window::WindowId;
use bevy::{prelude::*, utils::HashMap};
use bevy_inspector_egui::bevy_egui::{egui, EguiContext, EguiPlugin, EguiSettings};
use bevy_inspector_egui::{InspectableRegistry, WorldInspectorParams};
use indexmap::IndexMap;

use crate::drag_and_drop;
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
            .add_event::<EditorEvent>()
            .add_system_to_stage(
                CoreStage::PostUpdate,
                Editor::system.exclusive_system().at_start(),
            );
    }
}

#[non_exhaustive]
pub enum EditorEvent {
    Toggle { now_active: bool },
}

pub struct EditorState {
    pub active: bool,
    pub pointer_in_viewport: bool,
    pub pointer_on_floating_window: bool,
    pub viewport: egui::Rect,
    pub listening_for_text: bool,
}

impl EditorState {
    fn is_in_viewport(&self, pos: egui::Pos2) -> bool {
        self.viewport.contains(pos)
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            active: false,
            pointer_in_viewport: false,
            pointer_on_floating_window: false,
            viewport: egui::Rect::NOTHING,
            listening_for_text: false,
        }
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
    menu_ui_fn: UiFn,
    default_size: (f32, f32),
}

pub(crate) struct EditorInternalState {
    left_panel: Option<TypeId>,
    right_panel: Option<TypeId>,
    bottom_panel: Option<TypeId>,
    pub(crate) floating_windows: Vec<FloatingWindow>,
    active_drag_window: Option<WindowPosition>,
    active_drop_location: Option<DropLocation>,

    next_floating_window_id: u32,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) enum EditorPanel {
    Left,
    Right,
    Bottom,
}

#[derive(Clone)]
pub(crate) struct FloatingWindow {
    pub(crate) window: TypeId,
    pub(crate) id: u32,
    pub(crate) original_panel: Option<EditorPanel>,
    pub(crate) initial_position: Option<egui::Pos2>,
}

#[derive(Debug)]
enum WindowPosition {
    Panel(EditorPanel),
    #[allow(dead_code)]
    FloatingWindow(u32),
}
impl WindowPosition {
    fn panel(self) -> Option<EditorPanel> {
        match self {
            WindowPosition::Panel(panel) => Some(panel),
            WindowPosition::FloatingWindow(_) => None,
        }
    }
}

#[derive(Debug)]
enum DropLocation {
    Panel(EditorPanel),
    NewFloatingWindow,
}

impl EditorInternalState {
    pub(crate) fn next_floating_window_id(&mut self) -> u32 {
        let id = self.next_floating_window_id;
        self.next_floating_window_id += 1;
        id
    }

    fn active_panel(&self, panel: EditorPanel) -> Option<TypeId> {
        match panel {
            EditorPanel::Left => self.left_panel.clone(),
            EditorPanel::Right => self.right_panel.clone(),
            EditorPanel::Bottom => self.bottom_panel.clone(),
        }
    }
    fn active_panel_mut(&mut self, panel: EditorPanel) -> &mut Option<TypeId> {
        match panel {
            EditorPanel::Left => &mut self.left_panel,
            EditorPanel::Right => &mut self.right_panel,
            EditorPanel::Bottom => &mut self.bottom_panel,
        }
    }

    fn set_window(&mut self, location: WindowPosition, window: TypeId) {
        match location {
            WindowPosition::Panel(panel) => *self.active_panel_mut(panel) = Some(window),
            WindowPosition::FloatingWindow(id) => {
                if let Some(floating_window) = self.floating_windows.iter_mut().find(|a| a.id == id)
                {
                    floating_window.window = window;
                }
            }
        }
    }
}

fn ui_fn<W: EditorWindow>(world: &mut World, cx: EditorWindowContext, ui: &mut egui::Ui) {
    W::ui(world, cx, ui);
}
fn menu_ui_fn<W: EditorWindow>(world: &mut World, cx: EditorWindowContext, ui: &mut egui::Ui) {
    W::menu_ui(world, cx, ui);
}

impl Editor {
    pub fn add_window<W: EditorWindow>(&mut self) {
        let type_id = std::any::TypeId::of::<W>();
        let ui_fn = Box::new(ui_fn::<W>);
        let menu_ui_fn = Box::new(menu_ui_fn::<W>);
        let data = EditorWindowData {
            ui_fn,
            menu_ui_fn,
            name: W::NAME,
            default_size: W::DEFAULT_SIZE,
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
                active_drag_window: None,
                active_drop_location: None,
            };
            world.insert_resource(state);
        }

        let ctx = world.get_resource_mut::<EguiContext>().unwrap().ctx_for_window_mut(WindowId::primary()).clone();
        world.resource_scope(|world, mut editor: Mut<Editor>| {
            world.resource_scope(|world, mut editor_state: Mut<EditorState>| {
                world.resource_scope(
                    |world, mut editor_internal_state: Mut<EditorInternalState>| {
                        world.resource_scope(
                            |world, mut editor_events: Mut<Events<EditorEvent>>| {
                                editor.editor_ui(
                                    world,
                                    &ctx,
                                    &mut editor_state,
                                    &mut editor_internal_state,
                                    &mut editor_events,
                                );
                            },
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
        editor_events: &mut Events<EditorEvent>,
    ) {
        self.editor_menu_bar(world, ctx, editor_state, internal_state, editor_events);

        if !editor_state.active {
            editor_state.pointer_on_floating_window =
                self.editor_floating_windows(world, ctx, internal_state);
            return;
        }
        let res = egui::SidePanel::left("left_panel")
            .resizable(true)
            .show(ctx, |ui| {
                self.editor_window(world, internal_state, ui, EditorPanel::Left);
            });
        self.editor_window_context_menu(res.response, internal_state, EditorPanel::Left);

        let res = egui::SidePanel::right("right_panel")
            .resizable(true)
            .show(ctx, |ui| {
                self.editor_window(world, internal_state, ui, EditorPanel::Right);
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
                        self.editor_window(world, internal_state, ui, EditorPanel::Bottom);
                    });
                self.editor_window_context_menu(res.response, internal_state, EditorPanel::Bottom);

                let (viewport, _) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
                editor_state.viewport = viewport;
            });

        editor_state.pointer_on_floating_window =
            self.editor_floating_windows(world, ctx, internal_state);

        self.handle_drag_and_drop(editor_state, internal_state, ctx);

        if let Some(interact_pos) = ctx.input().pointer.interact_pos() {
            editor_state.pointer_in_viewport = editor_state
                .viewport
                .expand(-ctx.style().interaction.resize_grab_radius_side)
                .contains(interact_pos);
        };

        editor_state.listening_for_text = ctx.wants_keyboard_input();
    }

    fn editor_menu_bar(
        &mut self,
        world: &mut World,
        ctx: &egui::CtxRef,
        editor_state: &mut EditorState,
        internal_state: &mut EditorInternalState,
        editor_events: &mut Events<EditorEvent>,
    ) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if play_pause_button(editor_state.active, ui).clicked() {
                    editor_state.active = !editor_state.active;
                    editor_events.send(EditorEvent::Toggle {
                        now_active: editor_state.active,
                    });
                }

                ui.menu_button("Open window", |ui| {
                    for (&_, window) in self.windows.iter() {
                        let cx = EditorWindowContext {
                            window_states: &mut self.window_states,
                            internal_state,
                        };
                        (window.menu_ui_fn)(world, cx, ui);
                    }
                });
            });
        });
    }

    fn editor_window(
        &mut self,
        world: &mut World,
        internal_state: &mut EditorInternalState,
        ui: &mut egui::Ui,
        panel: EditorPanel,
    ) {
        let id = egui::Id::new(panel);
        let drag_id = id.with("drag");

        let selected_text = internal_state
            .active_panel(panel)
            .clone()
            .map_or_else(|| "Select a window", |id| self.windows[&id].name);

        egui::menu::bar(ui, |ui| {
            egui::ComboBox::from_id_source("panel select")
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for (id, window) in &self.windows {
                        if ui.selectable_label(false, window.name).clicked() {
                            *internal_state.active_panel_mut(panel) = Some(*id);
                        }
                    }
                    if ui.selectable_label(false, "None").clicked() {
                        *internal_state.active_panel_mut(panel) = None;
                    }
                });

            ui.with_layout(egui::Layout::right_to_left(), |ui| {
                let can_drag = internal_state.active_panel(panel).is_some();

                let is_being_dragged = drag_and_drop::drag_source(ui, drag_id, can_drag, |ui| {
                    ui.add_enabled(can_drag, egui::Button::new("☰").frame(false));
                });
                if is_being_dragged {
                    internal_state.active_drag_window = Some(WindowPosition::Panel(panel));
                }
            });
        });

        let some_window_is_being_dragged = internal_state.active_drag_window.is_some();
        let drop_response = drag_and_drop::drop_target(ui, some_window_is_being_dragged, |ui| {
            if let Some(selected) = internal_state.active_panel(panel) {
                self.editor_window_inner(world, internal_state, selected, ui);
            }

            ui.allocate_space(ui.available_size());
        })
        .response;

        if ui.memory().is_anything_being_dragged() && drop_response.hovered() {
            internal_state.active_drop_location = Some(DropLocation::Panel(panel));
        } else {
            if let Some(DropLocation::Panel(drop_location)) = internal_state.active_drop_location {
                if drop_location == panel {
                    internal_state.active_drop_location = None;
                }
            }
        }
    }

    fn editor_window_inner(
        &mut self,
        world: &mut World,
        internal_state: &mut EditorInternalState,
        selected: TypeId,
        ui: &mut egui::Ui,
    ) {
        let cx = EditorWindowContext {
            window_states: &mut self.window_states,
            internal_state,
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
            let window_is_set = internal_state.active_panel_mut(panel).is_some();

            if ui
                .add_enabled(window_is_set, egui::Button::new("Pop out"))
                .clicked()
            {
                let window = std::mem::take(internal_state.active_panel_mut(panel));
                if let Some(window) = window {
                    let id = internal_state.next_floating_window_id();
                    internal_state.floating_windows.push(FloatingWindow {
                        window,
                        id,
                        original_panel: Some(panel),
                        initial_position: None,
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
    ) -> bool {
        let mut close_floating_windows = Vec::new();
        let floating_windows = internal_state.floating_windows.clone();

        let mut cursor_on_floating_window = false;

        for (i, floating_window) in floating_windows.into_iter().enumerate() {
            let id = egui::Id::new(floating_window.id);
            let title = self.windows[&floating_window.window].name;

            let mut open = true;
            let default_size = self.windows[&floating_window.window].default_size;
            let mut window = egui::Window::new(title)
                .id(id)
                .open(&mut open)
                .resizable(true)
                .default_size(default_size);
            if let Some(initial_position) = floating_window.initial_position {
                window = window.default_pos(initial_position - egui::Vec2::new(10.0, 10.0))
            }
            let response = window.show(ctx, |ui| {
                self.editor_window_inner(world, internal_state, floating_window.window, ui);
                ui.allocate_space(ui.available_size() - (5.0, 5.0).into());
            });

            cursor_on_floating_window |= response
                .and_then(|response| {
                    let interact_pos = ctx.input().pointer.interact_pos()?;
                    let in_bounds = response
                        .response
                        .rect
                        .expand(-ctx.style().interaction.resize_grab_radius_side)
                        .contains(interact_pos);
                    Some(in_bounds)
                })
                .unwrap_or(false);

            if !open {
                close_floating_windows.push(i);
            }
        }

        for &to_remove in close_floating_windows.iter().rev() {
            let floating_window = internal_state.floating_windows.swap_remove(to_remove);

            if let Some(original_panel) = floating_window.original_panel {
                internal_state
                    .active_panel_mut(original_panel)
                    .get_or_insert(floating_window.window);
            }
        }

        cursor_on_floating_window
    }

    fn handle_drag_and_drop(
        &mut self,
        editor_state: &mut EditorState,
        internal_state: &mut EditorInternalState,
        ctx: &egui::CtxRef,
    ) -> Option<()> {
        if !ctx.input().pointer.any_released() {
            return None;
        }

        let active_window = std::mem::take(&mut internal_state.active_drag_window)?;
        let drop_location = match std::mem::take(&mut internal_state.active_drop_location) {
            Some(drop_location) => drop_location,
            None => {
                let pos = ctx.input().pointer.interact_pos()?;
                if editor_state.is_in_viewport(pos) {
                    DropLocation::NewFloatingWindow
                } else {
                    return None;
                }
            }
        };

        let window_id = match active_window {
            WindowPosition::Panel(panel) => {
                let window_id = std::mem::take(internal_state.active_panel_mut(panel)).unwrap();
                window_id
            }
            WindowPosition::FloatingWindow(id) => {
                let index = internal_state
                    .floating_windows
                    .iter()
                    .position(|floating_window| floating_window.id == id)
                    .unwrap();
                let floating_window = internal_state.floating_windows.swap_remove(index);
                floating_window.window
            }
        };

        match drop_location {
            DropLocation::Panel(panel) => {
                let previous_window = std::mem::take(internal_state.active_panel_mut(panel));
                *internal_state.active_panel_mut(panel) = Some(window_id);

                if let Some(previous_window) = previous_window {
                    internal_state.set_window(active_window, previous_window);
                }
            }
            DropLocation::NewFloatingWindow => {
                let id = internal_state.next_floating_window_id();
                internal_state.floating_windows.push(FloatingWindow {
                    window: window_id,
                    id,
                    original_panel: active_window.panel(),
                    initial_position: ctx.input().pointer.interact_pos(),
                });
            }
        }

        Some(())
    }
}

fn play_pause_button(active: bool, ui: &mut egui::Ui) -> egui::Response {
    let icon = match active {
        true => "▶",
        false => "⏸",
    };
    ui.add(egui::Button::new(icon).frame(false))
}
