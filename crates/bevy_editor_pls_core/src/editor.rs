use std::any::{Any, TypeId};

use bevy::ecs::event::Events;
use bevy::window::WindowMode;
use bevy::{prelude::*, utils::HashMap};
use bevy_inspector_egui::bevy_egui::{egui, EguiContext};
use egui_dock::{NodeIndex, TabBarStyle, TabIndex};
use indexmap::IndexMap;

use crate::editor_window::{EditorWindow, EditorWindowContext};

#[non_exhaustive]
#[derive(Event)]
pub enum EditorEvent {
    Toggle { now_active: bool },
    FocusSelected,
}

#[derive(Debug)]
enum ActiveEditorInteraction {
    Viewport,
    Editor,
}

#[derive(Resource)]
pub struct Editor {
    on_window: Entity,
    always_active: bool,

    active: bool,

    pointer_used: bool,
    active_editor_interaction: Option<ActiveEditorInteraction>,
    listening_for_text: bool,
    viewport: egui::Rect,

    windows: IndexMap<TypeId, EditorWindowData>,
    window_states: HashMap<TypeId, EditorWindowState>,
}
impl Editor {
    pub fn new(on_window: Entity, always_active: bool) -> Self {
        Editor {
            on_window,
            always_active,

            active: always_active,
            pointer_used: false,
            active_editor_interaction: None,
            listening_for_text: false,
            viewport: egui::Rect::NOTHING,

            windows: IndexMap::default(),
            window_states: HashMap::default(),
        }
    }

    pub fn window(&self) -> Entity {
        self.on_window
    }
    pub fn always_active(&self) -> bool {
        self.always_active
    }
    pub fn active(&self) -> bool {
        self.active
    }

    /// Panics if `self.always_active` is true
    pub fn set_active(&mut self, active: bool) {
        if !active && self.always_active {
            warn!("cannot call set_active on always-active editor");
        }

        self.active = active;
    }

    pub fn viewport(&self) -> egui::Rect {
        self.viewport
    }
    pub fn is_in_viewport(&self, pos: egui::Pos2) -> bool {
        self.viewport.contains(pos)
    }

    pub fn pointer_used(&self) -> bool {
        self.pointer_used
            || matches!(
                self.active_editor_interaction,
                Some(ActiveEditorInteraction::Editor)
            )
    }

    pub fn listening_for_text(&self) -> bool {
        self.listening_for_text
    }

    pub fn viewport_interaction_active(&self) -> bool {
        !self.pointer_used
            || matches!(
                self.active_editor_interaction,
                Some(ActiveEditorInteraction::Viewport)
            )
    }
}

pub(crate) type UiFn =
    Box<dyn Fn(&mut World, EditorWindowContext, &mut egui::Ui) + Send + Sync + 'static>;
pub(crate) type EditorWindowState = Box<dyn Any + Send + Sync>;

struct EditorWindowData {
    name: &'static str,
    ui_fn: UiFn,
    menu_ui_fn: UiFn,
    viewport_toolbar_ui_fn: UiFn,
    viewport_ui_fn: UiFn,
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
fn viewport_ui_fn<W: EditorWindow>(world: &mut World, cx: EditorWindowContext, ui: &mut egui::Ui) {
    W::viewport_ui(world, cx, ui);
}

impl Editor {
    pub fn add_window<W: EditorWindow>(&mut self) {
        let type_id = std::any::TypeId::of::<W>();
        let ui_fn = Box::new(ui_fn::<W>);
        let menu_ui_fn = Box::new(menu_ui_fn::<W>);
        let viewport_toolbar_ui_fn = Box::new(viewport_toolbar_ui_fn::<W>);
        let viewport_ui_fn = Box::new(viewport_ui_fn::<W>);
        let data = EditorWindowData {
            ui_fn,
            menu_ui_fn,
            viewport_toolbar_ui_fn,
            viewport_ui_fn,
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
            .insert(type_id, Box::<<W as EditorWindow>::State>::default());
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
    pub(crate) fn system(world: &mut World) {
        world.resource_scope(|world, mut editor: Mut<Editor>| {
            let Ok(mut egui_context) = world
                .query::<&mut EguiContext>()
                .get_mut(world, editor.on_window)
            else {
                return;
            };
            let egui_context = egui_context.get_mut().clone();

            world.resource_scope(
                |world, mut editor_internal_state: Mut<EditorInternalState>| {
                    world.resource_scope(|world, mut editor_events: Mut<Events<EditorEvent>>| {
                        editor.editor_ui(
                            world,
                            &egui_context,
                            &mut editor_internal_state,
                            &mut editor_events,
                        );
                    });
                },
            );
        });
    }

    fn editor_ui(
        &mut self,
        world: &mut World,
        ctx: &egui::Context,
        internal_state: &mut EditorInternalState,
        editor_events: &mut Events<EditorEvent>,
    ) {
        self.editor_menu_bar(world, ctx, internal_state, editor_events);

        if !self.active {
            self.editor_floating_windows(world, ctx, internal_state);
            self.pointer_used = ctx.wants_pointer_input();
            return;
        }

        let mut tree = std::mem::replace(
            &mut internal_state.tree,
            egui_dock::Tree::new(Vec::default()),
        );

        egui_dock::DockArea::new(&mut tree)
            .style(egui_dock::Style {
                tab_bar: TabBarStyle {
                    bg_fill: ctx.style().visuals.window_fill(),
                    ..default()
                },
                ..egui_dock::Style::from_egui(ctx.style().as_ref())
            })
            .show(
                ctx,
                &mut TabViewer {
                    editor: self,
                    internal_state,
                    world,
                },
            );
        internal_state.tree = tree;

        let pointer_pos = ctx.input(|input| input.pointer.interact_pos());
        self.pointer_used = pointer_pos.map_or(false, |pos| !self.is_in_viewport(pos));

        self.editor_floating_windows(world, ctx, internal_state);

        self.listening_for_text = ctx.wants_keyboard_input();

        let is_pressed = ctx.input(|input| input.pointer.press_start_time().is_some());
        match (&self.active_editor_interaction, is_pressed) {
            (_, false) => self.active_editor_interaction = None,
            (None, true) => {
                self.active_editor_interaction = Some(match self.pointer_used {
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
        internal_state: &mut EditorInternalState,
        editor_events: &mut Events<EditorEvent>,
    ) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            let bar_response = egui::menu::bar(ui, |ui| {
                if !self.always_active && play_pause_button(self.active, ui).clicked() {
                    self.active = !self.active;
                    editor_events.send(EditorEvent::Toggle {
                        now_active: self.active,
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
                let mut window = world
                    .query::<&mut Window>()
                    .get_mut(world, self.on_window)
                    .unwrap();

                match window.mode {
                    WindowMode::Windowed => window.mode = WindowMode::BorderlessFullscreen,
                    _ => window.mode = WindowMode::Windowed,
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

    fn editor_viewport_toolbar_ui(
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

            (window.viewport_ui_fn)(world, cx, ui);
        }
    }
}

struct TabViewer<'a> {
    editor: &'a mut Editor,
    internal_state: &'a mut EditorInternalState,
    world: &'a mut World,
}
impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = TreeTab;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match *tab {
            TreeTab::GameView => {
                let viewport = ui.clip_rect();

                ui.horizontal(|ui| {
                    ui.style_mut().spacing.button_padding = egui::vec2(2.0, 0.0);
                    let height = ui.spacing().interact_size.y;
                    ui.set_min_size(egui::vec2(ui.available_width(), height));

                    self.editor
                        .editor_viewport_toolbar_ui(self.world, ui, self.internal_state);
                });

                self.editor.viewport = viewport;

                self.editor
                    .editor_viewport_ui(self.world, ui, self.internal_state);
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
