use bevy::{app::Events, ecs::component::Component, prelude::*};
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_mod_picking::{pick_labels::MESH_FOCUS, InteractablePickingPlugin, PickingPlugin, PickingPluginState};

use crate::{
    systems::EditorEvent,
    systems::{maintain_inspected_entities, send_editor_events},
    ui::{currently_inspected_system, menu_system},
};

/// See the [crate-level docs](index.html) for usage
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // bevy-inspector-egui
        app.world_mut().get_resource_or_insert_with(|| WorldInspectorParams {
            enabled: false,
            ..Default::default()
        });
        app.add_plugin(WorldInspectorPlugin::new());

        // bevy-mod-picking
        if !app.world().contains_resource::<PickingPluginState>() {
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

pub type ExclusiveAccessFn = Box<dyn Fn(&mut World) + Send + Sync + 'static>;

/// Configuration for for editor
pub struct EditorSettings {
    pub(crate) events_to_send: Vec<(String, ExclusiveAccessFn)>,
    pub(crate) state_transition_handlers: Vec<(String, ExclusiveAccessFn)>,
    /// controls whether clicking meshes with a [PickableBundle](bevy_mod_picking::PickableBundle) opens the inspector
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
    /// Adds a event to the **Events** menu.
    /// When the menu item is clicked, the event provided by `get_event` will be sent.
    pub fn add_event<T, F>(&mut self, name: &'static str, get_event: F)
    where
        T: Component,
        F: Fn() -> T + Send + Sync + 'static,
    {
        let f = Box::new(move |world: &mut World| {
            let mut events = world
                .get_resource_mut::<Events<T>>()
                .unwrap_or_else(|| panic!("no resource for Events<{}>", std::any::type_name::<T>()));
            events.send(get_event());
        });

        self.events_to_send.push((name.to_string(), f));
    }

    /// Adds an app to the **States** menu.
    /// When the menu item is clicked, the game will transition to that state.
    pub fn add_state<S: Component + Clone>(&mut self, name: &'static str, state: S) {
        let f = Box::new(move |world: &mut World| {
            let mut events = world.get_resource_mut::<State<S>>().unwrap();
            if let Err(e) = events.set_next(state.clone()) {
                warn!("{}", e);
            }
        });

        self.state_transition_handlers.push((name.to_string(), f));
    }
}
