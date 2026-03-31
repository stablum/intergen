fn cycle_from_all<T>(all: &[T], current: T, direction: isize) -> T
where
    T: Copy + Eq,
{
    let current_index = all
        .iter()
        .position(|candidate| *candidate == current)
        .unwrap_or(0) as isize;
    let next_index = (current_index + direction).rem_euclid(all.len() as isize) as usize;
    all[next_index]
}

fn cycle_shape_kind(current: ShapeKind, direction: isize) -> ShapeKind {
    const ALL: [ShapeKind; 4] = [
        ShapeKind::Cube,
        ShapeKind::Tetrahedron,
        ShapeKind::Octahedron,
        ShapeKind::Dodecahedron,
    ];
    cycle_from_all(&ALL, current, direction)
}

fn cycle_spawn_placement_mode(current: SpawnPlacementMode, direction: isize) -> SpawnPlacementMode {
    const ALL: [SpawnPlacementMode; 3] = [
        SpawnPlacementMode::Vertex,
        SpawnPlacementMode::Edge,
        SpawnPlacementMode::Face,
    ];
    cycle_from_all(&ALL, current, direction)
}

fn cycle_spawn_add_mode(current: SpawnAddMode, direction: isize) -> SpawnAddMode {
    const ALL: [SpawnAddMode; 2] = [SpawnAddMode::Single, SpawnAddMode::FillLevel];
    cycle_from_all(&ALL, current, direction)
}

fn cycle_bool(current: bool, direction: f32) -> bool {
    cycle_from_all(&[false, true], current, direction as isize)
}

fn cycle_material_surface_mode(
    current: MaterialSurfaceMode,
    direction: isize,
) -> MaterialSurfaceMode {
    const ALL: [MaterialSurfaceMode; 2] =
        [MaterialSurfaceMode::Legacy, MaterialSurfaceMode::Procedural];
    cycle_from_all(&ALL, current, direction)
}

fn cycle_material_surface_family(
    current: MaterialSurfaceFamily,
    direction: isize,
) -> MaterialSurfaceFamily {
    const ALL: [MaterialSurfaceFamily; 6] = [
        MaterialSurfaceFamily::Legacy,
        MaterialSurfaceFamily::Matte,
        MaterialSurfaceFamily::Satin,
        MaterialSurfaceFamily::Glossy,
        MaterialSurfaceFamily::Metal,
        MaterialSurfaceFamily::Frosted,
    ];
    cycle_from_all(&ALL, current, direction)
}

fn shape_kind_value_text(kind: ShapeKind) -> &'static str {
    match kind {
        ShapeKind::Cube => "cube",
        ShapeKind::Tetrahedron => "tetrahedron",
        ShapeKind::Octahedron => "octahedron",
        ShapeKind::Dodecahedron => "dodecahedron",
    }
}

fn boolean_value_text(enabled: bool) -> &'static str {
    if enabled { "on" } else { "off" }
}

fn material_surface_mode_value_text(mode: MaterialSurfaceMode) -> &'static str {
    match mode {
        MaterialSurfaceMode::Legacy => "legacy",
        MaterialSurfaceMode::Procedural => "procedural",
    }
}

fn material_surface_family_value_text(family: MaterialSurfaceFamily) -> &'static str {
    match family {
        MaterialSurfaceFamily::Legacy => "legacy",
        MaterialSurfaceFamily::Matte => "matte",
        MaterialSurfaceFamily::Satin => "satin",
        MaterialSurfaceFamily::Glossy => "glossy",
        MaterialSurfaceFamily::Metal => "metal",
        MaterialSurfaceFamily::Frosted => "frosted",
    }
}

fn lfo_parameters() -> impl Iterator<Item = EffectTunerParameter> {
    EffectNumericParameter::all()
        .iter()
        .copied()
        .map(EffectTunerParameter::Effect)
        .chain(
            EffectTunerSceneParameter::lfo_capable()
                .iter()
                .copied()
                .map(EffectTunerParameter::Scene),
        )
}

fn default_lfos() -> Vec<ParameterLfo> {
    lfo_parameters()
        .map(|parameter| match parameter {
            EffectTunerParameter::Effect(parameter) => ParameterLfo::default_for(parameter),
            EffectTunerParameter::Scene(_) => ParameterLfo::new(0.0),
        })
        .collect()
}

fn default_scene_lfo_bases() -> Vec<f32> {
    let camera_config = CameraConfig::default();
    let camera_rig = CameraRig::from_config(&camera_config);
    let generation_config = GenerationConfig::default();
    let generation_state = GenerationState::from_config(&generation_config);
    let rendering_config = RenderingConfig::default();
    let rendering_state = RenderingState::from_config(&rendering_config);
    let lighting_config = LightingConfig::default();
    let lighting_state = LightingState::from_config(&lighting_config);
    let material_config = MaterialConfig::default();
    let material_state = MaterialState::from_config(&material_config);
    let stage_state = StageState::from_config(&rendering_config.stage);
    let context = EffectTunerViewContext {
        camera_config: &camera_config,
        camera_rig: &camera_rig,
        generation_config: &generation_config,
        generation_state: &generation_state,
        rendering_config: &rendering_config,
        rendering_state: &rendering_state,
        lighting_config: &lighting_config,
        lighting_state: &lighting_state,
        material_config: &material_config,
        material_state: &material_state,
        stage_state: &stage_state,
    };
    EffectTunerSceneParameter::lfo_capable()
        .iter()
        .map(|parameter| parameter.value(&context))
        .collect()
}

fn effect_parameter_index(parameter: EffectNumericParameter) -> Option<usize> {
    EffectNumericParameter::all()
        .iter()
        .position(|candidate| *candidate == parameter)
}

fn lfo_index_for_parameter(parameter: EffectTunerParameter) -> Option<usize> {
    match parameter {
        EffectTunerParameter::Effect(parameter) => effect_parameter_index(parameter),
        EffectTunerParameter::Scene(parameter) => parameter
            .lfo_scene_index()
            .map(|index| EffectNumericParameter::all().len() + index),
    }
}

fn lfo_seed_for_index(index: usize) -> u32 {
    index as u32 + 1
}
