use std::{any::Any, path::Path};

use bevy::{app::Events, ecs::world::WorldCell, utils::StableHashMap, window::WindowId};
use bevy::{ecs::component::Component, prelude::*};

use bevy_inspector_egui::egui;

type UiFn = Box<dyn Fn(&mut egui::Ui, &mut dyn Any, &WorldCell) + Send + Sync>;
type DragAndDropHandler = Box<dyn Fn(&Path, &mut World) + Send + Sync>;

/// Configuration for for editor
pub struct EditorSettings {
    pub(crate) menu_items:
        StableHashMap<&'static str, Vec<(Option<&'static str>, Box<dyn Any + Send + Sync + 'static>, UiFn)>>,
    pub(crate) drag_and_drop_handlers: Vec<(&'static [&'static str], DragAndDropHandler)>,

    /// Whether clicking meshes with a [PickableBundle](bevy_mod_picking::PickableBundle) opens the inspector.
    /// Can be toggled in the editor UI.
    pub click_to_inspect: bool,
    /// Whether wireframe should be shown.
    /// Can be toggled in the editor UI.
    pub show_wireframes: bool,
    /// Whether the camera can be controlled with WASD.
    /// Can be toggled in the editor UI.
    pub fly_camera: bool,
    pub orbit_camera: bool,

    /// If enabled, [`PickableBundle`](bevy_mod_picking::PickableBundle) will be added to all meshes
    pub auto_pickable: bool,
    /// If enabled, adds [`PickingCameraBundle`](bevy_mod_picking::PickingCameraBundle) all cameras
    pub auto_pickable_camera: bool,
    /// If enabled, [`FlyCamera`](bevy_fly_camera::FlyCamera) will automatically be added to your camera
    pub auto_flycam: bool,
    /// If enabled, [`PanCam`](bevy_pancam::Pancam) will automatically be added to your 2d cameras
    pub auto_pancam: bool,

    /// If any of `add_gizmo_plugin`, `auto_gizmo_target` or `auto_gizmo_camera` are `true`, the [`TransformGizmoPlugin`](bevy_transform_gizmo::TransformGizmoPlugin) will be added.
    pub add_gizmo_plugin: bool,
    /// If enabled, [`GizmoTransformable`](bevy_transform_gizmo::GizmoTransformable) will added to all meshes
    pub auto_gizmo_target: bool,
    /// If enabled, a [`GizmoPickSource`](bevy_transform_gizmo::GizmoPickSource) will automatically be added to your camera
    pub auto_gizmo_camera: bool,

    /// Shows a panel displaying the current FPS. Only available if the [`FrameTimeDiagnosticsPlugin`](bevy::diagnostic::FrameTimeDiagnosticsPlugin) is active.
    pub performance_panel: bool,

    /// Controls whether the editor panel is shown.
    pub display_ui: bool,

    /// The window to display the editor in.
    pub window: WindowId,
}
impl Default for EditorSettings {
    fn default() -> Self {
        EditorSettings {
            menu_items: StableHashMap::default(),
            drag_and_drop_handlers: Vec::new(),
            click_to_inspect: true,
            show_wireframes: false,
            fly_camera: false,
            orbit_camera: false,
            auto_pickable: false,
            auto_pickable_camera: false,
            auto_flycam: false,
<<<<<<< HEAD
            auto_pancam: false,
=======
            add_gizmo_plugin: false,
            auto_gizmo_target: false,
            auto_gizmo_camera: false,
>>>>>>> f8a6629 (add gizmo plugin)
            performance_panel: false,
            display_ui: true,
            window: WindowId::primary(),
        }
    }
}
impl EditorSettings {
    pub fn new() -> Self {
        EditorSettings::default()
    }

    /// Returns `EditorSettings` with all the `auto_*` settings enabled:
    /// All meshes will automatically be [made pickable](EditorSettings::auto_pickable) and the camera gets a [flycam](EditorSettings::auto_flycam) attached.
    pub fn automagic() -> Self {
        let mut settings = EditorSettings::default();
        settings.auto_pickable = true;
        settings.auto_pickable_camera = true;
        settings.auto_flycam = true;
<<<<<<< HEAD
        settings.auto_pancam = true;
=======
        settings.auto_gizmo_target = true;
        settings.auto_gizmo_camera = true;
>>>>>>> f8a6629 (add gizmo plugin)
        settings
    }

    pub fn on_window(mut self, window_id: WindowId) -> Self {
        self.window = window_id;
        self
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
    pub fn add_state<S: Component + Clone + Eq + std::hash::Hash + std::fmt::Debug>(
        &mut self,
        name: &'static str,
        state: S,
    ) {
        self.add_menu_item("States", move |ui, world| {
            let mut state_resource = world.get_resource_mut::<State<S>>().unwrap();
            if ui.button(name).clicked() {
                if let Err(e) = state_resource.replace(state.clone()) {
                    warn!("{}", e);
                }
            }
        });
    }
}

impl EditorSettings {
    /// Displays `ui` in menu bar `menu`.
    pub fn add_menu_item<U>(&mut self, menu: &'static str, ui: U)
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

    /// Registeres a handler for drag and drop events.
    /// # Example
    /// ```rust,no_run
    /// # use bevy::{prelude::*, asset::AssetPath};
    /// # let mut settings = bevy_editor_pls::EditorSettings::new();
    /// settings.on_file_drop(&["gltf", "glb"], |path, world| {
    ///     let asset_path = AssetPath::new_ref(path, Some("Scene0".into()));
    ///     let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
    ///     let scene_handle = asset_server.load(asset_path);
    ///
    ///     let mut spawner = world.get_resource_mut::<SceneSpawner>().unwrap();
    ///     spawner.spawn(scene_handle);
    /// });
    /// ```
    pub fn on_file_drop<S>(&mut self, extensions: &'static [&'static str], handler: S)
    where
        S: Fn(&Path, &mut World) + Send + Sync + 'static,
    {
        self.drag_and_drop_handlers.push((extensions, Box::new(handler)))
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
