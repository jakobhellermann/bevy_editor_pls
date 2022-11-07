use bevy::{prelude::*, reflect::TypeRegistry, scene::DynamicScene};
use bevy_editor_pls_core::editor_window::{EditorWindow, EditorWindowContext};
use bevy_inspector_egui::egui::{self, RichText};

const DEFAULT_FILENAME: &str = "scene.scn.ron";

#[derive(Default, Component)]
pub struct NotInScene;

#[derive(Default)]
pub struct SceneWindowState {
    filename: String,
    scene_save_result: Option<Result<(), Box<dyn std::error::Error + Send + Sync>>>,
}

pub struct SceneWindow;

impl EditorWindow for SceneWindow {
    type State = SceneWindowState;
    const NAME: &'static str = "Scenes";

    fn ui(world: &mut World, mut cx: EditorWindowContext, ui: &mut egui::Ui) {
        let state = cx.state_mut::<SceneWindow>().unwrap();

        ui.horizontal(|ui| {
            let res = egui::TextEdit::singleline(&mut state.filename)
                .hint_text(DEFAULT_FILENAME)
                .desired_width(120.0)
                .show(ui);

            if res.response.changed() {
                state.scene_save_result = None;
            }

            let enter_pressed = ui.input().key_pressed(egui::Key::Enter);

            if ui.button("Save").clicked() || enter_pressed {
                let filename = if state.filename.is_empty() {
                    DEFAULT_FILENAME
                } else {
                    &state.filename
                };
                let mut query = world.query_filtered::<Entity, Without<NotInScene>>();
                let entitys = query.iter(world).collect();
                state.scene_save_result = Some(save_world(world, filename, entitys));
            }
        });

        if let Some(status) = &state.scene_save_result {
            match status {
                Ok(()) => {
                    ui.label(RichText::new("Success!").color(egui::Color32::GREEN));
                }
                Err(error) => {
                    ui.label(RichText::new(error.to_string()).color(egui::Color32::RED));
                }
            }
        }
    }
}

fn save_world(
    world: &World,
    name: &str,
    entitys: std::collections::HashSet<Entity>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let type_registry = world.get_resource::<TypeRegistry>().unwrap();
    let scene = from_world(&world, type_registry, entitys);

    let ron = scene.serialize_ron(type_registry)?;
    std::fs::write(name, ron)?;

    Ok(())
}

/// Create a new dynamic scene from a given world and etity set;
fn from_world(
    world: &World,
    type_registry: &TypeRegistry,
    entitys: std::collections::HashSet<Entity>,
) -> DynamicScene {
    use bevy::scene::DynamicEntity;
    let mut scene = DynamicScene::default();
    let type_registry = type_registry.read();

    for archetype in world.archetypes().iter() {
        // a map of an entity to its index in the dynamic scene;
        // can't use simple offset like bevy's because it may skip an entity
        let mut entity_map = std::collections::HashMap::new();
        // Create a new dynamic entity for each entity of the given archetype
        // and insert it into the dynamic scene.
        // skip entitys not in the set
        for entity in archetype.entities() {
            if !entitys.contains(entity) {
                continue;
            }
            entity_map.insert(entity, scene.entities.len());
            scene.entities.push(DynamicEntity {
                entity: entity.id(),
                components: Vec::new(),
            });
        }

        // Add each reflection-powered component to the entity it belongs to.
        for component_id in archetype.components() {
            let reflect_component = world
                .components()
                .get_info(component_id)
                .and_then(|info| type_registry.get(info.type_id().unwrap()))
                .and_then(|registration| registration.data::<ReflectComponent>());
            if let Some(reflect_component) = reflect_component {
                for entity in archetype.entities() {
                    if !entitys.contains(entity) {
                        continue;
                    }
                    if let Some(component) = reflect_component.reflect(world, *entity) {
                        scene.entities[*entity_map
                            .get(entity)
                            .expect("entity to have been added to map")]
                        .components
                        .push(component.clone_value());
                    }
                }
            }
        }
    }
    scene
}
