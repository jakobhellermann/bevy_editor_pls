// adapter from the bevy cookbook example <https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html>

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    render::camera::Projection,
};

pub struct PanOrbitCameraPlugin;
impl Plugin for PanOrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(pan_orbit_camera.label(CameraSystem::Movement));
    }
}

/// Tags an entity as capable of panning and orbiting.
#[derive(Component)]
pub struct PanOrbitCamera {
    pub enabled: bool,

    /// The "focus point" to orbit around. It is automatically updated when panning the camera
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,

    pub orbit_button: MouseButton,
    pub pan_button: MouseButton,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        PanOrbitCamera {
            enabled: true,

            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,

            orbit_button: MouseButton::Right,
            pan_button: MouseButton::Middle,
        }
    }
}

#[derive(SystemLabel, PartialEq, Eq, Clone, Hash, Debug)]
pub(crate) enum CameraSystem {
    Movement,
}

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
fn pan_orbit_camera(
    windows: Res<Windows>,

    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<Input<MouseButton>>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
) {
    // change input mapping for orbit and panning here
    let (mut pan_orbit, mut transform, projection) = query.single_mut();

    if !pan_orbit.enabled {
        return;
    }

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;

    if input_mouse.pressed(pan_orbit.orbit_button) {
        for ev in ev_motion.iter() {
            rotation_move += ev.delta;
        }
    } else if input_mouse.pressed(pan_orbit.pan_button) {
        // Pan only if we're not rotating at the moment
        for ev in ev_motion.iter() {
            pan += ev.delta;
        }
    }
    for ev in ev_scroll.iter() {
        scroll += ev.y;
    }
    if input_mouse.just_released(pan_orbit.orbit_button)
        || input_mouse.just_pressed(pan_orbit.orbit_button)
    {
        orbit_button_changed = true;
    }

    if orbit_button_changed {
        // only check for upside down when orbiting started or ended this frame
        // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
        let up = transform.rotation * Vec3::Y;
        pan_orbit.upside_down = up.y <= 0.0;
    }

    let mut any = false;
    if rotation_move.length_squared() > 0.0 {
        any = true;
        let delta_x = {
            let delta = rotation_move.x / 180.0;
            if pan_orbit.upside_down {
                -delta
            } else {
                delta
            }
        };
        let delta_y = rotation_move.y / 180.0;
        let yaw = Quat::from_rotation_y(-delta_x);
        let pitch = Quat::from_rotation_x(-delta_y);
        transform.rotation = yaw * transform.rotation; // rotate around global y axis
        transform.rotation = transform.rotation * pitch; // rotate around local x axis
    } else if pan.length_squared() > 0.0 {
        any = true;
        // make panning distance independent of resolution and FOV,
        let window_size = get_primary_window_size(&windows);
        if let Projection::Perspective(projection) = projection {
            pan *=
                Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window_size;
        }
        // translate by local axes
        let right = transform.rotation * Vec3::X * -pan.x;
        let up = transform.rotation * Vec3::Y * pan.y;
        // make panning proportional to distance away from focus point
        let translation = (right + up) * pan_orbit.radius;
        pan_orbit.focus += translation;
    } else if scroll.abs() > 0.0 {
        any = true;
        pan_orbit.radius -= scroll * pan_orbit.radius * 0.1;
        // dont allow zoom to reach zero or you get stuck
        pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
    }

    if any {
        // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
        // parent = x and y rotation
        // child = z-offset
        let rot_matrix = Mat3::from_quat(transform.rotation);
        transform.translation =
            pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
    }
}

fn get_primary_window_size(windows: &Windows) -> Vec2 {
    let window = windows.get_primary().unwrap();
    let window = Vec2::new(window.width() as f32, window.height() as f32);
    window
}
