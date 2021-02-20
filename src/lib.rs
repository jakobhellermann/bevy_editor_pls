use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::EguiContext, egui, WorldInspectorParams, WorldInspectorPlugin,
};
use egui::menu;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // bevy-inspector-egui
        app.insert_resource(WorldInspectorParams {
            enabled: false,
            ..Default::default()
        })
        .add_plugin(WorldInspectorPlugin);

        // ui systems
        app.add_system(menu_system.system());
    }
}

fn menu_system(egui_context: Res<EguiContext>, mut inspector_params: ResMut<WorldInspectorParams>) {
    egui::TopPanel::top("editor-pls top panel").show(&egui_context.ctx, |ui| {
        menu::bar(ui, |ui| {
            menu::menu(ui, "Inspector", |ui| {
                ui.horizontal(|ui| checkbox(ui, &mut inspector_params.enabled, "World Inspector"));
            });
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
