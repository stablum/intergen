use std::fs;
use std::path::Path;

use bevy::prelude::*;
use bevy::window::PresentMode;
use serde::{Deserialize, Serialize};

use crate::polyhedra::{PolyhedronKind, SpawnTuning};

#[path = "config_effects.rs"]
mod effects;
#[cfg(test)]
pub(crate) use effects::{
    BloomConfig, ColorWavefolderConfig, EdgeDetectionConfig, GaussianBlurConfig,
    LensDistortionConfig,
};
pub(crate) use effects::{EffectGroup, EffectNumericParameter, EffectsConfig};

const DEFAULT_CONFIG_PATH: &str = "config.toml";

#[derive(Clone, Default, Resource, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct AppConfig {
    pub(crate) window: WindowConfig,
    pub(crate) rendering: RenderingConfig,
    pub(crate) camera: CameraConfig,
    pub(crate) generation: GenerationConfig,
    pub(crate) lighting: LightingConfig,
    pub(crate) effects: EffectsConfig,
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

fn parse_config(contents: &str) -> Result<AppConfig, toml::de::Error> {
    toml::from_str(contents)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PresentModeSetting {
    #[default]
    AutoVsync,
    AutoNoVsync,
    Immediate,
    Mailbox,
    Fifo,
    FifoRelaxed,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct CameraConfig {
    pub(crate) initial_yaw: f32,
    pub(crate) initial_pitch: f32,
    pub(crate) initial_roll: f32,
    pub(crate) initial_distance: f32,
    pub(crate) rotation_accel: f32,
    pub(crate) preserve_angular_momentum: bool,
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
            preserve_angular_momentum: true,
            zoom_accel: 24.0,
            angular_damping: 2.2,
            zoom_damping: 4.0,
            min_distance: 4.0,
            max_distance: 48.0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
    pub(crate) twist_hold_delay_secs: f32,
    pub(crate) twist_repeat_interval_secs: f32,
    pub(crate) min_twist_per_vertex_radians: f32,
    pub(crate) max_twist_per_vertex_radians: f32,
    pub(crate) default_vertex_offset_ratio: f32,
    pub(crate) vertex_offset_adjust_step: f32,
    pub(crate) vertex_offset_hold_delay_secs: f32,
    pub(crate) vertex_offset_repeat_interval_secs: f32,
    pub(crate) min_vertex_offset_ratio: f32,
    pub(crate) max_vertex_offset_ratio: f32,
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
        let min = self.min_twist_per_vertex_radians.max(0.0);
        let max = self.max_twist_per_vertex_radians.max(min);
        (min, max)
    }

    pub(crate) fn default_twist_per_vertex_radians_clamped(&self) -> f32 {
        let (min, max) = self.twist_bounds();
        self.twist_per_vertex_radians.clamp(min, max)
    }

    pub(crate) fn vertex_offset_bounds(&self) -> (f32, f32) {
        let min = self.min_vertex_offset_ratio.max(0.0);
        let max = self.max_vertex_offset_ratio.max(min);
        (min, max)
    }

    pub(crate) fn default_vertex_offset_ratio_clamped(&self) -> f32 {
        let (min, max) = self.vertex_offset_bounds();
        self.default_vertex_offset_ratio.clamp(min, max)
    }

    pub(crate) fn spawn_tuning(
        &self,
        twist_per_vertex_radians: f32,
        vertex_offset_ratio: f32,
    ) -> SpawnTuning {
        let (min_scale_ratio, max_scale_ratio) = self.scale_bounds();
        let (min_twist_per_vertex_radians, max_twist_per_vertex_radians) = self.twist_bounds();
        let (min_vertex_offset_ratio, max_vertex_offset_ratio) = self.vertex_offset_bounds();
        SpawnTuning {
            min_scale_ratio,
            max_scale_ratio,
            containment_epsilon: self.containment_epsilon,
            twist_per_vertex_radians: twist_per_vertex_radians
                .clamp(min_twist_per_vertex_radians, max_twist_per_vertex_radians),
            vertex_offset_ratio: vertex_offset_ratio
                .clamp(min_vertex_offset_ratio, max_vertex_offset_ratio),
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
            twist_hold_delay_secs: 0.24,
            twist_repeat_interval_secs: 0.07,
            min_twist_per_vertex_radians: 0.0,
            max_twist_per_vertex_radians: std::f32::consts::PI,
            default_vertex_offset_ratio: 0.0,
            vertex_offset_adjust_step: 0.1,
            vertex_offset_hold_delay_secs: 0.24,
            vertex_offset_repeat_interval_secs: 0.07,
            min_vertex_offset_ratio: 0.0,
            max_vertex_offset_ratio: 6.0,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct LightingConfig {
    pub(crate) directional: DirectionalLightConfig,
    pub(crate) point: PointLightConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
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
                "fonts/carbonplus-regular-bl.otf".to_string(),
                "fonts/CarbonPlus-Regular.ttf".to_string(),
                "fonts/CarbonPlus-Regular.otf".to_string(),
                "fonts/Carbon Plus Regular.ttf".to_string(),
                "fonts/Carbon Plus Regular.otf".to_string(),
                "fonts/CarbonPlus.ttf".to_string(),
                "fonts/Carbon Plus.ttf".to_string(),
                "fonts/carbonplus-bold-bl.otf".to_string(),
                "fonts/carbonplus-light-bl.otf".to_string(),
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
    fn camera_preserves_angular_momentum_by_default() {
        let config = parse_config("").expect("empty config should parse");

        assert!(config.camera.preserve_angular_momentum);
    }

    #[test]
    fn color_wavefolder_settings_parse_from_config() {
        let config = parse_config(
            r#"
            [effects.color_wavefolder]
            enabled = false
            gain = 3.25
            modulus = 0.5
            "#,
        )
        .expect("effects config should parse");

        assert!(!config.effects.color_wavefolder.enabled);
        assert_eq!(config.effects.color_wavefolder.gain_clamped(), 3.25);
        assert_eq!(config.effects.color_wavefolder.modulus_clamped(), 0.5);
    }

    #[test]
    fn lens_distortion_settings_parse_from_config() {
        let config = parse_config(
            r#"
            [effects.lens_distortion]
            enabled = true
            strength = 2.25
            radial_k2 = -0.75
            radial_k3 = 5.0
            zoom = 0.05
            center = [1.2, -0.4]
            scale = [0.05, 5.0]
            tangential = [3.0, -3.0]
            chromatic_aberration = 2.0
            "#,
        )
        .expect("lens distortion config should parse");

        assert!(config.effects.lens_distortion.enabled);
        assert_eq!(config.effects.lens_distortion.strength_clamped(), 2.25);
        assert_eq!(config.effects.lens_distortion.radial_k2_clamped(), -0.75);
        assert_eq!(config.effects.lens_distortion.radial_k3_clamped(), 4.0);
        assert_eq!(config.effects.lens_distortion.zoom_clamped(), 0.1);
        assert_eq!(config.effects.lens_distortion.center_clamped(), [1.0, 0.0]);
        assert_eq!(config.effects.lens_distortion.scale_clamped(), [0.1, 4.0]);
        assert_eq!(
            config.effects.lens_distortion.tangential_clamped(),
            [2.0, -2.0]
        );
        assert_eq!(
            config
                .effects
                .lens_distortion
                .chromatic_aberration_clamped(),
            0.5
        );
    }

    #[test]
    fn gaussian_blur_settings_parse_from_config() {
        let config = parse_config(
            r#"
            [effects.gaussian_blur]
            enabled = true
            sigma = 2.75
            radius_pixels = 7
            "#,
        )
        .expect("gaussian blur config should parse");

        assert!(config.effects.gaussian_blur.enabled);
        assert_eq!(config.effects.gaussian_blur.sigma_clamped(), 2.75);
        assert_eq!(config.effects.gaussian_blur.radius_pixels_clamped(), 7);
    }

    #[test]
    fn bloom_settings_parse_from_config() {
        let config = parse_config(
            r#"
            [effects.bloom]
            enabled = true
            threshold = -0.25
            intensity = 1.4
            radius_pixels = 99
            "#,
        )
        .expect("bloom config should parse");

        assert!(config.effects.bloom.enabled);
        assert_eq!(config.effects.bloom.threshold_clamped(), 0.0);
        assert_eq!(config.effects.bloom.intensity_clamped(), 1.4);
        assert_eq!(config.effects.bloom.radius_pixels_clamped(), 16);
    }

    #[test]
    fn edge_detection_settings_parse_from_config() {
        let config = parse_config(
            r#"
            [effects.edge_detection]
            enabled = true
            strength = 4.25
            threshold = 0.35
            mix = 2.0
            color = [0.9, 0.2, 1.4]
            "#,
        )
        .expect("edge detection config should parse");

        assert!(config.effects.edge_detection.enabled);
        assert_eq!(config.effects.edge_detection.strength_clamped(), 4.25);
        assert_eq!(config.effects.edge_detection.threshold_clamped(), 0.35);
        assert_eq!(config.effects.edge_detection.mix_clamped(), 1.0);
        assert_eq!(config.effects.edge_detection.color, [0.9, 0.2, 1.4]);
    }

    #[test]
    fn twist_default_is_clamped_to_bounds() {
        let config = parse_config(
            r#"
            [generation]
            twist_per_vertex_radians = 10.0
            min_twist_per_vertex_radians = 0.0
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

    #[test]
    fn twist_hold_timings_parse_from_config() {
        let config = parse_config(
            r#"
            [generation]
            twist_hold_delay_secs = 0.12
            twist_repeat_interval_secs = 0.03
            "#,
        )
        .expect("twist config should parse");

        assert_eq!(config.generation.twist_hold_delay_secs, 0.12);
        assert_eq!(config.generation.twist_repeat_interval_secs, 0.03);
    }

    #[test]
    fn vertex_offset_default_is_clamped_to_bounds() {
        let config = parse_config(
            r#"
            [generation]
            default_vertex_offset_ratio = 10.0
            min_vertex_offset_ratio = 0.0
            max_vertex_offset_ratio = 0.75
            "#,
        )
        .expect("vertex offset config should parse");

        assert_eq!(
            config.generation.default_vertex_offset_ratio_clamped(),
            0.75
        );
    }

    #[test]
    fn vertex_offset_hold_timings_parse_from_config() {
        let config = parse_config(
            r#"
            [generation]
            vertex_offset_hold_delay_secs = 0.11
            vertex_offset_repeat_interval_secs = 0.02
            "#,
        )
        .expect("vertex offset config should parse");

        assert_eq!(config.generation.vertex_offset_hold_delay_secs, 0.11);
        assert_eq!(config.generation.vertex_offset_repeat_interval_secs, 0.02);
    }

    #[test]
    fn twist_bounds_never_allow_negative_floor() {
        let config = parse_config(
            r#"
            [generation]
            min_twist_per_vertex_radians = -1.0
            max_twist_per_vertex_radians = 0.75
            "#,
        )
        .expect("twist config should parse");

        assert_eq!(config.generation.twist_bounds(), (0.0, 0.75));
    }

    #[test]
    fn vertex_offset_bounds_never_allow_negative_floor() {
        let config = parse_config(
            r#"
            [generation]
            min_vertex_offset_ratio = -1.0
            max_vertex_offset_ratio = 0.75
            "#,
        )
        .expect("vertex offset config should parse");

        assert_eq!(config.generation.vertex_offset_bounds(), (0.0, 0.75));
    }
}
