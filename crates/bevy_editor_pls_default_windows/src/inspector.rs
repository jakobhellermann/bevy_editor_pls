use std::any::TypeId;

use super::add::{AddWindow, AddWindowState};
use super::hierarchy::HierarchyWindow;
use bevy::asset::UntypedAssetId;
use bevy::prelude::{AppTypeRegistry, Entity, World};
use bevy::reflect::TypeRegistry;
use bevy_editor_pls_core::editor_window::{EditorWindow, EditorWindowContext};
use bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities;
use bevy_inspector_egui::{bevy_inspector, egui};

#[derive(Eq, PartialEq)]
pub enum InspectorSelection {
    Entities,
    Resource(TypeId, String),
    Asset(TypeId, String, UntypedAssetId),
}

pub struct InspectorState {
    pub selected: InspectorSelection,
}

impl Default for InspectorState {
    fn default() -> Self {
        Self {
            selected: InspectorSelection::Entities,
        }
    }
}

pub struct InspectorWindow;
impl EditorWindow for InspectorWindow {
    type State = InspectorState;
    const NAME: &'static str = "Inspector";

    fn ui(world: &mut World, cx: EditorWindowContext, ui: &mut egui::Ui) {
        let type_registry = world.resource::<AppTypeRegistry>().0.clone();
        let type_registry = type_registry.read();

        let selected = &cx.state::<Self>().unwrap().selected;
        let selected_entities = &cx.state::<HierarchyWindow>().unwrap().selected;

        let add_window_state = cx.state::<AddWindow>();
        inspector(
            world,
            selected,
            selected_entities,
            ui,
            add_window_state,
            &type_registry,
        );
    }
}

fn inspector(
    world: &mut World,
    selected: &InspectorSelection,
    selected_entities: &SelectedEntities,
    ui: &mut egui::Ui,
    add_window_state: Option<&AddWindowState>,
    type_registry: &TypeRegistry,
) {
    egui::ScrollArea::vertical().show(ui, |ui| match *selected {
        InspectorSelection::Entities => match selected_entities.as_slice() {
            [] => {
                ui.label("No entity selected");
            }
            &[entity] => {
                bevy_inspector::ui_for_entity(world, entity, ui);
                add_ui(ui, &[entity], world, add_window_state);
            }
            entities => {
                bevy_inspector::ui_for_entities_shared_components(world, entities, ui);
                add_ui(ui, entities, world, add_window_state);
            }
        },
        InspectorSelection::Resource(type_id, ref name) => {
            ui.label(name);
            bevy_inspector::by_type_id::ui_for_resource(world, type_id, ui, name, type_registry)
        }
        InspectorSelection::Asset(type_id, ref name, handle) => {
            ui.label(name);
            bevy_inspector::by_type_id::ui_for_asset(world, type_id, handle, ui, type_registry);
        }
    });
}

fn add_ui(
    ui: &mut egui::Ui,
    entities: &[Entity],
    world: &mut World,
    add_window_state: Option<&AddWindowState>,
) {
    if let Some(add_window_state) = add_window_state {
        let layout = egui::Layout::top_down(egui::Align::Center).with_cross_justify(true);
        ui.with_layout(layout, |ui| {
            ui.menu_button("+", |ui| {
                if let Some(add_item) = crate::add::add_ui(ui, add_window_state) {
                    for entity in entities {
                        add_item.add_to_entity(world, *entity);
                    }
                }
            });
        });
    }
}

pub fn label_button(ui: &mut egui::Ui, text: &str, text_color: egui::Color32) -> bool {
    ui.add(egui::Button::new(egui::RichText::new(text).color(text_color)).frame(false))
        .clicked()
}
