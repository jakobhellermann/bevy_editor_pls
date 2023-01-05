use bevy::{
    math::{IVec2, Vec2},
    prelude::{AppTypeRegistry, World},
    reflect::TypeRegistryInternal,
    window::{PresentMode, WindowId, WindowMode, Windows},
};
use bevy_editor_pls_core::editor_window::{EditorWindow, EditorWindowContext};
use bevy_inspector_egui::{
    egui,
    inspector_options::std_options::NumberOptions,
    reflect_inspector::{Context, InspectorUi},
};

pub struct WindowsWindow;

impl EditorWindow for WindowsWindow {
    type State = ();

    const NAME: &'static str = "Windows";

    fn ui(world: &mut World, _: EditorWindowContext, ui: &mut egui::Ui) {
        let type_registry = world.resource::<AppTypeRegistry>().clone();
        let type_registry = type_registry.read();

        let mut windows = world.get_resource_mut::<Windows>().unwrap();
        window_ui(&mut *windows, ui, &type_registry);
    }
}

fn window_ui(windows: &mut Windows, ui: &mut egui::Ui, type_registry: &TypeRegistryInternal) {
    let mut context = Context::default();
    let mut env = InspectorUi::new_no_short_circuit(&type_registry, &mut context);
    let id = egui::Id::null();

    for window in windows.iter_mut() {
        let name = match window.id() {
            id if id == WindowId::primary() => "Primary",
            _ => window.title(),
        };

        ui.heading(name);

        egui::Grid::new(window.id()).show(ui, |ui| {
            ui.label("mode");
            let mut mode = window.mode();
            let mut mode_changed = false;
            egui::ComboBox::from_id_source("window_mode")
                .selected_text(format!("{:?}", mode))
                .show_ui(ui, |ui| {
                    mode_changed |= ui
                        .selectable_value(
                            &mut mode,
                            WindowMode::BorderlessFullscreen,
                            "BorderlessFullscreen",
                        )
                        .changed();
                    mode_changed |= ui
                        .selectable_value(&mut mode, WindowMode::Fullscreen, "Fullscreen")
                        .changed();
                    mode_changed |= ui
                        .selectable_value(&mut mode, WindowMode::SizedFullscreen, "SizedFullscreen")
                        .changed();
                    mode_changed |= ui
                        .selectable_value(&mut mode, WindowMode::Windowed, "Windowed")
                        .changed();
                });
            if mode_changed {
                window.set_mode(mode);
            }
            ui.end_row();

            ui.label("present_mode");
            let mut present_mode = window.present_mode();
            let mut present_mode_changed = false;
            egui::ComboBox::from_id_source("present_mode")
                .selected_text(format!("{:?}", present_mode))
                .show_ui(ui, |ui| {
                    present_mode_changed |= ui
                        .selectable_value(&mut present_mode, PresentMode::Fifo, "Fifo")
                        .changed();
                    present_mode_changed |= ui
                        .selectable_value(&mut present_mode, PresentMode::Immediate, "Immediate")
                        .changed();
                    present_mode_changed |= ui
                        .selectable_value(&mut present_mode, PresentMode::Mailbox, "Mailbox")
                        .changed();
                });
            if present_mode_changed {
                window.set_present_mode(present_mode);
            }
            ui.end_row();

            ui.label("size");
            let mut size = Vec2::new(window.width(), window.height());
            let mut size_attributes = NumberOptions::at_least(Vec2::ONE);
            size_attributes.speed = 1.0;
            if env.ui_for_reflect_with_options(&mut size, ui, id, &size_attributes) {
                window.set_resolution(size.x, size.y);
            }
            ui.end_row();

            ui.label("position");
            let mut position = window.position().unwrap_or_default();
            if env.ui_for_reflect_with_options(
                &mut position,
                ui,
                id,
                &NumberOptions::at_least(IVec2::ZERO),
            ) {
                window.set_position(bevy::window::MonitorSelection::Current, position);
            }
            ui.end_row();

            ui.label("scale_factor_override");
            let mut scale_factor_override = window.scale_factor();
            let mut scale_factor_attrs = NumberOptions::default();
            scale_factor_attrs.min = Some(0.01);
            scale_factor_attrs.speed = 0.001;
            if env.ui_for_reflect_with_options(
                &mut scale_factor_override,
                ui,
                id,
                &scale_factor_attrs,
            ) {
                window.set_scale_factor_override(Some(scale_factor_override));
            }
            ui.end_row();

            ui.label("title");
            let mut title = window.title().to_string();
            if env.ui_for_reflect(&mut title, ui) {
                window.set_title(title);
            }
            ui.end_row();

            ui.label("resizable");
            let mut resizable = window.resizable();
            if env.ui_for_reflect(&mut resizable, ui) {
                window.set_resizable(resizable);
            }
            ui.end_row();

            ui.label("cursor_visible");
            let mut cursor_visible = window.cursor_visible();
            if env.ui_for_reflect(&mut cursor_visible, ui) {
                window.set_cursor_visibility(cursor_visible);
            }
            ui.end_row();

            // TODO: reenable
            /*ui.label("cursor_locked");
            let mut cursor_locked = window.cursor_grab_mode();
            if cursor_locked.ui(ui, Default::default(), &mut cx) {
                window.set_cursor_grab_mode(cursor_locked);
            }*/
            ui.end_row();
        });
    }
}
