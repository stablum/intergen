use std::f32::consts::FRAC_PI_4;

use bevy::prelude::*;

const CAMERA_DISTANCE: f32 = 14.0;
const CAMERA_ROTATION_ACCEL: f32 = 1.9;
const CAMERA_ZOOM_ACCEL: f32 = 24.0;
const ANGULAR_DAMPING: f32 = 2.2;
const ZOOM_DAMPING: f32 = 4.0;
const MIN_CAMERA_DISTANCE: f32 = 4.0;
const MAX_CAMERA_DISTANCE: f32 = 48.0;

#[derive(Resource)]
pub(crate) struct CameraRig {
    pub(crate) orientation: Quat,
    pub(crate) angular_velocity: Vec3,
    pub(crate) distance: f32,
    pub(crate) zoom_velocity: f32,
}

impl Default for CameraRig {
    fn default() -> Self {
        Self {
            orientation: Quat::from_euler(EulerRot::YXZ, -FRAC_PI_4, -0.45, 0.15),
            angular_velocity: Vec3::ZERO,
            distance: CAMERA_DISTANCE,
            zoom_velocity: 0.0,
        }
    }
}

#[derive(Component)]
pub(crate) struct SceneCamera;

pub(crate) fn camera_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera_rig: ResMut<CameraRig>,
) {
    let dt = time.delta_secs();
    let torque_step = CAMERA_ROTATION_ACCEL * dt;

    if keys.pressed(KeyCode::ArrowUp) {
        camera_rig.angular_velocity.x += torque_step;
    }
    if keys.pressed(KeyCode::ArrowDown) {
        camera_rig.angular_velocity.x -= torque_step;
    }
    if keys.pressed(KeyCode::ArrowLeft) {
        camera_rig.angular_velocity.y += torque_step;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        camera_rig.angular_velocity.y -= torque_step;
    }
    if keys.pressed(KeyCode::KeyQ) {
        camera_rig.angular_velocity.z += torque_step;
    }
    if keys.pressed(KeyCode::KeyE) {
        camera_rig.angular_velocity.z -= torque_step;
    }
    if keys.pressed(KeyCode::KeyW) {
        camera_rig.zoom_velocity -= CAMERA_ZOOM_ACCEL * dt;
    }
    if keys.pressed(KeyCode::KeyS) {
        camera_rig.zoom_velocity += CAMERA_ZOOM_ACCEL * dt;
    }
}

pub(crate) fn camera_motion_system(
    time: Res<Time>,
    mut camera_rig: ResMut<CameraRig>,
    mut query: Query<&mut Transform, With<SceneCamera>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    if camera_rig.angular_velocity.length_squared() > 0.0 {
        let delta = Quat::from_scaled_axis(camera_rig.angular_velocity * dt);
        camera_rig.orientation = (delta * camera_rig.orientation).normalize();
    }

    camera_rig.angular_velocity *= f32::exp(-ANGULAR_DAMPING * dt);
    camera_rig.zoom_velocity *= f32::exp(-ZOOM_DAMPING * dt);
    camera_rig.distance = (camera_rig.distance + camera_rig.zoom_velocity * dt)
        .clamp(MIN_CAMERA_DISTANCE, MAX_CAMERA_DISTANCE);

    let translation = camera_rig.orientation * Vec3::new(0.0, 0.0, camera_rig.distance);
    *transform = Transform::from_translation(translation)
        .looking_at(Vec3::ZERO, camera_rig.orientation * Vec3::Y);
}
