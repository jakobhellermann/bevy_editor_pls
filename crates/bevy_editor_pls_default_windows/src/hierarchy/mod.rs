pub mod picking;

use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_editor_pls_core::{EditorEvent, EditorState};
use bevy_inspector_egui::egui::{self, CollapsingHeader, RichText};

use bevy_editor_pls_core::{
    editor_window::{EditorWindow, EditorWindowContext},
    Editor,
};

#[derive(Component)]
pub struct HideInEditor;

use self::picking::EditorRayCastState;

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
    mut select_mesh_events: EventReader<EditorHierarchyEvent>,
    mut editor_events: EventReader<EditorEvent>,
    mut raycast_state: ResMut<EditorRayCastState>,
    editor_camera: Query<&picking::EditorRayCastSource, With<super::cameras::EditorCamera>>,
    mut editor: ResMut<Editor>,
    editor_state: Res<EditorState>,
) {
    for event in editor_events.iter() {
        match *event {
            EditorEvent::Toggle { now_active: false } => {
                raycast_state.build_rays = false;
                raycast_state.update_raycast = false;
            }
            EditorEvent::Toggle { now_active: true } => {
                raycast_state.build_rays = true;
                raycast_state.update_raycast = true;
            }
            _ => {}
        }
    }

    for event in select_mesh_events.iter() {
        match event {
            EditorHierarchyEvent::SelectMesh => {
                let raycast_source = if editor_state.active {
                    let raycast_source = editor_camera.single();
                    Some(raycast_source)
                } else {
                    None
                };
                let state = editor.window_state_mut::<HierarchyWindow>().unwrap();

                if let Some(raycast_source) = raycast_source {
                    if let Some((entity, _interaction)) = raycast_source.intersect_top() {
                        info!("Selecting mesh, found {:?}", entity);
                        state.selected = Some(entity);
                    } else {
                        info!("Selecting mesh, found none");
                        state.selected = None;
                    }
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
        let mut root_query = self
            .world
            .query_filtered::<Entity, (Without<Parent>, Without<HideInEditor>)>();

        let parents: HashSet<Entity> = std::iter::successors(self.state.selected, |&entity| {
            self.world.get::<Parent>(entity).map(|parent| parent.0)
        })
        .skip(1)
        .collect();

        let entities: Vec<_> = root_query.iter(self.world).collect();
        for entity in entities {
            self.entity_ui(entity, ui, &parents);
        }
    }
    fn entity_ui(&mut self, entity: Entity, ui: &mut egui::Ui, always_open: &HashSet<Entity>) {
        let active = self.state.selected == Some(entity);

        let entity_name = bevy_inspector_egui::world_inspector::entity_name(self.world, entity);
        let mut text = RichText::new(entity_name);
        if active {
            text = text.strong();
        }

        let selected = self.state.selected == Some(entity);
        let has_children = self
            .world
            .get::<Children>(entity)
            .map_or(false, |children| children.len() > 0);

        let open = if !has_children {
            Some(false)
        } else if always_open.contains(&entity) {
            Some(true)
        } else {
            None
        };

        let response = CollapsingHeader::new(text)
            .id_source(entity)
            .selectable(true)
            .selected(selected)
            .open(open)
            .show(ui, |ui| {
                let children = self.world.get::<Children>(entity);
                if let Some(children) = children {
                    let children = children.clone();
                    ui.label("Children");
                    for &child in children.iter() {
                        self.entity_ui(child, ui, always_open);
                    }
                } else {
                    ui.label("No children");
                }
            });
        if response.header_response.clicked() {
            self.state.selected = (!selected).then(|| entity);
        }
    }
}
