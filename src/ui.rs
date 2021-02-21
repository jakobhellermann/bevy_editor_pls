use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::EguiContext,
    egui::{self, menu},
    WorldInspectorParams,
};

use crate::{systems::EditorEvent, EditorSettings};

pub(crate) fn menu_system(
    egui_context: Res<EguiContext>,
    editor_settings: Res<EditorSettings>,
    mut editor_events: ResMut<Events<EditorEvent>>,
    mut inspector_params: ResMut<WorldInspectorParams>,
) {
    egui::TopPanel::top("editor-pls top panel").show(&egui_context.ctx, |ui| {
        menu::bar(ui, |ui| {
            menu::menu(ui, "Inspector", |ui| {
                ui.horizontal(|ui| checkbox(ui, &mut inspector_params.enabled, "World Inspector"));
            });

            if !editor_settings.events_to_send_order.is_empty() {
                menu::menu(ui, "Events", |ui| {
                    for (name, type_id) in &editor_settings.events_to_send_order {
                        if ui.button(name).clicked() {
                            editor_events.send(EditorEvent(*type_id));
                        }
                    }
                });
            }
        });
    });
}

fn checkbox(ui: &mut egui::Ui, selected: &mut bool, text: &str) {
    if ui.selectable_label(false, text).clicked() {
        *selected = !*selected;
    }
    ui.wrap(|ui| {
        let style = &mut ui.style_mut().visuals.widgets;
        style.inactive.bg_fill = style.active.bg_fill;
        ui.spacing_mut().icon_spacing = 0.0;
        ui.checkbox(selected, "");
    });
}
