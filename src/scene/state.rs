#[derive(Resource)]
pub(crate) struct ShapeAssets {
    pub(crate) catalog: ShapeCatalog,
    cube: ShapeRuntime,
    tetrahedron: ShapeRuntime,
    octahedron: ShapeRuntime,
    dodecahedron: ShapeRuntime,
}

impl ShapeAssets {
    pub(crate) fn new(meshes: &mut Assets<Mesh>) -> Self {
        let catalog = ShapeCatalog::new();

        Self {
            cube: ShapeRuntime::new(catalog.geometry(ShapeKind::Cube), meshes),
            tetrahedron: ShapeRuntime::new(catalog.geometry(ShapeKind::Tetrahedron), meshes),
            octahedron: ShapeRuntime::new(catalog.geometry(ShapeKind::Octahedron), meshes),
            dodecahedron: ShapeRuntime::new(catalog.geometry(ShapeKind::Dodecahedron), meshes),
            catalog,
        }
    }

    pub(crate) fn mesh(&self, kind: ShapeKind) -> &Handle<Mesh> {
        match kind {
            ShapeKind::Cube => &self.cube.mesh,
            ShapeKind::Tetrahedron => &self.tetrahedron.mesh,
            ShapeKind::Octahedron => &self.octahedron.mesh,
            ShapeKind::Dodecahedron => &self.dodecahedron.mesh,
        }
    }
}

struct ShapeRuntime {
    mesh: Handle<Mesh>,
}

impl ShapeRuntime {
    fn new(geometry: &ShapeGeometry, meshes: &mut Assets<Mesh>) -> Self {
        Self {
            mesh: meshes.add(build_mesh(geometry)),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct GenerationParameters {
    scale_ratio: ScalarParameterState,
    child_twist: ScalarParameterState,
    child_offset: ScalarParameterState,
    child_spawn_exclusion_probability: ScalarParameterState,
}

impl GenerationParameters {
    pub(crate) fn from_config(generation_config: &GenerationConfig) -> Self {
        Self {
            scale_ratio: ScalarParameterState::new(
                generation_config.parameter_spec(GenerationParameter::ChildScaleRatio),
            ),
            child_twist: ScalarParameterState::new(
                generation_config.parameter_spec(GenerationParameter::ChildTwistPerVertexRadians),
            ),
            child_offset: ScalarParameterState::new(
                generation_config.parameter_spec(GenerationParameter::ChildOutwardOffsetRatio),
            ),
            child_spawn_exclusion_probability: ScalarParameterState::new(
                generation_config
                    .parameter_spec(GenerationParameter::ChildSpawnExclusionProbability),
            ),
        }
    }

    pub(crate) fn from_base_values(
        scale_ratio: f32,
        child_twist: f32,
        child_offset: f32,
        child_spawn_exclusion_probability: f32,
    ) -> Self {
        Self {
            scale_ratio: ScalarParameterState::from_base(scale_ratio),
            child_twist: ScalarParameterState::from_base(child_twist),
            child_offset: ScalarParameterState::from_base(child_offset),
            child_spawn_exclusion_probability: ScalarParameterState::from_base(
                child_spawn_exclusion_probability,
            ),
        }
    }

    fn parameter(&self, parameter: GenerationParameter) -> &ScalarParameterState {
        match parameter {
            GenerationParameter::ChildScaleRatio => &self.scale_ratio,
            GenerationParameter::ChildTwistPerVertexRadians => &self.child_twist,
            GenerationParameter::ChildOutwardOffsetRatio => &self.child_offset,
            GenerationParameter::ChildSpawnExclusionProbability => {
                &self.child_spawn_exclusion_probability
            }
        }
    }

    fn parameter_mut(&mut self, parameter: GenerationParameter) -> &mut ScalarParameterState {
        match parameter {
            GenerationParameter::ChildScaleRatio => &mut self.scale_ratio,
            GenerationParameter::ChildTwistPerVertexRadians => &mut self.child_twist,
            GenerationParameter::ChildOutwardOffsetRatio => &mut self.child_offset,
            GenerationParameter::ChildSpawnExclusionProbability => {
                &mut self.child_spawn_exclusion_probability
            }
        }
    }

    fn base_value(&self, parameter: GenerationParameter) -> f32 {
        self.parameter(parameter).base_value()
    }

    fn evaluated(
        &self,
        parameter: GenerationParameter,
        generation_config: &GenerationConfig,
    ) -> f32 {
        self.parameter(parameter)
            .evaluated(generation_config.parameter_spec(parameter))
    }

    fn spawn_tuning(
        &self,
        generation_config: &GenerationConfig,
        spawn_placement_mode: SpawnPlacementMode,
    ) -> SpawnTuning {
        generation_config.spawn_tuning(
            self.evaluated(
                GenerationParameter::ChildTwistPerVertexRadians,
                generation_config,
            ),
            self.evaluated(
                GenerationParameter::ChildOutwardOffsetRatio,
                generation_config,
            ),
            self.evaluated(
                GenerationParameter::ChildSpawnExclusionProbability,
                generation_config,
            ),
            spawn_placement_mode,
        )
    }

    fn clear_runtime_state(&mut self) {
        for parameter in GenerationParameter::ALL {
            self.parameter_mut(parameter).clear_runtime_state();
        }
    }
}

#[derive(Clone, Resource)]
pub(crate) struct GenerationState {
    pub(crate) nodes: Vec<ShapeNode>,
    pub(crate) selected_shape_kind: ShapeKind,
    pub(crate) spawn_placement_mode: SpawnPlacementMode,
    pub(crate) spawn_add_mode: SpawnAddMode,
    pub(crate) parameters: GenerationParameters,
    pub(crate) spawn_hold: HoldRepeatState,
}

#[cfg_attr(not(test), allow(dead_code))]
impl GenerationState {
    pub(crate) fn from_config(generation_config: &GenerationConfig) -> Self {
        let shape_catalog = ShapeCatalog::new();
        let root = root_generation_node(&shape_catalog, generation_config);

        Self {
            nodes: vec![root],
            selected_shape_kind: generation_config.default_child_shape_kind,
            spawn_placement_mode: generation_config.default_spawn_placement_mode,
            spawn_add_mode: SpawnAddMode::default(),
            parameters: GenerationParameters::from_config(generation_config),
            spawn_hold: HoldRepeatState::default(),
        }
    }

    pub(crate) fn parameter(&self, parameter: GenerationParameter) -> &ScalarParameterState {
        self.parameters.parameter(parameter)
    }

    pub(crate) fn parameter_mut(
        &mut self,
        parameter: GenerationParameter,
    ) -> &mut ScalarParameterState {
        self.parameters.parameter_mut(parameter)
    }

    pub(crate) fn scale_ratio(&self, generation_config: &GenerationConfig) -> f32 {
        self.parameters
            .evaluated(GenerationParameter::ChildScaleRatio, generation_config)
    }

    pub(crate) fn scale_ratio_base(&self) -> f32 {
        self.parameters
            .base_value(GenerationParameter::ChildScaleRatio)
    }

    pub(crate) fn twist_per_vertex_radians(&self, generation_config: &GenerationConfig) -> f32 {
        self.parameters.evaluated(
            GenerationParameter::ChildTwistPerVertexRadians,
            generation_config,
        )
    }

    pub(crate) fn twist_per_vertex_radians_base(&self) -> f32 {
        self.parameters
            .base_value(GenerationParameter::ChildTwistPerVertexRadians)
    }

    pub(crate) fn vertex_offset_ratio(&self, generation_config: &GenerationConfig) -> f32 {
        self.parameters.evaluated(
            GenerationParameter::ChildOutwardOffsetRatio,
            generation_config,
        )
    }

    pub(crate) fn vertex_offset_ratio_base(&self) -> f32 {
        self.parameters
            .base_value(GenerationParameter::ChildOutwardOffsetRatio)
    }

    pub(crate) fn vertex_spawn_exclusion_probability_base(&self) -> f32 {
        self.parameters
            .base_value(GenerationParameter::ChildSpawnExclusionProbability)
    }

    pub(crate) fn spawn_tuning(&self, generation_config: &GenerationConfig) -> SpawnTuning {
        self.parameters
            .spawn_tuning(generation_config, self.spawn_placement_mode)
    }
}

#[derive(Clone, Resource)]
pub(crate) struct MaterialState {
    pub(crate) opacity: f32,
    pub(crate) hue_step_per_level: f32,
    pub(crate) saturation: f32,
    pub(crate) lightness: f32,
    pub(crate) metallic: f32,
    pub(crate) perceptual_roughness: f32,
    pub(crate) reflectance: f32,
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

#[derive(Clone, Debug)]
pub(crate) struct StageSurfaceState {
    pub(crate) color: [f32; 3],
    pub(crate) translation: [f32; 3],
    pub(crate) rotation_degrees: [f32; 3],
    pub(crate) size: [f32; 2],
    pub(crate) thickness: f32,
    pub(crate) metallic: f32,
    pub(crate) perceptual_roughness: f32,
    pub(crate) reflectance: f32,
}

impl StageSurfaceState {
    fn from_config(surface: &StageSurfaceConfig) -> Self {
        Self {
            color: surface.color,
            translation: surface.translation,
            rotation_degrees: surface.rotation_degrees,
            size: surface.size,
            thickness: surface.thickness,
            metallic: surface.metallic,
            perceptual_roughness: surface.perceptual_roughness,
            reflectance: surface.reflectance,
        }
    }

    fn runtime_stage_surface_config(&self, defaults: &StageSurfaceConfig) -> StageSurfaceConfig {
        let mut surface = defaults.clone();
        surface.color = self.color;
        surface.translation = self.translation;
        surface.rotation_degrees = self.rotation_degrees;
        surface.size = [self.size[0].max(0.01), self.size[1].max(0.01)];
        surface.thickness = self.thickness.max(0.01);
        surface.metallic = self.metallic.clamp(0.0, 1.0);
        surface.perceptual_roughness = self.perceptual_roughness.clamp(0.0, 1.0);
        surface.reflectance = self.reflectance.clamp(0.0, 1.0);
        surface
    }
}

#[derive(Clone, Resource)]
pub(crate) struct StageState {
    pub(crate) enabled: bool,
    pub(crate) floor_enabled: bool,
    pub(crate) backdrop_enabled: bool,
    pub(crate) floor: StageSurfaceState,
    pub(crate) backdrop: StageSurfaceState,
}

impl StageState {
    pub(crate) fn from_config(stage_config: &StageConfig) -> Self {
        Self {
            enabled: stage_config.enabled,
            floor_enabled: stage_config.floor.enabled,
            backdrop_enabled: stage_config.backdrop.enabled,
            floor: StageSurfaceState::from_config(&stage_config.floor),
            backdrop: StageSurfaceState::from_config(&stage_config.backdrop),
        }
    }

    pub(crate) fn runtime_stage_config(&self, defaults: &StageConfig) -> StageConfig {
        let mut stage = defaults.clone();
        stage.enabled = self.enabled;
        stage.floor = self.floor.runtime_stage_surface_config(&defaults.floor);
        stage.backdrop = self.backdrop.runtime_stage_surface_config(&defaults.backdrop);
        stage.floor.enabled = self.floor_enabled;
        stage.backdrop.enabled = self.backdrop_enabled;
        stage
    }
}

#[derive(Clone, Resource)]
pub(crate) struct RenderingState {
    pub(crate) clear_color: [f32; 3],
    pub(crate) ambient_light_color: [f32; 3],
    pub(crate) ambient_light_brightness: f32,
}

impl RenderingState {
    pub(crate) fn from_config(rendering_config: &RenderingConfig) -> Self {
        Self {
            clear_color: rendering_config.clear_color,
            ambient_light_color: rendering_config.ambient_light_color,
            ambient_light_brightness: rendering_config.ambient_light_brightness,
        }
    }

    pub(crate) fn runtime_rendering_config(
        &self,
        defaults: &RenderingConfig,
        stage_state: &StageState,
    ) -> RenderingConfig {
        let mut rendering = defaults.clone();
        rendering.clear_color = self.clear_color;
        rendering.ambient_light_color = self.ambient_light_color;
        rendering.ambient_light_brightness = self.ambient_light_brightness.max(0.0);
        rendering.stage = stage_state.runtime_stage_config(&defaults.stage);
        rendering
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DirectionalLightState {
    pub(crate) color: [f32; 3],
    pub(crate) illuminance: f32,
    pub(crate) translation: [f32; 3],
    pub(crate) look_at: [f32; 3],
}

impl DirectionalLightState {
    fn from_config(config: &crate::config::DirectionalLightConfig) -> Self {
        Self {
            color: config.color,
            illuminance: config.illuminance,
            translation: config.translation,
            look_at: config.look_at,
        }
    }

    fn runtime_directional_light_config(
        &self,
        defaults: &crate::config::DirectionalLightConfig,
    ) -> crate::config::DirectionalLightConfig {
        let mut config = defaults.clone();
        config.color = self.color;
        config.illuminance = self.illuminance.max(0.0);
        config.translation = self.translation;
        config.look_at = self.look_at;
        config
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PointLightState {
    pub(crate) color: [f32; 3],
    pub(crate) intensity: f32,
    pub(crate) range: f32,
    pub(crate) translation: [f32; 3],
}

impl PointLightState {
    fn from_point_config(config: &crate::config::PointLightConfig) -> Self {
        Self {
            color: config.color,
            intensity: config.intensity,
            range: config.range,
            translation: config.translation,
        }
    }

    fn from_accent_config(config: &crate::config::AccentLightConfig) -> Self {
        Self {
            color: config.color,
            intensity: config.intensity,
            range: config.range,
            translation: config.translation,
        }
    }

    fn runtime_point_light_config(
        &self,
        defaults: &crate::config::PointLightConfig,
    ) -> crate::config::PointLightConfig {
        let mut config = defaults.clone();
        config.color = self.color;
        config.intensity = self.intensity.max(0.0);
        config.range = self.range.max(0.0);
        config.translation = self.translation;
        config
    }

    fn runtime_accent_light_config(
        &self,
        defaults: &crate::config::AccentLightConfig,
    ) -> crate::config::AccentLightConfig {
        let mut config = defaults.clone();
        config.color = self.color;
        config.intensity = self.intensity.max(0.0);
        config.range = self.range.max(0.0);
        config.translation = self.translation;
        config
    }
}

#[derive(Clone, Resource)]
pub(crate) struct LightingState {
    pub(crate) directional: DirectionalLightState,
    pub(crate) point: PointLightState,
    pub(crate) accent: PointLightState,
}

impl LightingState {
    pub(crate) fn from_config(lighting_config: &LightingConfig) -> Self {
        Self {
            directional: DirectionalLightState::from_config(&lighting_config.directional),
            point: PointLightState::from_point_config(&lighting_config.point),
            accent: PointLightState::from_accent_config(&lighting_config.accent),
        }
    }

    pub(crate) fn runtime_lighting_config(&self, defaults: &LightingConfig) -> LightingConfig {
        let mut lighting = defaults.clone();
        lighting.directional = self
            .directional
            .runtime_directional_light_config(&defaults.directional);
        lighting.point = self.point.runtime_point_light_config(&defaults.point);
        lighting.accent = self.accent.runtime_accent_light_config(&defaults.accent);
        lighting
    }
}

impl MaterialState {
    pub(crate) fn from_config(material_config: &MaterialConfig) -> Self {
        Self {
            opacity: material_config.default_opacity_clamped(),
            hue_step_per_level: material_config.hue_step_per_level,
            saturation: material_config.saturation,
            lightness: material_config.lightness,
            metallic: material_config.metallic,
            perceptual_roughness: material_config.perceptual_roughness,
            reflectance: material_config.reflectance,
            cube_hue_bias: material_config.cube_hue_bias,
            tetrahedron_hue_bias: material_config.tetrahedron_hue_bias,
            octahedron_hue_bias: material_config.octahedron_hue_bias,
            dodecahedron_hue_bias: material_config.dodecahedron_hue_bias,
            surface_mode: material_config.surface_mode,
            base_surface: material_config.base_surface,
            root_surface: material_config.root_surface,
            accent_surface: material_config.accent_surface,
            accent_every_n_levels: material_config.accent_every_n_levels,
            level_lightness_shift: material_config.level_lightness_shift,
            level_saturation_shift: material_config.level_saturation_shift,
            level_metallic_shift: material_config.level_metallic_shift,
            level_roughness_shift: material_config.level_roughness_shift,
            level_reflectance_shift: material_config.level_reflectance_shift,
        }
    }

    pub(crate) fn runtime_material_config(&self, defaults: &MaterialConfig) -> MaterialConfig {
        let mut config = defaults.clone();
        config.hue_step_per_level = self.hue_step_per_level;
        config.saturation = self.saturation;
        config.lightness = self.lightness;
        config.metallic = self.metallic;
        config.perceptual_roughness = self.perceptual_roughness;
        config.reflectance = self.reflectance;
        config.cube_hue_bias = self.cube_hue_bias;
        config.tetrahedron_hue_bias = self.tetrahedron_hue_bias;
        config.octahedron_hue_bias = self.octahedron_hue_bias;
        config.dodecahedron_hue_bias = self.dodecahedron_hue_bias;
        config.surface_mode = self.surface_mode;
        config.base_surface = self.base_surface;
        config.root_surface = self.root_surface;
        config.accent_surface = self.accent_surface;
        config.accent_every_n_levels = self.accent_every_n_levels;
        config.level_lightness_shift = self.level_lightness_shift;
        config.level_saturation_shift = self.level_saturation_shift;
        config.level_metallic_shift = self.level_metallic_shift;
        config.level_roughness_shift = self.level_roughness_shift;
        config.level_reflectance_shift = self.level_reflectance_shift;
        config
    }
}

#[derive(Component)]
pub(crate) struct ShapeEntity {
    pub(crate) node_index: usize,
}

#[derive(Component)]
pub(crate) struct SceneLightEntity;

#[derive(Component)]
pub(crate) struct SceneDirectionalLight;

#[derive(Component)]
pub(crate) struct ScenePointLight;

#[derive(Component)]
pub(crate) struct SceneAccentLight;

#[derive(Component)]
pub(crate) struct SceneStageEntity;

pub(crate) fn root_generation_node(
    shape_catalog: &ShapeCatalog,
    generation_config: &GenerationConfig,
) -> ShapeNode {
    root_node(
        generation_config.root_shape_kind,
        generation_config.root_scale,
        shape_catalog,
    )
}

pub(crate) fn reset_generation_state(
    generation_state: &mut GenerationState,
    shape_catalog: &ShapeCatalog,
    generation_config: &GenerationConfig,
) -> ShapeNode {
    let root = root_node(
        generation_state.selected_shape_kind,
        generation_config.root_scale,
        shape_catalog,
    );
    generation_state.nodes = vec![root.clone()];
    generation_state.spawn_hold.reset();
    generation_state.parameters.clear_runtime_state();
    root
}
