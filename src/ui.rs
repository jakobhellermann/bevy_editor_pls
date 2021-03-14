use bevy::render::wireframe::WireframeConfig;
use bevy::{app::Events, prelude::*};
use bevy_fly_camera::FlyCamera;

use crate::{plugin::EditorState, EditorSettings};
use bevy_inspector_egui::{
    bevy_egui::EguiContext,
    egui::{self, menu},
    options::EntityAttributes,
    Context, Inspectable, WorldInspectorParams,
};

pub(crate) enum EditorMenuEvent {
    EnableFlyCams(bool),
}

pub(crate) fn handle_menu_event(mut events: EventReader<EditorMenuEvent>, mut flycam_query: Query<&mut FlyCamera>) {
    for event in events.iter() {
        match *event {
            EditorMenuEvent::EnableFlyCams(enabled) => {
                for mut cam in flycam_query.iter_mut() {
                    cam.enabled = enabled;
                }
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

    egui::TopPanel::top("editor_pls top panel").show(&egui_context.ctx, |ui| {
        menu::bar(ui, |ui| {
            menu::menu(ui, "Inspector", |ui| {
                egui::Grid::new("inspector settings").show(ui, |ui| {
                    checkbox(ui, &mut inspector_params.enabled, "World Inspector");
                    ui.end_row();
                    checkbox(ui, &mut editor_settings.click_to_inspect, "Click to inspect");
                    ui.end_row();

                    if let Some(wireframe_config) = &mut wireframe_config {
                        checkbox(ui, &mut wireframe_config.global, "Wireframes");
                        ui.end_row();
                    }

                    let flycam_before = editor_settings.fly_camera;
                    checkbox(ui, &mut editor_settings.fly_camera, "Fly camera");
                    ui.end_row();
                    let flycam_after = editor_settings.fly_camera;
                    if flycam_before != flycam_after {
                        menu_events.send(EditorMenuEvent::EnableFlyCams(flycam_after));
                    }
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

pub(crate) fn currently_inspected_system(world: &mut World) {
    let world_ptr = world as *mut _;

    let mut currently_inspected = match world.get_resource_mut::<EditorState>().unwrap().currently_inspected {
        Some(entity) => entity,
        None => return,
    };
    let name = entity_name(world, currently_inspected);

    let world_cell = world.cell();
    let egui_context = world_cell.get_resource::<EguiContext>().unwrap();
    let editor_settings = world_cell.get_resource_mut::<EditorSettings>().unwrap();
    let mut editor_state = world_cell.get_resource_mut::<EditorState>().unwrap();

    if !editor_settings.click_to_inspect {
        return;
    }

    let context = unsafe { Context::new_ptr(&egui_context.ctx, world_ptr) };

    let mut is_open = true;
    egui::Window::new("Inspector")
        .open(&mut is_open)
        .id(egui::Id::new("editor inspector"))
        .show(&egui_context.ctx, |ui| {
            ui.wrap(|ui| {
                ui.style_mut().visuals.override_text_color = Some(ui.style().visuals.widgets.hovered.text_color());
                ui.heading(name);
            });

            ui.style_mut().wrap = Some(false);
            let options = EntityAttributes { despawnable: true };
            currently_inspected.ui(ui, options, &context);
        });

    if !is_open {
        editor_state.currently_inspected = None;
    }
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

fn entity_name(world: &World, entity: Entity) -> String {
    match world.get::<Name>(entity) {
        Some(name) => name.as_str().to_string(),
        None => format!("Entity {}", entity.id()),
    }
}
