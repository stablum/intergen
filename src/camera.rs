use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;

use crate::config::{AppConfig, CameraConfig};
use crate::control_page::{
    ControlPageInputMask, just_pressed_unmasked, just_released_unmasked, pressed_unmasked,
};
use crate::recent_changes::RecentChangesState;

const CAMERA_CHANGE_THROTTLE_SECS: f32 = 0.25;

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
    control_page_input_mask: Res<ControlPageInputMask>,
    mut camera_rig: ResMut<CameraRig>,
    mut recent_changes: ResMut<RecentChangesState>,
) {
    let input_mask = *control_page_input_mask;
    let now_secs = time.elapsed_secs();

    if just_pressed_unmasked(&keys, input_mask, KeyCode::Backspace) {
        let previous_angular_velocity = camera_rig.angular_velocity;
        stop_angular_momentum(&mut camera_rig);
        record_stopped_angular_velocity(
            &mut recent_changes,
            previous_angular_velocity,
            camera_rig.angular_velocity,
            now_secs,
        );
        println!("Stopped camera rotation momentum.");
    }

    let dt = time.delta_secs();
    let torque_step = app_config.camera.rotation_accel * dt;
    let ctrl_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    let pitch_input = signed_axis_input(
        &keys,
        input_mask,
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
        !ctrl_pressed,
    );
    camera_rig.angular_velocity.x += torque_step * pitch_input.delta_sign;
    record_camera_parameter_change(
        &mut recent_changes,
        "camera.angular_velocity.x",
        camera_rig.angular_velocity.x,
        pitch_input,
        now_secs,
    );

    let yaw_input = signed_axis_input(
        &keys,
        input_mask,
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
        !ctrl_pressed,
    );
    camera_rig.angular_velocity.y += torque_step * yaw_input.delta_sign;
    record_camera_parameter_change(
        &mut recent_changes,
        "camera.angular_velocity.y",
        camera_rig.angular_velocity.y,
        yaw_input,
        now_secs,
    );

    let roll_input = signed_axis_input(&keys, input_mask, KeyCode::KeyQ, KeyCode::KeyE, true);
    camera_rig.angular_velocity.z += torque_step * roll_input.delta_sign;
    record_camera_parameter_change(
        &mut recent_changes,
        "camera.angular_velocity.z",
        camera_rig.angular_velocity.z,
        roll_input,
        now_secs,
    );

    let zoom_input = signed_axis_input(&keys, input_mask, KeyCode::KeyS, KeyCode::KeyW, true);
    camera_rig.zoom_velocity += app_config.camera.zoom_accel * dt * zoom_input.delta_sign;
    record_camera_parameter_change(
        &mut recent_changes,
        "camera.zoom_velocity",
        camera_rig.zoom_velocity,
        zoom_input,
        now_secs,
    );
}

pub(crate) fn camera_motion_system(
    time: Res<Time>,
    app_config: Res<AppConfig>,
    mut camera_rig: ResMut<CameraRig>,
    mut query: Query<&mut Transform, With<SceneCamera>>,
) {
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

    sync_scene_camera_transform(&camera_rig, &mut query);
}

pub(crate) fn sync_scene_camera_transform<F: QueryFilter>(
    camera_rig: &CameraRig,
    query: &mut Query<&mut Transform, F>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let translation = camera_rig.orientation * Vec3::new(0.0, 0.0, camera_rig.distance);
    *transform = Transform::from_translation(translation)
        .looking_at(Vec3::ZERO, camera_rig.orientation * Vec3::Y);
}

fn stop_angular_momentum(camera_rig: &mut CameraRig) {
    camera_rig.angular_velocity = Vec3::ZERO;
}

#[derive(Clone, Copy, Debug, Default)]
struct SignedAxisInput {
    delta_sign: f32,
    just_started: bool,
    just_released: bool,
}

fn signed_axis_input(
    keys: &ButtonInput<KeyCode>,
    input_mask: ControlPageInputMask,
    positive_key: KeyCode,
    negative_key: KeyCode,
    enabled: bool,
) -> SignedAxisInput {
    if !enabled {
        return SignedAxisInput::default();
    }

    let positive_pressed = pressed_unmasked(keys, input_mask, positive_key);
    let negative_pressed = pressed_unmasked(keys, input_mask, negative_key);
    SignedAxisInput {
        delta_sign: f32::from(positive_pressed as u8) - f32::from(negative_pressed as u8),
        just_started: just_pressed_unmasked(keys, input_mask, positive_key)
            || just_pressed_unmasked(keys, input_mask, negative_key),
        just_released: just_released_unmasked(keys, input_mask, positive_key)
            || just_released_unmasked(keys, input_mask, negative_key),
    }
}

fn record_stopped_angular_velocity(
    recent_changes: &mut RecentChangesState,
    previous_angular_velocity: Vec3,
    angular_velocity: Vec3,
    now_secs: f32,
) {
    if previous_angular_velocity.x != angular_velocity.x {
        recent_changes.record(
            "camera.angular_velocity.x",
            format_camera_parameter_value(angular_velocity.x),
            now_secs,
        );
    }
    if previous_angular_velocity.y != angular_velocity.y {
        recent_changes.record(
            "camera.angular_velocity.y",
            format_camera_parameter_value(angular_velocity.y),
            now_secs,
        );
    }
    if previous_angular_velocity.z != angular_velocity.z {
        recent_changes.record(
            "camera.angular_velocity.z",
            format_camera_parameter_value(angular_velocity.z),
            now_secs,
        );
    }
}

fn record_camera_parameter_change(
    recent_changes: &mut RecentChangesState,
    label: &'static str,
    value: f32,
    input: SignedAxisInput,
    now_secs: f32,
) {
    if input.delta_sign == 0.0 && !input.just_released {
        return;
    }

    let formatted_value = format_camera_parameter_value(value);
    if input.just_started || input.just_released {
        recent_changes.record(label, formatted_value, now_secs);
    } else {
        recent_changes.record_throttled(
            label,
            formatted_value,
            now_secs,
            CAMERA_CHANGE_THROTTLE_SECS,
        );
    }
}

fn format_camera_parameter_value(value: f32) -> String {
    format!("{value:+.3}")
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
    use super::{
        CameraRig, damped_angular_velocity, format_camera_parameter_value,
        record_stopped_angular_velocity, stop_angular_momentum,
    };
    use bevy::prelude::{Quat, Vec3};

    use crate::recent_changes::RecentChangesState;

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

    #[test]
    fn camera_parameter_values_use_signed_fixed_precision() {
        assert_eq!(format_camera_parameter_value(0.12549), "+0.125");
        assert_eq!(format_camera_parameter_value(-1.2), "-1.200");
        assert_eq!(format_camera_parameter_value(0.0), "+0.000");
    }

    #[test]
    fn stopping_angular_momentum_records_changed_velocity_axes() {
        let mut recent_changes = RecentChangesState::default();

        record_stopped_angular_velocity(
            &mut recent_changes,
            Vec3::new(1.0, 0.0, -0.5),
            Vec3::ZERO,
            2.0,
        );

        let rows = recent_changes.snapshot(2.0).rows;

        assert_eq!(rows.len(), 2);
        assert!(
            rows.iter()
                .any(|row| row.label == "camera.angular_velocity.x" && row.value == "+0.000")
        );
        assert!(
            rows.iter()
                .any(|row| row.label == "camera.angular_velocity.z" && row.value == "+0.000")
        );
    }
}
