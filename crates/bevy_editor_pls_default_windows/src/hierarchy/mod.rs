pub mod picking;

use bevy::ecs::entity::Entities;
use bevy::ecs::query::QuerySingleError;
use bevy::pbr::wireframe::Wireframe;
use bevy::prelude::*;
use bevy::reflect::TypeRegistryInternal;
use bevy::render::{Extract, RenderApp, RenderStage};
use bevy_editor_pls_core::EditorState;
use bevy_inspector_egui::bevy_inspector::guess_entity_name;
use bevy_inspector_egui::bevy_inspector::hierarchy::{SelectedEntities, SelectionMode};
use bevy_inspector_egui::egui::{self, ScrollArea};

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
                let type_registry = world.resource::<AppTypeRegistry>().clone();
                let type_registry = type_registry.read();
                Hierarchy {
                    world,
                    state: hierarchy_state,
                    type_registry: &type_registry,
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
                    source.and_then(|source| {
                        source.get_nearest_intersection().map(|(entity, _)| entity)
                    })
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
                    .and_then(|source| source.get_nearest_intersection())
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
                    .select(mode, entity, |_, _| std::iter::once(entity));
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
pub struct HierarchyState {
    pub selected: SelectedEntities,
    rename_info: Option<RenameInfo>,
}

pub struct RenameInfo {
    entity: Entity,
    renaming: bool,
    current_rename: String,
}

struct Hierarchy<'a> {
    world: &'a mut World,
    state: &'a mut HierarchyState,
    type_registry: &'a TypeRegistryInternal,
    add_state: Option<&'a AddWindowState>,
}

impl<'a> Hierarchy<'a> {
    fn show(&mut self, ui: &mut egui::Ui) {
        let mut despawn_recursive = None;
        let mut despawn = None;

        let HierarchyState {
            selected,
            rename_info,
        } = self.state;

        bevy_inspector_egui::bevy_inspector::hierarchy::Hierarchy {
            extra_state: rename_info,
            world: self.world,
            type_registry: self.type_registry,
            selected,
            context_menu: Some(&mut |ui, entity, world, rename_info| {
                if ui.button("Despawn").clicked() {
                    despawn_recursive = Some(entity);
                }

                if ui.button("Remove keeping children").clicked() {
                    despawn = Some(entity);
                }

                if ui.button("Rename").clicked() {
                    let entity_name = guess_entity_name(world, self.type_registry, entity);
                    *rename_info = Some(RenameInfo {
                        entity,
                        renaming: true,
                        current_rename: entity_name,
                    });
                    ui.close_menu();
                }

                if let Some(add_state) = self.add_state {
                    ui.menu_button("Add", |ui| {
                        if let Some(add_item) = add_ui(ui, add_state) {
                            add_item.add_to_entity(world, entity);
                            ui.close_menu();
                        }
                    });
                }
            }),
            shortcircuit_entity: Some(&mut |ui, entity, world, rename_info| {
                if let Some(rename_info) = rename_info {
                    if rename_info.renaming && rename_info.entity == entity {
                        rename_entity_ui(ui, rename_info, world);

                        return true;
                    }
                }

                false
            }),
        }
        .show::<Without<HideInEditor>>(ui);

        if let Some(entity) = despawn_recursive {
            bevy::hierarchy::despawn_with_children_recursive(self.world, entity);
        }
        if let Some(entity) = despawn {
            self.world.entity_mut(entity).despawn();
            self.state.selected.remove(entity);
        }

        if ui.input().key_pressed(egui::Key::Delete) {
            for entity in self.state.selected.iter() {
                self.world.entity_mut(entity).despawn_recursive();
            }
            self.state.selected.clear();
        }
    }
}

fn rename_entity_ui(ui: &mut egui::Ui, rename_info: &mut RenameInfo, world: &mut World) {
    use egui::epaint::text::cursor::CCursor;
    use egui::widgets::text_edit::{CCursorRange, TextEdit, TextEditOutput};

    let id = egui::Id::new(rename_info.entity);

    let edit = TextEdit::singleline(&mut rename_info.current_rename).id(id);
    let TextEditOutput {
        response,
        state: mut edit_state,
        ..
    } = edit.show(ui);

    // Runs once to end renaming
    if response.lost_focus() {
        rename_info.renaming = false;

        match world.get_entity_mut(rename_info.entity) {
            Some(mut ent_mut) => match ent_mut.get_mut::<Name>() {
                Some(mut name) => {
                    name.set(rename_info.current_rename.clone());
                }
                None => {
                    ent_mut.insert(Name::new(rename_info.current_rename.clone()));
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
            CCursor::new(rename_info.current_rename.len()),
        )));
    }

    TextEdit::store_state(ui.ctx(), id, edit_state);
}
