use std::{any::TypeId, hash::Hash};

use bevy::{prelude::*, utils::StableHashMap};
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_mod_picking::{
    pick_labels::MESH_FOCUS, InteractablePickingPlugin, PickingPlugin, PickingPluginState,
};

use crate::{
    systems::EditorEvent,
    systems::{maintain_inspected_entities, send_editor_events},
    ui::{currently_inspected_system, menu_system},
};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // bevy-inspector-egui
        app.insert_resource(WorldInspectorParams {
            enabled: false,
            ..Default::default()
        })
        .add_plugin(WorldInspectorPlugin);

        // bevy-mod-picking
        if !app.resources().contains::<PickingPluginState>() {
            app.add_plugin(PickingPlugin)
                .add_plugin(InteractablePickingPlugin);
        };

        // resources
        app.init_resource::<EditorSettings>()
            .init_resource::<EditorState>()
            .add_event::<EditorEvent>();

        // systems
        app.add_system(menu_system.system());

        app.add_system(currently_inspected_system.exclusive_system());
        app.add_system(send_editor_events.exclusive_system());

        app.add_system_to_stage(
            CoreStage::PostUpdate,
            maintain_inspected_entities.system().after(MESH_FOCUS),
        );
    }
}

#[derive(Default)]
pub struct EditorState {
    pub currently_inspected: Option<Entity>,
}

type StableTypeMap<V> = StableHashMap<TypeId, V>;
pub type ExclusiveAccessFn = Box<dyn Fn(&mut World, &mut Resources) + Send + Sync + 'static>;

#[derive(Clone)]
enum Never {}

pub struct EditorSettings {
    pub(crate) events_to_send: StableTypeMap<(String, ExclusiveAccessFn)>,
    pub(crate) state_transition_handlers: StableHashMap<(TypeId, u64), (String, ExclusiveAccessFn)>,
    pub click_to_inspect: bool,
}
impl Default for EditorSettings {
    fn default() -> Self {
        EditorSettings {
            events_to_send: Default::default(),
            state_transition_handlers: Default::default(),
            click_to_inspect: true,
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

    pub fn add_state<S: Hash + Resource + Clone>(&mut self, name: &'static str, state: S) {
        let variant = hash_value(&state);

        let f = Box::new(move |_: &mut World, resources: &mut Resources| {
            let mut events = resources.get_mut::<State<S>>().unwrap();
            if let Err(e) = events.set_next(state.clone()) {
                warn!("{}", e);
            }
        });

        self.state_transition_handlers
            .insert((TypeId::of::<S>(), variant), (name.to_string(), f));
    }
}

fn hash_value(val: impl Hash) -> u64 {
    use std::hash::Hasher;
    let mut hasher = std::collections::hash_map::DefaultHasher::default();
    val.hash(&mut hasher);
    hasher.finish()
}
