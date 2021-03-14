use std::any::Any;

use bevy::{app::Events, ecs::world::WorldCell, render::wireframe::WireframeConfig, utils::StableHashMap};
use bevy::{ecs::component::Component, prelude::*};

use bevy_fly_camera::FlyCameraPlugin;
use bevy_inspector_egui::{egui, WorldInspectorParams, WorldInspectorPlugin};
use bevy_mod_picking::{pick_labels::MESH_FOCUS, InteractablePickingPlugin, PickingPlugin, PickingPluginState};

use crate::{systems, ui};

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
        app.add_system(systems::esc_cursor_grab.system());

        // resources
        app.init_resource::<EditorState>().add_event::<ui::EditorMenuEvent>();

        let show_wireframes = app
            .world_mut()
            .get_resource_or_insert_with(EditorSettings::default)
            .show_wireframes;
        if app.world().contains_resource::<WireframeConfig>() {
            app.world_mut().get_resource_mut::<WireframeConfig>().unwrap().global = show_wireframes;
        }

        // systems
        app.add_system(ui::menu_system.exclusive_system());
        app.add_system(ui::currently_inspected_system.exclusive_system());
        app.add_system(ui::handle_menu_event.system());

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

type UiFn = Box<dyn Fn(&mut egui::Ui, &mut dyn Any, &WorldCell) + Send + Sync>;

/// Configuration for for editor
pub struct EditorSettings {
    pub(crate) menu_items:
        StableHashMap<&'static str, Vec<(Option<&'static str>, Box<dyn Any + Send + Sync + 'static>, UiFn)>>,
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
            menu_items: Default::default(),
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
        EditorSettings::default()
    }

    /// Adds a event to the **Events** menu.
    /// When the menu item is clicked, the event provided by `get_event` will be sent.
    pub fn add_event<T: Component, F>(&mut self, name: &'static str, get_event: F)
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.add_menu_item("Events", move |ui, world| {
            let mut events = world.get_resource_mut::<Events<T>>().unwrap();
            if ui.button(name).clicked() {
                events.send(get_event());
            }
        });
    }

    /// Adds an app to the **States** menu.
    /// When the menu item is clicked, the game will transition to that state.
    pub fn add_state<S: Component + Clone>(&mut self, name: &'static str, state: S) {
        self.add_menu_item("States", move |ui, world| {
            let mut state_resource = world.get_resource_mut::<State<S>>().unwrap();
            if ui.button(name).clicked() {
                if let Err(e) = state_resource.set_next(state.clone()) {
                    warn!("{}", e);
                }
            }
        });
    }
}

impl EditorSettings {
    /// Displays `ui` in menu bar `menu`.
    pub(crate) fn add_menu_item<U>(&mut self, menu: &'static str, ui: U)
    where
        U: Fn(&mut egui::Ui, &WorldCell) + Send + Sync + 'static,
    {
        self.add_menu_item_stateful(menu, None, (), move |_ui, _, world| ui(_ui, world))
    }

    /// Displays `ui` in menu bar `menu` with access to some state.
    ///
    /// The state can later be retrieved using [EditorSettings::menu_state] and [EditorSettings::menu_state_mut].
    pub(crate) fn add_menu_item_stateful<S, U>(
        &mut self,
        menu: &'static str,
        state_label: Option<&'static str>,
        state: S,
        ui: U,
    ) where
        S: Send + Sync + 'static,
        U: Fn(&mut egui::Ui, &mut S, &WorldCell) + Send + Sync + 'static,
    {
        const INVALID_NAMES: &[&str] = &["Inspector"];
        assert!(!INVALID_NAMES.contains(&menu), "can't use menu name `{}`", menu);

        let ui_fn: UiFn = Box::new(move |_ui, state, world| {
            let state = state.downcast_mut::<S>().unwrap();
            ui(_ui, state, world);
        });
        let items = self.menu_items.entry(menu).or_default();
        items.push((state_label, Box::new(state), ui_fn));
    }


    #[rustfmt::skip]
    #[allow(unused)]
    pub(crate) fn menu_state<S: Send + Sync + 'static>(&self, name: &'static str, label: &'static str) -> &S {
        let items = self.menu_items.get(name).unwrap_or_else(|| panic!("no menu `{}` exists", name));
        let state = items
            .iter()
            .find_map(|(lbl, state, _)| (*lbl == Some(label)).then(|| state))
            .unwrap_or_else(|| panic!("no menu item `{}/{}`", name, label));
        state.downcast_ref().unwrap_or_else(|| panic!("menu state of `{}/{}` is not a {}", name, label, std::any::type_name::<S>()))
    }
    #[rustfmt::skip]
    #[allow(unused)]
    pub(crate) fn menu_state_mut<S: Send + Sync + 'static>(&mut self, name: &'static str, label: &'static str) -> &mut S {
        let items = self.menu_items.get_mut(name).unwrap_or_else(|| panic!("no menu {} exists", name));
        let state = items
            .iter_mut()
            .find_map(|(lbl, state, _)| (*lbl == Some(label)).then(|| state))
            .unwrap_or_else(|| panic!("no menu item `{}/{}`", name, label));
        state.downcast_mut().unwrap_or_else(|| panic!("menu state of `{}/{}` is not a {}", name, label, std::any::type_name::<S>()))
    }
}
