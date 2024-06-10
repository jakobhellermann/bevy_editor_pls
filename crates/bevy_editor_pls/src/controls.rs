use bevy::{prelude::*, utils::HashMap};
use bevy_editor_pls_core::{editor_window::EditorWindow, Editor, EditorEvent};

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
    fn evaluate(&self, editor: &Editor) -> bool {
        match *self {
            BindingCondition::InViewport(in_viewport) => {
                if in_viewport {
                    !editor.pointer_used()
                } else {
                    editor.pointer_used()
                }
            }
            BindingCondition::EditorActive(editor_active) => editor_active == editor.active(),
            BindingCondition::ListeningForText(listening) => {
                listening == editor.listening_for_text()
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

impl From<UserInput> for Binding {
    fn from(input: UserInput) -> Self {
        Binding {
            input,
            conditions: Vec::new(),
        }
    }
}

impl Button {
    fn just_pressed(
        &self,
        keyboard_input: &ButtonInput<KeyCode>,
        mouse_input: &ButtonInput<MouseButton>,
    ) -> bool {
        match self {
            Button::Keyboard(code) => keyboard_input.just_pressed(*code),
            Button::Mouse(button) => mouse_input.just_pressed(*button),
        }
    }
    fn pressed(
        &self,
        keyboard_input: &ButtonInput<KeyCode>,
        mouse_input: &ButtonInput<MouseButton>,
    ) -> bool {
        match self {
            Button::Keyboard(code) => keyboard_input.pressed(*code),
            Button::Mouse(button) => mouse_input.pressed(*button),
        }
    }
}

impl UserInput {
    fn just_pressed(
        &self,
        keyboard_input: &ButtonInput<KeyCode>,
        mouse_input: &ButtonInput<MouseButton>,
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
        keyboard_input: &ButtonInput<KeyCode>,
        mouse_input: &ButtonInput<MouseButton>,
        editor: &Editor,
    ) -> bool {
        let can_trigger = self
            .conditions
            .iter()
            .all(|condition| condition.evaluate(editor));
        if !can_trigger {
            return false;
        }

        self.input.just_pressed(keyboard_input, mouse_input)
    }
}

#[derive(PartialEq, Eq, Hash)]
pub enum Action {
    PlayPauseEditor,
    PauseUnpauseTime,
    FocusSelected,

    #[cfg(feature = "default_windows")]
    SetGizmoModeTranslate,
    #[cfg(feature = "default_windows")]
    SetGizmoModeRotate,
    #[cfg(feature = "default_windows")]
    SetGizmoModeScale,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::PlayPauseEditor => write!(f, "Play/Pause editor"),
            Action::PauseUnpauseTime => write!(f, "Pause/Unpause time"),
            Action::FocusSelected => write!(f, "Focus Selected Entity"),
            #[cfg(feature = "default_windows")]
            Action::SetGizmoModeTranslate => write!(f, "Activate translation gizmo"),
            #[cfg(feature = "default_windows")]
            Action::SetGizmoModeRotate => write!(f, "Activate rotation gizmo"),
            #[cfg(feature = "default_windows")]
            Action::SetGizmoModeScale => write!(f, "Activate scale gizmo"),
        }
    }
}

/// Resource mapping input bindings to [`Action`]s
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
        keyboard_input: &ButtonInput<KeyCode>,
        mouse_input: &ButtonInput<MouseButton>,
        editor: &Editor,
    ) -> bool {
        let bindings = &self.actions.get(&action).unwrap();
        bindings
            .iter()
            .any(|binding| binding.just_pressed(keyboard_input, mouse_input, editor))
    }
}

pub fn editor_controls_system(
    controls: Res<EditorControls>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut editor_events: EventWriter<EditorEvent>,
    mut editor: ResMut<Editor>,
) {
    if controls.just_pressed(
        Action::PlayPauseEditor,
        &keyboard_input,
        &mouse_input,
        &editor,
    ) && !editor.always_active()
    {
        let was_active = editor.active();
        editor.set_active(!was_active);
        editor_events.send(EditorEvent::Toggle {
            now_active: !was_active,
        });
    }

    if controls.just_pressed(
        Action::PauseUnpauseTime,
        &keyboard_input,
        &mouse_input,
        &editor,
    ) {
        if let Some(default_window) = editor.window_state_mut::<bevy_editor_pls_default_windows::debug_settings::DebugSettingsWindow>() {
            default_window.pause_time = !default_window.pause_time;
        }
    }

    if controls.just_pressed(
        Action::FocusSelected,
        &keyboard_input,
        &mouse_input,
        &editor,
    ) {
        editor_events.send(EditorEvent::FocusSelected);
    }

    #[cfg(feature = "default_windows")]
    {
        if controls.just_pressed(
            Action::SetGizmoModeTranslate,
            &keyboard_input,
            &mouse_input,
            &editor,
        ) {
            editor
                .window_state_mut::<bevy_editor_pls_default_windows::gizmos::GizmoWindow>()
                .unwrap()
                .gizmo_modes = transform_gizmo_bevy::GizmoMode::all_translate();
        }
        if controls.just_pressed(
            Action::SetGizmoModeRotate,
            &keyboard_input,
            &mouse_input,
            &editor,
        ) {
            editor
                .window_state_mut::<bevy_editor_pls_default_windows::gizmos::GizmoWindow>()
                .unwrap()
                .gizmo_modes = transform_gizmo_bevy::GizmoMode::all_rotate();
        }
        if controls.just_pressed(
            Action::SetGizmoModeScale,
            &keyboard_input,
            &mouse_input,
            &editor,
        ) {
            editor
                .window_state_mut::<bevy_editor_pls_default_windows::gizmos::GizmoWindow>()
                .unwrap()
                .gizmo_modes = transform_gizmo_bevy::GizmoMode::all_scale();
        }
    }
}

impl EditorControls {
    pub fn unbind(&mut self, action: Action) {
        self.actions.remove(&action);
    }

    /// - `C-Enter`: pause time
    /// - `E`: toggle editor
    /// - `F`: focus on selected entity
    /// `T/R/S`: show translate/rotate/scale gizmo
    pub fn default_bindings() -> Self {
        let mut controls = EditorControls::default();

        controls.insert(
            Action::PauseUnpauseTime,
            Binding {
                input: UserInput::Chord(vec![
                    Button::Keyboard(KeyCode::ControlLeft),
                    Button::Keyboard(KeyCode::Enter),
                ]),
                conditions: vec![BindingCondition::ListeningForText(false)],
            },
        );

        controls.insert(
            Action::PlayPauseEditor,
            Binding {
                input: UserInput::Single(Button::Keyboard(KeyCode::KeyE)),
                conditions: vec![BindingCondition::ListeningForText(false)],
            },
        );

        controls.insert(
            Action::FocusSelected,
            Binding {
                input: UserInput::Single(Button::Keyboard(KeyCode::KeyF)),
                conditions: vec![BindingCondition::EditorActive(true)],
            },
        );

        #[cfg(feature = "default_windows")]
        {
            controls.insert(
                Action::SetGizmoModeTranslate,
                UserInput::Single(Button::Keyboard(KeyCode::KeyT)).into(),
            );
            controls.insert(
                Action::SetGizmoModeRotate,
                UserInput::Single(Button::Keyboard(KeyCode::KeyR)).into(),
            );
            controls.insert(
                Action::SetGizmoModeScale,
                UserInput::Single(Button::Keyboard(KeyCode::KeyS)).into(),
            );
        }

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
