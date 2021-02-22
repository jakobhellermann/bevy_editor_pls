use bevy::prelude::*;
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_mod_picking::{pick_labels::MESH_FOCUS, InteractablePickingPlugin, PickingPlugin, PickingPluginState};

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
            app.add_plugin(PickingPlugin).add_plugin(InteractablePickingPlugin);
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

pub type ExclusiveAccessFn = Box<dyn Fn(&mut World, &mut Resources) + Send + Sync + 'static>;

pub struct EditorSettings {
    pub(crate) events_to_send: Vec<(String, ExclusiveAccessFn)>,
    pub(crate) state_transition_handlers: Vec<(String, ExclusiveAccessFn)>,
    pub click_to_inspect: bool,
}
impl Default for EditorSettings {
    fn default() -> Self {
        EditorSettings {
            events_to_send: Default::default(),
            state_transition_handlers: Default::default(),
            click_to_inspect: false,
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
            let mut events = resources
                .get_mut::<Events<T>>()
                .unwrap_or_else(|| panic!("no resource for Events<{}>", std::any::type_name::<T>()));
            events.send(get_event());
        });

        self.events_to_send.push((name.to_string(), f));
    }

    pub fn add_state<S: Resource + Clone>(&mut self, name: &'static str, state: S) {
        let f = Box::new(move |_: &mut World, resources: &mut Resources| {
            let mut events = resources.get_mut::<State<S>>().unwrap();
            if let Err(e) = events.set_next(state.clone()) {
                warn!("{}", e);
            }
        });

        self.state_transition_handlers.push((name.to_string(), f));
    }
}
