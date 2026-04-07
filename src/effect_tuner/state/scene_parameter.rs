#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EffectTunerSceneParameter {
    ChildKind,
    SpawnPlacementMode,
    SpawnAddMode,
    ChildScaleRatio,
    ChildAxisScaleX,
    ChildAxisScaleY,
    ChildAxisScaleZ,
    ChildTwistPerVertexRadians,
    ChildOutwardOffsetRatio,
    ChildSpawnExclusionProbability,
    StageEnabled,
    StageFloorEnabled,
    StageBackdropEnabled,
    GlobalOpacity,
    MaterialHueStepPerLevel,
    MaterialSaturation,
    MaterialLightness,
    MaterialMetallic,
    MaterialPerceptualRoughness,
    MaterialReflectance,
    MaterialCubeHueBias,
    MaterialTetrahedronHueBias,
    MaterialOctahedronHueBias,
    MaterialDodecahedronHueBias,
    MaterialSurfaceMode,
    MaterialBaseSurface,
    MaterialRootSurface,
    MaterialAccentSurface,
    MaterialAccentEveryNLevels,
    MaterialLevelLightnessShift,
    MaterialLevelSaturationShift,
    MaterialLevelMetallicShift,
    MaterialLevelRoughnessShift,
    MaterialLevelReflectanceShift,
    CameraDistance,
    CameraAngularVelocityX,
    CameraAngularVelocityY,
    CameraAngularVelocityZ,
    CameraZoomVelocity,
    RenderingClearColorR,
    RenderingClearColorG,
    RenderingClearColorB,
    RenderingAmbientColorR,
    RenderingAmbientColorG,
    RenderingAmbientColorB,
    RenderingAmbientBrightness,
    StageFloorColorR,
    StageFloorColorG,
    StageFloorColorB,
    StageFloorTranslationX,
    StageFloorTranslationY,
    StageFloorTranslationZ,
    StageFloorRotationX,
    StageFloorRotationY,
    StageFloorRotationZ,
    StageFloorSizeX,
    StageFloorSizeY,
    StageFloorThickness,
    StageFloorMetallic,
    StageFloorPerceptualRoughness,
    StageFloorReflectance,
    StageBackdropColorR,
    StageBackdropColorG,
    StageBackdropColorB,
    StageBackdropTranslationX,
    StageBackdropTranslationY,
    StageBackdropTranslationZ,
    StageBackdropRotationX,
    StageBackdropRotationY,
    StageBackdropRotationZ,
    StageBackdropSizeX,
    StageBackdropSizeY,
    StageBackdropThickness,
    StageBackdropMetallic,
    StageBackdropPerceptualRoughness,
    StageBackdropReflectance,
    LightingDirectionalColorR,
    LightingDirectionalColorG,
    LightingDirectionalColorB,
    LightingDirectionalIlluminance,
    LightingDirectionalTranslationX,
    LightingDirectionalTranslationY,
    LightingDirectionalTranslationZ,
    LightingDirectionalLookAtX,
    LightingDirectionalLookAtY,
    LightingDirectionalLookAtZ,
    LightingPointColorR,
    LightingPointColorG,
    LightingPointColorB,
    LightingPointIntensity,
    LightingPointRange,
    LightingPointTranslationX,
    LightingPointTranslationY,
    LightingPointTranslationZ,
    LightingAccentColorR,
    LightingAccentColorG,
    LightingAccentColorB,
    LightingAccentIntensity,
    LightingAccentRange,
    LightingAccentTranslationX,
    LightingAccentTranslationY,
    LightingAccentTranslationZ,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SceneChangeTarget {
    None,
    Generation,
    Materials,
    Stage,
    Rendering,
    Lighting,
    Camera,
}

impl EffectTunerSceneParameter {
    const ALL: [Self; 102] = [
        Self::ChildKind,
        Self::SpawnPlacementMode,
        Self::SpawnAddMode,
        Self::ChildScaleRatio,
        Self::ChildAxisScaleX,
        Self::ChildAxisScaleY,
        Self::ChildAxisScaleZ,
        Self::ChildTwistPerVertexRadians,
        Self::ChildOutwardOffsetRatio,
        Self::ChildSpawnExclusionProbability,
        Self::StageEnabled,
        Self::StageFloorEnabled,
        Self::StageBackdropEnabled,
        Self::GlobalOpacity,
        Self::MaterialHueStepPerLevel,
        Self::MaterialSaturation,
        Self::MaterialLightness,
        Self::MaterialMetallic,
        Self::MaterialPerceptualRoughness,
        Self::MaterialReflectance,
        Self::MaterialCubeHueBias,
        Self::MaterialTetrahedronHueBias,
        Self::MaterialOctahedronHueBias,
        Self::MaterialDodecahedronHueBias,
        Self::MaterialSurfaceMode,
        Self::MaterialBaseSurface,
        Self::MaterialRootSurface,
        Self::MaterialAccentSurface,
        Self::MaterialAccentEveryNLevels,
        Self::MaterialLevelLightnessShift,
        Self::MaterialLevelSaturationShift,
        Self::MaterialLevelMetallicShift,
        Self::MaterialLevelRoughnessShift,
        Self::MaterialLevelReflectanceShift,
        Self::CameraDistance,
        Self::CameraAngularVelocityX,
        Self::CameraAngularVelocityY,
        Self::CameraAngularVelocityZ,
        Self::CameraZoomVelocity,
        Self::RenderingClearColorR,
        Self::RenderingClearColorG,
        Self::RenderingClearColorB,
        Self::RenderingAmbientColorR,
        Self::RenderingAmbientColorG,
        Self::RenderingAmbientColorB,
        Self::RenderingAmbientBrightness,
        Self::StageFloorColorR,
        Self::StageFloorColorG,
        Self::StageFloorColorB,
        Self::StageFloorTranslationX,
        Self::StageFloorTranslationY,
        Self::StageFloorTranslationZ,
        Self::StageFloorRotationX,
        Self::StageFloorRotationY,
        Self::StageFloorRotationZ,
        Self::StageFloorSizeX,
        Self::StageFloorSizeY,
        Self::StageFloorThickness,
        Self::StageFloorMetallic,
        Self::StageFloorPerceptualRoughness,
        Self::StageFloorReflectance,
        Self::StageBackdropColorR,
        Self::StageBackdropColorG,
        Self::StageBackdropColorB,
        Self::StageBackdropTranslationX,
        Self::StageBackdropTranslationY,
        Self::StageBackdropTranslationZ,
        Self::StageBackdropRotationX,
        Self::StageBackdropRotationY,
        Self::StageBackdropRotationZ,
        Self::StageBackdropSizeX,
        Self::StageBackdropSizeY,
        Self::StageBackdropThickness,
        Self::StageBackdropMetallic,
        Self::StageBackdropPerceptualRoughness,
        Self::StageBackdropReflectance,
        Self::LightingDirectionalColorR,
        Self::LightingDirectionalColorG,
        Self::LightingDirectionalColorB,
        Self::LightingDirectionalIlluminance,
        Self::LightingDirectionalTranslationX,
        Self::LightingDirectionalTranslationY,
        Self::LightingDirectionalTranslationZ,
        Self::LightingDirectionalLookAtX,
        Self::LightingDirectionalLookAtY,
        Self::LightingDirectionalLookAtZ,
        Self::LightingPointColorR,
        Self::LightingPointColorG,
        Self::LightingPointColorB,
        Self::LightingPointIntensity,
        Self::LightingPointRange,
        Self::LightingPointTranslationX,
        Self::LightingPointTranslationY,
        Self::LightingPointTranslationZ,
        Self::LightingAccentColorR,
        Self::LightingAccentColorG,
        Self::LightingAccentColorB,
        Self::LightingAccentIntensity,
        Self::LightingAccentRange,
        Self::LightingAccentTranslationX,
        Self::LightingAccentTranslationY,
        Self::LightingAccentTranslationZ,
    ];

    const LFO_CAPABLE: [Self; 92] = [
        Self::ChildTwistPerVertexRadians,
        Self::ChildOutwardOffsetRatio,
        Self::GlobalOpacity,
        Self::MaterialHueStepPerLevel,
        Self::MaterialSaturation,
        Self::MaterialLightness,
        Self::MaterialMetallic,
        Self::MaterialPerceptualRoughness,
        Self::MaterialReflectance,
        Self::MaterialCubeHueBias,
        Self::MaterialTetrahedronHueBias,
        Self::MaterialOctahedronHueBias,
        Self::MaterialDodecahedronHueBias,
        Self::MaterialAccentEveryNLevels,
        Self::MaterialLevelLightnessShift,
        Self::MaterialLevelSaturationShift,
        Self::MaterialLevelMetallicShift,
        Self::MaterialLevelRoughnessShift,
        Self::MaterialLevelReflectanceShift,
        Self::CameraDistance,
        Self::CameraAngularVelocityX,
        Self::CameraAngularVelocityY,
        Self::CameraAngularVelocityZ,
        Self::CameraZoomVelocity,
        Self::RenderingClearColorR,
        Self::RenderingClearColorG,
        Self::RenderingClearColorB,
        Self::RenderingAmbientColorR,
        Self::RenderingAmbientColorG,
        Self::RenderingAmbientColorB,
        Self::RenderingAmbientBrightness,
        Self::StageFloorColorR,
        Self::StageFloorColorG,
        Self::StageFloorColorB,
        Self::StageFloorTranslationX,
        Self::StageFloorTranslationY,
        Self::StageFloorTranslationZ,
        Self::StageFloorRotationX,
        Self::StageFloorRotationY,
        Self::StageFloorRotationZ,
        Self::StageFloorSizeX,
        Self::StageFloorSizeY,
        Self::StageFloorThickness,
        Self::StageFloorMetallic,
        Self::StageFloorPerceptualRoughness,
        Self::StageFloorReflectance,
        Self::StageBackdropColorR,
        Self::StageBackdropColorG,
        Self::StageBackdropColorB,
        Self::StageBackdropTranslationX,
        Self::StageBackdropTranslationY,
        Self::StageBackdropTranslationZ,
        Self::StageBackdropRotationX,
        Self::StageBackdropRotationY,
        Self::StageBackdropRotationZ,
        Self::StageBackdropSizeX,
        Self::StageBackdropSizeY,
        Self::StageBackdropThickness,
        Self::StageBackdropMetallic,
        Self::StageBackdropPerceptualRoughness,
        Self::StageBackdropReflectance,
        Self::LightingDirectionalColorR,
        Self::LightingDirectionalColorG,
        Self::LightingDirectionalColorB,
        Self::LightingDirectionalIlluminance,
        Self::LightingDirectionalTranslationX,
        Self::LightingDirectionalTranslationY,
        Self::LightingDirectionalTranslationZ,
        Self::LightingDirectionalLookAtX,
        Self::LightingDirectionalLookAtY,
        Self::LightingDirectionalLookAtZ,
        Self::LightingPointColorR,
        Self::LightingPointColorG,
        Self::LightingPointColorB,
        Self::LightingPointIntensity,
        Self::LightingPointRange,
        Self::LightingPointTranslationX,
        Self::LightingPointTranslationY,
        Self::LightingPointTranslationZ,
        Self::LightingAccentColorR,
        Self::LightingAccentColorG,
        Self::LightingAccentColorB,
        Self::LightingAccentIntensity,
        Self::LightingAccentRange,
        Self::LightingAccentTranslationX,
        Self::LightingAccentTranslationY,
        Self::LightingAccentTranslationZ,
        Self::ChildScaleRatio,
        Self::ChildAxisScaleX,
        Self::ChildAxisScaleY,
        Self::ChildAxisScaleZ,
        Self::ChildSpawnExclusionProbability,
    ];

    const GENERATION_LFO_CAPABLE: [Self; 7] = [
        Self::ChildScaleRatio,
        Self::ChildAxisScaleX,
        Self::ChildAxisScaleY,
        Self::ChildAxisScaleZ,
        Self::ChildTwistPerVertexRadians,
        Self::ChildOutwardOffsetRatio,
        Self::ChildSpawnExclusionProbability,
    ];

    const MATERIAL_LFO_CAPABLE: [Self; 17] = [
        Self::GlobalOpacity,
        Self::MaterialHueStepPerLevel,
        Self::MaterialSaturation,
        Self::MaterialLightness,
        Self::MaterialMetallic,
        Self::MaterialPerceptualRoughness,
        Self::MaterialReflectance,
        Self::MaterialCubeHueBias,
        Self::MaterialTetrahedronHueBias,
        Self::MaterialOctahedronHueBias,
        Self::MaterialDodecahedronHueBias,
        Self::MaterialAccentEveryNLevels,
        Self::MaterialLevelLightnessShift,
        Self::MaterialLevelSaturationShift,
        Self::MaterialLevelMetallicShift,
        Self::MaterialLevelRoughnessShift,
        Self::MaterialLevelReflectanceShift,
    ];

    pub(crate) fn all() -> &'static [Self] {
        &Self::ALL
    }

    fn lfo_capable() -> &'static [Self] {
        &Self::LFO_CAPABLE
    }

    fn generation_lfo_capable() -> &'static [Self] {
        &Self::GENERATION_LFO_CAPABLE
    }

    fn material_lfo_capable() -> &'static [Self] {
        &Self::MATERIAL_LFO_CAPABLE
    }

    fn supports_lfo(self) -> bool {
        Self::lfo_capable().contains(&self)
    }

    fn stable_id(self) -> &'static str {
        match self {
            Self::ChildKind => "generation.child_kind",
            Self::SpawnPlacementMode => "generation.spawn_placement_mode",
            Self::SpawnAddMode => "generation.spawn_add_mode",
            Self::ChildScaleRatio => "generation.child_scale_ratio",
            Self::ChildAxisScaleX => "generation.child_axis_scale.x",
            Self::ChildAxisScaleY => "generation.child_axis_scale.y",
            Self::ChildAxisScaleZ => "generation.child_axis_scale.z",
            Self::ChildTwistPerVertexRadians => "generation.child_twist_per_vertex_radians",
            Self::ChildOutwardOffsetRatio => "generation.child_outward_offset_ratio",
            Self::ChildSpawnExclusionProbability => "generation.child_spawn_exclusion_probability",
            Self::StageEnabled => "stage.enabled",
            Self::StageFloorEnabled => "stage.floor.enabled",
            Self::StageBackdropEnabled => "stage.backdrop.enabled",
            Self::GlobalOpacity => "materials.opacity",
            Self::MaterialHueStepPerLevel => "materials.hue_step_per_level",
            Self::MaterialSaturation => "materials.saturation",
            Self::MaterialLightness => "materials.lightness",
            Self::MaterialMetallic => "materials.metallic",
            Self::MaterialPerceptualRoughness => "materials.perceptual_roughness",
            Self::MaterialReflectance => "materials.reflectance",
            Self::MaterialCubeHueBias => "materials.cube_hue_bias",
            Self::MaterialTetrahedronHueBias => "materials.tetrahedron_hue_bias",
            Self::MaterialOctahedronHueBias => "materials.octahedron_hue_bias",
            Self::MaterialDodecahedronHueBias => "materials.dodecahedron_hue_bias",
            Self::MaterialSurfaceMode => "materials.surface_mode",
            Self::MaterialBaseSurface => "materials.base_surface",
            Self::MaterialRootSurface => "materials.root_surface",
            Self::MaterialAccentSurface => "materials.accent_surface",
            Self::MaterialAccentEveryNLevels => "materials.accent_every_n_levels",
            Self::MaterialLevelLightnessShift => "materials.level_lightness_shift",
            Self::MaterialLevelSaturationShift => "materials.level_saturation_shift",
            Self::MaterialLevelMetallicShift => "materials.level_metallic_shift",
            Self::MaterialLevelRoughnessShift => "materials.level_roughness_shift",
            Self::MaterialLevelReflectanceShift => "materials.level_reflectance_shift",
            Self::CameraDistance => "camera.distance",
            Self::CameraAngularVelocityX => "camera.angular_velocity.x",
            Self::CameraAngularVelocityY => "camera.angular_velocity.y",
            Self::CameraAngularVelocityZ => "camera.angular_velocity.z",
            Self::CameraZoomVelocity => "camera.zoom_velocity",
            Self::RenderingClearColorR => "rendering.clear_color.r",
            Self::RenderingClearColorG => "rendering.clear_color.g",
            Self::RenderingClearColorB => "rendering.clear_color.b",
            Self::RenderingAmbientColorR => "rendering.ambient_light_color.r",
            Self::RenderingAmbientColorG => "rendering.ambient_light_color.g",
            Self::RenderingAmbientColorB => "rendering.ambient_light_color.b",
            Self::RenderingAmbientBrightness => "rendering.ambient_light_brightness",
            Self::StageFloorColorR => "stage.floor.color.r",
            Self::StageFloorColorG => "stage.floor.color.g",
            Self::StageFloorColorB => "stage.floor.color.b",
            Self::StageFloorTranslationX => "stage.floor.translation.x",
            Self::StageFloorTranslationY => "stage.floor.translation.y",
            Self::StageFloorTranslationZ => "stage.floor.translation.z",
            Self::StageFloorRotationX => "stage.floor.rotation_degrees.x",
            Self::StageFloorRotationY => "stage.floor.rotation_degrees.y",
            Self::StageFloorRotationZ => "stage.floor.rotation_degrees.z",
            Self::StageFloorSizeX => "stage.floor.size.x",
            Self::StageFloorSizeY => "stage.floor.size.y",
            Self::StageFloorThickness => "stage.floor.thickness",
            Self::StageFloorMetallic => "stage.floor.metallic",
            Self::StageFloorPerceptualRoughness => "stage.floor.perceptual_roughness",
            Self::StageFloorReflectance => "stage.floor.reflectance",
            Self::StageBackdropColorR => "stage.backdrop.color.r",
            Self::StageBackdropColorG => "stage.backdrop.color.g",
            Self::StageBackdropColorB => "stage.backdrop.color.b",
            Self::StageBackdropTranslationX => "stage.backdrop.translation.x",
            Self::StageBackdropTranslationY => "stage.backdrop.translation.y",
            Self::StageBackdropTranslationZ => "stage.backdrop.translation.z",
            Self::StageBackdropRotationX => "stage.backdrop.rotation_degrees.x",
            Self::StageBackdropRotationY => "stage.backdrop.rotation_degrees.y",
            Self::StageBackdropRotationZ => "stage.backdrop.rotation_degrees.z",
            Self::StageBackdropSizeX => "stage.backdrop.size.x",
            Self::StageBackdropSizeY => "stage.backdrop.size.y",
            Self::StageBackdropThickness => "stage.backdrop.thickness",
            Self::StageBackdropMetallic => "stage.backdrop.metallic",
            Self::StageBackdropPerceptualRoughness => "stage.backdrop.perceptual_roughness",
            Self::StageBackdropReflectance => "stage.backdrop.reflectance",
            Self::LightingDirectionalColorR => "lighting.directional.color.r",
            Self::LightingDirectionalColorG => "lighting.directional.color.g",
            Self::LightingDirectionalColorB => "lighting.directional.color.b",
            Self::LightingDirectionalIlluminance => "lighting.directional.illuminance",
            Self::LightingDirectionalTranslationX => "lighting.directional.translation.x",
            Self::LightingDirectionalTranslationY => "lighting.directional.translation.y",
            Self::LightingDirectionalTranslationZ => "lighting.directional.translation.z",
            Self::LightingDirectionalLookAtX => "lighting.directional.look_at.x",
            Self::LightingDirectionalLookAtY => "lighting.directional.look_at.y",
            Self::LightingDirectionalLookAtZ => "lighting.directional.look_at.z",
            Self::LightingPointColorR => "lighting.point.color.r",
            Self::LightingPointColorG => "lighting.point.color.g",
            Self::LightingPointColorB => "lighting.point.color.b",
            Self::LightingPointIntensity => "lighting.point.intensity",
            Self::LightingPointRange => "lighting.point.range",
            Self::LightingPointTranslationX => "lighting.point.translation.x",
            Self::LightingPointTranslationY => "lighting.point.translation.y",
            Self::LightingPointTranslationZ => "lighting.point.translation.z",
            Self::LightingAccentColorR => "lighting.accent.color.r",
            Self::LightingAccentColorG => "lighting.accent.color.g",
            Self::LightingAccentColorB => "lighting.accent.color.b",
            Self::LightingAccentIntensity => "lighting.accent.intensity",
            Self::LightingAccentRange => "lighting.accent.range",
            Self::LightingAccentTranslationX => "lighting.accent.translation.x",
            Self::LightingAccentTranslationY => "lighting.accent.translation.y",
            Self::LightingAccentTranslationZ => "lighting.accent.translation.z",
        }
    }

    fn from_stable_id(stable_id: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|parameter| parameter.stable_id() == stable_id)
    }

    fn label(self) -> &'static str {
        self.stable_id()
    }

    fn short_label(self) -> &'static str {
        match self {
            Self::ChildKind => "shape",
            Self::SpawnPlacementMode => "placement",
            Self::SpawnAddMode => "add mode",
            Self::ChildScaleRatio => "scale",
            Self::ChildAxisScaleX => "axis x",
            Self::ChildAxisScaleY => "axis y",
            Self::ChildAxisScaleZ => "axis z",
            Self::ChildTwistPerVertexRadians => "twist",
            Self::ChildOutwardOffsetRatio => "offset",
            Self::ChildSpawnExclusionProbability => "spawn%",
            Self::StageEnabled => "enabled",
            Self::StageFloorEnabled => "floor",
            Self::StageBackdropEnabled => "backdrop",
            Self::GlobalOpacity => "opacity",
            Self::MaterialHueStepPerLevel => "hue step",
            Self::MaterialSaturation => "sat",
            Self::MaterialLightness => "light",
            Self::MaterialMetallic => "metallic",
            Self::MaterialPerceptualRoughness => "roughness",
            Self::MaterialReflectance => "reflect",
            Self::MaterialCubeHueBias => "cube hue",
            Self::MaterialTetrahedronHueBias => "tetra hue",
            Self::MaterialOctahedronHueBias => "octa hue",
            Self::MaterialDodecahedronHueBias => "dodec hue",
            Self::MaterialSurfaceMode => "surface",
            Self::MaterialBaseSurface => "base surf",
            Self::MaterialRootSurface => "root surf",
            Self::MaterialAccentSurface => "accent srf",
            Self::MaterialAccentEveryNLevels => "accent nth",
            Self::MaterialLevelLightnessShift => "lvl light",
            Self::MaterialLevelSaturationShift => "lvl sat",
            Self::MaterialLevelMetallicShift => "lvl metal",
            Self::MaterialLevelRoughnessShift => "lvl rough",
            Self::MaterialLevelReflectanceShift => "lvl refl",
            Self::CameraDistance => "distance",
            Self::CameraAngularVelocityX => "ang x",
            Self::CameraAngularVelocityY => "ang y",
            Self::CameraAngularVelocityZ => "ang z",
            Self::CameraZoomVelocity => "zoom vel",
            Self::RenderingClearColorR => "clear r",
            Self::RenderingClearColorG => "clear g",
            Self::RenderingClearColorB => "clear b",
            Self::RenderingAmbientColorR => "amb r",
            Self::RenderingAmbientColorG => "amb g",
            Self::RenderingAmbientColorB => "amb b",
            Self::RenderingAmbientBrightness => "amb brt",
            Self::StageFloorColorR
            | Self::StageBackdropColorR
            | Self::LightingDirectionalColorR
            | Self::LightingPointColorR
            | Self::LightingAccentColorR => "color r",
            Self::StageFloorColorG
            | Self::StageBackdropColorG
            | Self::LightingDirectionalColorG
            | Self::LightingPointColorG
            | Self::LightingAccentColorG => "color g",
            Self::StageFloorColorB
            | Self::StageBackdropColorB
            | Self::LightingDirectionalColorB
            | Self::LightingPointColorB
            | Self::LightingAccentColorB => "color b",
            Self::StageFloorTranslationX
            | Self::StageBackdropTranslationX
            | Self::LightingDirectionalTranslationX
            | Self::LightingPointTranslationX
            | Self::LightingAccentTranslationX => "pos x",
            Self::StageFloorTranslationY
            | Self::StageBackdropTranslationY
            | Self::LightingDirectionalTranslationY
            | Self::LightingPointTranslationY
            | Self::LightingAccentTranslationY => "pos y",
            Self::StageFloorTranslationZ
            | Self::StageBackdropTranslationZ
            | Self::LightingDirectionalTranslationZ
            | Self::LightingPointTranslationZ
            | Self::LightingAccentTranslationZ => "pos z",
            Self::StageFloorRotationX | Self::StageBackdropRotationX => "rot x",
            Self::StageFloorRotationY | Self::StageBackdropRotationY => "rot y",
            Self::StageFloorRotationZ | Self::StageBackdropRotationZ => "rot z",
            Self::StageFloorSizeX | Self::StageBackdropSizeX => "size x",
            Self::StageFloorSizeY | Self::StageBackdropSizeY => "size y",
            Self::StageFloorThickness | Self::StageBackdropThickness => "thick",
            Self::StageFloorMetallic | Self::StageBackdropMetallic => "metal",
            Self::StageFloorPerceptualRoughness | Self::StageBackdropPerceptualRoughness => {
                "rough"
            }
            Self::StageFloorReflectance | Self::StageBackdropReflectance => "refl",
            Self::LightingDirectionalIlluminance => "illum",
            Self::LightingDirectionalLookAtX => "look x",
            Self::LightingDirectionalLookAtY => "look y",
            Self::LightingDirectionalLookAtZ => "look z",
            Self::LightingPointIntensity | Self::LightingAccentIntensity => "intensity",
            Self::LightingPointRange | Self::LightingAccentRange => "range",
        }
    }

    pub(crate) fn group_label(self) -> &'static str {
        match self {
            Self::StageEnabled | Self::StageFloorEnabled | Self::StageBackdropEnabled => "stage",
            Self::GlobalOpacity
            | Self::MaterialHueStepPerLevel
            | Self::MaterialSaturation
            | Self::MaterialLightness
            | Self::MaterialMetallic
            | Self::MaterialPerceptualRoughness
            | Self::MaterialReflectance
            | Self::MaterialCubeHueBias
            | Self::MaterialTetrahedronHueBias
            | Self::MaterialOctahedronHueBias
            | Self::MaterialDodecahedronHueBias
            | Self::MaterialSurfaceMode
            | Self::MaterialBaseSurface
            | Self::MaterialRootSurface
            | Self::MaterialAccentSurface
            | Self::MaterialAccentEveryNLevels
            | Self::MaterialLevelLightnessShift
            | Self::MaterialLevelSaturationShift
            | Self::MaterialLevelMetallicShift
            | Self::MaterialLevelRoughnessShift
            | Self::MaterialLevelReflectanceShift => "mat",
            Self::ChildKind
            | Self::SpawnPlacementMode
            | Self::SpawnAddMode
            | Self::ChildScaleRatio
            | Self::ChildAxisScaleX
            | Self::ChildAxisScaleY
            | Self::ChildAxisScaleZ
            | Self::ChildTwistPerVertexRadians
            | Self::ChildOutwardOffsetRatio
            | Self::ChildSpawnExclusionProbability => "scene",
            Self::CameraDistance
            | Self::CameraAngularVelocityX
            | Self::CameraAngularVelocityY
            | Self::CameraAngularVelocityZ
            | Self::CameraZoomVelocity => "camera",
            Self::RenderingClearColorR
            | Self::RenderingClearColorG
            | Self::RenderingClearColorB
            | Self::RenderingAmbientColorR
            | Self::RenderingAmbientColorG
            | Self::RenderingAmbientColorB
            | Self::RenderingAmbientBrightness => "render",
            Self::StageFloorColorR
            | Self::StageFloorColorG
            | Self::StageFloorColorB
            | Self::StageFloorTranslationX
            | Self::StageFloorTranslationY
            | Self::StageFloorTranslationZ
            | Self::StageFloorRotationX
            | Self::StageFloorRotationY
            | Self::StageFloorRotationZ
            | Self::StageFloorSizeX
            | Self::StageFloorSizeY
            | Self::StageFloorThickness
            | Self::StageFloorMetallic
            | Self::StageFloorPerceptualRoughness
            | Self::StageFloorReflectance => "floor",
            Self::StageBackdropColorR
            | Self::StageBackdropColorG
            | Self::StageBackdropColorB
            | Self::StageBackdropTranslationX
            | Self::StageBackdropTranslationY
            | Self::StageBackdropTranslationZ
            | Self::StageBackdropRotationX
            | Self::StageBackdropRotationY
            | Self::StageBackdropRotationZ
            | Self::StageBackdropSizeX
            | Self::StageBackdropSizeY
            | Self::StageBackdropThickness
            | Self::StageBackdropMetallic
            | Self::StageBackdropPerceptualRoughness
            | Self::StageBackdropReflectance => "backdrop",
            Self::LightingDirectionalColorR
            | Self::LightingDirectionalColorG
            | Self::LightingDirectionalColorB
            | Self::LightingDirectionalIlluminance
            | Self::LightingDirectionalTranslationX
            | Self::LightingDirectionalTranslationY
            | Self::LightingDirectionalTranslationZ
            | Self::LightingDirectionalLookAtX
            | Self::LightingDirectionalLookAtY
            | Self::LightingDirectionalLookAtZ => "dir",
            Self::LightingPointColorR
            | Self::LightingPointColorG
            | Self::LightingPointColorB
            | Self::LightingPointIntensity
            | Self::LightingPointRange
            | Self::LightingPointTranslationX
            | Self::LightingPointTranslationY
            | Self::LightingPointTranslationZ => "point",
            Self::LightingAccentColorR
            | Self::LightingAccentColorG
            | Self::LightingAccentColorB
            | Self::LightingAccentIntensity
            | Self::LightingAccentRange
            | Self::LightingAccentTranslationX
            | Self::LightingAccentTranslationY
            | Self::LightingAccentTranslationZ => "accent",
        }
    }

    pub(crate) fn change_target(self) -> SceneChangeTarget {
        match self {
            Self::ChildTwistPerVertexRadians | Self::ChildOutwardOffsetRatio => {
                SceneChangeTarget::Generation
            }
            Self::GlobalOpacity
            | Self::MaterialHueStepPerLevel
            | Self::MaterialSaturation
            | Self::MaterialLightness
            | Self::MaterialMetallic
            | Self::MaterialPerceptualRoughness
            | Self::MaterialReflectance
            | Self::MaterialCubeHueBias
            | Self::MaterialTetrahedronHueBias
            | Self::MaterialOctahedronHueBias
            | Self::MaterialDodecahedronHueBias
            | Self::MaterialSurfaceMode
            | Self::MaterialBaseSurface
            | Self::MaterialRootSurface
            | Self::MaterialAccentSurface
            | Self::MaterialAccentEveryNLevels
            | Self::MaterialLevelLightnessShift
            | Self::MaterialLevelSaturationShift
            | Self::MaterialLevelMetallicShift
            | Self::MaterialLevelRoughnessShift
            | Self::MaterialLevelReflectanceShift => SceneChangeTarget::Materials,
            Self::StageEnabled
            | Self::StageFloorEnabled
            | Self::StageBackdropEnabled
            | Self::StageFloorColorR
            | Self::StageFloorColorG
            | Self::StageFloorColorB
            | Self::StageFloorTranslationX
            | Self::StageFloorTranslationY
            | Self::StageFloorTranslationZ
            | Self::StageFloorRotationX
            | Self::StageFloorRotationY
            | Self::StageFloorRotationZ
            | Self::StageFloorSizeX
            | Self::StageFloorSizeY
            | Self::StageFloorThickness
            | Self::StageFloorMetallic
            | Self::StageFloorPerceptualRoughness
            | Self::StageFloorReflectance
            | Self::StageBackdropColorR
            | Self::StageBackdropColorG
            | Self::StageBackdropColorB
            | Self::StageBackdropTranslationX
            | Self::StageBackdropTranslationY
            | Self::StageBackdropTranslationZ
            | Self::StageBackdropRotationX
            | Self::StageBackdropRotationY
            | Self::StageBackdropRotationZ
            | Self::StageBackdropSizeX
            | Self::StageBackdropSizeY
            | Self::StageBackdropThickness
            | Self::StageBackdropMetallic
            | Self::StageBackdropPerceptualRoughness
            | Self::StageBackdropReflectance => SceneChangeTarget::Stage,
            Self::RenderingClearColorR
            | Self::RenderingClearColorG
            | Self::RenderingClearColorB
            | Self::RenderingAmbientColorR
            | Self::RenderingAmbientColorG
            | Self::RenderingAmbientColorB
            | Self::RenderingAmbientBrightness => SceneChangeTarget::Rendering,
            Self::LightingDirectionalColorR
            | Self::LightingDirectionalColorG
            | Self::LightingDirectionalColorB
            | Self::LightingDirectionalIlluminance
            | Self::LightingDirectionalTranslationX
            | Self::LightingDirectionalTranslationY
            | Self::LightingDirectionalTranslationZ
            | Self::LightingDirectionalLookAtX
            | Self::LightingDirectionalLookAtY
            | Self::LightingDirectionalLookAtZ
            | Self::LightingPointColorR
            | Self::LightingPointColorG
            | Self::LightingPointColorB
            | Self::LightingPointIntensity
            | Self::LightingPointRange
            | Self::LightingPointTranslationX
            | Self::LightingPointTranslationY
            | Self::LightingPointTranslationZ
            | Self::LightingAccentColorR
            | Self::LightingAccentColorG
            | Self::LightingAccentColorB
            | Self::LightingAccentIntensity
            | Self::LightingAccentRange
            | Self::LightingAccentTranslationX
            | Self::LightingAccentTranslationY
            | Self::LightingAccentTranslationZ => SceneChangeTarget::Lighting,
            Self::CameraDistance
            | Self::CameraAngularVelocityX
            | Self::CameraAngularVelocityY
            | Self::CameraAngularVelocityZ
            | Self::CameraZoomVelocity => SceneChangeTarget::Camera,
            Self::ChildKind
            | Self::SpawnPlacementMode
            | Self::SpawnAddMode
            | Self::ChildScaleRatio
            | Self::ChildAxisScaleX
            | Self::ChildAxisScaleY
            | Self::ChildAxisScaleZ
            | Self::ChildSpawnExclusionProbability => SceneChangeTarget::None,
        }
    }

    fn is_numeric(self) -> bool {
        !matches!(
            self,
            Self::ChildKind
                | Self::SpawnPlacementMode
                | Self::SpawnAddMode
                | Self::StageEnabled
                | Self::StageFloorEnabled
                | Self::StageBackdropEnabled
                | Self::MaterialSurfaceMode
                | Self::MaterialBaseSurface
                | Self::MaterialRootSurface
                | Self::MaterialAccentSurface
        )
    }

    fn is_generation_lfo_parameter(self) -> bool {
        Self::generation_lfo_capable().contains(&self)
    }

    fn lfo_scene_index(self) -> Option<usize> {
        Self::lfo_capable()
            .iter()
            .position(|candidate| *candidate == self)
    }

    fn runtime_default_lfo_amplitude(self, context: &EffectTunerViewContext<'_>) -> f32 {
        self.adjustment_step(context, false, false) * 5.0
    }

    fn generation_parameter(self) -> Option<GenerationParameter> {
        match self {
            Self::ChildScaleRatio => Some(GenerationParameter::ChildScaleRatio),
            Self::ChildAxisScaleX => Some(GenerationParameter::ChildAxisScaleX),
            Self::ChildAxisScaleY => Some(GenerationParameter::ChildAxisScaleY),
            Self::ChildAxisScaleZ => Some(GenerationParameter::ChildAxisScaleZ),
            Self::ChildTwistPerVertexRadians => {
                Some(GenerationParameter::ChildTwistPerVertexRadians)
            }
            Self::ChildOutwardOffsetRatio => Some(GenerationParameter::ChildOutwardOffsetRatio),
            Self::ChildSpawnExclusionProbability => {
                Some(GenerationParameter::ChildSpawnExclusionProbability)
            }
            _ => None,
        }
    }

    fn base_step(self, context: &EffectTunerViewContext<'_>) -> f32 {
        match self.generation_parameter() {
            Some(parameter) => context.generation_config.parameter_spec(parameter).step(),
            None => match self {
                Self::GlobalOpacity => context.material_config.opacity_adjust_step.abs(),
                Self::MaterialHueStepPerLevel
                | Self::MaterialCubeHueBias
                | Self::MaterialTetrahedronHueBias
                | Self::MaterialOctahedronHueBias
                | Self::MaterialDodecahedronHueBias
                | Self::StageFloorRotationX
                | Self::StageFloorRotationY
                | Self::StageFloorRotationZ
                | Self::StageBackdropRotationX
                | Self::StageBackdropRotationY
                | Self::StageBackdropRotationZ => 5.0,
                Self::MaterialAccentEveryNLevels => 1.0,
                Self::MaterialSaturation
                | Self::MaterialLightness
                | Self::MaterialMetallic
                | Self::MaterialPerceptualRoughness
                | Self::MaterialReflectance
                | Self::MaterialLevelLightnessShift
                | Self::MaterialLevelSaturationShift
                | Self::MaterialLevelMetallicShift
                | Self::MaterialLevelRoughnessShift
                | Self::MaterialLevelReflectanceShift
                | Self::RenderingClearColorR
                | Self::RenderingClearColorG
                | Self::RenderingClearColorB
                | Self::RenderingAmbientColorR
                | Self::RenderingAmbientColorG
                | Self::RenderingAmbientColorB
                | Self::StageFloorColorR
                | Self::StageFloorColorG
                | Self::StageFloorColorB
                | Self::StageBackdropColorR
                | Self::StageBackdropColorG
                | Self::StageBackdropColorB
                | Self::StageFloorMetallic
                | Self::StageFloorPerceptualRoughness
                | Self::StageFloorReflectance
                | Self::StageBackdropMetallic
                | Self::StageBackdropPerceptualRoughness
                | Self::StageBackdropReflectance
                | Self::LightingDirectionalColorR
                | Self::LightingDirectionalColorG
                | Self::LightingDirectionalColorB
                | Self::LightingPointColorR
                | Self::LightingPointColorG
                | Self::LightingPointColorB
                | Self::LightingAccentColorR
                | Self::LightingAccentColorG
                | Self::LightingAccentColorB => 0.05,
                Self::CameraDistance => 0.5,
                Self::CameraAngularVelocityX
                | Self::CameraAngularVelocityY
                | Self::CameraAngularVelocityZ => 0.1,
                Self::CameraZoomVelocity => 1.0,
                Self::RenderingAmbientBrightness => 0.5,
                Self::StageFloorTranslationX
                | Self::StageFloorTranslationY
                | Self::StageFloorTranslationZ
                | Self::StageBackdropTranslationX
                | Self::StageBackdropTranslationY
                | Self::StageBackdropTranslationZ
                | Self::LightingDirectionalTranslationX
                | Self::LightingDirectionalTranslationY
                | Self::LightingDirectionalTranslationZ
                | Self::LightingDirectionalLookAtX
                | Self::LightingDirectionalLookAtY
                | Self::LightingDirectionalLookAtZ
                | Self::LightingPointTranslationX
                | Self::LightingPointTranslationY
                | Self::LightingPointTranslationZ
                | Self::LightingAccentTranslationX
                | Self::LightingAccentTranslationY
                | Self::LightingAccentTranslationZ => 0.5,
                Self::StageFloorSizeX
                | Self::StageFloorSizeY
                | Self::StageBackdropSizeX
                | Self::StageBackdropSizeY => 0.5,
                Self::StageFloorThickness | Self::StageBackdropThickness => 0.05,
                Self::LightingDirectionalIlluminance => 500.0,
                Self::LightingPointIntensity | Self::LightingAccentIntensity => 50_000.0,
                Self::LightingPointRange | Self::LightingAccentRange => 1.0,
                Self::ChildKind
                | Self::SpawnPlacementMode
                | Self::SpawnAddMode
                | Self::StageEnabled
                | Self::StageFloorEnabled
                | Self::StageBackdropEnabled
                | Self::MaterialSurfaceMode
                | Self::MaterialBaseSurface
                | Self::MaterialRootSurface
                | Self::MaterialAccentSurface
                | Self::ChildScaleRatio
                | Self::ChildAxisScaleX
                | Self::ChildAxisScaleY
                | Self::ChildAxisScaleZ
                | Self::ChildTwistPerVertexRadians
                | Self::ChildOutwardOffsetRatio
                | Self::ChildSpawnExclusionProbability => 1.0,
            },
        }
    }

    fn adjustment_step(
        self,
        context: &EffectTunerViewContext<'_>,
        shift_pressed: bool,
        alt_pressed: bool,
    ) -> f32 {
        let mut step = self.base_step(context);
        if shift_pressed {
            step *= 10.0;
        }
        if alt_pressed {
            step *= 0.1;
        }
        step
    }

    fn value(self, context: &EffectTunerViewContext<'_>) -> f32 {
        match self {
            Self::ChildKind
            | Self::SpawnPlacementMode
            | Self::SpawnAddMode
            | Self::MaterialSurfaceMode
            | Self::MaterialBaseSurface
            | Self::MaterialRootSurface
            | Self::MaterialAccentSurface => 0.0,
            Self::ChildScaleRatio => context.generation_state.scale_ratio_base(),
            Self::ChildAxisScaleX => context.generation_state.child_axis_scale_base().x,
            Self::ChildAxisScaleY => context.generation_state.child_axis_scale_base().y,
            Self::ChildAxisScaleZ => context.generation_state.child_axis_scale_base().z,
            Self::ChildTwistPerVertexRadians => {
                context.generation_state.twist_per_vertex_radians_base()
            }
            Self::ChildOutwardOffsetRatio => context.generation_state.vertex_offset_ratio_base(),
            Self::ChildSpawnExclusionProbability => context
                .generation_state
                .vertex_spawn_exclusion_probability_base(),
            Self::StageEnabled => bool_to_value(context.stage_state.enabled),
            Self::StageFloorEnabled => bool_to_value(context.stage_state.floor_enabled),
            Self::StageBackdropEnabled => bool_to_value(context.stage_state.backdrop_enabled),
            _ => self
                .camera_numeric_value(context.camera_rig)
                .or_else(|| self.rendering_numeric_value(context.rendering_state))
                .or_else(|| self.stage_numeric_value(context.stage_state))
                .or_else(|| self.lighting_numeric_value(context.lighting_state))
                .or_else(|| self.material_numeric_value(context.material_state))
                .unwrap_or_default(),
        }
    }

    fn set_value(self, context: &mut EffectTunerEditContext<'_>, value: f32) -> f32 {
        match self.generation_parameter() {
            Some(parameter) => {
                let spec = context.generation_config.parameter_spec(parameter);
                let parameter_state = context.generation_state.parameter_mut(parameter);
                let current = parameter_state.base_value();
                parameter_state.adjust_clamped_base_value(value - current, spec)
            }
            None => {
                if let Some(applied) =
                    self.apply_camera_numeric_value(context.camera_config, context.camera_rig, value)
                {
                    applied
                } else if let Some(applied) =
                    self.apply_rendering_numeric_value(context.rendering_state, value)
                {
                    applied
                } else if let Some(applied) = self.apply_stage_numeric_value(context.stage_state, value)
                {
                    applied
                } else if let Some(applied) =
                    self.apply_lighting_numeric_value(context.lighting_state, value)
                {
                    applied
                } else if let Some(applied) = self.apply_material_numeric_value(
                    context.material_config,
                    context.material_state,
                    value,
                ) {
                    applied
                } else {
                    0.0
                }
            }
        }
    }

    fn apply_camera_numeric_value(
        self,
        camera_config: &crate::config::CameraConfig,
        camera_rig: &mut CameraRig,
        value: f32,
    ) -> Option<f32> {
        Some(match self {
            Self::CameraDistance => {
                let (min_distance, max_distance) = camera_config.distance_bounds();
                camera_rig.distance = value.clamp(min_distance, max_distance);
                camera_rig.distance
            }
            Self::CameraAngularVelocityX => {
                camera_rig.angular_velocity.x = value;
                camera_rig.angular_velocity.x
            }
            Self::CameraAngularVelocityY => {
                camera_rig.angular_velocity.y = value;
                camera_rig.angular_velocity.y
            }
            Self::CameraAngularVelocityZ => {
                camera_rig.angular_velocity.z = value;
                camera_rig.angular_velocity.z
            }
            Self::CameraZoomVelocity => {
                camera_rig.zoom_velocity = value;
                camera_rig.zoom_velocity
            }
            _ => return None,
        })
    }

    fn apply_rendering_numeric_value(
        self,
        rendering_state: &mut RenderingState,
        value: f32,
    ) -> Option<f32> {
        Some(match self {
            Self::RenderingClearColorR => {
                rendering_state.clear_color[0] = value;
                rendering_state.clear_color[0]
            }
            Self::RenderingClearColorG => {
                rendering_state.clear_color[1] = value;
                rendering_state.clear_color[1]
            }
            Self::RenderingClearColorB => {
                rendering_state.clear_color[2] = value;
                rendering_state.clear_color[2]
            }
            Self::RenderingAmbientColorR => {
                rendering_state.ambient_light_color[0] = value;
                rendering_state.ambient_light_color[0]
            }
            Self::RenderingAmbientColorG => {
                rendering_state.ambient_light_color[1] = value;
                rendering_state.ambient_light_color[1]
            }
            Self::RenderingAmbientColorB => {
                rendering_state.ambient_light_color[2] = value;
                rendering_state.ambient_light_color[2]
            }
            Self::RenderingAmbientBrightness => {
                rendering_state.ambient_light_brightness = value.max(0.0);
                rendering_state.ambient_light_brightness
            }
            _ => return None,
        })
    }

    fn apply_stage_numeric_value(
        self,
        stage_state: &mut StageState,
        value: f32,
    ) -> Option<f32> {
        Some(match self {
            Self::StageFloorColorR => {
                stage_state.floor.color[0] = value;
                stage_state.floor.color[0]
            }
            Self::StageFloorColorG => {
                stage_state.floor.color[1] = value;
                stage_state.floor.color[1]
            }
            Self::StageFloorColorB => {
                stage_state.floor.color[2] = value;
                stage_state.floor.color[2]
            }
            Self::StageFloorTranslationX => {
                stage_state.floor.translation[0] = value;
                stage_state.floor.translation[0]
            }
            Self::StageFloorTranslationY => {
                stage_state.floor.translation[1] = value;
                stage_state.floor.translation[1]
            }
            Self::StageFloorTranslationZ => {
                stage_state.floor.translation[2] = value;
                stage_state.floor.translation[2]
            }
            Self::StageFloorRotationX => {
                stage_state.floor.rotation_degrees[0] = value;
                stage_state.floor.rotation_degrees[0]
            }
            Self::StageFloorRotationY => {
                stage_state.floor.rotation_degrees[1] = value;
                stage_state.floor.rotation_degrees[1]
            }
            Self::StageFloorRotationZ => {
                stage_state.floor.rotation_degrees[2] = value;
                stage_state.floor.rotation_degrees[2]
            }
            Self::StageFloorSizeX => {
                stage_state.floor.size[0] = value.max(0.01);
                stage_state.floor.size[0]
            }
            Self::StageFloorSizeY => {
                stage_state.floor.size[1] = value.max(0.01);
                stage_state.floor.size[1]
            }
            Self::StageFloorThickness => {
                stage_state.floor.thickness = value.max(0.01);
                stage_state.floor.thickness
            }
            Self::StageFloorMetallic => {
                stage_state.floor.metallic = value.clamp(0.0, 1.0);
                stage_state.floor.metallic
            }
            Self::StageFloorPerceptualRoughness => {
                stage_state.floor.perceptual_roughness = value.clamp(0.0, 1.0);
                stage_state.floor.perceptual_roughness
            }
            Self::StageFloorReflectance => {
                stage_state.floor.reflectance = value.clamp(0.0, 1.0);
                stage_state.floor.reflectance
            }
            Self::StageBackdropColorR => {
                stage_state.backdrop.color[0] = value;
                stage_state.backdrop.color[0]
            }
            Self::StageBackdropColorG => {
                stage_state.backdrop.color[1] = value;
                stage_state.backdrop.color[1]
            }
            Self::StageBackdropColorB => {
                stage_state.backdrop.color[2] = value;
                stage_state.backdrop.color[2]
            }
            Self::StageBackdropTranslationX => {
                stage_state.backdrop.translation[0] = value;
                stage_state.backdrop.translation[0]
            }
            Self::StageBackdropTranslationY => {
                stage_state.backdrop.translation[1] = value;
                stage_state.backdrop.translation[1]
            }
            Self::StageBackdropTranslationZ => {
                stage_state.backdrop.translation[2] = value;
                stage_state.backdrop.translation[2]
            }
            Self::StageBackdropRotationX => {
                stage_state.backdrop.rotation_degrees[0] = value;
                stage_state.backdrop.rotation_degrees[0]
            }
            Self::StageBackdropRotationY => {
                stage_state.backdrop.rotation_degrees[1] = value;
                stage_state.backdrop.rotation_degrees[1]
            }
            Self::StageBackdropRotationZ => {
                stage_state.backdrop.rotation_degrees[2] = value;
                stage_state.backdrop.rotation_degrees[2]
            }
            Self::StageBackdropSizeX => {
                stage_state.backdrop.size[0] = value.max(0.01);
                stage_state.backdrop.size[0]
            }
            Self::StageBackdropSizeY => {
                stage_state.backdrop.size[1] = value.max(0.01);
                stage_state.backdrop.size[1]
            }
            Self::StageBackdropThickness => {
                stage_state.backdrop.thickness = value.max(0.01);
                stage_state.backdrop.thickness
            }
            Self::StageBackdropMetallic => {
                stage_state.backdrop.metallic = value.clamp(0.0, 1.0);
                stage_state.backdrop.metallic
            }
            Self::StageBackdropPerceptualRoughness => {
                stage_state.backdrop.perceptual_roughness = value.clamp(0.0, 1.0);
                stage_state.backdrop.perceptual_roughness
            }
            Self::StageBackdropReflectance => {
                stage_state.backdrop.reflectance = value.clamp(0.0, 1.0);
                stage_state.backdrop.reflectance
            }
            _ => return None,
        })
    }

    fn apply_lighting_numeric_value(
        self,
        lighting_state: &mut LightingState,
        value: f32,
    ) -> Option<f32> {
        Some(match self {
            Self::LightingDirectionalColorR => {
                lighting_state.directional.color[0] = value;
                lighting_state.directional.color[0]
            }
            Self::LightingDirectionalColorG => {
                lighting_state.directional.color[1] = value;
                lighting_state.directional.color[1]
            }
            Self::LightingDirectionalColorB => {
                lighting_state.directional.color[2] = value;
                lighting_state.directional.color[2]
            }
            Self::LightingDirectionalIlluminance => {
                lighting_state.directional.illuminance = value.max(0.0);
                lighting_state.directional.illuminance
            }
            Self::LightingDirectionalTranslationX => {
                lighting_state.directional.translation[0] = value;
                lighting_state.directional.translation[0]
            }
            Self::LightingDirectionalTranslationY => {
                lighting_state.directional.translation[1] = value;
                lighting_state.directional.translation[1]
            }
            Self::LightingDirectionalTranslationZ => {
                lighting_state.directional.translation[2] = value;
                lighting_state.directional.translation[2]
            }
            Self::LightingDirectionalLookAtX => {
                lighting_state.directional.look_at[0] = value;
                lighting_state.directional.look_at[0]
            }
            Self::LightingDirectionalLookAtY => {
                lighting_state.directional.look_at[1] = value;
                lighting_state.directional.look_at[1]
            }
            Self::LightingDirectionalLookAtZ => {
                lighting_state.directional.look_at[2] = value;
                lighting_state.directional.look_at[2]
            }
            Self::LightingPointColorR => {
                lighting_state.point.color[0] = value;
                lighting_state.point.color[0]
            }
            Self::LightingPointColorG => {
                lighting_state.point.color[1] = value;
                lighting_state.point.color[1]
            }
            Self::LightingPointColorB => {
                lighting_state.point.color[2] = value;
                lighting_state.point.color[2]
            }
            Self::LightingPointIntensity => {
                lighting_state.point.intensity = value.max(0.0);
                lighting_state.point.intensity
            }
            Self::LightingPointRange => {
                lighting_state.point.range = value.max(0.0);
                lighting_state.point.range
            }
            Self::LightingPointTranslationX => {
                lighting_state.point.translation[0] = value;
                lighting_state.point.translation[0]
            }
            Self::LightingPointTranslationY => {
                lighting_state.point.translation[1] = value;
                lighting_state.point.translation[1]
            }
            Self::LightingPointTranslationZ => {
                lighting_state.point.translation[2] = value;
                lighting_state.point.translation[2]
            }
            Self::LightingAccentColorR => {
                lighting_state.accent.color[0] = value;
                lighting_state.accent.color[0]
            }
            Self::LightingAccentColorG => {
                lighting_state.accent.color[1] = value;
                lighting_state.accent.color[1]
            }
            Self::LightingAccentColorB => {
                lighting_state.accent.color[2] = value;
                lighting_state.accent.color[2]
            }
            Self::LightingAccentIntensity => {
                lighting_state.accent.intensity = value.max(0.0);
                lighting_state.accent.intensity
            }
            Self::LightingAccentRange => {
                lighting_state.accent.range = value.max(0.0);
                lighting_state.accent.range
            }
            Self::LightingAccentTranslationX => {
                lighting_state.accent.translation[0] = value;
                lighting_state.accent.translation[0]
            }
            Self::LightingAccentTranslationY => {
                lighting_state.accent.translation[1] = value;
                lighting_state.accent.translation[1]
            }
            Self::LightingAccentTranslationZ => {
                lighting_state.accent.translation[2] = value;
                lighting_state.accent.translation[2]
            }
            _ => return None,
        })
    }

    fn apply_material_numeric_value(
        self,
        material_config: &MaterialConfig,
        material_state: &mut MaterialState,
        value: f32,
    ) -> Option<f32> {
        Some(match self {
            Self::GlobalOpacity => {
                let (min_opacity, max_opacity) = material_config.opacity_bounds();
                material_state.opacity = value.clamp(min_opacity, max_opacity);
                material_state.opacity
            }
            Self::MaterialHueStepPerLevel => {
                material_state.hue_step_per_level = value;
                material_state.hue_step_per_level
            }
            Self::MaterialSaturation => {
                material_state.saturation = value.clamp(0.0, 1.0);
                material_state.saturation
            }
            Self::MaterialLightness => {
                material_state.lightness = value.clamp(0.0, 1.0);
                material_state.lightness
            }
            Self::MaterialMetallic => {
                material_state.metallic = value.clamp(0.0, 1.0);
                material_state.metallic
            }
            Self::MaterialPerceptualRoughness => {
                material_state.perceptual_roughness = value.clamp(0.0, 1.0);
                material_state.perceptual_roughness
            }
            Self::MaterialReflectance => {
                material_state.reflectance = value.clamp(0.0, 1.0);
                material_state.reflectance
            }
            Self::MaterialCubeHueBias => {
                material_state.cube_hue_bias = value;
                material_state.cube_hue_bias
            }
            Self::MaterialTetrahedronHueBias => {
                material_state.tetrahedron_hue_bias = value;
                material_state.tetrahedron_hue_bias
            }
            Self::MaterialOctahedronHueBias => {
                material_state.octahedron_hue_bias = value;
                material_state.octahedron_hue_bias
            }
            Self::MaterialDodecahedronHueBias => {
                material_state.dodecahedron_hue_bias = value;
                material_state.dodecahedron_hue_bias
            }
            Self::MaterialAccentEveryNLevels => {
                material_state.accent_every_n_levels = value.round().max(0.0) as usize;
                material_state.accent_every_n_levels as f32
            }
            Self::MaterialLevelLightnessShift => {
                material_state.level_lightness_shift = value;
                material_state.level_lightness_shift
            }
            Self::MaterialLevelSaturationShift => {
                material_state.level_saturation_shift = value;
                material_state.level_saturation_shift
            }
            Self::MaterialLevelMetallicShift => {
                material_state.level_metallic_shift = value;
                material_state.level_metallic_shift
            }
            Self::MaterialLevelRoughnessShift => {
                material_state.level_roughness_shift = value;
                material_state.level_roughness_shift
            }
            Self::MaterialLevelReflectanceShift => {
                material_state.level_reflectance_shift = value;
                material_state.level_reflectance_shift
            }
            _ => return None,
        })
    }

    fn default_value(self, context: &EffectTunerViewContext<'_>) -> f32 {
        match self.generation_parameter() {
            Some(parameter) => context
                .generation_config
                .parameter_spec(parameter)
                .default_value(),
            None => match self {
                Self::GlobalOpacity => context.material_config.default_opacity_clamped(),
                Self::MaterialHueStepPerLevel => context.material_config.hue_step_per_level,
                Self::MaterialSaturation => context.material_config.saturation,
                Self::MaterialLightness => context.material_config.lightness,
                Self::MaterialMetallic => context.material_config.metallic,
                Self::MaterialPerceptualRoughness => context.material_config.perceptual_roughness,
                Self::MaterialReflectance => context.material_config.reflectance,
                Self::MaterialCubeHueBias => context.material_config.cube_hue_bias,
                Self::MaterialTetrahedronHueBias => context.material_config.tetrahedron_hue_bias,
                Self::MaterialOctahedronHueBias => context.material_config.octahedron_hue_bias,
                Self::MaterialDodecahedronHueBias => context.material_config.dodecahedron_hue_bias,
                Self::MaterialAccentEveryNLevels => {
                    context.material_config.accent_every_n_levels as f32
                }
                Self::MaterialLevelLightnessShift => context.material_config.level_lightness_shift,
                Self::MaterialLevelSaturationShift => {
                    context.material_config.level_saturation_shift
                }
                Self::MaterialLevelMetallicShift => context.material_config.level_metallic_shift,
                Self::MaterialLevelRoughnessShift => context.material_config.level_roughness_shift,
                Self::MaterialLevelReflectanceShift => {
                    context.material_config.level_reflectance_shift
                }
                Self::CameraDistance => context.camera_config.initial_distance,
                Self::CameraAngularVelocityX
                | Self::CameraAngularVelocityY
                | Self::CameraAngularVelocityZ
                | Self::CameraZoomVelocity => 0.0,
                Self::RenderingClearColorR => context.rendering_config.clear_color[0],
                Self::RenderingClearColorG => context.rendering_config.clear_color[1],
                Self::RenderingClearColorB => context.rendering_config.clear_color[2],
                Self::RenderingAmbientColorR => context.rendering_config.ambient_light_color[0],
                Self::RenderingAmbientColorG => context.rendering_config.ambient_light_color[1],
                Self::RenderingAmbientColorB => context.rendering_config.ambient_light_color[2],
                Self::RenderingAmbientBrightness => context.rendering_config.ambient_light_brightness,
                Self::StageFloorColorR => context.rendering_config.stage.floor.color[0],
                Self::StageFloorColorG => context.rendering_config.stage.floor.color[1],
                Self::StageFloorColorB => context.rendering_config.stage.floor.color[2],
                Self::StageFloorTranslationX => context.rendering_config.stage.floor.translation[0],
                Self::StageFloorTranslationY => context.rendering_config.stage.floor.translation[1],
                Self::StageFloorTranslationZ => context.rendering_config.stage.floor.translation[2],
                Self::StageFloorRotationX => context.rendering_config.stage.floor.rotation_degrees[0],
                Self::StageFloorRotationY => context.rendering_config.stage.floor.rotation_degrees[1],
                Self::StageFloorRotationZ => context.rendering_config.stage.floor.rotation_degrees[2],
                Self::StageFloorSizeX => context.rendering_config.stage.floor.size[0],
                Self::StageFloorSizeY => context.rendering_config.stage.floor.size[1],
                Self::StageFloorThickness => context.rendering_config.stage.floor.thickness,
                Self::StageFloorMetallic => context.rendering_config.stage.floor.metallic,
                Self::StageFloorPerceptualRoughness => {
                    context.rendering_config.stage.floor.perceptual_roughness
                }
                Self::StageFloorReflectance => context.rendering_config.stage.floor.reflectance,
                Self::StageBackdropColorR => context.rendering_config.stage.backdrop.color[0],
                Self::StageBackdropColorG => context.rendering_config.stage.backdrop.color[1],
                Self::StageBackdropColorB => context.rendering_config.stage.backdrop.color[2],
                Self::StageBackdropTranslationX => {
                    context.rendering_config.stage.backdrop.translation[0]
                }
                Self::StageBackdropTranslationY => {
                    context.rendering_config.stage.backdrop.translation[1]
                }
                Self::StageBackdropTranslationZ => {
                    context.rendering_config.stage.backdrop.translation[2]
                }
                Self::StageBackdropRotationX => {
                    context.rendering_config.stage.backdrop.rotation_degrees[0]
                }
                Self::StageBackdropRotationY => {
                    context.rendering_config.stage.backdrop.rotation_degrees[1]
                }
                Self::StageBackdropRotationZ => {
                    context.rendering_config.stage.backdrop.rotation_degrees[2]
                }
                Self::StageBackdropSizeX => context.rendering_config.stage.backdrop.size[0],
                Self::StageBackdropSizeY => context.rendering_config.stage.backdrop.size[1],
                Self::StageBackdropThickness => context.rendering_config.stage.backdrop.thickness,
                Self::StageBackdropMetallic => context.rendering_config.stage.backdrop.metallic,
                Self::StageBackdropPerceptualRoughness => {
                    context.rendering_config.stage.backdrop.perceptual_roughness
                }
                Self::StageBackdropReflectance => {
                    context.rendering_config.stage.backdrop.reflectance
                }
                Self::LightingDirectionalColorR => context.lighting_config.directional.color[0],
                Self::LightingDirectionalColorG => context.lighting_config.directional.color[1],
                Self::LightingDirectionalColorB => context.lighting_config.directional.color[2],
                Self::LightingDirectionalIlluminance => {
                    context.lighting_config.directional.illuminance
                }
                Self::LightingDirectionalTranslationX => {
                    context.lighting_config.directional.translation[0]
                }
                Self::LightingDirectionalTranslationY => {
                    context.lighting_config.directional.translation[1]
                }
                Self::LightingDirectionalTranslationZ => {
                    context.lighting_config.directional.translation[2]
                }
                Self::LightingDirectionalLookAtX => context.lighting_config.directional.look_at[0],
                Self::LightingDirectionalLookAtY => context.lighting_config.directional.look_at[1],
                Self::LightingDirectionalLookAtZ => context.lighting_config.directional.look_at[2],
                Self::LightingPointColorR => context.lighting_config.point.color[0],
                Self::LightingPointColorG => context.lighting_config.point.color[1],
                Self::LightingPointColorB => context.lighting_config.point.color[2],
                Self::LightingPointIntensity => context.lighting_config.point.intensity,
                Self::LightingPointRange => context.lighting_config.point.range,
                Self::LightingPointTranslationX => context.lighting_config.point.translation[0],
                Self::LightingPointTranslationY => context.lighting_config.point.translation[1],
                Self::LightingPointTranslationZ => context.lighting_config.point.translation[2],
                Self::LightingAccentColorR => context.lighting_config.accent.color[0],
                Self::LightingAccentColorG => context.lighting_config.accent.color[1],
                Self::LightingAccentColorB => context.lighting_config.accent.color[2],
                Self::LightingAccentIntensity => context.lighting_config.accent.intensity,
                Self::LightingAccentRange => context.lighting_config.accent.range,
                Self::LightingAccentTranslationX => context.lighting_config.accent.translation[0],
                Self::LightingAccentTranslationY => context.lighting_config.accent.translation[1],
                Self::LightingAccentTranslationZ => context.lighting_config.accent.translation[2],
                _ => 0.0,
            },
        }
    }

    fn display_value(self, context: &EffectTunerViewContext<'_>) -> String {
        match self {
            Self::ChildKind => {
                shape_kind_value_text(context.generation_state.selected_shape_kind).to_string()
            }
            Self::SpawnPlacementMode => context
                .generation_state
                .spawn_placement_mode
                .plural_label()
                .to_string(),
            Self::SpawnAddMode => context.generation_state.spawn_add_mode.label().to_string(),
            Self::StageEnabled => boolean_value_text(context.stage_state.enabled).to_string(),
            Self::StageFloorEnabled => {
                boolean_value_text(context.stage_state.floor_enabled).to_string()
            }
            Self::StageBackdropEnabled => {
                boolean_value_text(context.stage_state.backdrop_enabled).to_string()
            }
            Self::MaterialSurfaceMode => {
                material_surface_mode_value_text(context.material_state.surface_mode).to_string()
            }
            Self::MaterialBaseSurface => {
                material_surface_family_value_text(context.material_state.base_surface).to_string()
            }
            Self::MaterialRootSurface => {
                material_surface_family_value_text(context.material_state.root_surface).to_string()
            }
            Self::MaterialAccentSurface => {
                material_surface_family_value_text(context.material_state.accent_surface)
                    .to_string()
            }
            Self::MaterialAccentEveryNLevels => {
                context.material_state.accent_every_n_levels.to_string()
            }
            _ => format_scene_numeric_value(self.value(context)),
        }
    }

    fn display_numeric_value(self, value: f32) -> Option<String> {
        if self == Self::MaterialAccentEveryNLevels {
            return Some(value.round().max(0.0).to_string());
        }

        self.is_numeric()
            .then(|| format_scene_numeric_value(value))
    }

    fn live_value(self, context: &EffectTunerViewContext<'_>) -> Option<f32> {
        match self {
            Self::ChildScaleRatio => {
                Some(context.generation_state.scale_ratio(context.generation_config))
            }
            Self::ChildAxisScaleX => Some(context.generation_state.child_axis_scale(context.generation_config).x),
            Self::ChildAxisScaleY => Some(context.generation_state.child_axis_scale(context.generation_config).y),
            Self::ChildAxisScaleZ => Some(context.generation_state.child_axis_scale(context.generation_config).z),
            Self::ChildTwistPerVertexRadians => Some(
                context
                    .generation_state
                    .twist_per_vertex_radians(context.generation_config),
            ),
            Self::ChildOutwardOffsetRatio => Some(
                context
                    .generation_state
                    .vertex_offset_ratio(context.generation_config),
            ),
            Self::ChildSpawnExclusionProbability => Some(
                context
                    .generation_state
                    .vertex_spawn_exclusion_probability(context.generation_config),
            ),
            _ if self.is_numeric() => Some(self.value(context)),
            _ => None,
        }
    }

    fn camera_numeric_value(self, camera_rig: &CameraRig) -> Option<f32> {
        Some(match self {
            Self::CameraDistance => camera_rig.distance,
            Self::CameraAngularVelocityX => camera_rig.angular_velocity.x,
            Self::CameraAngularVelocityY => camera_rig.angular_velocity.y,
            Self::CameraAngularVelocityZ => camera_rig.angular_velocity.z,
            Self::CameraZoomVelocity => camera_rig.zoom_velocity,
            _ => return None,
        })
    }

    fn rendering_numeric_value(self, rendering_state: &RenderingState) -> Option<f32> {
        Some(match self {
            Self::RenderingClearColorR => rendering_state.clear_color[0],
            Self::RenderingClearColorG => rendering_state.clear_color[1],
            Self::RenderingClearColorB => rendering_state.clear_color[2],
            Self::RenderingAmbientColorR => rendering_state.ambient_light_color[0],
            Self::RenderingAmbientColorG => rendering_state.ambient_light_color[1],
            Self::RenderingAmbientColorB => rendering_state.ambient_light_color[2],
            Self::RenderingAmbientBrightness => rendering_state.ambient_light_brightness,
            _ => return None,
        })
    }

    fn stage_numeric_value(self, stage_state: &StageState) -> Option<f32> {
        Some(match self {
            Self::StageFloorColorR => stage_state.floor.color[0],
            Self::StageFloorColorG => stage_state.floor.color[1],
            Self::StageFloorColorB => stage_state.floor.color[2],
            Self::StageFloorTranslationX => stage_state.floor.translation[0],
            Self::StageFloorTranslationY => stage_state.floor.translation[1],
            Self::StageFloorTranslationZ => stage_state.floor.translation[2],
            Self::StageFloorRotationX => stage_state.floor.rotation_degrees[0],
            Self::StageFloorRotationY => stage_state.floor.rotation_degrees[1],
            Self::StageFloorRotationZ => stage_state.floor.rotation_degrees[2],
            Self::StageFloorSizeX => stage_state.floor.size[0],
            Self::StageFloorSizeY => stage_state.floor.size[1],
            Self::StageFloorThickness => stage_state.floor.thickness,
            Self::StageFloorMetallic => stage_state.floor.metallic,
            Self::StageFloorPerceptualRoughness => stage_state.floor.perceptual_roughness,
            Self::StageFloorReflectance => stage_state.floor.reflectance,
            Self::StageBackdropColorR => stage_state.backdrop.color[0],
            Self::StageBackdropColorG => stage_state.backdrop.color[1],
            Self::StageBackdropColorB => stage_state.backdrop.color[2],
            Self::StageBackdropTranslationX => stage_state.backdrop.translation[0],
            Self::StageBackdropTranslationY => stage_state.backdrop.translation[1],
            Self::StageBackdropTranslationZ => stage_state.backdrop.translation[2],
            Self::StageBackdropRotationX => stage_state.backdrop.rotation_degrees[0],
            Self::StageBackdropRotationY => stage_state.backdrop.rotation_degrees[1],
            Self::StageBackdropRotationZ => stage_state.backdrop.rotation_degrees[2],
            Self::StageBackdropSizeX => stage_state.backdrop.size[0],
            Self::StageBackdropSizeY => stage_state.backdrop.size[1],
            Self::StageBackdropThickness => stage_state.backdrop.thickness,
            Self::StageBackdropMetallic => stage_state.backdrop.metallic,
            Self::StageBackdropPerceptualRoughness => stage_state.backdrop.perceptual_roughness,
            Self::StageBackdropReflectance => stage_state.backdrop.reflectance,
            _ => return None,
        })
    }

    fn lighting_numeric_value(self, lighting_state: &LightingState) -> Option<f32> {
        Some(match self {
            Self::LightingDirectionalColorR => lighting_state.directional.color[0],
            Self::LightingDirectionalColorG => lighting_state.directional.color[1],
            Self::LightingDirectionalColorB => lighting_state.directional.color[2],
            Self::LightingDirectionalIlluminance => lighting_state.directional.illuminance,
            Self::LightingDirectionalTranslationX => lighting_state.directional.translation[0],
            Self::LightingDirectionalTranslationY => lighting_state.directional.translation[1],
            Self::LightingDirectionalTranslationZ => lighting_state.directional.translation[2],
            Self::LightingDirectionalLookAtX => lighting_state.directional.look_at[0],
            Self::LightingDirectionalLookAtY => lighting_state.directional.look_at[1],
            Self::LightingDirectionalLookAtZ => lighting_state.directional.look_at[2],
            Self::LightingPointColorR => lighting_state.point.color[0],
            Self::LightingPointColorG => lighting_state.point.color[1],
            Self::LightingPointColorB => lighting_state.point.color[2],
            Self::LightingPointIntensity => lighting_state.point.intensity,
            Self::LightingPointRange => lighting_state.point.range,
            Self::LightingPointTranslationX => lighting_state.point.translation[0],
            Self::LightingPointTranslationY => lighting_state.point.translation[1],
            Self::LightingPointTranslationZ => lighting_state.point.translation[2],
            Self::LightingAccentColorR => lighting_state.accent.color[0],
            Self::LightingAccentColorG => lighting_state.accent.color[1],
            Self::LightingAccentColorB => lighting_state.accent.color[2],
            Self::LightingAccentIntensity => lighting_state.accent.intensity,
            Self::LightingAccentRange => lighting_state.accent.range,
            Self::LightingAccentTranslationX => lighting_state.accent.translation[0],
            Self::LightingAccentTranslationY => lighting_state.accent.translation[1],
            Self::LightingAccentTranslationZ => lighting_state.accent.translation[2],
            _ => return None,
        })
    }

    fn material_numeric_value(self, material_state: &MaterialState) -> Option<f32> {
        Some(match self {
            Self::GlobalOpacity => material_state.opacity,
            Self::MaterialHueStepPerLevel => material_state.hue_step_per_level,
            Self::MaterialSaturation => material_state.saturation,
            Self::MaterialLightness => material_state.lightness,
            Self::MaterialMetallic => material_state.metallic,
            Self::MaterialPerceptualRoughness => material_state.perceptual_roughness,
            Self::MaterialReflectance => material_state.reflectance,
            Self::MaterialCubeHueBias => material_state.cube_hue_bias,
            Self::MaterialTetrahedronHueBias => material_state.tetrahedron_hue_bias,
            Self::MaterialOctahedronHueBias => material_state.octahedron_hue_bias,
            Self::MaterialDodecahedronHueBias => material_state.dodecahedron_hue_bias,
            Self::MaterialAccentEveryNLevels => material_state.accent_every_n_levels as f32,
            Self::MaterialLevelLightnessShift => material_state.level_lightness_shift,
            Self::MaterialLevelSaturationShift => material_state.level_saturation_shift,
            Self::MaterialLevelMetallicShift => material_state.level_metallic_shift,
            Self::MaterialLevelRoughnessShift => material_state.level_roughness_shift,
            Self::MaterialLevelReflectanceShift => material_state.level_reflectance_shift,
            _ => return None,
        })
    }

    fn apply_numeric_input(&self, context: &mut EffectTunerEditContext<'_>, value: f32) -> bool {
        if !self.is_numeric() {
            return false;
        }

        let _ = self.set_value(context, value);
        true
    }

    fn adjust_value(
        self,
        context: &mut EffectTunerEditContext<'_>,
        direction: f32,
        shift_pressed: bool,
        alt_pressed: bool,
    ) {
        match self {
            Self::ChildKind => {
                context.generation_state.selected_shape_kind = cycle_shape_kind(
                    context.generation_state.selected_shape_kind,
                    direction as isize,
                );
            }
            Self::SpawnPlacementMode => {
                context.generation_state.spawn_placement_mode = cycle_spawn_placement_mode(
                    context.generation_state.spawn_placement_mode,
                    direction as isize,
                );
            }
            Self::SpawnAddMode => {
                context.generation_state.spawn_add_mode = cycle_spawn_add_mode(
                    context.generation_state.spawn_add_mode,
                    direction as isize,
                );
            }
            Self::StageEnabled => {
                context.stage_state.enabled = cycle_bool(context.stage_state.enabled, direction);
            }
            Self::StageFloorEnabled => {
                context.stage_state.floor_enabled =
                    cycle_bool(context.stage_state.floor_enabled, direction);
            }
            Self::StageBackdropEnabled => {
                context.stage_state.backdrop_enabled =
                    cycle_bool(context.stage_state.backdrop_enabled, direction);
            }
            Self::MaterialSurfaceMode => {
                context.material_state.surface_mode = cycle_material_surface_mode(
                    context.material_state.surface_mode,
                    direction as isize,
                );
            }
            Self::MaterialBaseSurface => {
                context.material_state.base_surface = cycle_material_surface_family(
                    context.material_state.base_surface,
                    direction as isize,
                );
            }
            Self::MaterialRootSurface => {
                context.material_state.root_surface = cycle_material_surface_family(
                    context.material_state.root_surface,
                    direction as isize,
                );
            }
            Self::MaterialAccentSurface => {
                context.material_state.accent_surface = cycle_material_surface_family(
                    context.material_state.accent_surface,
                    direction as isize,
                );
            }
            _ => {
                let current_value = self.value(&context.view());
                let next_value = current_value
                    + direction * self.adjustment_step(&context.view(), shift_pressed, alt_pressed);
                let _ = self.set_value(context, next_value);
            }
        }
    }

    fn reset_value(self, context: &mut EffectTunerEditContext<'_>) {
        match self {
            Self::ChildKind => {
                context.generation_state.selected_shape_kind =
                    context.generation_config.default_child_shape_kind;
            }
            Self::SpawnPlacementMode => {
                context.generation_state.spawn_placement_mode =
                    context.generation_config.default_spawn_placement_mode;
            }
            Self::SpawnAddMode => {
                context.generation_state.spawn_add_mode = SpawnAddMode::default();
            }
            Self::StageEnabled => {
                context.stage_state.enabled = context.rendering_config.stage.enabled;
            }
            Self::StageFloorEnabled => {
                context.stage_state.floor_enabled = context.rendering_config.stage.floor.enabled;
            }
            Self::StageBackdropEnabled => {
                context.stage_state.backdrop_enabled =
                    context.rendering_config.stage.backdrop.enabled;
            }
            Self::MaterialSurfaceMode => {
                context.material_state.surface_mode = context.material_config.surface_mode;
            }
            Self::MaterialBaseSurface => {
                context.material_state.base_surface = context.material_config.base_surface;
            }
            Self::MaterialRootSurface => {
                context.material_state.root_surface = context.material_config.root_surface;
            }
            Self::MaterialAccentSurface => {
                context.material_state.accent_surface = context.material_config.accent_surface;
            }
            _ => {
                let default_value = self.default_value(&context.view());
                let _ = self.set_value(context, default_value);
            }
        }
    }
}

fn bool_to_value(enabled: bool) -> f32 {
    if enabled { 1.0 } else { 0.0 }
}

fn format_scene_numeric_value(value: f32) -> String {
    let magnitude = value.abs();
    if magnitude >= 10_000.0 {
        format!("{value:.0}")
    } else if magnitude >= 1_000.0 {
        format!("{value:.1}")
    } else if magnitude >= 100.0 {
        format!("{value:.2}")
    } else {
        format!("{value:.3}")
    }
}
