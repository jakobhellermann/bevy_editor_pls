use bevy::{app::ManualEventReader, prelude::*};
use std::any::TypeId;

use crate::{
    plugin::{EditorState, ExclusiveAccessFn},
    EditorSettings,
};

pub enum EditorEvent {
    SendEvent(TypeId),
    StateTransition(TypeId, u64),
}

impl EditorEvent {
    fn handler_fn<'e>(&self, editor: &'e EditorSettings) -> &'e ExclusiveAccessFn {
        match self {
            EditorEvent::SendEvent(type_id) => &editor.events_to_send[type_id].1,
            EditorEvent::StateTransition(type_id, variant) => {
                &editor.state_transition_handlers[&(*type_id, *variant)].1
            }
        }
    }
}

pub(crate) fn send_editor_events(world: &mut World, resources: &mut Resources) {
    let editor_events = resources.get::<Events<EditorEvent>>().unwrap();
    let mut editor_event_reader = ManualEventReader::<EditorEvent>::default();
    // we need to take ownership of `EditorSettings` so that we can run the handler functions with access to the `Resources`
    let editor_settings = {
        let mut res = resources.get_mut::<EditorSettings>().unwrap();
        std::mem::take(&mut *res)
    };

    let mut fns: Vec<_> = editor_event_reader
        .iter(&editor_events)
        .map(|event| event.handler_fn(&editor_settings))
        .collect();

    drop(editor_events);
    drop(editor_event_reader);

    for f in &mut fns {
        f(world, resources);
    }

    let mut editor_settings_res = resources.get_mut::<EditorSettings>().unwrap();
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
