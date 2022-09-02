pub mod picking;

use bevy::ecs::entity::Entities;
use bevy::ecs::query::QuerySingleError;
use bevy::pbr::wireframe::Wireframe;
use bevy::prelude::*;
use bevy::render::{Extract, RenderApp, RenderStage};
use bevy::utils::HashSet;
use bevy_editor_pls_core::EditorState;
use bevy_inspector_egui::egui::{self, CollapsingHeader, RichText, ScrollArea};

use bevy_editor_pls_core::{
    editor_window::{EditorWindow, EditorWindowContext},
    Editor,
};

use crate::add::{add_ui, AddWindow, AddWindowState};
use crate::cameras::ActiveEditorCamera;
use crate::debug_settings::DebugSettingsWindow;

#[derive(Component)]
pub struct HideInEditor;

pub struct HierarchyWindow;
impl EditorWindow for HierarchyWindow {
    type State = HierarchyState;
    const NAME: &'static str = "Hierarchy";

    fn ui(world: &mut World, mut cx: EditorWindowContext, ui: &mut egui::Ui) {
        let (hierarchy_state, add_state) = cx.state_mut_pair::<HierarchyWindow, AddWindow>();
        let hierarchy_state = hierarchy_state.unwrap();

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                Hierarchy {
                    world,
                    state: hierarchy_state,
                    add_state: add_state.as_deref(),
                }
                .show(ui);
            });
    }

    fn app_setup(app: &mut bevy::prelude::App) {
        picking::setup(app);
        app.add_event::<EditorHierarchyEvent>()
            .add_system_to_stage(CoreStage::PostUpdate, clear_removed_entites)
            .add_system(handle_events);

        app.sub_app_mut(RenderApp)
            .add_system_to_stage(RenderStage::Extract, extract_wireframe_for_selected);
    }
}

fn clear_removed_entites(mut editor: ResMut<Editor>, entities: &Entities) {
    let state = editor.window_state_mut::<HierarchyWindow>().unwrap();
    state.selected.retain(|entity| entities.contains(entity));
}

pub enum EditorHierarchyEvent {
    SelectMesh,
}

fn handle_events(
    mut select_mesh_events: EventReader<EditorHierarchyEvent>,
    // mut editor_events: EventReader<EditorEvent>,
    // mut raycast_state: ResMut<EditorRayCastState>,
    editor_camera: Query<Option<&picking::EditorRayCastSource>, With<ActiveEditorCamera>>,
    non_editor_camera: Query<&picking::EditorRayCastSource, Without<super::cameras::EditorCamera>>,
    mut editor: ResMut<Editor>,
    editor_state: Res<EditorState>,
    input: Res<Input<KeyCode>>,
) {
    // TODO: reenable once bevy_mod_raycast has per-source configuration
    /*for event in editor_events.iter() {
        match *event {
            EditorEvent::Toggle { now_active: false } => {
                raycast_state.build_rays = false;
                raycast_state.update_raycast = false;
            }
            EditorEvent::Toggle { now_active: true } => {
                raycast_state.build_rays = true;
                raycast_state.update_raycast = true;
            }
            _ => {}
        }
    }*/

    for event in select_mesh_events.iter() {
        #[allow(irrefutable_let_patterns)]
        if let EditorHierarchyEvent::SelectMesh = event {
            let picked_entity = if editor_state.active {
                editor_camera.get_single().ok().and_then(|source| {
                    source.and_then(|source| source.intersect_top().map(|(entity, _)| entity))
                })
            } else {
                let source = match non_editor_camera.get_single() {
                    Ok(source) => Some(source),
                    Err(QuerySingleError::NoEntities(_)) => {
                        error!("No cameras with `EditorRayCastSource` found, can't click to inspect when the editor is inactive!");
                        continue;
                    }
                    Err(QuerySingleError::MultipleEntities(_)) => {
                        error!("Multiple cameras with `EditorRayCastSource` found!");
                        continue;
                    }
                };
                source
                    .and_then(|source| source.intersect_top())
                    .map(|(entity, _)| entity)
            };

            let state = editor.window_state_mut::<HierarchyWindow>().unwrap();

            let ctrl = input.any_pressed([KeyCode::LControl, KeyCode::RControl]);
            let shift = input.any_pressed([KeyCode::LShift, KeyCode::RShift]);
            let mode = SelectionMode::from_ctrl_shift(ctrl, shift);

            if let Some(entity) = picked_entity {
                info!("Selecting mesh, found {:?}", entity);
                state
                    .selected
                    .select(mode, entity, || std::iter::once(entity));
            } else {
                info!("Selecting mesh, found none");

                match mode {
                    SelectionMode::Replace | SelectionMode::Add => state.selected.clear(),
                    SelectionMode::Extend => {}
                }
            }
        }
    }
}

fn extract_wireframe_for_selected(editor: Extract<Res<Editor>>, mut commands: Commands) {
    let wireframe_for_selected = editor
        .window_state::<DebugSettingsWindow>()
        .map_or(false, |settings| settings.highlight_selected);

    if wireframe_for_selected {
        let selected = &editor.window_state::<HierarchyWindow>().unwrap().selected;
        for selected in selected.iter() {
            commands.get_or_spawn(selected).insert(Wireframe);
        }
    }
}

#[derive(Default)]
pub struct SelectedEntities {
    entities: Vec<Entity>,
}

pub enum SelectionMode {
    Replace,
    Add,
    Extend,
}

impl SelectionMode {
    pub fn from_ctrl_shift(ctrl: bool, shift: bool) -> SelectionMode {
        match (ctrl, shift) {
            (true, _) => SelectionMode::Add,
            (false, true) => SelectionMode::Extend,
            (false, false) => SelectionMode::Replace,
        }
    }
}

impl SelectedEntities {
    pub fn select<I: IntoIterator<Item = Entity>>(
        &mut self,
        mode: SelectionMode,
        entity: Entity,
        extend_with: impl Fn() -> I,
    ) {
        match (self.len(), mode) {
            (0, _) => {
                self.insert(entity);
            }
            (_, SelectionMode::Replace) => {
                self.insert_replace(entity);
            }
            (_, SelectionMode::Add) => {
                self.toggle(entity);
            }
            (_, SelectionMode::Extend) => {
                for entity in extend_with() {
                    self.insert(entity);
                }
            }
        }
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.entities.contains(&entity)
    }
    pub fn insert(&mut self, entity: Entity) {
        if !self.contains(entity) {
            self.entities.push(entity);
        }
    }

    pub fn insert_replace(&mut self, entity: Entity) {
        self.entities.clear();
        self.entities.push(entity);
    }

    pub fn toggle(&mut self, entity: Entity) {
        if self.remove(entity).is_none() {
            self.entities.push(entity);
        }
    }

    pub fn remove(&mut self, entity: Entity) -> Option<Entity> {
        if let Some(idx) = self.entities.iter().position(|&e| e == entity) {
            Some(self.entities.remove(idx))
        } else {
            None
        }
    }
    pub fn clear(&mut self) {
        self.entities.clear();
    }
    pub fn retain(&mut self, f: impl Fn(Entity) -> bool) {
        self.entities.retain(|entity| f(*entity));
    }
    pub fn len(&self) -> usize {
        self.entities.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entities.len() == 0
    }
    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entities.iter().copied()
    }
}

#[derive(Default)]
pub struct HierarchyState {
    pub selected: SelectedEntities,
    pub rename_info: (u64, bool, String),
}

struct Hierarchy<'a> {
    world: &'a mut World,
    state: &'a mut HierarchyState,
    add_state: Option<&'a AddWindowState>,
}

impl<'a> Hierarchy<'a> {
    fn show(&mut self, ui: &mut egui::Ui) {
        let mut root_query = self
            .world
            .query_filtered::<Entity, (Without<Parent>, Without<HideInEditor>)>();

        let always_open: HashSet<Entity> = self
            .state
            .selected
            .iter()
            .flat_map(|selected| {
                std::iter::successors(Some(selected), |&entity| {
                    self.world.get::<Parent>(entity).map(|parent| parent.get())
                })
                .skip(1)
            })
            .collect();

        let mut entities: Vec<_> = root_query.iter(self.world).collect();
        entities.sort();

        for entity in entities {
            self.entity_ui(entity, ui, &always_open);
        }
    }

    fn entity_ui(&mut self, entity: Entity, ui: &mut egui::Ui, always_open: &HashSet<Entity>) {
        let selected = self.state.selected.contains(entity);

        let entity_name = bevy_inspector_egui::world_inspector::entity_name(self.world, entity);
        let mut text = RichText::new(entity_name.clone());
        if selected {
            text = text.strong();
        }

        let has_children = self
            .world
            .get::<Children>(entity)
            .map_or(false, |children| children.len() > 0);

        let open = if !has_children {
            Some(false)
        } else if always_open.contains(&entity) {
            Some(true)
        } else {
            None
        };

        let (renamed_entity_bits, renaming, current_rename) = &mut self.state.rename_info;
        if *renaming && *renamed_entity_bits == entity.to_bits() {
            rename_entity_ui(ui, entity, current_rename, renaming, self.world);

            return;
        }

        #[allow(deprecated)]
        // no idea how to do this with CollapsingState::show_header, TODO figure out
        let response = CollapsingHeader::new(text)
            .id_source(entity)
            .selectable(true)
            .selected(selected)
            .open(open)
            .show(ui, |ui| {
                let children = self.world.get::<Children>(entity);
                if let Some(children) = children {
                    let children = children.to_vec();
                    ui.label("Children");
                    for &child in children.iter() {
                        self.entity_ui(child, ui, always_open);
                    }
                } else {
                    ui.label("No children");
                }
            });

        let mut despawn = false;
        let mut despawn_recursive = false;
        let header_response = response.header_response.context_menu(|ui| {
            if ui.button("Despawn").clicked() {
                despawn_recursive = true;
            }

            if ui.button("Remove keeping children").clicked() {
                despawn = true;
            }

            if ui.button("Rename").clicked() {
                self.state.rename_info = (entity.to_bits(), true, entity_name);
                ui.close_menu();
            }

            if let Some(add_state) = self.add_state {
                ui.menu_button("Add", |ui| {
                    if let Some(add_item) = add_ui(ui, add_state) {
                        add_item.add_to_entity(self.world, entity);
                        ui.close_menu();
                    }
                });
            }
        });

        if selected && ui.input().key_pressed(egui::Key::Delete) {
            despawn_recursive = true;
        }

        if despawn_recursive {
            despawn_with_children_recursive(self.world, entity);
        }
        if despawn {
            for entity in self.state.selected.iter() {
                if let Some(parent) = self.world.get::<Parent>(entity) {
                    if let Some(mut children) = self.world.get_mut::<Children>(parent.get()) {
                        let new_children: Vec<_> = children
                            .iter()
                            .copied()
                            .filter(|&child| child != entity)
                            .collect();
                        *children = Children::with(new_children.as_slice());
                    }
                }

                self.world.despawn(entity);
            }
            self.state.selected.clear();
        }

        if header_response.clicked() {
            let selection_mode = SelectionMode::from_ctrl_shift(
                ui.input().modifiers.ctrl,
                ui.input().modifiers.shift,
            );
            let extend_with = || std::iter::once(entity); // TODO implement extending selection
            self.state
                .selected
                .select(selection_mode, entity, extend_with);
        }
    }
}

fn rename_entity_ui(
    ui: &mut egui::Ui,
    entity: Entity,
    current_rename: &mut String,
    renaming: &mut bool,
    world: &mut World,
) {
    use egui::epaint::text::cursor::CCursor;
    use egui::widgets::text_edit::{CCursorRange, TextEdit, TextEditOutput};

    let id = egui::Id::new(entity);

    let edit = TextEdit::singleline(current_rename).id(id);
    let TextEditOutput {
        response,
        state: mut edit_state,
        ..
    } = edit.show(ui);

    // Runs once to end renaming
    if response.lost_focus() {
        *renaming = false;

        match world.get_entity_mut(entity) {
            Some(mut ent_mut) => match ent_mut.get_mut::<Name>() {
                Some(mut name) => {
                    name.set(current_rename.clone());
                }
                None => {
                    ent_mut.insert(Name::new(current_rename.clone()));
                }
            },
            None => {
                error!("Failed to get renamed entity");
            }
        }
    }

    // Runs once when renaming begins
    if !response.has_focus() {
        response.request_focus();

        edit_state.set_ccursor_range(Some(CCursorRange::two(
            CCursor::new(0),
            CCursor::new(current_rename.len()),
        )));
    }

    TextEdit::store_state(ui.ctx(), id, edit_state);
}
