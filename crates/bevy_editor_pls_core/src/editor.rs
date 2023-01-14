use std::any::{Any, TypeId};

use bevy::ecs::event::Events;
use bevy::window::{WindowId, WindowMode};
use bevy::{prelude::*, utils::HashMap};
use bevy_inspector_egui::bevy_egui::{egui, EguiContext, EguiPlugin, EguiSystem};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use egui_dock::{NodeIndex, TabIndex};
use indexmap::IndexMap;

use crate::editor_window::{EditorWindow, EditorWindowContext};

pub struct EditorPlugin;
impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<DefaultInspectorConfigPlugin>() {
            app.add_plugin(DefaultInspectorConfigPlugin);
        }
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugin(EguiPlugin);
        }

        app.init_resource::<Editor>()
            .init_resource::<EditorInternalState>()
            .init_resource::<EditorState>()
            .add_event::<EditorEvent>()
            .add_system_to_stage(
                CoreStage::PostUpdate,
                Editor::system.at_start().label(EguiSystem::ProcessOutput),
            );
    }
}

#[non_exhaustive]
pub enum EditorEvent {
    Toggle { now_active: bool },
    FocusSelected,
}

#[derive(Resource)]
pub struct EditorState {
    pub active: bool,
    pointer_used: bool,
    active_editor_interaction: Option<ActiveEditorInteraction>,
    pub listening_for_text: bool,
    pub viewport: egui::Rect,
}

#[derive(Debug)]
enum ActiveEditorInteraction {
    Viewport,
    Editor,
}

impl EditorState {
    fn is_in_viewport(&self, pos: egui::Pos2) -> bool {
        self.viewport.contains(pos)
    }

    pub fn pointer_used(&self) -> bool {
        self.pointer_used
            || matches!(
                self.active_editor_interaction,
                Some(ActiveEditorInteraction::Editor)
            )
    }

    pub fn viewport_interaction_active(&self) -> bool {
        !self.pointer_used
            || matches!(
                self.active_editor_interaction,
                Some(ActiveEditorInteraction::Viewport)
            )
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            active: false,
            pointer_used: false,
            active_editor_interaction: None,
            listening_for_text: false,
            viewport: egui::Rect::NOTHING,
        }
    }
}

#[derive(Resource, Default)]
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
    viewport_toolbar_ui_fn: UiFn,
    default_size: (f32, f32),
}

#[derive(Resource)]
pub struct EditorInternalState {
    tree: egui_dock::Tree<TreeTab>,
    pub(crate) floating_windows: Vec<FloatingWindow>,

    next_floating_window_id: u32,
}

impl Default for EditorInternalState {
    fn default() -> Self {
        Self {
            tree: egui_dock::Tree::new(vec![TreeTab::GameView]),
            floating_windows: Default::default(),
            next_floating_window_id: Default::default(),
        }
    }
}

#[derive(Copy, Clone)]
enum TreeTab {
    GameView,
    CustomWindow(TypeId),
}

impl EditorInternalState {
    pub fn push_to_focused_leaf<W: EditorWindow>(&mut self) {
        self.tree
            .push_to_focused_leaf(TreeTab::CustomWindow(TypeId::of::<W>()));
        if let Some(focus) = self.tree.focused_leaf() {
            self.tree.set_active_tab(focus, TabIndex(0));
        };
    }

    pub fn split<W: EditorWindow>(
        &mut self,
        parent: NodeIndex,
        split: egui_dock::Split,
        fraction: f32,
    ) -> [NodeIndex; 2] {
        let node = egui_dock::Node::leaf(TreeTab::CustomWindow(TypeId::of::<W>()));
        self.tree.split(parent, split, fraction, node)
    }

    pub fn split_right<W: EditorWindow>(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
    ) -> [NodeIndex; 2] {
        self.split::<W>(parent, egui_dock::Split::Right, fraction)
    }
    pub fn split_left<W: EditorWindow>(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
    ) -> [NodeIndex; 2] {
        self.split::<W>(parent, egui_dock::Split::Left, fraction)
    }
    pub fn split_above<W: EditorWindow>(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
    ) -> [NodeIndex; 2] {
        self.split::<W>(parent, egui_dock::Split::Above, fraction)
    }
    pub fn split_below<W: EditorWindow>(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
    ) -> [NodeIndex; 2] {
        self.split::<W>(parent, egui_dock::Split::Below, fraction)
    }

    pub fn split_many(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
        split: egui_dock::Split,
        windows: &[TypeId],
    ) -> [NodeIndex; 2] {
        let tabs = windows.iter().copied().map(TreeTab::CustomWindow).collect();
        let node = egui_dock::Node::leaf_with(tabs);
        self.tree.split(parent, split, fraction, node)
    }
}

#[derive(Clone)]
pub(crate) struct FloatingWindow {
    pub(crate) window: TypeId,
    pub(crate) id: u32,
    pub(crate) initial_position: Option<egui::Pos2>,
}

impl EditorInternalState {
    pub(crate) fn next_floating_window_id(&mut self) -> u32 {
        let id = self.next_floating_window_id;
        self.next_floating_window_id += 1;
        id
    }
}

fn ui_fn<W: EditorWindow>(world: &mut World, cx: EditorWindowContext, ui: &mut egui::Ui) {
    W::ui(world, cx, ui);
}
fn menu_ui_fn<W: EditorWindow>(world: &mut World, cx: EditorWindowContext, ui: &mut egui::Ui) {
    W::menu_ui(world, cx, ui);
}
fn viewport_toolbar_ui_fn<W: EditorWindow>(
    world: &mut World,
    cx: EditorWindowContext,
    ui: &mut egui::Ui,
) {
    W::viewport_toolbar_ui(world, cx, ui);
}

impl Editor {
    pub fn add_window<W: EditorWindow>(&mut self) {
        let type_id = std::any::TypeId::of::<W>();
        let ui_fn = Box::new(ui_fn::<W>);
        let menu_ui_fn = Box::new(menu_ui_fn::<W>);
        let viewport_toolbar_ui_fn = Box::new(viewport_toolbar_ui_fn::<W>);
        let data = EditorWindowData {
            ui_fn,
            menu_ui_fn,
            viewport_toolbar_ui_fn,
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
        let ctx = if let Some(ctx) = world
            .get_resource_mut::<EguiContext>()
            .unwrap()
            .try_ctx_for_window_mut(WindowId::primary())
            .cloned()
        {
            ctx
        } else {
            return;
        };

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
        ctx: &egui::Context,
        editor_state: &mut EditorState,
        internal_state: &mut EditorInternalState,
        editor_events: &mut Events<EditorEvent>,
    ) {
        self.editor_menu_bar(world, ctx, editor_state, internal_state, editor_events);

        if !editor_state.active {
            self.editor_floating_windows(world, ctx, internal_state);
            editor_state.pointer_used = ctx.is_pointer_over_area();
            return;
        }

        let mut tree = std::mem::replace(
            &mut internal_state.tree,
            egui_dock::Tree::new(Vec::default()),
        );
        egui_dock::DockArea::new(&mut tree).show(
            ctx,
            &mut TabViewer {
                editor: self,
                internal_state,
                world,
                editor_state,
            },
        );
        internal_state.tree = tree;

        let pointer_pos = ctx.input().pointer.interact_pos();
        editor_state.pointer_used =
            pointer_pos.map_or(false, |pos| !editor_state.is_in_viewport(pos));

        self.editor_floating_windows(world, ctx, internal_state);

        editor_state.listening_for_text = ctx.wants_keyboard_input();
        editor_state.pointer_used |= ctx.is_using_pointer();

        let is_pressed = ctx.input().pointer.press_start_time().is_some();
        match (&editor_state.active_editor_interaction, is_pressed) {
            (_, false) => editor_state.active_editor_interaction = None,
            (None, true) => {
                editor_state.active_editor_interaction = Some(match editor_state.pointer_used {
                    true => ActiveEditorInteraction::Editor,
                    false => ActiveEditorInteraction::Viewport,
                });
            }
            (Some(_), true) => {}
        }
    }

    fn editor_menu_bar(
        &mut self,
        world: &mut World,
        ctx: &egui::Context,
        editor_state: &mut EditorState,
        internal_state: &mut EditorInternalState,
        editor_events: &mut Events<EditorEvent>,
    ) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            let bar_response = egui::menu::bar(ui, |ui| {
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
            })
            .response
            .interact(egui::Sense::click());

            if bar_response.double_clicked() {
                if let Some(window) = world
                    .get_resource_mut::<Windows>()
                    .unwrap()
                    .get_primary_mut()
                {
                    match window.mode() {
                        WindowMode::Windowed => window.set_mode(WindowMode::Fullscreen),
                        _ => window.set_mode(WindowMode::Windowed),
                    }
                }
            }
        });
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
        ui: &mut egui::Ui,
        internal_state: &mut EditorInternalState,
        tab: TreeTab,
    ) {
        if ui.button("Pop out").clicked() {
            if let TreeTab::CustomWindow(window) = tab {
                let id = internal_state.next_floating_window_id();
                internal_state.floating_windows.push(FloatingWindow {
                    window,
                    id,
                    initial_position: None,
                });
            }

            ui.close_menu();
        }
    }

    fn editor_floating_windows(
        &mut self,
        world: &mut World,
        ctx: &egui::Context,
        internal_state: &mut EditorInternalState,
    ) {
        let mut close_floating_windows = Vec::new();
        let floating_windows = internal_state.floating_windows.clone();

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
            window.show(ctx, |ui| {
                self.editor_window_inner(world, internal_state, floating_window.window, ui);
                ui.allocate_space(ui.available_size() - (5.0, 5.0).into());
            });

            if !open {
                close_floating_windows.push(i);
            }
        }

        for &to_remove in close_floating_windows.iter().rev() {
            let _floating_window = internal_state.floating_windows.swap_remove(to_remove);
        }
    }

    fn editor_viewport_ui(
        &mut self,
        world: &mut World,
        ui: &mut egui::Ui,
        internal_state: &mut EditorInternalState,
    ) {
        for (_, window) in self.windows.iter() {
            let cx = EditorWindowContext {
                window_states: &mut self.window_states,
                internal_state,
            };
            (window.viewport_toolbar_ui_fn)(world, cx, ui);
        }
    }
}

struct TabViewer<'a> {
    editor: &'a mut Editor,
    internal_state: &'a mut EditorInternalState,
    editor_state: &'a mut EditorState,
    world: &'a mut World,
}
impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = TreeTab;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match *tab {
            TreeTab::GameView => {
                // self.editor
                // .editor_viewport_ui(self.world, ui, self.internal_state);

                ui.horizontal(|ui| {
                    ui.style_mut().spacing.button_padding = egui::vec2(2.0, 0.0);
                    let height = ui.spacing().interact_size.y;
                    ui.set_min_size(egui::vec2(ui.available_width(), height));

                    self.editor
                        .editor_viewport_ui(self.world, ui, self.internal_state);
                });

                let (viewport, _) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
                self.editor_state.viewport = viewport;
            }
            TreeTab::CustomWindow(window_id) => {
                self.editor
                    .editor_window_inner(self.world, self.internal_state, window_id, ui);
            }
        }
    }

    fn context_menu(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        self.editor
            .editor_window_context_menu(ui, self.internal_state, *tab);
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match *tab {
            TreeTab::GameView => "Viewport".into(),
            TreeTab::CustomWindow(window_id) => {
                self.editor.windows.get(&window_id).unwrap().name.into()
            }
        }
    }

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, TreeTab::GameView)
    }
}

fn play_pause_button(active: bool, ui: &mut egui::Ui) -> egui::Response {
    let icon = match active {
        true => "▶",
        false => "⏸",
    };
    ui.add(egui::Button::new(icon).frame(false))
}
