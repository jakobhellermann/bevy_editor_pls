use std::ffi::OsStr;

use bevy::{
    app::{Events, ManualEventReader},
    prelude::*,
};

use crate::EditorSettings;

pub(crate) fn drag_and_drop_system(world: &mut World) {
    let paths = {
        let cell = world.cell();
        let events = cell.get_resource_mut::<Events<FileDragAndDrop>>().unwrap();
        let mut event_reader = ManualEventReader::default();

        let mut paths = Vec::new();

        for event in event_reader.iter(&events) {
            match event {
                FileDragAndDrop::DroppedFile { path_buf, .. } => {
                    paths.push(path_buf.to_path_buf());
                }
                _ => {}
            }
        }
        paths
    };

    world.resource_scope(|mut editor_settings: Mut<EditorSettings>, world| {
        for path in paths {
            let extension = match path.extension().and_then(OsStr::to_str) {
                Some(extension) => extension,
                None => continue,
            };
            let handlers = editor_settings
                .drag_and_drop_handlers
                .iter_mut()
                .filter(|(extensions, _)| extensions.iter().any(|e| extension.eq_ignore_ascii_case(e)))
                .map(|(_, handler)| handler);

            for handler in handlers {
                handler(&path, world);
            }
        }
    });
}
