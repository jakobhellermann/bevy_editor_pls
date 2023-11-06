use bevy::{
    prelude::{AppTypeRegistry, ReflectResource, World},
    reflect::TypeRegistry,
};
use bevy_editor_pls_core::editor_window::{EditorWindow, EditorWindowContext};
use bevy_inspector_egui::egui;

use crate::inspector::{InspectorSelection, InspectorWindow};

pub struct ResourcesWindow;

impl EditorWindow for ResourcesWindow {
    type State = ();

    const NAME: &'static str = "Resources";

    fn ui(world: &mut World, mut cx: EditorWindowContext, ui: &mut egui::Ui) {
        let selection = &mut cx.state_mut::<InspectorWindow>().unwrap().selected;
        let type_registry = world.resource::<AppTypeRegistry>();
        let type_registry = type_registry.read();

        select_resource(ui, &type_registry, selection);
    }
}

fn select_resource(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    selection: &mut InspectorSelection,
) {
    let mut resources: Vec<_> = type_registry
        .iter()
        .filter(|registration| registration.data::<ReflectResource>().is_some())
        .map(|registration| {
            (
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
            )
        })
        .collect();
    resources.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

    for (resource_name, type_id) in resources {
        let selected = match *selection {
            InspectorSelection::Resource(selected, _) => selected == type_id,
            _ => false,
        };

        if ui.selectable_label(selected, resource_name).clicked() {
            *selection = InspectorSelection::Resource(type_id, resource_name.to_owned());
        }
    }
}
