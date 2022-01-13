use bevy::prelude::*;
use bevy_editor_pls_core::{editor_window::EditorWindow, EditorState};
use bevy_editor_pls_default_windows::hierarchy::EditorHierarchyEvent;

pub enum Button {
    Keyboard(KeyCode),
    Mouse(MouseButton),
}

pub enum UserInput {
    Single(Button),
    Chord(Vec<Button>),
}

enum BindingCondition {
    InViewport(bool),
}

impl BindingCondition {
    fn evaluate(&self, editor_state: &EditorState) -> bool {
        match *self {
            BindingCondition::InViewport(in_viewport) => {
                if in_viewport {
                    return (editor_state.pointer_in_viewport || !editor_state.active)
                        && !editor_state.pointer_on_floating_window;
                } else {
                    return !editor_state.pointer_in_viewport && editor_state.active
                        || editor_state.pointer_on_floating_window;
                }
            }
        }
    }
}

pub struct Binding {
    input: UserInput,
    conditions: Vec<BindingCondition>,
}

pub struct EditorControls {
    pub select_mesh: Binding,
    pub play_pause_editor: Binding,
}

impl Button {
    fn just_pressed(
        &self,
        keyboard_input: &Input<KeyCode>,
        mouse_input: &Input<MouseButton>,
    ) -> bool {
        match self {
            Button::Keyboard(code) => keyboard_input.just_pressed(*code),
            Button::Mouse(button) => mouse_input.just_pressed(*button),
        }
    }
}

impl UserInput {
    fn just_pressed(
        &self,
        keyboard_input: &Input<KeyCode>,
        mouse_input: &Input<MouseButton>,
    ) -> bool {
        match self {
            UserInput::Single(single) => single.just_pressed(keyboard_input, mouse_input),
            UserInput::Chord(_) => todo!(),
        }
    }
}

impl Binding {
    fn just_pressed(
        &self,
        keyboard_input: &Input<KeyCode>,
        mouse_input: &Input<MouseButton>,
        editor_state: &EditorState,
    ) -> bool {
        let can_trigger = self
            .conditions
            .iter()
            .all(|condition| condition.evaluate(editor_state));
        if !can_trigger {
            return false;
        }

        self.input.just_pressed(keyboard_input, mouse_input)
    }
}

pub fn editor_controls_system(
    controls: Res<EditorControls>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut editor_state: ResMut<EditorState>,

    mut editor_hierarchy_event: EventWriter<EditorHierarchyEvent>,
) {
    if controls
        .select_mesh
        .just_pressed(&keyboard_input, &mouse_input, &editor_state)
    {
        editor_hierarchy_event.send(EditorHierarchyEvent::SelectMesh)
    }

    if controls
        .play_pause_editor
        .just_pressed(&keyboard_input, &mouse_input, &editor_state)
    {
        editor_state.active = !editor_state.active;
    }
}

impl Default for EditorControls {
    fn default() -> Self {
        Self {
            select_mesh: Binding {
                input: UserInput::Single(Button::Mouse(MouseButton::Left)),
                conditions: vec![BindingCondition::InViewport(true)],
            },
            play_pause_editor: Binding {
                input: UserInput::Single(Button::Keyboard(KeyCode::E)),
                conditions: Vec::new(),
            },
        }
    }
}

impl std::fmt::Display for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Button::Keyboard(key) => write!(f, "{:?}", key),
            Button::Mouse(mouse) => write!(f, "{:?}", mouse),
        }
    }
}

impl std::fmt::Display for UserInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserInput::Single(single) => {
                write!(f, "{}", single)?;
            }
            UserInput::Chord(chord) => {
                let mut iter = chord.iter();
                let first = iter.next();
                if let Some(first) = first {
                    write!(f, "{}", first)?;
                }

                for remaining in iter {
                    write!(f, "+ {}", remaining)?;
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for Binding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.input)
    }
}

pub struct ControlsWindow;

impl EditorWindow for ControlsWindow {
    type State = ();
    const NAME: &'static str = "Controls";

    fn ui(
        world: &mut World,
        _: bevy_editor_pls_core::editor_window::EditorWindowContext,
        ui: &mut egui::Ui,
    ) {
        let controls = world.get_resource::<EditorControls>().unwrap();

        ui.label(egui::RichText::new("Play/Pause editor").strong());
        ui.label(format!("{}", controls.play_pause_editor));

        ui.label(egui::RichText::new("Select mesh:").strong());
        ui.label(format!("{}", controls.select_mesh));
    }
}
