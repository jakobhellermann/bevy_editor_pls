use bevy::{prelude::*, utils::HashMap};
use bevy_editor_pls_core::{editor_window::EditorWindow, Editor, EditorEvent, EditorState};
use bevy_editor_pls_default_windows::hierarchy::EditorHierarchyEvent;

#[derive(Debug)]
pub enum Button {
    Keyboard(KeyCode),
    Mouse(MouseButton),
}

#[derive(Debug)]
pub enum UserInput {
    Single(Button),
    Chord(Vec<Button>),
}

#[derive(Debug)]
pub enum BindingCondition {
    InViewport(bool),
    EditorActive(bool),
    ListeningForText(bool),
}

impl BindingCondition {
    fn evaluate(&self, editor_state: &EditorState) -> bool {
        match *self {
            BindingCondition::InViewport(in_viewport) => {
                if in_viewport {
                    return !editor_state.pointer_used();
                } else {
                    return editor_state.pointer_used();
                }
            }
            BindingCondition::EditorActive(editor_active) => editor_active == editor_state.active,
            BindingCondition::ListeningForText(listening) => {
                listening == editor_state.listening_for_text
            }
        }
    }
}

impl std::fmt::Display for BindingCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            BindingCondition::InViewport(true) => "mouse in viewport",
            BindingCondition::InViewport(false) => "mouse not in viewport",
            BindingCondition::EditorActive(true) => "editor is active",
            BindingCondition::EditorActive(false) => "editor is not active",
            BindingCondition::ListeningForText(true) => "a ui field is listening for text",
            BindingCondition::ListeningForText(false) => "no ui fields are listening for text",
        };
        f.write_str(str)
    }
}

#[derive(Debug)]
pub struct Binding {
    pub input: UserInput,
    pub conditions: Vec<BindingCondition>,
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
    fn pressed(&self, keyboard_input: &Input<KeyCode>, mouse_input: &Input<MouseButton>) -> bool {
        match self {
            Button::Keyboard(code) => keyboard_input.pressed(*code),
            Button::Mouse(button) => mouse_input.pressed(*button),
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
            UserInput::Chord(chord) => match chord.as_slice() {
                [modifiers @ .., final_key] => {
                    let modifiers_pressed = modifiers
                        .iter()
                        .all(|key| key.pressed(keyboard_input, mouse_input));
                    modifiers_pressed && final_key.just_pressed(keyboard_input, mouse_input)
                }
                [] => false,
            },
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

#[derive(PartialEq, Eq, Hash)]
pub enum Action {
    PlayPauseEditor,
    SelectMesh,
    PauseUnpauseTime,
    FocusSelected,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::PlayPauseEditor => write!(f, "Play/Pause editor"),
            Action::SelectMesh => write!(f, "Select mesh to inspect"),
            Action::PauseUnpauseTime => write!(f, "Pause/Unpause time"),
            Action::FocusSelected => write!(f, "Focus Selected Entity"),
        }
    }
}

#[derive(Resource, Default)]
pub struct EditorControls {
    pub actions: HashMap<Action, Vec<Binding>>,
}

impl EditorControls {
    pub fn insert(&mut self, action: Action, binding: Binding) {
        self.actions.entry(action).or_default().push(binding);
    }
    fn get(&self, action: &Action) -> &[Binding] {
        self.actions.get(action).map_or(&[], Vec::as_slice)
    }

    fn just_pressed(
        &self,
        action: Action,
        keyboard_input: &Input<KeyCode>,
        mouse_input: &Input<MouseButton>,
        editor_state: &EditorState,
    ) -> bool {
        let bindings = &self.actions.get(&action).unwrap();
        bindings
            .iter()
            .any(|binding| binding.just_pressed(keyboard_input, mouse_input, editor_state))
    }
}

pub fn editor_controls_system(
    controls: Res<EditorControls>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut editor_state: ResMut<EditorState>,
    mut editor_events: EventWriter<EditorEvent>,
    mut editor_hierarchy_event: EventWriter<EditorHierarchyEvent>,
    mut editor: ResMut<Editor>,
) {
    if controls.just_pressed(
        Action::SelectMesh,
        &keyboard_input,
        &mouse_input,
        &editor_state,
    ) {
        editor_hierarchy_event.send(EditorHierarchyEvent::SelectMesh)
    }

    if controls.just_pressed(
        Action::PlayPauseEditor,
        &keyboard_input,
        &mouse_input,
        &editor_state,
    ) {
        editor_state.active = !editor_state.active;
        editor_events.send(EditorEvent::Toggle {
            now_active: editor_state.active,
        });
    }

    if controls.just_pressed(
        Action::PauseUnpauseTime,
        &keyboard_input,
        &mouse_input,
        &editor_state,
    ) {
        if let Some(default_window) = editor.window_state_mut::<bevy_editor_pls_default_windows::debug_settings::DebugSettingsWindow>() {
            default_window.pause_time = !default_window.pause_time;
        }
    }

    if controls.just_pressed(
        Action::FocusSelected,
        &keyboard_input,
        &mouse_input,
        &editor_state,
    ) {
        editor_events.send(EditorEvent::FocusSelected);
    }
}

impl EditorControls {
    pub fn unbind(&mut self, action: Action) {
        self.actions.remove(&action);
    }

    pub fn default_bindings() -> Self {
        let mut controls = EditorControls::default();

        controls.insert(
            Action::PauseUnpauseTime,
            Binding {
                input: UserInput::Chord(vec![
                    Button::Keyboard(KeyCode::LControl),
                    Button::Keyboard(KeyCode::Return),
                ]),
                conditions: vec![BindingCondition::ListeningForText(false)],
            },
        );

        controls.insert(
            Action::SelectMesh,
            Binding {
                input: UserInput::Single(Button::Mouse(MouseButton::Left)),
                conditions: vec![
                    BindingCondition::EditorActive(true),
                    BindingCondition::InViewport(true),
                ],
            },
        );
        controls.insert(
            Action::SelectMesh,
            Binding {
                input: UserInput::Chord(vec![
                    Button::Keyboard(KeyCode::LControl),
                    Button::Mouse(MouseButton::Left),
                ]),
                conditions: vec![BindingCondition::EditorActive(false)],
            },
        );
        controls.insert(
            Action::PlayPauseEditor,
            Binding {
                input: UserInput::Single(Button::Keyboard(KeyCode::E)),
                conditions: vec![BindingCondition::ListeningForText(false)],
            },
        );

        controls.insert(
            Action::FocusSelected,
            Binding {
                input: UserInput::Single(Button::Keyboard(KeyCode::F)),
                conditions: vec![BindingCondition::EditorActive(true)],
            },
        );

        controls
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
                    write!(f, " + {}", remaining)?;
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for Binding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.input)?;

        let mut conditions = self.conditions.iter();
        let first_condition = conditions.next();
        if let Some(first) = first_condition {
            write!(f, "\n    when {}", first)?;
        }
        for remaining in conditions {
            write!(f, " and {}", remaining)?;
        }

        Ok(())
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

        for action in &[
            Action::PlayPauseEditor,
            Action::PauseUnpauseTime,
            Action::SelectMesh,
            Action::FocusSelected,
        ] {
            ui.label(egui::RichText::new(action.to_string()).strong());
            let bindings = controls.get(action);
            for binding in bindings {
                ui.add(egui::Label::new(format!("{}", binding)).wrap(false));
            }
        }
    }
}
