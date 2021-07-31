use bevy::{
    app::Events,
    diagnostic::{Diagnostic, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy::{diagnostic::Diagnostics, render::wireframe::WireframeConfig};
use bevy_fly_camera::FlyCamera;
use bevy_orbit_controls::OrbitCamera;

use crate::{plugin::EditorState, EditorSettings};
use bevy_inspector_egui::{
    bevy_egui::EguiContext,
    egui::{self, menu},
    options::EntityAttributes,
    Context, Inspectable, WorldInspectorParams,
};

pub(crate) enum EditorMenuEvent {
    EnableFlyCams(bool),
    DisableOrbitCam,
}

pub(crate) fn handle_menu_event(
    mut commands: Commands,
    mut events: EventReader<EditorMenuEvent>,
    mut flycam_query: Query<&mut FlyCamera>,
    orbit_cam_query: Query<Entity, With<OrbitCamera>>,
) {
    for event in events.iter() {
        match *event {
            EditorMenuEvent::EnableFlyCams(enabled) => {
                for mut cam in flycam_query.iter_mut() {
                    cam.enabled = enabled;
                }
            }
            EditorMenuEvent::DisableOrbitCam => {
                orbit_cam_query.for_each(|entity| drop(commands.entity(entity).remove::<OrbitCamera>()))
            }
        }
    }
}

pub(crate) fn menu_system(world: &mut World) {
    let world = world.cell();
    let mut menu_events = world.get_resource_mut::<Events<EditorMenuEvent>>().unwrap();
    let egui_context = world.get_resource::<EguiContext>().unwrap();
    let mut editor_settings = world.get_resource_mut::<EditorSettings>().unwrap();
    let mut inspector_params = world.get_resource_mut::<WorldInspectorParams>().unwrap();
    let mut wireframe_config = world.get_resource_mut::<WireframeConfig>();
    let diagnostics = world.get_resource::<Diagnostics>().unwrap();

    if inspector_params.window != editor_settings.window {
        inspector_params.window = editor_settings.window;
    }

    if !editor_settings.display_ui {
        return;
    }

    let frame_time_diagnostics = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS).is_some();

    let ctx = match egui_context.try_ctx_for_window(editor_settings.window) {
        Some(ctx) => ctx,
        None => return,
    };
    egui::TopBottomPanel::top("editor_pls top panel").show(ctx, |ui| {
        menu::bar(ui, |ui| {
            menu::menu(ui, "Editor", |ui| {
                egui::Grid::new("inspector settings").show(ui, |ui| {
                    checkbox(ui, &mut inspector_params.enabled, "World Inspector");
                    ui.end_row();
                    checkbox(ui, &mut editor_settings.click_to_inspect, "Click to inspect");
                    ui.end_row();

                    if let Some(wireframe_config) = &mut wireframe_config {
                        checkbox(ui, &mut wireframe_config.global, "Wireframes");
                        ui.end_row();
                    }

                    if checkbox_changed(ui, &mut editor_settings.fly_camera, "Fly camera") {
                        menu_events.send(EditorMenuEvent::EnableFlyCams(editor_settings.fly_camera));
                    }
                    ui.end_row();

                    if checkbox_changed(ui, &mut editor_settings.orbit_camera, "Orbit camera") {
                        if !editor_settings.orbit_camera {
                            menu_events.send(EditorMenuEvent::DisableOrbitCam);
                        }
                    }
                    ui.end_row();

                    if frame_time_diagnostics {
                        checkbox(ui, &mut editor_settings.performance_panel, "Performance Panel");
                    }

                    ui.end_row();
                });
            });

            for (menu_title, items) in &mut editor_settings.menu_items {
                menu::menu(ui, *menu_title, |ui| {
                    for (_, state, ui_fn) in items.iter_mut() {
                        ui_fn(ui, &mut **state, &world);
                    }
                });
            }
        });
    });
}

pub(crate) fn performance_panel(
    egui_context: Res<EguiContext>,
    mut editor_settings: ResMut<EditorSettings>,
    diagnostics: ResMut<Diagnostics>,
) {
    if !editor_settings.performance_panel {
        return;
    };

    let diagnostics = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .and_then(Diagnostic::average)
        .zip(
            diagnostics
                .get(FrameTimeDiagnosticsPlugin::FRAME_TIME)
                .and_then(Diagnostic::average),
        );
    let (fps, frame_time) = match diagnostics {
        Some(value) => value,
        None => {
            warn!("Add the `FrameTimeDiagnosticsPlugin` to see the performance editor panel.");
            return;
        }
    };

    let ctx = match egui_context.try_ctx_for_window(editor_settings.window) {
        Some(ctx) => ctx,
        None => return,
    };

    egui::Window::new("Performance")
        .open(&mut editor_settings.performance_panel)
        .resizable(false)
        .show(ctx, |ui| {
            egui::Grid::new("frame time diagnostics").show(ui, |ui| {
                ui.label("FPS");
                ui.label(format!("{:.2}", fps));
                ui.end_row();
                ui.label("Frame Time");
                ui.label(format!("{:.4}", frame_time));
                ui.end_row();
            });
        });
}

pub(crate) fn currently_inspected_system(world: &mut World) {
    let world_ptr = world as *mut _;

    let mut currently_inspected = match world.get_resource_mut::<EditorState>().unwrap().currently_inspected {
        Some(entity) => entity,
        None => return,
    };
    let entity_exists = world.get_entity(currently_inspected).is_some();
    let parent = world.get::<Parent>(currently_inspected).map(|parent| parent.0);
    let mut go_to_parent = None;

    let name = entity_name(world, currently_inspected);

    let world_cell = world.cell();
    let egui_context = world_cell.get_resource::<EguiContext>().unwrap();
    let editor_settings = world_cell.get_resource_mut::<EditorSettings>().unwrap();
    let mut editor_state = world_cell.get_resource_mut::<EditorState>().unwrap();

    if !editor_settings.click_to_inspect {
        return;
    }

    if !entity_exists {
        editor_state.currently_inspected = None;
        return;
    }

    let ctx = match egui_context.try_ctx_for_window(editor_settings.window) {
        Some(ctx) => ctx,
        None => return,
    };

    let context = unsafe { Context::new_ptr(Some(ctx), world_ptr) };

    let mut is_open = true;
    egui::Window::new("Inspector")
        .open(&mut is_open)
        .id(egui::Id::new("editor inspector"))
        .show(ctx, |ui| {
            ui.scope(|ui| {
                ui.style_mut().visuals.override_text_color = Some(ui.style().visuals.widgets.hovered.text_color());
                ui.horizontal(|ui| {
                    ui.heading(name);

                    if let Some(parent) = parent {
                        if ui.heading("⬆").clicked() {
                            go_to_parent = Some(parent);
                        }
                    }
                });
            });

            ui.style_mut().wrap = Some(false);
            let options = EntityAttributes { despawnable: true };
            currently_inspected.ui(ui, options, &context);
        });

    if !is_open {
        editor_state.currently_inspected = None;
    }

    if let Some(entity) = go_to_parent {
        editor_state.currently_inspected = Some(entity);
    }
}

fn checkbox(ui: &mut egui::Ui, selected: &mut bool, text: &str) {
    if ui.selectable_label(false, text).clicked() {
        *selected = !*selected;
    }
    ui.scope(|ui| {
        let style = &mut ui.style_mut().visuals.widgets;
        style.inactive.bg_fill = style.active.bg_fill;
        ui.spacing_mut().icon_spacing = 0.0;
        ui.checkbox(selected, "");
    });
}

fn checkbox_changed(ui: &mut egui::Ui, selected: &mut bool, text: &str) -> bool {
    let before = *selected;
    checkbox(ui, selected, text);
    before != *selected
}
fn entity_name(world: &World, entity: Entity) -> String {
    match world.get::<Name>(entity) {
        Some(name) => name.as_str().to_string(),
        None => format!("Entity {}", entity.id()),
    }
}
