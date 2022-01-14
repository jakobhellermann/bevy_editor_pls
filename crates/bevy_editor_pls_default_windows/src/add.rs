use std::borrow::Cow;

use bevy::prelude::*;
use bevy_editor_pls_core::editor_window::{EditorWindow, EditorWindowContext};
use bevy_inspector_egui::egui;
use indexmap::IndexMap;

pub struct AddItem {
    name: Cow<'static, str>,
    add_to_entity: fn(&mut World, Entity),
}

impl AddItem {
    pub fn new(name: Cow<'static, str>, add_to_entity: fn(&mut World, Entity)) -> Self {
        AddItem {
            name,
            add_to_entity,
        }
    }

    pub fn bundle<T: FromWorld + Bundle>() -> Self {
        AddItem::bundle_named::<T>(pretty_type_name::pretty_type_name::<T>().into())
    }

    pub fn bundle_named<T: FromWorld + Bundle>(name: Cow<'static, str>) -> Self {
        AddItem::new(name, |world, entity| {
            let bundle = T::from_world(world);
            world.entity_mut(entity).insert_bundle(bundle);
        })
    }
}

pub struct AddWindowState {
    sections: IndexMap<&'static str, Vec<AddItem>>,
}

impl AddWindowState {
    pub fn add(&mut self, name: &'static str, item: AddItem) {
        self.sections.entry(name).or_default().push(item);
    }
}

pub struct AddWindow;

impl EditorWindow for AddWindow {
    type State = AddWindowState;

    const NAME: &'static str = "Add";

    fn ui(world: &mut World, cx: EditorWindowContext, ui: &mut egui::Ui) {
        let state = cx.state::<Self>().unwrap();
        add_ui(world, ui, state);
    }

    fn menu_ui(world: &mut World, cx: EditorWindowContext, ui: &mut egui::Ui) {
        let state = cx.state::<Self>().unwrap();
        add_ui(world, ui, state);
    }
}

fn add_ui(world: &mut World, ui: &mut egui::Ui, state: &AddWindowState) {
    ui.menu_button("Add", |ui| {
        for (section_name, items) in &state.sections {
            ui.menu_button(*section_name, |ui| {
                for item in items {
                    if ui.button(item.name.as_ref()).clicked() {
                        let entity = world.spawn().id();
                        (item.add_to_entity)(world, entity);

                        ui.close_menu();
                    }
                }
            });
        }
    });
}

impl Default for AddWindowState {
    fn default() -> Self {
        let mut state = AddWindowState {
            sections: IndexMap::default(),
        };
        state.add("3D", AddItem::bundle_named::<PbrBundle>("PbrBundle".into()));
        state.add(
            "3D",
            AddItem::new("Cube".into(), |world, entity| {
                let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
                let mesh = meshes.add(shape::Cube::default().into());

                let mut materials = world
                    .get_resource_mut::<Assets<StandardMaterial>>()
                    .unwrap();
                let material = materials.add(StandardMaterial::default());

                world.entity_mut(entity).insert_bundle(PbrBundle {
                    mesh,
                    material,
                    ..Default::default()
                });
            }),
        );

        state.add("2D", AddItem::bundle::<SpriteBundle>());

        state.add(
            "Cameras",
            AddItem::new("Perspective 3D".into(), |world, entity| {
                world
                    .entity_mut(entity)
                    .insert_bundle(PerspectiveCameraBundle::new_3d());
            }),
        );
        state.add(
            "Cameras",
            AddItem::new("Orthographic 3D".into(), |world, entity| {
                world
                    .entity_mut(entity)
                    .insert_bundle(OrthographicCameraBundle::new_3d());
            }),
        );
        state.add(
            "Cameras",
            AddItem::new("Orthographic 2D".into(), |world, entity| {
                world
                    .entity_mut(entity)
                    .insert_bundle(OrthographicCameraBundle::new_2d());
            }),
        );

        state
    }
}
