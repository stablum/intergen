pub(crate) fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    app_config: Res<AppConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_rig: Res<CameraRig>,
    mut effect_tuner: ResMut<crate::effect_tuner::EffectTunerState>,
) {
    let ui_theme = load_ui_theme(&asset_server, &app_config.ui);
    let shape_assets = ShapeAssets::new(&mut meshes);
    let root = root_generation_node(&shape_assets.catalog, &app_config.generation);
    let stage_state = StageState::from_config(&app_config.rendering.stage);
    let material_state = MaterialState::from_config(&app_config.materials);
    effect_tuner.sync_material_scene_lfo_bases(&material_state);
    let runtime_material_config = material_state.runtime_material_config(&app_config.materials);
    let initial_opacity = material_state.opacity;

    spawn_scene_lights(&mut commands, &app_config);
    let mut runtime_rendering = app_config.rendering.clone();
    runtime_rendering.stage = stage_state.runtime_stage_config(&app_config.rendering.stage);
    spawn_stage_entities(
        &mut commands,
        &mut meshes,
        &mut materials,
        &runtime_rendering,
    );

    spawn_shape_entity(
        &mut commands,
        &mut materials,
        shape_assets.mesh(root.kind),
        &root,
        &runtime_material_config,
        initial_opacity,
        0,
    );

    let camera_translation = camera_rig.orientation * Vec3::new(0.0, 0.0, camera_rig.distance);
    let scene_camera = commands
        .spawn((
            Camera3d::default(),
            Tonemapping::AcesFitted,
            Transform::from_translation(camera_translation)
                .looking_at(Vec3::ZERO, camera_rig.orientation * Vec3::Y),
            SceneCamera,
            IsDefaultUiCamera,
            camera_effects_from_config(&app_config.effects),
        ))
        .id();

    spawn_help_ui(&mut commands, &ui_theme, scene_camera, &app_config.ui);

    let initial_parameters = GenerationParameters::from_config(&app_config.generation);
    let initial_scale_ratio = initial_parameters.base_value(GenerationParameter::ChildScaleRatio);
    let initial_spawn_placement_mode = app_config.generation.default_spawn_placement_mode;
    let initial_twist =
        initial_parameters.base_value(GenerationParameter::ChildTwistPerVertexRadians);
    let initial_vertex_offset =
        initial_parameters.base_value(GenerationParameter::ChildOutwardOffsetRatio);
    let initial_vertex_spawn_exclusion =
        initial_parameters.base_value(GenerationParameter::ChildSpawnExclusionProbability);
    commands.insert_resource(ui_theme.clone());
    commands.insert_resource(shape_assets);
    commands.insert_resource(GenerationState {
        nodes: vec![root],
        selected_shape_kind: app_config.generation.default_child_shape_kind,
        spawn_placement_mode: initial_spawn_placement_mode,
        spawn_add_mode: SpawnAddMode::default(),
        parameters: initial_parameters,
        spawn_hold: HoldRepeatState::default(),
    });
    commands.insert_resource(material_state);
    commands.insert_resource(stage_state);

    println!("{}", startup_controls_message());
    println!("{}", startup_fx_message());
    println!(
        "Selected child shape: {:?}, ratio: {:.2}",
        app_config.generation.default_child_shape_kind, initial_scale_ratio
    );
    println!(
        "{}",
        spawn_placement_mode_status_message(initial_spawn_placement_mode)
    );
    println!("{}", spawn_add_mode_status_message(SpawnAddMode::default()));
    println!("{}", twist_status_message(initial_twist));
    println!("{}", vertex_offset_status_message(initial_vertex_offset));
    println!(
        "{}",
        vertex_exclusion_status_message(initial_vertex_spawn_exclusion)
    );
    println!("{}", opacity_status_message(initial_opacity));
    for message in effects_status_messages(&app_config.effects) {
        println!("{message}");
    }
    if ui_theme.source == UiFontSource::Fallback {
        eprintln!(
            "Carbon Plus was not found in assets/fonts. Using Bevy's fallback font for UI text."
        );
    }
}

pub(crate) fn spawn_scene_lights(commands: &mut Commands, app_config: &AppConfig) {
    commands.spawn((
        SceneLightEntity,
        SceneDirectionalLight,
        DirectionalLight {
            color: app_config.lighting.directional.color(),
            illuminance: app_config.lighting.directional.illuminance,
            shadows_enabled: app_config.lighting.directional.shadows_enabled,
            ..default()
        },
        Transform::from_translation(app_config.lighting.directional.translation())
            .looking_at(app_config.lighting.directional.look_at(), Vec3::Y),
    ));

    commands.spawn((
        SceneLightEntity,
        ScenePointLight,
        PointLight {
            color: app_config.lighting.point.color(),
            intensity: app_config.lighting.point.intensity,
            range: app_config.lighting.point.range,
            shadows_enabled: app_config.lighting.point.shadows_enabled,
            ..default()
        },
        Transform::from_translation(app_config.lighting.point.translation()),
    ));

    if app_config.lighting.accent.enabled {
        commands.spawn((
            SceneLightEntity,
            SceneAccentLight,
            PointLight {
                color: app_config.lighting.accent.color(),
                intensity: app_config.lighting.accent.intensity,
                range: app_config.lighting.accent.range,
                shadows_enabled: app_config.lighting.accent.shadows_enabled,
                ..default()
            },
            Transform::from_translation(app_config.lighting.accent.translation()),
        ));
    }
}

pub(crate) fn spawn_stage_entities(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    rendering: &RenderingConfig,
) {
    if !rendering.stage.enabled {
        return;
    }

    if rendering.stage.floor.enabled {
        spawn_stage_surface(
            commands,
            meshes,
            materials,
            &rendering.stage.floor,
            StageSurfaceOrientation::Horizontal,
        );
    }

    if rendering.stage.backdrop.enabled {
        spawn_stage_surface(
            commands,
            meshes,
            materials,
            &rendering.stage.backdrop,
            StageSurfaceOrientation::Vertical,
        );
    }
}

pub(crate) fn sync_stage_entities(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    rendering: &RenderingConfig,
    stage_state: &StageState,
    stage_entities: &Query<Entity, With<SceneStageEntity>>,
) {
    for entity in stage_entities.iter() {
        commands.entity(entity).despawn();
    }

    let mut runtime_rendering = rendering.clone();
    runtime_rendering.stage = stage_state.runtime_stage_config(&rendering.stage);
    spawn_stage_entities(commands, meshes, materials, &runtime_rendering);
}

fn spawn_stage_surface(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    surface: &StageSurfaceConfig,
    orientation: StageSurfaceOrientation,
) {
    let size = surface.size();
    let mesh = match orientation {
        StageSurfaceOrientation::Horizontal => {
            Mesh::from(Cuboid::new(size.x, surface.thickness(), size.y))
        }
        StageSurfaceOrientation::Vertical => {
            Mesh::from(Cuboid::new(size.x, size.y, surface.thickness()))
        }
    };
    let material = materials.add(stage_surface_material(surface));

    commands.spawn((
        SceneStageEntity,
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        Transform {
            translation: surface.translation(),
            rotation: surface.rotation(),
            ..default()
        },
        Visibility::Visible,
    ));
}

fn stage_surface_material(surface: &StageSurfaceConfig) -> StandardMaterial {
    StandardMaterial {
        base_color: surface.color(),
        metallic: surface.metallic.clamp(0.0, 1.0),
        perceptual_roughness: surface.perceptual_roughness.clamp(0.0, 1.0),
        reflectance: surface.reflectance.clamp(0.0, 1.0),
        ..default()
    }
}

#[derive(Clone, Copy)]
enum StageSurfaceOrientation {
    Horizontal,
    Vertical,
}

pub(crate) fn spawn_shape_entity(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    mesh: &Handle<Mesh>,
    node: &ShapeNode,
    material_config: &MaterialConfig,
    opacity: f32,
    node_index: usize,
) {
    let material = materials.add(shape_material(node, material_config, opacity));

    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material),
        ShapeEntity { node_index },
        Transform {
            translation: node.center,
            rotation: node.rotation,
            scale: Vec3::splat(node.scale),
        },
        Visibility::Visible,
    ));
}

pub(crate) fn sync_shape_transforms(
    nodes: &[ShapeNode],
    shape_transforms: &mut Query<(&ShapeEntity, &mut Transform)>,
) {
    for (shape_entity, mut transform) in shape_transforms.iter_mut() {
        let Some(node) = nodes.get(shape_entity.node_index) else {
            continue;
        };

        transform.translation = node.center;
        transform.rotation = node.rotation;
        transform.scale = Vec3::splat(node.scale);
    }
}
