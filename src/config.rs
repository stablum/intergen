use std::fs;
use std::path::Path;

use bevy::prelude::*;
use bevy::window::PresentMode;
use serde::Deserialize;

use crate::polyhedra::{PolyhedronKind, SpawnTuning};

const DEFAULT_CONFIG_PATH: &str = "config.toml";

#[derive(Clone, Resource, Deserialize)]
#[serde(default)]
pub(crate) struct AppConfig {
    pub(crate) window: WindowConfig,
    pub(crate) rendering: RenderingConfig,
    pub(crate) camera: CameraConfig,
    pub(crate) generation: GenerationConfig,
    pub(crate) lighting: LightingConfig,
    pub(crate) materials: MaterialConfig,
    pub(crate) capture: CaptureConfig,
    pub(crate) ui: UiConfig,
}

impl AppConfig {
    pub(crate) fn load_from_default_path() -> Result<Self, String> {
        Self::load_from_path(Path::new(DEFAULT_CONFIG_PATH))
    }

    pub(crate) fn load_from_path(path: &Path) -> Result<Self, String> {
        if !path.is_file() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(path)
            .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
        parse_config(&contents)
            .map_err(|error| format!("Could not parse {}: {error}", path.display()))
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig::default(),
            rendering: RenderingConfig::default(),
            camera: CameraConfig::default(),
            generation: GenerationConfig::default(),
            lighting: LightingConfig::default(),
            materials: MaterialConfig::default(),
            capture: CaptureConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

fn parse_config(contents: &str) -> Result<AppConfig, toml::de::Error> {
    toml::from_str(contents)
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub(crate) struct WindowConfig {
    pub(crate) title: String,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) present_mode: PresentModeSetting,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "intergen".to_string(),
            width: 1440,
            height: 960,
            present_mode: PresentModeSetting::AutoVsync,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PresentModeSetting {
    AutoVsync,
    AutoNoVsync,
    Immediate,
    Mailbox,
    Fifo,
    FifoRelaxed,
}

impl Default for PresentModeSetting {
    fn default() -> Self {
        Self::AutoVsync
    }
}

impl From<PresentModeSetting> for PresentMode {
    fn from(value: PresentModeSetting) -> Self {
        match value {
            PresentModeSetting::AutoVsync => PresentMode::AutoVsync,
            PresentModeSetting::AutoNoVsync => PresentMode::AutoNoVsync,
            PresentModeSetting::Immediate => PresentMode::Immediate,
            PresentModeSetting::Mailbox => PresentMode::Mailbox,
            PresentModeSetting::Fifo => PresentMode::Fifo,
            PresentModeSetting::FifoRelaxed => PresentMode::FifoRelaxed,
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub(crate) struct RenderingConfig {
    pub(crate) clear_color: [f32; 3],
    pub(crate) ambient_light_color: [f32; 3],
    pub(crate) ambient_light_brightness: f32,
}

impl RenderingConfig {
    pub(crate) fn clear_color(&self) -> Color {
        srgb(self.clear_color)
    }

    pub(crate) fn ambient_light_color(&self) -> Color {
        srgb(self.ambient_light_color)
    }
}

impl Default for RenderingConfig {
    fn default() -> Self {
        Self {
            clear_color: [0.035, 0.04, 0.06],
            ambient_light_color: [0.7, 0.74, 0.82],
            ambient_light_brightness: 12.0,
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub(crate) struct CameraConfig {
    pub(crate) initial_yaw: f32,
    pub(crate) initial_pitch: f32,
    pub(crate) initial_roll: f32,
    pub(crate) initial_distance: f32,
    pub(crate) rotation_accel: f32,
    pub(crate) zoom_accel: f32,
    pub(crate) angular_damping: f32,
    pub(crate) zoom_damping: f32,
    pub(crate) min_distance: f32,
    pub(crate) max_distance: f32,
}

impl CameraConfig {
    pub(crate) fn initial_orientation(&self) -> Quat {
        Quat::from_euler(
            EulerRot::YXZ,
            self.initial_yaw,
            self.initial_pitch,
            self.initial_roll,
        )
    }

    pub(crate) fn distance_bounds(&self) -> (f32, f32) {
        ordered_pair(self.min_distance, self.max_distance)
    }
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            initial_yaw: -std::f32::consts::FRAC_PI_4,
            initial_pitch: -0.45,
            initial_roll: 0.15,
            initial_distance: 14.0,
            rotation_accel: 1.9,
            zoom_accel: 24.0,
            angular_damping: 2.2,
            zoom_damping: 4.0,
            min_distance: 4.0,
            max_distance: 48.0,
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub(crate) struct GenerationConfig {
    pub(crate) root_kind: PolyhedronKind,
    pub(crate) root_scale: f32,
    pub(crate) default_child_kind: PolyhedronKind,
    pub(crate) default_scale_ratio: f32,
    pub(crate) scale_adjust_step: f32,
    pub(crate) min_scale_ratio: f32,
    pub(crate) max_scale_ratio: f32,
    pub(crate) spawn_hold_delay_secs: f32,
    pub(crate) spawn_repeat_interval_secs: f32,
    pub(crate) containment_epsilon: f32,
    pub(crate) twist_per_vertex_radians: f32,
    pub(crate) twist_adjust_step: f32,
    pub(crate) min_twist_per_vertex_radians: f32,
    pub(crate) max_twist_per_vertex_radians: f32,
}

impl GenerationConfig {
    pub(crate) fn scale_bounds(&self) -> (f32, f32) {
        ordered_pair(self.min_scale_ratio, self.max_scale_ratio)
    }

    pub(crate) fn default_scale_ratio_clamped(&self) -> f32 {
        let (min, max) = self.scale_bounds();
        self.default_scale_ratio.clamp(min, max)
    }

    pub(crate) fn twist_bounds(&self) -> (f32, f32) {
        ordered_pair(
            self.min_twist_per_vertex_radians,
            self.max_twist_per_vertex_radians,
        )
    }

    pub(crate) fn default_twist_per_vertex_radians_clamped(&self) -> f32 {
        let (min, max) = self.twist_bounds();
        self.twist_per_vertex_radians.clamp(min, max)
    }

    pub(crate) fn spawn_tuning(&self, twist_per_vertex_radians: f32) -> SpawnTuning {
        let (min_scale_ratio, max_scale_ratio) = self.scale_bounds();
        let (min_twist_per_vertex_radians, max_twist_per_vertex_radians) = self.twist_bounds();
        SpawnTuning {
            min_scale_ratio,
            max_scale_ratio,
            containment_epsilon: self.containment_epsilon,
            twist_per_vertex_radians: twist_per_vertex_radians
                .clamp(min_twist_per_vertex_radians, max_twist_per_vertex_radians),
        }
    }
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            root_kind: PolyhedronKind::Cube,
            root_scale: 1.9,
            default_child_kind: PolyhedronKind::Dodecahedron,
            default_scale_ratio: 0.58,
            scale_adjust_step: 0.05,
            min_scale_ratio: 0.15,
            max_scale_ratio: 1.0,
            spawn_hold_delay_secs: 0.24,
            spawn_repeat_interval_secs: 0.07,
            containment_epsilon: 0.02,
            twist_per_vertex_radians: std::f32::consts::PI / 5.0,
            twist_adjust_step: std::f32::consts::PI / 18.0,
            min_twist_per_vertex_radians: -std::f32::consts::PI,
            max_twist_per_vertex_radians: std::f32::consts::PI,
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub(crate) struct LightingConfig {
    pub(crate) directional: DirectionalLightConfig,
    pub(crate) point: PointLightConfig,
}

impl Default for LightingConfig {
    fn default() -> Self {
        Self {
            directional: DirectionalLightConfig::default(),
            point: PointLightConfig::default(),
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub(crate) struct DirectionalLightConfig {
    pub(crate) color: [f32; 3],
    pub(crate) illuminance: f32,
    pub(crate) shadows_enabled: bool,
    pub(crate) translation: [f32; 3],
    pub(crate) look_at: [f32; 3],
}

impl DirectionalLightConfig {
    pub(crate) fn color(&self) -> Color {
        srgb(self.color)
    }

    pub(crate) fn translation(&self) -> Vec3 {
        vec3(self.translation)
    }

    pub(crate) fn look_at(&self) -> Vec3 {
        vec3(self.look_at)
    }
}

impl Default for DirectionalLightConfig {
    fn default() -> Self {
        Self {
            color: [1.0, 0.97, 0.93],
            illuminance: 22_000.0,
            shadows_enabled: true,
            translation: [12.0, 18.0, 9.0],
            look_at: [0.0, 0.0, 0.0],
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub(crate) struct PointLightConfig {
    pub(crate) color: [f32; 3],
    pub(crate) intensity: f32,
    pub(crate) range: f32,
    pub(crate) shadows_enabled: bool,
    pub(crate) translation: [f32; 3],
}

impl PointLightConfig {
    pub(crate) fn color(&self) -> Color {
        srgb(self.color)
    }

    pub(crate) fn translation(&self) -> Vec3 {
        vec3(self.translation)
    }
}

impl Default for PointLightConfig {
    fn default() -> Self {
        Self {
            color: [0.5, 0.6, 0.85],
            intensity: 1_200_000.0,
            range: 60.0,
            shadows_enabled: false,
            translation: [-9.0, 5.0, -12.0],
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub(crate) struct MaterialConfig {
    pub(crate) hue_step_per_level: f32,
    pub(crate) saturation: f32,
    pub(crate) lightness: f32,
    pub(crate) metallic: f32,
    pub(crate) perceptual_roughness: f32,
    pub(crate) reflectance: f32,
    pub(crate) default_opacity: f32,
    pub(crate) opacity_adjust_step: f32,
    pub(crate) min_opacity: f32,
    pub(crate) max_opacity: f32,
    pub(crate) cube_hue_bias: f32,
    pub(crate) tetrahedron_hue_bias: f32,
    pub(crate) octahedron_hue_bias: f32,
    pub(crate) dodecahedron_hue_bias: f32,
}

impl MaterialConfig {
    pub(crate) fn hue_bias(&self, kind: PolyhedronKind) -> f32 {
        match kind {
            PolyhedronKind::Cube => self.cube_hue_bias,
            PolyhedronKind::Tetrahedron => self.tetrahedron_hue_bias,
            PolyhedronKind::Octahedron => self.octahedron_hue_bias,
            PolyhedronKind::Dodecahedron => self.dodecahedron_hue_bias,
        }
    }

    pub(crate) fn opacity_bounds(&self) -> (f32, f32) {
        ordered_pair(self.min_opacity, self.max_opacity)
    }

    pub(crate) fn default_opacity_clamped(&self) -> f32 {
        let (min, max) = self.opacity_bounds();
        self.default_opacity.clamp(min, max)
    }
}

impl Default for MaterialConfig {
    fn default() -> Self {
        Self {
            hue_step_per_level: 45.0,
            saturation: 0.68,
            lightness: 0.56,
            metallic: 0.05,
            perceptual_roughness: 0.86,
            reflectance: 0.24,
            default_opacity: 1.0,
            opacity_adjust_step: 0.1,
            min_opacity: 0.1,
            max_opacity: 1.0,
            cube_hue_bias: 35.0,
            tetrahedron_hue_bias: 110.0,
            octahedron_hue_bias: 205.0,
            dodecahedron_hue_bias: 290.0,
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub(crate) struct CaptureConfig {
    pub(crate) output_dir: String,
    pub(crate) default_capture_delay_frames: u32,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            output_dir: "screenshots".to_string(),
            default_capture_delay_frames: 8,
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub(crate) struct UiConfig {
    pub(crate) font_candidates: Vec<String>,
    pub(crate) hint_top: f32,
    pub(crate) hint_left: f32,
    pub(crate) hint_padding_x: f32,
    pub(crate) hint_padding_y: f32,
    pub(crate) hint_background: [f32; 4],
    pub(crate) hint_text: [f32; 3],
    pub(crate) hint_font_size: f32,
    pub(crate) overlay_background: [f32; 4],
    pub(crate) overlay_padding: f32,
    pub(crate) panel_max_width: f32,
    pub(crate) panel_row_gap: f32,
    pub(crate) panel_padding: f32,
    pub(crate) panel_background: [f32; 4],
    pub(crate) panel_radius: f32,
    pub(crate) title_font_size: f32,
    pub(crate) title_text: [f32; 3],
    pub(crate) body_font_size: f32,
    pub(crate) body_text: [f32; 3],
    pub(crate) body_max_width: f32,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            font_candidates: vec![
                "fonts/CarbonPlus-Regular.ttf".to_string(),
                "fonts/CarbonPlus-Regular.otf".to_string(),
                "fonts/Carbon Plus Regular.ttf".to_string(),
                "fonts/Carbon Plus Regular.otf".to_string(),
                "fonts/CarbonPlus.ttf".to_string(),
                "fonts/Carbon Plus.ttf".to_string(),
            ],
            hint_top: 18.0,
            hint_left: 18.0,
            hint_padding_x: 12.0,
            hint_padding_y: 8.0,
            hint_background: [0.06, 0.08, 0.13, 0.86],
            hint_text: [0.93, 0.95, 0.99],
            hint_font_size: 14.0,
            overlay_background: [0.01, 0.02, 0.04, 0.72],
            overlay_padding: 24.0,
            panel_max_width: 460.0,
            panel_row_gap: 12.0,
            panel_padding: 20.0,
            panel_background: [0.07, 0.1, 0.16, 0.95],
            panel_radius: 20.0,
            title_font_size: 28.0,
            title_text: [0.98, 0.99, 1.0],
            body_font_size: 16.0,
            body_text: [0.89, 0.92, 0.96],
            body_max_width: 420.0,
        }
    }
}

pub(crate) fn srgb(color: [f32; 3]) -> Color {
    Color::srgb(color[0], color[1], color[2])
}

pub(crate) fn srgba(color: [f32; 4]) -> Color {
    Color::srgba(color[0], color[1], color[2], color[3])
}

pub(crate) fn vec3(vector: [f32; 3]) -> Vec3 {
    Vec3::new(vector[0], vector[1], vector[2])
}

fn ordered_pair(left: f32, right: f32) -> (f32, f32) {
    if left <= right {
        (left, right)
    } else {
        (right, left)
    }
}

#[cfg(test)]
mod tests {
    use super::{AppConfig, PresentModeSetting, parse_config};

    #[test]
    fn partial_config_uses_defaults_for_unspecified_sections() {
        let config = parse_config(
            r#"
            [camera]
            initial_distance = 21.0
            "#,
        )
        .expect("partial config should parse");

        assert_eq!(config.camera.initial_distance, 21.0);
        assert_eq!(config.window.width, 1440);
        assert_eq!(
            config.generation.default_child_kind,
            super::PolyhedronKind::Dodecahedron
        );
    }

    #[test]
    fn default_config_uses_auto_vsync() {
        let config = AppConfig::default();

        assert_eq!(config.window.present_mode, PresentModeSetting::AutoVsync);
    }

    #[test]
    fn twist_default_is_clamped_to_bounds() {
        let config = parse_config(
            r#"
            [generation]
            twist_per_vertex_radians = 10.0
            min_twist_per_vertex_radians = -0.5
            max_twist_per_vertex_radians = 0.75
            "#,
        )
        .expect("twist config should parse");

        assert_eq!(
            config.generation.default_twist_per_vertex_radians_clamped(),
            0.75
        );
    }

    #[test]
    fn material_opacity_default_is_clamped_to_bounds() {
        let config = parse_config(
            r#"
            [materials]
            default_opacity = 2.0
            min_opacity = 0.2
            max_opacity = 0.8
            "#,
        )
        .expect("material config should parse");

        assert_eq!(config.materials.default_opacity_clamped(), 0.8);
    }
}
