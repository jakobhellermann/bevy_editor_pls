use std::borrow::Cow;

use bevy::ecs::world::EntityRef;
use bevy::prelude::*;
use bevy_inspector_egui::egui::{self, CollapsingHeader};

use bevy_editor_pls_core::editor_window::{EditorWindow, EditorWindowContext};

pub struct HierarchyWindow;
impl EditorWindow for HierarchyWindow {
    type State = HierarchyState;
    const NAME: &'static str = "Hierarchy";

    fn ui(world: &mut World, mut cx: EditorWindowContext, ui: &mut egui::Ui) {
        let state = cx.state_mut::<HierarchyWindow>().unwrap();
        Hierarchy { world, state }.show(ui);
    }
}

#[derive(Default)]
pub struct HierarchyState {
    pub selected: Option<Entity>,
}

struct Hierarchy<'a> {
    world: &'a mut World,
    state: &'a mut HierarchyState,
}

impl<'a> Hierarchy<'a> {
    fn show(&mut self, ui: &mut egui::Ui) {
        let mut root_query = self.world.query_filtered::<Entity, Without<Parent>>();
        let entities: Vec<_> = root_query.iter(self.world).collect();
        for entity in entities {
            self.entity_ui(entity, ui);
        }
    }
    fn entity_ui(&mut self, entity: Entity, ui: &mut egui::Ui) {
        let response = CollapsingHeader::new(self.entity_name(entity).to_string()).show(ui, |ui| {
            let children = self.world.get::<Children>(entity);
            if let Some(children) = children {
                let children = children.clone();
                ui.label("Children");
                for &child in children.iter() {
                    self.entity_ui(child, ui);
                }
            } else {
                ui.label("No children");
            }
        });
        if response.header_response.clicked() {
            self.state.selected = Some(entity);
        }
    }

    fn entity_name(&self, entity: Entity) -> Cow<'_, str> {
        match self.world.get_entity(entity) {
            Some(entity) => guess_entity_name(entity),
            None => format!("Entity {} (inexistent)", entity.id()).into(),
        }
    }
}

fn guess_entity_name(entity: EntityRef) -> Cow<'_, str> {
    if let Some(name) = entity.get::<Name>() {
        return name.as_str().into();
    }

    if let Some(camera) = entity.get::<Camera>() {
        match &camera.name {
            Some(name) => return name.as_str().into(),
            None => return "Camera".into(),
        }
    }

    format!("Entity {:?}", entity.id()).into()
}
