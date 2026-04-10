use std::fs;
use std::path::Path;

use bevy::prelude::*;
use bevy::window::PresentMode;
use serde::{Deserialize, Serialize};

use crate::parameters::{GenerationParameter, ScalarParameterSpec};
use crate::shapes::{ShapeKind, SpawnPlacementMode, SpawnTuning};

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
    pub(crate) stage: StageConfig,
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
            stage: StageConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct StageConfig {
    pub(crate) enabled: bool,
    #[serde(default = "StageSurfaceConfig::floor_default")]
    pub(crate) floor: StageSurfaceConfig,
    #[serde(default = "StageSurfaceConfig::backdrop_default")]
    pub(crate) backdrop: StageSurfaceConfig,
}

impl Default for StageConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            floor: StageSurfaceConfig::floor_default(),
            backdrop: StageSurfaceConfig::backdrop_default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct StageSurfaceConfig {
    pub(crate) enabled: bool,
    pub(crate) color: [f32; 3],
    pub(crate) translation: [f32; 3],
    pub(crate) rotation_degrees: [f32; 3],
    pub(crate) size: [f32; 2],
    pub(crate) thickness: f32,
    pub(crate) metallic: f32,
    pub(crate) perceptual_roughness: f32,
    pub(crate) reflectance: f32,
}

impl Default for StageSurfaceConfig {
    fn default() -> Self {
        Self::floor_default()
    }
}

impl StageSurfaceConfig {
    fn floor_default() -> Self {
        Self {
            enabled: false,
            color: [0.09, 0.1, 0.12],
            translation: [0.0, -4.4, 0.0],
            rotation_degrees: [0.0, 0.0, 0.0],
            size: [42.0, 42.0],
            thickness: 0.2,
            metallic: 0.08,
            perceptual_roughness: 0.64,
            reflectance: 0.26,
        }
    }

    fn backdrop_default() -> Self {
        Self {
            enabled: false,
            color: [0.05, 0.06, 0.09],
            translation: [0.0, 4.0, -18.0],
            rotation_degrees: [0.0, 0.0, 0.0],
            size: [38.0, 20.0],
            thickness: 0.2,
            metallic: 0.02,
            perceptual_roughness: 0.72,
            reflectance: 0.2,
        }
    }

    pub(crate) fn color(&self) -> Color {
        srgb(self.color)
    }

    pub(crate) fn translation(&self) -> Vec3 {
        vec3(self.translation)
    }

    pub(crate) fn rotation(&self) -> Quat {
        let [x, y, z] = self.rotation_degrees;
        Quat::from_euler(
            EulerRot::XYZ,
            x.to_radians(),
            y.to_radians(),
            z.to_radians(),
        )
    }

    pub(crate) fn size(&self) -> Vec2 {
        Vec2::new(self.size[0].max(0.01), self.size[1].max(0.01))
    }

    pub(crate) fn thickness(&self) -> f32 {
        self.thickness.max(0.01)
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
    #[serde(alias = "root_kind")]
    pub(crate) root_shape_kind: ShapeKind,
    pub(crate) root_scale: f32,
    #[serde(alias = "default_child_kind")]
    pub(crate) default_child_shape_kind: ShapeKind,
    pub(crate) default_spawn_placement_mode: SpawnPlacementMode,
    pub(crate) default_scale_ratio: f32,
    pub(crate) scale_adjust_step: f32,
    pub(crate) min_scale_ratio: f32,
    pub(crate) max_scale_ratio: f32,
    pub(crate) default_child_axis_scale: [f32; 3],
    pub(crate) child_axis_scale_adjust_step: f32,
    pub(crate) min_child_axis_scale: f32,
    pub(crate) max_child_axis_scale: f32,
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
    pub(crate) default_child_position_offset: [f32; 3],
    pub(crate) child_position_offset_adjust_step: f32,
    pub(crate) default_vertex_spawn_exclusion_probability: f32,
    pub(crate) vertex_spawn_exclusion_adjust_step: f32,
    pub(crate) vertex_spawn_exclusion_hold_delay_secs: f32,
    pub(crate) vertex_spawn_exclusion_repeat_interval_secs: f32,
    pub(crate) min_vertex_spawn_exclusion_probability: f32,
    pub(crate) max_vertex_spawn_exclusion_probability: f32,
}

#[cfg_attr(not(test), allow(dead_code))]
impl GenerationConfig {
    pub(crate) fn parameter_spec(&self, parameter: GenerationParameter) -> ScalarParameterSpec {
        match parameter {
            GenerationParameter::ChildScaleRatio => ScalarParameterSpec::new(
                self.default_scale_ratio,
                self.min_scale_ratio,
                self.max_scale_ratio,
                self.scale_adjust_step,
                0.0,
                0.0,
            ),
            GenerationParameter::ChildAxisScaleX => {
                self.child_axis_scale_spec(self.default_child_axis_scale[0])
            }
            GenerationParameter::ChildAxisScaleY => {
                self.child_axis_scale_spec(self.default_child_axis_scale[1])
            }
            GenerationParameter::ChildAxisScaleZ => {
                self.child_axis_scale_spec(self.default_child_axis_scale[2])
            }
            GenerationParameter::ChildTwistPerVertexRadians => {
                ScalarParameterSpec::new_nonnegative(
                    self.twist_per_vertex_radians,
                    self.min_twist_per_vertex_radians,
                    self.max_twist_per_vertex_radians,
                    self.twist_adjust_step,
                    self.twist_hold_delay_secs,
                    self.twist_repeat_interval_secs,
                )
            }
            GenerationParameter::ChildOutwardOffsetRatio => ScalarParameterSpec::new_nonnegative(
                self.default_vertex_offset_ratio,
                self.min_vertex_offset_ratio,
                self.max_vertex_offset_ratio,
                self.vertex_offset_adjust_step,
                self.vertex_offset_hold_delay_secs,
                self.vertex_offset_repeat_interval_secs,
            ),
            GenerationParameter::ChildPositionOffsetX => {
                self.child_position_offset_spec(self.default_child_position_offset[0])
            }
            GenerationParameter::ChildPositionOffsetY => {
                self.child_position_offset_spec(self.default_child_position_offset[1])
            }
            GenerationParameter::ChildPositionOffsetZ => {
                self.child_position_offset_spec(self.default_child_position_offset[2])
            }
            GenerationParameter::ChildSpawnExclusionProbability => {
                ScalarParameterSpec::new_probability(
                    self.default_vertex_spawn_exclusion_probability,
                    self.min_vertex_spawn_exclusion_probability,
                    self.max_vertex_spawn_exclusion_probability,
                    self.vertex_spawn_exclusion_adjust_step,
                    self.vertex_spawn_exclusion_hold_delay_secs,
                    self.vertex_spawn_exclusion_repeat_interval_secs,
                )
            }
        }
    }

    fn child_axis_scale_spec(&self, default_value: f32) -> ScalarParameterSpec {
        let min_value = self.min_child_axis_scale.max(0.01);
        let max_value = self.max_child_axis_scale.max(min_value);
        ScalarParameterSpec::new(
            default_value,
            min_value,
            max_value,
            self.child_axis_scale_adjust_step,
            0.0,
            0.0,
        )
    }

    fn child_position_offset_spec(&self, default_value: f32) -> ScalarParameterSpec {
        ScalarParameterSpec::new(
            default_value,
            -1.0,
            1.0,
            self.child_position_offset_adjust_step,
            0.0,
            0.0,
        )
    }

    pub(crate) fn twist_bounds(&self) -> (f32, f32) {
        self.parameter_spec(GenerationParameter::ChildTwistPerVertexRadians)
            .bounds()
    }

    pub(crate) fn child_axis_scale_bounds(&self) -> (f32, f32) {
        self.parameter_spec(GenerationParameter::ChildAxisScaleX)
            .bounds()
    }

    pub(crate) fn default_child_axis_scale_clamped(&self) -> Vec3 {
        Vec3::new(
            self.parameter_spec(GenerationParameter::ChildAxisScaleX)
                .default_value(),
            self.parameter_spec(GenerationParameter::ChildAxisScaleY)
                .default_value(),
            self.parameter_spec(GenerationParameter::ChildAxisScaleZ)
                .default_value(),
        )
    }

    pub(crate) fn default_twist_per_vertex_radians_clamped(&self) -> f32 {
        self.parameter_spec(GenerationParameter::ChildTwistPerVertexRadians)
            .default_value()
    }

    pub(crate) fn vertex_offset_bounds(&self) -> (f32, f32) {
        self.parameter_spec(GenerationParameter::ChildOutwardOffsetRatio)
            .bounds()
    }

    pub(crate) fn default_vertex_offset_ratio_clamped(&self) -> f32 {
        self.parameter_spec(GenerationParameter::ChildOutwardOffsetRatio)
            .default_value()
    }

    pub(crate) fn default_child_position_offset_clamped(&self) -> Vec3 {
        Vec3::new(
            self.parameter_spec(GenerationParameter::ChildPositionOffsetX)
                .default_value(),
            self.parameter_spec(GenerationParameter::ChildPositionOffsetY)
                .default_value(),
            self.parameter_spec(GenerationParameter::ChildPositionOffsetZ)
                .default_value(),
        )
    }

    pub(crate) fn vertex_spawn_exclusion_bounds(&self) -> (f32, f32) {
        self.parameter_spec(GenerationParameter::ChildSpawnExclusionProbability)
            .bounds()
    }

    pub(crate) fn default_vertex_spawn_exclusion_probability_clamped(&self) -> f32 {
        self.parameter_spec(GenerationParameter::ChildSpawnExclusionProbability)
            .default_value()
    }

    pub(crate) fn spawn_tuning(
        &self,
        child_axis_scale: Vec3,
        twist_per_vertex_radians: f32,
        vertex_offset_ratio: f32,
        child_position_offset: Vec3,
        vertex_spawn_exclusion_probability: f32,
        spawn_placement_mode: SpawnPlacementMode,
    ) -> SpawnTuning {
        let scale_spec = self.parameter_spec(GenerationParameter::ChildScaleRatio);
        let axis_scale_x_spec = self.parameter_spec(GenerationParameter::ChildAxisScaleX);
        let axis_scale_y_spec = self.parameter_spec(GenerationParameter::ChildAxisScaleY);
        let axis_scale_z_spec = self.parameter_spec(GenerationParameter::ChildAxisScaleZ);
        let twist_spec = self.parameter_spec(GenerationParameter::ChildTwistPerVertexRadians);
        let offset_spec = self.parameter_spec(GenerationParameter::ChildOutwardOffsetRatio);
        let position_offset_x_spec = self.parameter_spec(GenerationParameter::ChildPositionOffsetX);
        let position_offset_y_spec = self.parameter_spec(GenerationParameter::ChildPositionOffsetY);
        let position_offset_z_spec = self.parameter_spec(GenerationParameter::ChildPositionOffsetZ);
        let exclusion_spec =
            self.parameter_spec(GenerationParameter::ChildSpawnExclusionProbability);
        let (min_scale_ratio, max_scale_ratio) = scale_spec.bounds();
        SpawnTuning {
            min_scale_ratio,
            max_scale_ratio,
            child_axis_scale: Vec3::new(
                axis_scale_x_spec.clamp(child_axis_scale.x),
                axis_scale_y_spec.clamp(child_axis_scale.y),
                axis_scale_z_spec.clamp(child_axis_scale.z),
            ),
            containment_epsilon: self.containment_epsilon,
            twist_per_vertex_radians: twist_spec.clamp(twist_per_vertex_radians),
            vertex_offset_ratio: offset_spec.clamp(vertex_offset_ratio),
            child_position_offset: Vec3::new(
                position_offset_x_spec.clamp(child_position_offset.x),
                position_offset_y_spec.clamp(child_position_offset.y),
                position_offset_z_spec.clamp(child_position_offset.z),
            ),
            vertex_spawn_exclusion_probability: exclusion_spec
                .clamp(vertex_spawn_exclusion_probability),
            spawn_placement_mode,
        }
    }
}
impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            root_shape_kind: ShapeKind::Cube,
            root_scale: 1.9,
            default_child_shape_kind: ShapeKind::Dodecahedron,
            default_spawn_placement_mode: SpawnPlacementMode::Vertex,
            default_scale_ratio: 0.58,
            scale_adjust_step: 0.05,
            min_scale_ratio: 0.15,
            max_scale_ratio: 1.0,
            default_child_axis_scale: [1.0, 1.0, 1.0],
            child_axis_scale_adjust_step: 0.05,
            min_child_axis_scale: 0.01,
            max_child_axis_scale: 100.0,
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
            default_child_position_offset: [0.0, 0.0, 0.0],
            child_position_offset_adjust_step: 0.05,
            default_vertex_spawn_exclusion_probability: 0.0,
            vertex_spawn_exclusion_adjust_step: 0.05,
            vertex_spawn_exclusion_hold_delay_secs: 0.24,
            vertex_spawn_exclusion_repeat_interval_secs: 0.07,
            min_vertex_spawn_exclusion_probability: 0.0,
            max_vertex_spawn_exclusion_probability: 1.0,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct LightingConfig {
    pub(crate) directional: DirectionalLightConfig,
    pub(crate) point: PointLightConfig,
    pub(crate) accent: AccentLightConfig,
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
pub(crate) struct AccentLightConfig {
    pub(crate) enabled: bool,
    pub(crate) color: [f32; 3],
    pub(crate) intensity: f32,
    pub(crate) range: f32,
    pub(crate) shadows_enabled: bool,
    pub(crate) translation: [f32; 3],
}

impl AccentLightConfig {
    pub(crate) fn color(&self) -> Color {
        srgb(self.color)
    }

    pub(crate) fn translation(&self) -> Vec3 {
        vec3(self.translation)
    }
}

impl Default for AccentLightConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            color: [0.9, 0.72, 0.96],
            intensity: 450_000.0,
            range: 48.0,
            shadows_enabled: false,
            translation: [11.0, 2.5, 13.0],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MaterialSurfaceMode {
    #[default]
    Legacy,
    Procedural,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MaterialSurfaceFamily {
    #[default]
    Legacy,
    Matte,
    Satin,
    Glossy,
    Metal,
    Frosted,
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
    pub(crate) surface_mode: MaterialSurfaceMode,
    pub(crate) base_surface: MaterialSurfaceFamily,
    pub(crate) root_surface: MaterialSurfaceFamily,
    pub(crate) accent_surface: MaterialSurfaceFamily,
    pub(crate) accent_every_n_levels: usize,
    pub(crate) level_lightness_shift: f32,
    pub(crate) level_saturation_shift: f32,
    pub(crate) level_metallic_shift: f32,
    pub(crate) level_roughness_shift: f32,
    pub(crate) level_reflectance_shift: f32,
}

impl MaterialConfig {
    pub(crate) fn hue_bias(&self, kind: ShapeKind) -> f32 {
        match kind {
            ShapeKind::Cube => self.cube_hue_bias,
            ShapeKind::Tetrahedron => self.tetrahedron_hue_bias,
            ShapeKind::Octahedron => self.octahedron_hue_bias,
            ShapeKind::Dodecahedron => self.dodecahedron_hue_bias,
        }
    }

    pub(crate) fn opacity_bounds(&self) -> (f32, f32) {
        ordered_pair(self.min_opacity, self.max_opacity)
    }

    pub(crate) fn default_opacity_clamped(&self) -> f32 {
        let (min, max) = self.opacity_bounds();
        self.default_opacity.clamp(min, max)
    }

    pub(crate) fn surface_family(&self, level: usize) -> MaterialSurfaceFamily {
        match self.surface_mode {
            MaterialSurfaceMode::Legacy => MaterialSurfaceFamily::Legacy,
            MaterialSurfaceMode::Procedural => {
                if level == 0 && self.root_surface != MaterialSurfaceFamily::Legacy {
                    return self.root_surface;
                }
                if self.accent_every_n_levels > 0
                    && level > 0
                    && level % self.accent_every_n_levels == 0
                    && self.accent_surface != MaterialSurfaceFamily::Legacy
                {
                    return self.accent_surface;
                }
                self.base_surface
            }
        }
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
            surface_mode: MaterialSurfaceMode::Legacy,
            base_surface: MaterialSurfaceFamily::Satin,
            root_surface: MaterialSurfaceFamily::Legacy,
            accent_surface: MaterialSurfaceFamily::Legacy,
            accent_every_n_levels: 3,
            level_lightness_shift: 0.0,
            level_saturation_shift: 0.0,
            level_metallic_shift: 0.0,
            level_roughness_shift: 0.0,
            level_reflectance_shift: 0.0,
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
    pub(crate) focus_background: [f32; 4],
    pub(crate) focus_text: [f32; 3],
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
            focus_background: [0.92, 0.78, 0.36, 0.98],
            focus_text: [0.05, 0.07, 0.10],
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
    use bevy::prelude::Vec3;

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
            config.generation.default_child_shape_kind,
            super::ShapeKind::Dodecahedron
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
    fn vertex_spawn_exclusion_default_is_clamped_to_bounds() {
        let config = parse_config(
            r#"
            [generation]
            default_vertex_spawn_exclusion_probability = 2.0
            min_vertex_spawn_exclusion_probability = 0.0
            max_vertex_spawn_exclusion_probability = 0.4
            "#,
        )
        .expect("vertex exclusion config should parse");

        assert_eq!(
            config
                .generation
                .default_vertex_spawn_exclusion_probability_clamped(),
            0.4
        );
    }

    #[test]
    fn vertex_spawn_exclusion_bounds_stay_within_probability_range() {
        let config = parse_config(
            r#"
            [generation]
            min_vertex_spawn_exclusion_probability = -1.0
            max_vertex_spawn_exclusion_probability = 2.0
            "#,
        )
        .expect("vertex exclusion config should parse");

        assert_eq!(
            config.generation.vertex_spawn_exclusion_bounds(),
            (0.0, 1.0)
        );
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

    #[test]
    fn child_axis_scale_defaults_are_clamped_to_positive_bounds() {
        let config = parse_config(
            r#"
            [generation]
            default_child_axis_scale = [-1.0, 4.0, 0.5]
            min_child_axis_scale = -2.0
            max_child_axis_scale = 2.0
            "#,
        )
        .expect("child axis scale config should parse");

        assert_eq!(config.generation.child_axis_scale_bounds(), (0.01, 2.0));
        assert_eq!(
            config.generation.default_child_axis_scale_clamped(),
            Vec3::new(0.01, 2.0, 0.5)
        );
    }

    #[test]
    fn child_position_offset_defaults_are_clamped_to_signed_unit_bounds() {
        let config = parse_config(
            r#"
            [generation]
            default_child_position_offset = [-2.0, 0.25, 3.0]
            "#,
        )
        .expect("child position offset config should parse");

        assert_eq!(
            config.generation.default_child_position_offset_clamped(),
            Vec3::new(-1.0, 0.25, 1.0)
        );
    }

    #[test]
    fn ui_focus_colors_default_when_not_overridden() {
        let config = parse_config(
            r#"
            [ui]
            body_text = [0.1, 0.2, 0.3]
            "#,
        )
        .expect("ui config should parse");

        assert_eq!(config.ui.body_text, [0.1, 0.2, 0.3]);
        assert_eq!(config.ui.focus_background, [0.92, 0.78, 0.36, 0.98]);
        assert_eq!(config.ui.focus_text, [0.05, 0.07, 0.10]);
    }

    #[test]
    fn legacy_rendering_and_lighting_defaults_keep_new_features_disabled() {
        let config = parse_config(
            r#"
            [rendering]
            clear_color = [0.0, 0.0, 0.0]
            ambient_light_color = [1.0, 0.0, 0.0]
            ambient_light_brightness = 12.0

            [lighting.directional]
            color = [1.0, 0.0, 0.0]
            illuminance = 22000.0
            shadows_enabled = true
            translation = [12.0, 18.0, 9.0]
            look_at = [0.0, 0.0, 0.0]

            [lighting.point]
            color = [0.5, 0.6, 0.85]
            intensity = 1200000.0
            range = 60.0
            shadows_enabled = false
            translation = [-9.0, 5.0, -12.0]
            "#,
        )
        .expect("legacy rendering and lighting config should parse");

        assert!(!config.rendering.stage.enabled);
        assert!(!config.lighting.accent.enabled);
    }

    #[test]
    fn legacy_material_defaults_keep_surface_mode_on_legacy() {
        let config = parse_config(
            r#"
            [materials]
            hue_step_per_level = 45.0
            saturation = 0.68
            lightness = 0.56
            metallic = 0.05
            perceptual_roughness = 0.86
            reflectance = 0.24
            default_opacity = 1.0
            opacity_adjust_step = 0.1
            min_opacity = 0.1
            max_opacity = 1.0
            cube_hue_bias = 35.0
            tetrahedron_hue_bias = 110.0
            octahedron_hue_bias = 205.0
            dodecahedron_hue_bias = 290.0
            "#,
        )
        .expect("legacy materials config should parse");

        assert_eq!(
            config.materials.surface_mode,
            super::MaterialSurfaceMode::Legacy
        );
        assert_eq!(
            config.materials.surface_family(0),
            super::MaterialSurfaceFamily::Legacy
        );
    }

    #[test]
    fn deprecated_shape_specific_surface_fields_do_not_break_material_parsing() {
        let config = parse_config(
            r#"
            [materials]
            surface_mode = "procedural"
            root_surface = "glossy"
            accent_surface = "metal"
            accent_every_n_levels = 3
            cube_surface = "satin"
            tetrahedron_surface = "matte"
            octahedron_surface = "metal"
            dodecahedron_surface = "glossy"
            "#,
        )
        .expect("deprecated shape surface fields should be ignored");

        assert_eq!(
            config.materials.base_surface,
            super::MaterialSurfaceFamily::Satin
        );
        assert_eq!(
            config.materials.surface_family(0),
            super::MaterialSurfaceFamily::Glossy
        );
        assert_eq!(
            config.materials.surface_family(1),
            super::MaterialSurfaceFamily::Satin
        );
        assert_eq!(
            config.materials.surface_family(3),
            super::MaterialSurfaceFamily::Metal
        );
    }
}
