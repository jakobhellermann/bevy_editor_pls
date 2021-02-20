use std::any::TypeId;

use bevy::{app::ManualEventReader, prelude::*, utils::HashMap};
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

        // resources
        app.init_resource::<EditorSettings>()
            .add_event::<EditorEvent>();

        // systems
        app.add_system(menu_system.system());
        app.add_system(send_editor_events.exclusive_system());
    }
}

pub struct EditorSettings {
    events_to_send: HashMap<TypeId, Box<dyn Fn(&mut Resources) + Send + Sync>>,
    events_to_send_order: Vec<(String, TypeId)>,
}
impl Default for EditorSettings {
    fn default() -> Self {
        EditorSettings {
            events_to_send: Default::default(),
            events_to_send_order: Default::default(),
        }
    }
}
impl EditorSettings {
    pub fn add_event<T, F>(&mut self, name: &str, get_event: F)
    where
        T: Resource,
        F: Fn() -> T + Send + Sync + 'static,
    {
        let f = move |resources: &mut Resources| {
            let mut events = resources.get_mut::<Events<T>>().unwrap();
            events.send(get_event());
        };

        self.events_to_send.insert(TypeId::of::<T>(), Box::new(f));
        assert!(!self.events_to_send_order.iter().any(|(n, _)| n == name));
        self.events_to_send_order
            .push((name.to_string(), TypeId::of::<T>()));
    }
}

struct EditorEvent(TypeId);

fn send_editor_events(_: &mut World, resources: &mut Resources) {
    let mut editor_settings = resources.get_mut::<EditorSettings>().unwrap();
    let editor_events = resources.get::<Events<EditorEvent>>().unwrap();
    let mut editor_event_reader = ManualEventReader::<EditorEvent>::default();

    let events_to_send = std::mem::take(&mut editor_settings.events_to_send);

    let fns: Vec<_> = editor_event_reader
        .iter(&editor_events)
        .map(|event| events_to_send.get(&event.0).unwrap())
        .collect();

    drop(editor_settings);
    drop(editor_events);
    drop(editor_event_reader);

    for f in fns {
        f(resources);
    }

    let mut editor_settings = resources.get_mut::<EditorSettings>().unwrap();
    editor_settings.events_to_send = events_to_send;
}

fn menu_system(
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

            menu::menu(ui, "Events", |ui| {
                for (name, type_id) in &editor_settings.events_to_send_order {
                    if ui.button(name).clicked() {
                        editor_events.send(EditorEvent(*type_id));
                    }
                }
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
