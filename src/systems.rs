use bevy::{
    app::{Events, ManualEventReader},
    prelude::*,
    render::{camera::Camera, render_graph::base::camera},
};
use bevy_mod_picking::{PickableBundle, PickableMesh, PickingCamera, PickingCameraBundle};

use crate::{
    plugin::{EditorState, ExclusiveAccessFn},
    EditorSettings,
};

pub enum EditorEvent {
    SendEvent(usize),
    StateTransition(usize),
}

impl EditorSettings {
    fn handler_fn<'e>(&'e self, event: &EditorEvent) -> &'e ExclusiveAccessFn {
        match *event {
            EditorEvent::SendEvent(index) => &self.events_to_send[index].1,
            EditorEvent::StateTransition(index) => &self.state_transition_handlers[index].1,
        }
    }
}

pub(crate) fn send_editor_events(world: &mut World) {
    let world_cell = world.cell();
    let editor_events = world_cell.get_resource::<Events<EditorEvent>>().unwrap();
    let mut editor_event_reader = ManualEventReader::<EditorEvent>::default();
    // we need to take ownership of `EditorSettings` so that we can run the handler functions with access to the `Resources`
    let editor_settings = {
        let mut res = world_cell.get_resource_mut::<EditorSettings>().unwrap();
        std::mem::take(&mut *res)
    };

    let mut fns: Vec<_> = editor_event_reader
        .iter(&editor_events)
        .map(|event| editor_settings.handler_fn(event))
        .collect();

    drop(editor_events);
    drop(editor_event_reader);
    drop(world_cell);

    for f in &mut fns {
        f(world);
    }

    let mut editor_settings_res = world.get_resource_mut::<EditorSettings>().unwrap();
    *editor_settings_res = editor_settings;
}

pub fn maintain_inspected_entities(
    editor_settings: ResMut<EditorSettings>,
    mut editor_state: ResMut<EditorState>,
    query: Query<(Entity, &Interaction), Changed<Interaction>>,
) {
    if !editor_settings.click_to_inspect {
        return;
    }

    let entity = query
        .iter()
        .filter(|(_, interaction)| matches!(interaction, Interaction::Clicked))
        .map(|(entity, _)| entity)
        .next();

    if let Some(entity) = entity {
        if editor_state.currently_inspected == Some(entity) {
            editor_state.currently_inspected = None;
        } else {
            editor_state.currently_inspected = Some(entity);
        }
    }
}

// systems for making everything pickable

pub fn make_everything_pickable(
    editor_settings: Res<EditorSettings>,
    mut commands: Commands,
    mut query: Query<Entity, (With<Draw>, Without<PickableMesh>, Without<Node>)>,
) {
    if !editor_settings.auto_pickable || !editor_settings.click_to_inspect {
        return;
    }

    for entity in query.iter_mut() {
        // dbg!(entity);
        commands.insert_bundle(entity, PickableBundle::default());
    }
}
pub fn make_camera_picksource(
    editor_settings: Res<EditorSettings>,
    mut commands: Commands,
    mut query: Query<(Entity, &Camera), Without<PickingCamera>>,
) {
    if !editor_settings.auto_pickable || !editor_settings.click_to_inspect {
        return;
    }

    for (entity, cam) in query.iter_mut() {
        if cam.name.as_ref().map_or(false, |name| name == camera::CAMERA_3D) {
            commands.insert_bundle(entity, PickingCameraBundle::default());
        }
    }
}
