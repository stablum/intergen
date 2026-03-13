use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bevy::app::AppExit;
use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};

use crate::config::AppConfig;

pub(crate) struct LaunchConfig {
    pub(crate) capture_path: Option<PathBuf>,
    pub(crate) capture_delay_frames: u32,
    pub(crate) blend_export_path: Option<PathBuf>,
    pub(crate) blend_export_delay_frames: u32,
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
    let mut blend_export_path = None;
    let mut blend_export_delay_frames = default_capture_delay_frames;
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
            "--export-blend" => {
                let Some(path) = args.next() else {
                    return Err("Missing path after --export-blend".to_string());
                };
                blend_export_path = Some(PathBuf::from(path));
            }
            "--export-blend-delay-frames" => {
                let Some(frame_count) = args.next() else {
                    return Err("Missing frame count after --export-blend-delay-frames".to_string());
                };
                let frame_count = frame_count.to_string_lossy();
                blend_export_delay_frames = frame_count.parse::<u32>().map_err(|_| {
                    format!("Invalid frame count for --export-blend-delay-frames: {frame_count}")
                })?;
            }
            "--help" | "-h" => {
                return Err(
                    "Usage: cargo run -- [--capture <output.png>] [--capture-delay-frames <n>] [--export-blend <output.blend>] [--export-blend-delay-frames <n>]\nF12 saves a screenshot during normal interactive runs. F4 exports a Blender .blend during normal interactive runs."
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
        blend_export_path,
        blend_export_delay_frames,
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

    let path = PathBuf::from(&app_config.capture.output_dir).join(manual_screenshot_filename(
        current_unix_timestamp(),
        screenshot_counter.next_index,
    ));
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

fn manual_screenshot_filename(timestamp: Duration, sequence: u32) -> String {
    format!(
        "intergen-{}-{:03}-{:04}.png",
        timestamp.as_secs(),
        timestamp.subsec_millis(),
        sequence
    )
}

fn current_unix_timestamp() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::Path;
    use std::time::Duration;

    use super::{manual_screenshot_filename, parse_launch_config};
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
        assert_eq!(config.blend_export_path, None);
        assert_eq!(
            config.blend_export_delay_frames,
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
        assert_eq!(config.blend_export_path, None);
    }

    #[test]
    fn launch_config_parses_blend_export_path_and_delay() {
        let config = parse_launch_config(
            [
                OsString::from("--export-blend"),
                OsString::from("blend-exports/test.blend"),
                OsString::from("--export-blend-delay-frames"),
                OsString::from("96"),
            ],
            CaptureConfig::default().default_capture_delay_frames,
        )
        .expect("blend export args should parse");

        assert_eq!(
            config.blend_export_path.as_deref(),
            Some(Path::new("blend-exports/test.blend"))
        );
        assert_eq!(config.blend_export_delay_frames, 96);
        assert_eq!(config.capture_path, None);
    }

    #[test]
    fn manual_screenshot_filename_includes_seconds_and_sequence() {
        let filename = manual_screenshot_filename(Duration::new(1_741_018_296, 45_000_000), 7);

        assert_eq!(filename, "intergen-1741018296-045-0007.png");
    }
}
