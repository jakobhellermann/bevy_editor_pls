use bevy::{
    math::{IVec2, Vec2},
    prelude::{World, warn},
    window::{PresentMode, WindowId, WindowMode, Windows, MonitorSelection, CursorGrabMode},
};
use bevy_editor_pls_core::editor_window::{EditorWindow, EditorWindowContext};
use bevy_inspector_egui::{
    egui,
    options::{NumberAttributes, Vec2dAttributes},
    Inspectable,
};

pub struct WindowsWindow;

impl EditorWindow for WindowsWindow {
    type State = ();

    const NAME: &'static str = "Windows";

    fn ui(world: &mut World, _: EditorWindowContext, ui: &mut egui::Ui) {
        let mut windows = world.get_resource_mut::<Windows>().unwrap();
        window_ui(&mut *windows, ui);
    }
}

fn window_ui(windows: &mut Windows, ui: &mut egui::Ui) {
    let mut cx = bevy_inspector_egui::Context::new_shared(None);

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
            let mut size_attributes = Vec2dAttributes::positive();
            size_attributes.speed = 1.0;
            if size.ui(ui, size_attributes, &mut cx) {
                window.set_resolution(size.x, size.y);
            }
            ui.end_row();

            ui.label("position");
            let mut position = window.position().unwrap_or_default();
            if position.ui(ui, NumberAttributes::min(IVec2::ZERO), &mut cx) {
                window.set_position(MonitorSelection::Current,  position);
            }
            ui.end_row();

            ui.label("scale_factor_override");
            let mut scale_factor_override = window.scale_factor();
            let scale_factor_attrs = NumberAttributes {
                min: Some(0.01),
                speed: 0.001,
                ..Default::default()
            };
            if scale_factor_override.ui(ui, scale_factor_attrs, &mut cx) {
                window.set_scale_factor_override(Some(scale_factor_override));
            }
            ui.end_row();

            ui.label("title");
            let mut title = window.title().to_string();
            if title.ui(ui, Default::default(), &mut cx) {
                window.set_title(title);
            }
            ui.end_row();

            ui.label("resizable");
            let mut resizable = window.resizable();
            if resizable.ui(ui, Default::default(), &mut cx) {
                window.set_resizable(resizable);
            }
            ui.end_row();

            ui.label("cursor_visible");
            let mut cursor_visible = window.cursor_visible();
            if cursor_visible.ui(ui, Default::default(), &mut cx) {
                window.set_cursor_visibility(cursor_visible);
            }
            ui.end_row();

            ui.label("cursor_grabed");
            warn!("This needs bevy_inspector to add ui for CursorGrabMode just checks if its CursorGrabMode::Locked");
            let mut cursor_locked = window.cursor_grab_mode() == CursorGrabMode::Locked;
            if cursor_locked.ui(ui, Default::default(), &mut cx) {
                if cursor_locked {
                    window.set_cursor_grab_mode(CursorGrabMode::Locked);
                } else {
                    window.set_cursor_grab_mode(CursorGrabMode::None);
                }
            }
            ui.end_row();
        });
    }
}
