use std::borrow::Cow;

use bevy::{
    core::Stopwatch,
    pbr::{wireframe::Wireframe, NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::view::RenderLayers,
};
use bevy_editor_pls_core::editor_window::{EditorWindow, EditorWindowContext};
use bevy_inspector_egui::egui;
use indexmap::IndexMap;

use crate::hierarchy::HierarchyWindow;

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

    pub fn component<T: FromWorld + Component>() -> Self {
        AddItem::component_named::<T>(pretty_type_name::pretty_type_name::<T>().into())
    }

    pub fn component_named<T: FromWorld + Component>(name: Cow<'static, str>) -> Self {
        AddItem::new(name, |world, entity| {
            let bundle = T::from_world(world);
            world.entity_mut(entity).insert(bundle);
        })
    }

    pub fn add_to_entity(&self, world: &mut World, entity: Entity) {
        (self.add_to_entity)(world, entity)
    }
}

pub struct AddWindowState {
    sections: IndexMap<&'static str, Vec<AddItem>>,
}

impl AddWindowState {
    pub fn add(&mut self, name: &'static str, item: AddItem) {
        self.sections.entry(name).or_default().push(item);
    }

    pub fn sections(&self) -> impl Iterator<Item = (&'static str, &[AddItem])> {
        self.sections
            .iter()
            .map(|(name, items)| (*name, items.as_slice()))
    }
}

pub struct AddWindow;

impl EditorWindow for AddWindow {
    type State = AddWindowState;

    const NAME: &'static str = "Add";

    fn ui(world: &mut World, cx: EditorWindowContext, ui: &mut egui::Ui) {
        add_ui_button(world, ui, cx);
    }

    fn menu_ui(world: &mut World, cx: EditorWindowContext, ui: &mut egui::Ui) {
        add_ui_button(world, ui, cx);
    }
}

fn add_ui_button(world: &mut World, ui: &mut egui::Ui, mut cx: EditorWindowContext) {
    let state = cx.state::<AddWindow>().unwrap();

    let response = ui.menu_button("Add", |ui| {
        add_ui(ui, state).map(|add_item| {
            let entity = world.spawn().id();
            add_item.add_to_entity(world, entity);
            entity
        })
    });

    if let Some(Some(entity)) = response.inner {
        if let Some(hierarchy_state) = cx.state_mut::<HierarchyWindow>() {
            hierarchy_state.selected = Some(entity);
        }
    }
}

pub fn add_ui<'a>(ui: &mut egui::Ui, state: &'a AddWindowState) -> Option<&'a AddItem> {
    for (section_name, items) in &state.sections {
        if section_name.is_empty() {
            for item in items {
                if ui.button(item.name.as_ref()).clicked() {
                    ui.close_menu();
                    return Some(item);
                }
            }
        } else {
            let value = ui.menu_button(*section_name, |ui| {
                for item in items {
                    if ui.button(item.name.as_ref()).clicked() {
                        ui.close_menu();
                        return Some(item);
                    }
                }
                None
            });
            if let Some(Some(value)) = value.inner {
                return Some(value);
            }
        }
    }
    None
}

impl Default for AddWindowState {
    fn default() -> Self {
        let mut state = AddWindowState {
            sections: IndexMap::default(),
        };

        state.add("", AddItem::bundle_named::<()>("Empty".into()));

        state.add("Core", AddItem::component::<Name>());
        state.add("Core", AddItem::component::<Timer>());
        state.add("Core", AddItem::component::<Stopwatch>());
        state.add(
            "Core",
            AddItem::bundle_named::<(Transform, GlobalTransform)>("Transform".into()),
        );

        state.add("Rendering", AddItem::component::<RenderLayers>());
        state.add("Rendering", AddItem::component::<Visibility>());
        state.add("Rendering", AddItem::component::<Wireframe>());
        state.add(
            "Rendering",
            AddItem::new("NotShadowCaster".into(), |world, entity| {
                world.entity_mut(entity).insert(NotShadowCaster);
            }),
        );
        state.add(
            "Rendering",
            AddItem::new("NotShadowReceiver".into(), |world, entity| {
                world.entity_mut(entity).insert(NotShadowReceiver);
            }),
        );

        state.add(
            "2D",
            AddItem::new("Orthographic Camera".into(), |world, entity| {
                world
                    .entity_mut(entity)
                    .insert_bundle(OrthographicCameraBundle::new_2d());
            }),
        );
        state.add("2D", AddItem::bundle::<SpriteBundle>());
        state.add("2D", AddItem::bundle::<Text2dBundle>());

        state.add(
            "3D",
            AddItem::new("Perspective Camera".into(), |world, entity| {
                world
                    .entity_mut(entity)
                    .insert_bundle(PerspectiveCameraBundle::new_3d());
            }),
        );
        state.add(
            "3D",
            AddItem::new("Orthographic Camera".into(), |world, entity| {
                world
                    .entity_mut(entity)
                    .insert_bundle(OrthographicCameraBundle::new_3d());
            }),
        );
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

        state.add("UI", AddItem::bundle::<UiCameraBundle>());
        state.add("UI", AddItem::bundle::<NodeBundle>());
        state.add("UI", AddItem::bundle::<TextBundle>());
        state.add("UI", AddItem::bundle::<ImageBundle>());
        state.add("UI", AddItem::bundle::<ButtonBundle>());

        state
    }
}
