use bevy::prelude::*;
use bevy::window::PresentMode;

use crate::camera::{CameraRig, camera_input_system, camera_motion_system};
use crate::capture::{
    AutomatedCapture, LaunchConfig, ScreenshotCounter, automated_capture_system,
    manual_screenshot_input_system,
};
use crate::generation::generation_input_system;
use crate::scene::setup_scene;
use crate::ui::{HelpOverlayState, toggle_help_overlay_system};

pub fn run() {
    let launch_config = match LaunchConfig::from_env() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            return;
        }
    };

    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.035, 0.04, 0.06)))
        .insert_resource(AmbientLight {
            color: Color::srgb(0.7, 0.74, 0.82),
            brightness: 12.0,
            ..default()
        })
        .insert_resource(CameraRig::default())
        .insert_resource(HelpOverlayState::default())
        .insert_resource(ScreenshotCounter::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "intergen".into(),
                resolution: (1440, 960).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                toggle_help_overlay_system,
                camera_input_system,
                camera_motion_system,
                generation_input_system,
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
