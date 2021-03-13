use bevy::{app::Events, render::wireframe::WireframeConfig};
use bevy::{ecs::component::Component, prelude::*};

use bevy_fly_camera::FlyCameraPlugin;
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_mod_picking::{pick_labels::MESH_FOCUS, InteractablePickingPlugin, PickingPlugin, PickingPluginState};

use crate::{systems, systems::EditorEvent, ui};

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

        // bevy_mod_picking
        if !app.world().contains_resource::<PickingPluginState>() {
            app.add_plugin(PickingPlugin).add_plugin(InteractablePickingPlugin);
        };

        // bevy_mod_flycamera
        app.add_plugin(FlyCameraPlugin);

        // resources
        app.init_resource::<EditorState>().add_event::<EditorEvent>();

        let show_wireframes = app
            .world_mut()
            .get_resource_or_insert_with(EditorSettings::default)
            .show_wireframes;
        if app.world().contains_resource::<WireframeConfig>() {
            app.world_mut().get_resource_mut::<WireframeConfig>().unwrap().global = show_wireframes;
        }

        // systems
        app.add_system(ui::menu_system.system());
        app.add_system(ui::currently_inspected_system.exclusive_system());

        app.add_system(systems::send_editor_events.exclusive_system());

        // auto add systems
        app.add_system(systems::make_everything_pickable.system());
        app.add_system(systems::make_camera_picksource.system());
        app.add_system(systems::make_cam_flycam.system());

        app.add_system_to_stage(
            CoreStage::PostUpdate,
            systems::maintain_inspected_entities.system().after(MESH_FOCUS),
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
    /// Whether clicking meshes with a [PickableBundle](bevy_mod_picking::PickableBundle) opens the inspector.
    /// Can be toggled in the editor UI.
    pub click_to_inspect: bool,
    /// Whether wireframe should be shown.
    /// Can be toggled in the editor UI.
    pub show_wireframes: bool,
    /// Whether the camera can be controlled with WASD.
    /// Can be toggled in the editor UI.
    pub fly_camera: bool,

    /// If enabled, [`PickableBundle`](bevy_mod_picking::PickableBundle) and [`PickingCameraBundle`](bevy_mod_picking::PickingCameraBundle) will automatically be added to your camera and objects
    pub auto_pickable: bool,
    /// If enabled, [`FlyCamera`](bevy_fly_camera::FlyCamera) will automatically be added to your camera
    pub auto_flycam: bool,
}
impl Default for EditorSettings {
    fn default() -> Self {
        EditorSettings {
            events_to_send: Default::default(),
            state_transition_handlers: Default::default(),
            click_to_inspect: false,
            show_wireframes: false,
            fly_camera: false,
            auto_pickable: false,
            auto_flycam: false,
        }
    }
}
impl EditorSettings {
    pub fn new() -> Self {
        Self::default()
    }

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
