pub mod picking;

use bevy::utils::HashSet;
use bevy::{ecs::system::QuerySingleError, prelude::*};
use bevy_editor_pls_core::EditorState;
use bevy_inspector_egui::egui::{self, CollapsingHeader, RichText};

use bevy_editor_pls_core::{
    editor_window::{EditorWindow, EditorWindowContext},
    Editor,
};

use crate::add::{add_ui, AddWindow, AddWindowState};
use crate::cameras::{CameraWindow, EditorCamKind};

#[derive(Component)]
pub struct HideInEditor;

pub struct HierarchyWindow;
impl EditorWindow for HierarchyWindow {
    type State = HierarchyState;
    const NAME: &'static str = "Hierarchy";

    fn ui(world: &mut World, mut cx: EditorWindowContext, ui: &mut egui::Ui) {
        let (hierarchy_state, add_state) = cx.state_mut_pair::<HierarchyWindow, AddWindow>();
        let hierarchy_state = hierarchy_state.unwrap();

        Hierarchy {
            world,
            state: hierarchy_state,
            add_state: add_state.as_deref(),
        }
        .show(ui);
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
    // mut editor_events: EventReader<EditorEvent>,
    // mut raycast_state: ResMut<EditorRayCastState>,
    _editor_camera_2d_panzoom: Query<
        (&GlobalTransform, &Camera),
        With<super::cameras::EditorCamera2dPanZoom>,
    >,
    editor_camera_3d_free: Query<
        &picking::EditorRayCastSource,
        With<super::cameras::EditorCamera3dFree>,
    >,
    non_editor_camera: Query<&picking::EditorRayCastSource, Without<super::cameras::EditorCamera>>,
    mut editor: ResMut<Editor>,
    editor_state: Res<EditorState>,
) {
    // TODO: reenable once bevy_mod_raycast has per-source configuration
    /*for event in editor_events.iter() {
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
    }*/

    for event in select_mesh_events.iter() {
        #[allow(irrefutable_let_patterns)]
        if let EditorHierarchyEvent::SelectMesh = event {
            let editor_camera = editor.window_state::<CameraWindow>().unwrap().editor_cam;

            let picked_entity = match (editor_state.active, editor_camera) {
                (false, _) => {
                    let source = match non_editor_camera.get_single() {
                        Ok(source) => Some(source),
                        Err(QuerySingleError::NoEntities(_)) => {
                            error!("No cameras with `EditorRayCastSource` found, can't click to inspect when the editor is inactive!");
                            continue;
                        }
                        Err(QuerySingleError::MultipleEntities(_)) => {
                            error!("Multiple cameras with `EditorRayCastSource` found!");
                            continue;
                        }
                    };
                    source
                        .and_then(|source| source.intersect_top())
                        .map(|(entity, _)| entity)
                }
                (true, EditorCamKind::D2PanZoom) => {
                    // TODO: pick sprites
                    // let (cam_transform, cam) = editor_camera_2d_panzoom.single();
                    continue;
                }
                (true, EditorCamKind::D3Free) => {
                    let source = editor_camera_3d_free.single();
                    source.intersect_top().map(|(entity, _)| entity)
                }
            };

            let state = editor.window_state_mut::<HierarchyWindow>().unwrap();

            if let Some(entity) = picked_entity {
                info!("Selecting mesh, found {:?}", entity);
                state.selected = Some(entity);
            } else {
                info!("Selecting mesh, found none");
                state.selected = None;
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
    add_state: Option<&'a AddWindowState>,
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

        let mut entities: Vec<_> = root_query.iter(self.world).collect();
        entities.sort();

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

        let mut despawn = false;
        let header_response = response.header_response.context_menu(|ui| {
            if ui.button("Despawn").clicked() {
                despawn = true;
            }

            if let Some(add_state) = self.add_state {
                ui.menu_button("Add", |ui| {
                    if let Some(add_item) = add_ui(ui, add_state) {
                        add_item.add_to_entity(self.world, entity);
                        self.state.selected = Some(entity);
                        ui.close_menu();
                    }
                });
            }
        });

        if selected && ui.input().key_pressed(egui::Key::Delete) {
            despawn = true;
        }

        if despawn {
            self.state.selected = None;

            if let Some(&parent) = self.world.get::<Parent>(entity) {
                if let Some(mut children) = self.world.get_mut::<Children>(parent.0) {
                    let new_children: Vec<_> = children
                        .iter()
                        .copied()
                        .filter(|&child| child != entity)
                        .collect();
                    *children = Children::with(new_children.as_slice());
                }
            }

            self.world.despawn(entity);
        }

        if header_response.clicked() {
            self.state.selected = (!selected).then(|| entity);
        }
    }
}
