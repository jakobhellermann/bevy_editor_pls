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

    world.resource_scope(|world, mut editor_settings: Mut<EditorSettings>| {
        for path in paths {
            let path_str = path.to_str().unwrap();
            let handlers = editor_settings
                .drag_and_drop_handlers
                .iter_mut()
                .filter(|(extensions, _)| extensions.iter().any(|e| path_str.ends_with(e)))
                .map(|(_, handler)| handler);

            for handler in handlers {
                handler(&path, world);
            }
        }
    });
}
