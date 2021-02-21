use bevy::{app::ManualEventReader, prelude::*};
use std::any::TypeId;

use crate::{plugin::EditorState, EditorSettings};

/// This event is sent when clicking on the `Events` menu bar
pub(crate) struct EditorEvent(pub TypeId);

pub(crate) fn send_editor_events(world: &mut World, resources: &mut Resources) {
    let mut editor_settings = resources.get_mut::<EditorSettings>().unwrap();
    let editor_events = resources.get::<Events<EditorEvent>>().unwrap();
    let mut editor_event_reader = ManualEventReader::<EditorEvent>::default();

    let events_to_send = std::mem::take(&mut editor_settings.events_to_send);

    let mut fns: Vec<_> = editor_event_reader
        .iter(&editor_events)
        .map(|event| events_to_send.get(&event.0).unwrap())
        .collect();

    drop(editor_settings);
    drop(editor_events);
    drop(editor_event_reader);

    for (_, f) in &mut fns {
        f(world, resources);
    }

    let mut editor_settings = resources.get_mut::<EditorSettings>().unwrap();
    editor_settings.events_to_send = events_to_send;
}

pub fn maintain_inspected_entities(
    mut editor_state: ResMut<EditorState>,
    query: Query<(Entity, &Interaction), Changed<Interaction>>,
) {
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
