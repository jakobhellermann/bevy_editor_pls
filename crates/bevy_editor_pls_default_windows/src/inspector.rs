use super::hierarchy::HierarchyWindow;
use bevy::prelude::{Entity, Mut, World};
use bevy_editor_pls_core::editor_window::{EditorWindow, EditorWindowContext};
use bevy_inspector_egui::egui;
use bevy_inspector_egui::{
    options::EntityAttributes, world_inspector::WorldUIContext, WorldInspectorParams,
};

pub struct InspectorWindow;
impl EditorWindow for InspectorWindow {
    type State = ();
    const NAME: &'static str = "Inspector";

    fn ui(world: &mut World, cx: EditorWindowContext, ui: &mut egui::Ui) {
        let inspected = cx.state::<HierarchyWindow>().unwrap().selected;
        inspector(world, inspected, ui);
    }
}

fn inspector(world: &mut World, inspected: Option<Entity>, ui: &mut egui::Ui) {
    let inspected = match inspected {
        Some(inspected) => inspected,
        None => {
            ui.label("No entity selected");
            return;
        }
    };

    if world.get_entity(inspected).is_none() {
        ui.label("No entity selected");
        return;
    }

    world.resource_scope(|world, params: Mut<WorldInspectorParams>| {
        let entity_options = EntityAttributes::default();
        WorldUIContext::new(world, None).entity_ui_inner(
            ui,
            inspected,
            &*params,
            egui::Id::new("inspector"),
            &entity_options,
        );
    });
}
