use bevy::prelude::*;
use bevy::window::PresentMode;

use crate::camera::{CameraRig, camera_input_system, camera_motion_system};
use crate::capture::{
    AutomatedCapture, LaunchConfig, ScreenshotCounter, automated_capture_system,
    manual_screenshot_input_system,
};
use crate::config::AppConfig;
use crate::effect_tuner::{EffectTunerState, apply_effect_tuner_system, effect_tuner_input_system};
use crate::effects::EffectsPlugin;
use crate::generation::generation_input_system;
use crate::scene::setup_scene;
use crate::ui::{HelpOverlayState, toggle_help_overlay_system, update_effect_tuner_overlay_system};

pub fn run() {
    let app_config = match AppConfig::load_from_default_path() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            return;
        }
    };

    let launch_config =
        match LaunchConfig::from_env(app_config.capture.default_capture_delay_frames) {
            Ok(config) => config,
            Err(error) => {
                eprintln!("{error}");
                return;
            }
        };

    let mut app = App::new();
    app.insert_resource(ClearColor(app_config.rendering.clear_color()))
        .insert_resource(AmbientLight {
            color: app_config.rendering.ambient_light_color(),
            brightness: app_config.rendering.ambient_light_brightness,
            ..default()
        })
        .insert_resource(app_config.clone())
        .insert_resource(CameraRig::from_config(&app_config.camera))
        .insert_resource(HelpOverlayState::default())
        .insert_resource(EffectTunerState::from_config(&app_config.effects))
        .insert_resource(ScreenshotCounter::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: app_config.window.title.clone(),
                resolution: (app_config.window.width, app_config.window.height).into(),
                present_mode: PresentMode::from(app_config.window.present_mode),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EffectsPlugin)
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                toggle_help_overlay_system,
                effect_tuner_input_system,
                camera_input_system,
                camera_motion_system,
                generation_input_system,
                apply_effect_tuner_system,
                update_effect_tuner_overlay_system,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (manual_screenshot_input_system, automated_capture_system),
        );

    if let Some(path) = launch_config.capture_path {
        app.insert_resource(AutomatedCapture::new(
            path,
            launch_config.capture_delay_frames,
        ));
    }

    app.run();
}
