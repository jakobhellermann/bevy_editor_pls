use std::any::TypeId;

use bevy::{prelude::*, utils::HashMap};
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};

use crate::{systems::send_editor_events, systems::EditorEvent, ui::menu_system};

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
    pub(crate) events_to_send: HashMap<TypeId, Box<dyn Fn(&mut Resources) + Send + Sync>>,
    pub(crate) events_to_send_order: Vec<(String, TypeId)>,
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
        let send_event_function = move |resources: &mut Resources| {
            let mut events = resources.get_mut::<Events<T>>().unwrap();
            events.send(get_event());
        };

        self.events_to_send
            .insert(TypeId::of::<T>(), Box::new(send_event_function));
        assert!(!self.events_to_send_order.iter().any(|(n, _)| n == name));
        self.events_to_send_order
            .push((name.to_string(), TypeId::of::<T>()));
    }
}
