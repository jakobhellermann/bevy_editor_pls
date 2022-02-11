# bevy_editor_pls

> :warning: **This is very much work in progress**: Take a look at the [missing features](#missing-features) to see if your use case isn't yet supported.

Adds debug tools to your bevy game, including
- hierarchy view and component inspector
- separate editor camera
- some builtin editor panels for diagnostics, debug settings
- scene export


## How to use:

Add the `EditorPlugin`:

```diff
+use bevy_editor_pls::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
+       .add_plugin(EditorPlugin)
        ...
        .run();
}
```


![editor preview](./docs/editor.png)

### Custom editor panels

```rust
fn main() {
    App::new()
        ...
        .add_editor_window::<MyEditorWindow>()
        ...
        .run();
}

pub struct MyEditorWindow;
impl EditorWindow for MyEditorWindow {
    type State = ();
    const NAME: &'static str = "Another editor panel";

    fn ui(world: &mut World, cx: EditorWindowContext, ui: &mut egui::Ui) {
        let currently_inspected = cx.state::<HierarchyWindow>().unwrap().selected;

        ui.label("Anything can go here");
    }
}
```


## Missing features

- scene import
- better editor camera controls (orbit camera, ...)
- transform gizmos
- visualization of invisible entities in editor (to see where the camera is for example)