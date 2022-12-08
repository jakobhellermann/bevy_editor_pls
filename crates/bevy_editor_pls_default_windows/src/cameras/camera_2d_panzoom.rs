// copied and extended from https://raw.githubusercontent.com/johanhelsing/bevy_pancam/main/src/lib.rs

use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    render::camera::OrthographicProjection,
};

#[derive(SystemLabel, PartialEq, Eq, Clone, Hash, Debug)]
pub(crate) enum CameraSystem {
    Movement,
}

#[derive(Default)]
pub(crate) struct PanCamPlugin;

impl Plugin for PanCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(camera_movement.label(CameraSystem::Movement))
            .add_system(camera_zoom.label(CameraSystem::Movement));
    }
}

// Zoom doesn't work on bevy 0.5 due to: https://github.com/bevyengine/bevy/pull/2015
fn camera_zoom(
    mut query: Query<(&PanCamControls, &mut OrthographicProjection)>,
    mut scroll_events: EventReader<MouseWheel>,
) {
    let pixels_per_line = 100.; // Maybe make configurable?
    let scroll = scroll_events
        .iter()
        .map(|ev| match ev.unit {
            MouseScrollUnit::Pixel => ev.y,
            MouseScrollUnit::Line => ev.y * pixels_per_line,
        })
        .sum::<f32>();

    if scroll == 0. {
        return;
    }

    for (_cam, mut projection) in query.iter_mut() {
        projection.scale = (projection.scale * (1. + -scroll * 0.001)).max(0.00001);
    }
}

fn camera_movement(
    mut windows: ResMut<Windows>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut query: Query<(&PanCamControls, &mut Transform, &OrthographicProjection)>,
    mut last_pos: Local<Option<Vec2>>,
) {
    let window = if let Some(window) = windows.get_primary_mut() {
        window
    } else {
        return;
    };

    // Use position instead of MouseMotion, otherwise we don't get acceleration movement
    let current_pos = match window.cursor_position() {
        Some(current_pos) => current_pos,
        None => return,
    };
    let delta = current_pos - last_pos.unwrap_or(current_pos);

    for (cam, mut transform, projection) in query.iter_mut() {
        if !cam.enabled {
            continue;
        }

        if cam
            .grab_buttons
            .iter()
            .any(|btn| mouse_buttons.pressed(*btn))
        {
            let scaling = Vec2::new(
                window.width() / (projection.right - projection.left),
                window.height() / (projection.top - projection.bottom),
            ) * projection.scale;

            transform.translation -= (delta * scaling).extend(0.);
        }
    }
    *last_pos = Some(current_pos);
}

#[derive(Component)]
pub struct PanCamControls {
    pub enabled: bool,
    pub grab_buttons: Vec<MouseButton>,
}

impl Default for PanCamControls {
    fn default() -> Self {
        Self {
            enabled: true,
            grab_buttons: vec![MouseButton::Left, MouseButton::Right, MouseButton::Middle],
        }
    }
}
