pub mod picking;

use bevy::ecs::system::QuerySingleError;
use bevy::prelude::*;
use bevy_inspector_egui::egui::{self, CollapsingHeader, RichText};

use bevy_editor_pls_core::{
    editor_window::{EditorWindow, EditorWindowContext},
    Editor,
};

pub struct HierarchyWindow;
impl EditorWindow for HierarchyWindow {
    type State = HierarchyState;
    const NAME: &'static str = "Hierarchy";

    fn ui(world: &mut World, mut cx: EditorWindowContext, ui: &mut egui::Ui) {
        let state = cx.state_mut::<HierarchyWindow>().unwrap();
        Hierarchy { world, state }.show(ui);
    }

    fn app_setup(app: &mut bevy::prelude::App) {
        picking::setup(app);
        app.add_event::<EditorHierarchyEvent>()
            .add_system(handle_events);
    }
}

pub enum EditorHierarchyEvent {
    SelectMesh,
}

fn handle_events(
    mut events: EventReader<EditorHierarchyEvent>,
    raycast_source: Query<&picking::EditorRayCastSource>,
    mut editor: ResMut<Editor>,
) {
    for event in events.iter() {
        match event {
            EditorHierarchyEvent::SelectMesh => {
                let raycast_source = match raycast_source.get_single() {
                    Ok(entity) => entity,
                    Err(QuerySingleError::NoEntities(_)) => continue,
                    Err(QuerySingleError::MultipleEntities(_)) => {
                        panic!("Multiple entities with EditorRayCastSource component!")
                    }
                };
                let state = editor.window_state_mut::<HierarchyWindow>().unwrap();

                if let Some((entity, _interaction)) = raycast_source.intersect_top() {
                    state.selected = Some(entity);
                } else {
                    state.selected = None;
                }
            }
        }
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
        let active = self.state.selected == Some(entity);

        let entity_name = bevy_inspector_egui::world_inspector::entity_name(self.world, entity);
        let mut text = RichText::new(entity_name);
        if active {
            text = text.strong();
        }
        let response = CollapsingHeader::new(text).show(ui, |ui| {
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
}
