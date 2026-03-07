use bevy::prelude::*;

use crate::config::{AppConfig, CameraConfig};

#[derive(Resource)]
pub(crate) struct CameraRig {
    pub(crate) orientation: Quat,
    pub(crate) angular_velocity: Vec3,
    pub(crate) distance: f32,
    pub(crate) zoom_velocity: f32,
}

impl CameraRig {
    pub(crate) fn from_config(config: &CameraConfig) -> Self {
        Self {
            orientation: config.initial_orientation(),
            angular_velocity: Vec3::ZERO,
            distance: config.initial_distance,
            zoom_velocity: 0.0,
        }
    }
}

#[derive(Component)]
pub(crate) struct SceneCamera;

pub(crate) fn camera_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    app_config: Res<AppConfig>,
    mut camera_rig: ResMut<CameraRig>,
) {
    if keys.just_pressed(KeyCode::Backspace) {
        stop_angular_momentum(&mut camera_rig);
        println!("Stopped camera rotation momentum.");
    }

    let dt = time.delta_secs();
    let torque_step = app_config.camera.rotation_accel * dt;
    let ctrl_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    if !ctrl_pressed && keys.pressed(KeyCode::ArrowUp) {
        camera_rig.angular_velocity.x += torque_step;
    }
    if !ctrl_pressed && keys.pressed(KeyCode::ArrowDown) {
        camera_rig.angular_velocity.x -= torque_step;
    }
    if !ctrl_pressed && keys.pressed(KeyCode::ArrowLeft) {
        camera_rig.angular_velocity.y += torque_step;
    }
    if !ctrl_pressed && keys.pressed(KeyCode::ArrowRight) {
        camera_rig.angular_velocity.y -= torque_step;
    }
    if keys.pressed(KeyCode::KeyQ) {
        camera_rig.angular_velocity.z += torque_step;
    }
    if keys.pressed(KeyCode::KeyE) {
        camera_rig.angular_velocity.z -= torque_step;
    }
    if keys.pressed(KeyCode::KeyW) {
        camera_rig.zoom_velocity -= app_config.camera.zoom_accel * dt;
    }
    if keys.pressed(KeyCode::KeyS) {
        camera_rig.zoom_velocity += app_config.camera.zoom_accel * dt;
    }
}

pub(crate) fn camera_motion_system(
    time: Res<Time>,
    app_config: Res<AppConfig>,
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

    camera_rig.angular_velocity = damped_angular_velocity(
        camera_rig.angular_velocity,
        app_config.camera.angular_damping,
        dt,
        app_config.camera.preserve_angular_momentum,
    );
    camera_rig.zoom_velocity *= f32::exp(-app_config.camera.zoom_damping * dt);
    let (min_distance, max_distance) = app_config.camera.distance_bounds();
    camera_rig.distance =
        (camera_rig.distance + camera_rig.zoom_velocity * dt).clamp(min_distance, max_distance);

    let translation = camera_rig.orientation * Vec3::new(0.0, 0.0, camera_rig.distance);
    *transform = Transform::from_translation(translation)
        .looking_at(Vec3::ZERO, camera_rig.orientation * Vec3::Y);
}

fn stop_angular_momentum(camera_rig: &mut CameraRig) {
    camera_rig.angular_velocity = Vec3::ZERO;
}

fn damped_angular_velocity(
    angular_velocity: Vec3,
    angular_damping: f32,
    dt: f32,
    preserve_angular_momentum: bool,
) -> Vec3 {
    if preserve_angular_momentum {
        return angular_velocity;
    }

    angular_velocity * f32::exp(-angular_damping * dt)
}

#[cfg(test)]
mod tests {
    use super::{CameraRig, damped_angular_velocity, stop_angular_momentum};
    use bevy::prelude::{Quat, Vec3};

    #[test]
    fn preserved_angular_momentum_keeps_velocity_constant() {
        let angular_velocity = Vec3::new(1.0, -0.5, 0.25);

        assert_eq!(
            damped_angular_velocity(angular_velocity, 2.2, 0.5, true),
            angular_velocity
        );
    }

    #[test]
    fn damped_angular_momentum_slows_velocity() {
        let angular_velocity = Vec3::new(1.0, -0.5, 0.25);
        let next_velocity = damped_angular_velocity(angular_velocity, 2.2, 0.5, false);

        assert!(next_velocity.length() < angular_velocity.length());
    }

    #[test]
    fn stop_angular_momentum_clears_only_rotation_velocity() {
        let mut camera_rig = CameraRig {
            orientation: Quat::IDENTITY,
            angular_velocity: Vec3::new(1.0, -0.5, 0.25),
            distance: 14.0,
            zoom_velocity: 3.0,
        };

        stop_angular_momentum(&mut camera_rig);

        assert_eq!(camera_rig.angular_velocity, Vec3::ZERO);
        assert_eq!(camera_rig.zoom_velocity, 3.0);
        assert_eq!(camera_rig.distance, 14.0);
    }
}
