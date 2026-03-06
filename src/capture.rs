use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use bevy::app::AppExit;
use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};

use crate::config::AppConfig;

pub(crate) struct LaunchConfig {
    pub(crate) capture_path: Option<PathBuf>,
    pub(crate) capture_delay_frames: u32,
}

impl LaunchConfig {
    pub(crate) fn from_env(default_capture_delay_frames: u32) -> Result<Self, String> {
        parse_launch_config(std::env::args_os().skip(1), default_capture_delay_frames)
    }
}

fn parse_launch_config(
    args: impl IntoIterator<Item = OsString>,
    default_capture_delay_frames: u32,
) -> Result<LaunchConfig, String> {
    let mut capture_path = None;
    let mut capture_delay_frames = default_capture_delay_frames;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.to_string_lossy().as_ref() {
            "--capture" => {
                let Some(path) = args.next() else {
                    return Err("Missing path after --capture".to_string());
                };
                capture_path = Some(PathBuf::from(path));
            }
            "--capture-delay-frames" => {
                let Some(frame_count) = args.next() else {
                    return Err("Missing frame count after --capture-delay-frames".to_string());
                };
                let frame_count = frame_count.to_string_lossy();
                capture_delay_frames = frame_count.parse::<u32>().map_err(|_| {
                    format!("Invalid frame count for --capture-delay-frames: {frame_count}")
                })?;
            }
            "--help" | "-h" => {
                return Err(
                    "Usage: cargo run -- [--capture <output.png>] [--capture-delay-frames <n>]\nF12 saves a screenshot during normal interactive runs."
                        .to_string(),
                );
            }
            other => {
                return Err(format!("Unknown argument: {other}"));
            }
        }
    }

    Ok(LaunchConfig {
        capture_path,
        capture_delay_frames,
    })
}

#[derive(Resource, Default)]
pub(crate) struct ScreenshotCounter {
    next_index: u32,
}

#[derive(Resource)]
pub(crate) struct AutomatedCapture {
    path: PathBuf,
    requested: bool,
    trigger_frame: u32,
}

impl AutomatedCapture {
    pub(crate) fn new(path: PathBuf, trigger_frame: u32) -> Self {
        Self {
            path,
            requested: false,
            trigger_frame,
        }
    }
}

pub(crate) fn manual_screenshot_input_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    app_config: Res<AppConfig>,
    mut screenshot_counter: ResMut<ScreenshotCounter>,
) {
    if !keys.just_pressed(KeyCode::F12) {
        return;
    }

    let path = PathBuf::from(&app_config.capture.output_dir)
        .join(format!("intergen-{:04}.png", screenshot_counter.next_index));
    screenshot_counter.next_index += 1;
    request_screenshot(&mut commands, path, false);
}

pub(crate) fn automated_capture_system(
    mut commands: Commands,
    frame_count: Res<FrameCount>,
    automated_capture: Option<ResMut<AutomatedCapture>>,
) {
    let Some(mut automated_capture) = automated_capture else {
        return;
    };

    if automated_capture.requested || frame_count.0 < automated_capture.trigger_frame {
        return;
    }

    automated_capture.requested = true;
    request_screenshot(&mut commands, automated_capture.path.clone(), true);
}

fn request_screenshot(commands: &mut Commands, path: PathBuf, exit_after_capture: bool) {
    if !ensure_parent_dir(&path) {
        return;
    }

    println!("Saving screenshot to {}", path.display());
    let mut entity = commands.spawn(Screenshot::primary_window());
    entity.observe(save_to_disk(path));
    if exit_after_capture {
        entity.observe(exit_after_screenshot_capture);
    }
}

fn ensure_parent_dir(path: &Path) -> bool {
    let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    else {
        return true;
    };

    if let Err(error) = fs::create_dir_all(parent) {
        eprintln!(
            "Could not create screenshot directory {}: {error}",
            parent.display()
        );
        return false;
    }

    true
}

fn exit_after_screenshot_capture(_: On<ScreenshotCaptured>, mut app_exit: MessageWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::Path;

    use super::parse_launch_config;
    use crate::config::CaptureConfig;

    #[test]
    fn launch_config_parses_capture_path() {
        let config = parse_launch_config(
            [
                OsString::from("--capture"),
                OsString::from("screenshots/test.png"),
            ],
            CaptureConfig::default().default_capture_delay_frames,
        )
        .expect("capture path should parse");

        assert_eq!(
            config.capture_path.as_deref(),
            Some(Path::new("screenshots/test.png"))
        );
        assert_eq!(
            config.capture_delay_frames,
            CaptureConfig::default().default_capture_delay_frames
        );
    }

    #[test]
    fn launch_config_parses_capture_delay_frames() {
        let config = parse_launch_config(
            [
                OsString::from("--capture-delay-frames"),
                OsString::from("64"),
            ],
            CaptureConfig::default().default_capture_delay_frames,
        )
        .expect("capture delay should parse");

        assert_eq!(config.capture_delay_frames, 64);
        assert_eq!(config.capture_path, None);
    }
}
