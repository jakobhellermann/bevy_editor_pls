use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_editor_pls_core::EditorState;

pub(crate) struct FlycamPlugin;
impl Plugin for FlycamPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(camera_movement.in_set(CameraSystem::Movement))
            .add_system(camera_look)
            .add_system(toggle_cursor);
    }
}

#[derive(SystemSet, PartialEq, Eq, Clone, Hash, Debug)]
pub(crate) enum CameraSystem {
    Movement,
}

#[derive(Component)]
pub struct FlycamControls {
    pub yaw: f32,
    pub pitch: f32,
    pub sensitivity: f32,
    pub enable_movement: bool,
    pub enable_look: bool,

    pub key_forward: KeyCode,
    pub key_back: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
    pub key_boost: KeyCode,
}
impl Default for FlycamControls {
    fn default() -> Self {
        Self {
            yaw: Default::default(),
            pitch: Default::default(),
            sensitivity: 1.0,
            enable_movement: false,
            enable_look: false,
            key_forward: KeyCode::W,
            key_back: KeyCode::S,
            key_left: KeyCode::A,
            key_right: KeyCode::D,
            key_up: KeyCode::Space,
            key_down: KeyCode::LControl,
            key_boost: KeyCode::LShift,
        }
    }
}

fn camera_movement(
    mut cam: Query<(&FlycamControls, &mut Transform)>,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let (flycam, mut cam_transform) = cam.single_mut();
    if !flycam.enable_movement {
        return;
    }

    let if_then_1 = |b| if b { 1.0 } else { 0.0 };
    let forward = if_then_1(keyboard_input.pressed(flycam.key_forward))
        - if_then_1(keyboard_input.pressed(flycam.key_back));
    let sideways = if_then_1(keyboard_input.pressed(flycam.key_right))
        - if_then_1(keyboard_input.pressed(flycam.key_left));
    let up = if_then_1(keyboard_input.pressed(flycam.key_up))
        - if_then_1(keyboard_input.pressed(flycam.key_down));

    if forward == 0.0 && sideways == 0.0 && up == 0.0 {
        return;
    }

    let speed = if keyboard_input.pressed(flycam.key_boost) {
        20.0
    } else {
        5.0
    };

    let movement =
        Vec3::new(sideways, forward, up).normalize_or_zero() * speed * time.raw_delta_seconds();

    let diff = cam_transform.forward() * movement.y
        + cam_transform.right() * movement.x
        + cam_transform.up() * movement.z;
    cam_transform.translation += diff;
}

fn camera_look(
    mouse_input: Res<Input<MouseButton>>,
    mut mouse_motion_event_reader: EventReader<MouseMotion>,
    mut query: Query<(&mut FlycamControls, &mut Transform)>,
) {
    let (mut flycam, mut transform) = query.single_mut();
    if !mouse_input.pressed(MouseButton::Right) {
        return;
    }
    if !flycam.enable_look {
        return;
    }
    let mut delta: Vec2 = Vec2::ZERO;
    for event in mouse_motion_event_reader.iter() {
        delta += event.delta;
    }
    if delta.is_nan() || delta.abs_diff_eq(Vec2::ZERO, f32::EPSILON) {
        return;
    }

    flycam.yaw -= delta.x / 180.0 * flycam.sensitivity;
    flycam.pitch -= delta.y / 180.0 * flycam.sensitivity;

    flycam.pitch = flycam
        .pitch
        .clamp(-std::f32::consts::PI / 2.0, std::f32::consts::PI / 2.0);

    transform.rotation = Quat::from_euler(EulerRot::YXZ, flycam.yaw, flycam.pitch, 0.0);
}

fn toggle_cursor(
    keyboard_input: Res<Input<KeyCode>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    editor_state: Res<EditorState>,
) {
    let Ok(mut window) = primary_window.get_single_mut() else { return };

    if !editor_state.active {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::LAlt) {
        window.cursor.grab_mode = CursorGrabMode::Confined;
        window.cursor.visible = false;
    }
    if keyboard_input.just_released(KeyCode::LAlt) {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }
}
