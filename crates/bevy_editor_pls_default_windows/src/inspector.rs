use super::add::{AddWindow, AddWindowState};
use super::hierarchy::HierarchyWindow;
use bevy::prelude::{Entity, World};
use bevy_editor_pls_core::editor_window::{EditorWindow, EditorWindowContext};
use bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities;
use bevy_inspector_egui::egui;

pub struct InspectorWindow;
impl EditorWindow for InspectorWindow {
    type State = ();
    const NAME: &'static str = "Inspector";

    fn ui(world: &mut World, cx: EditorWindowContext, ui: &mut egui::Ui) {
        let inspected = &cx.state::<HierarchyWindow>().unwrap().selected;

        let add_window_state = cx.state::<AddWindow>();
        inspector(world, inspected, ui, add_window_state);
    }
}

fn inspector(
    world: &mut World,
    inspected: &SelectedEntities,
    ui: &mut egui::Ui,
    add_window_state: Option<&AddWindowState>,
) {
    if inspected.is_empty() {
        ui.label("No entity selected");
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        match inspected.len() {
            1 => {
                let entity = inspected.iter().next().unwrap();
                bevy_inspector_egui::bevy_inspector::ui_for_entity(world, entity, ui, false);
                add_ui(ui, entity, world, add_window_state);
            }
            _ => {
                ui.label("Inspector for multiple entities not yet implemented.");
            }
        };
    });
}

fn add_ui(
    ui: &mut egui::Ui,
    entity: Entity,
    world: &mut World,
    add_window_state: Option<&AddWindowState>,
) {
    if let Some(add_window_state) = add_window_state {
        let layout = egui::Layout::top_down(egui::Align::Center).with_cross_justify(true);
        ui.with_layout(layout, |ui| {
            ui.menu_button("+", |ui| {
                if let Some(add_item) = crate::add::add_ui(ui, add_window_state) {
                    add_item.add_to_entity(world, entity);
                }
            });
        });
    }
}

pub fn label_button(ui: &mut egui::Ui, text: &str, text_color: egui::Color32) -> bool {
    ui.add(egui::Button::new(egui::RichText::new(text).color(text_color)).frame(false))
        .clicked()
}
