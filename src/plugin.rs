use std::any::TypeId;

use bevy::{prelude::*, utils::StableHashMap};
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

type StableTypeMap<V> = StableHashMap<TypeId, V>;
pub type ExclusiveAccessFn = Box<dyn Fn(&mut World, &mut Resources) + Send + Sync + 'static>;

pub struct EditorSettings {
    pub(crate) events_to_send: StableTypeMap<(String, ExclusiveAccessFn)>,
}
impl Default for EditorSettings {
    fn default() -> Self {
        EditorSettings {
            events_to_send: Default::default(),
        }
    }
}
impl EditorSettings {
    pub fn add_event<T, F>(&mut self, name: &'static str, get_event: F)
    where
        T: Resource,
        F: Fn() -> T + Send + Sync + 'static,
    {
        let f = Box::new(move |_: &mut World, resources: &mut Resources| {
            let mut events = resources.get_mut::<Events<T>>().unwrap();
            events.send(get_event());
        });

        self.events_to_send
            .insert(TypeId::of::<T>(), (name.to_string(), f));
    }
}
