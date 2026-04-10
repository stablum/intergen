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
    let rendering_state = RenderingState::from_config(&app_config.rendering);
    let material_state = MaterialState::from_config(&app_config.materials);
    let lighting_state = LightingState::from_config(&app_config.lighting);
    effect_tuner.sync_scene_lfo_bases(&crate::effect_tuner::EffectTunerViewContext {
        camera_config: &app_config.camera,
        camera_rig: &camera_rig,
        generation_config: &app_config.generation,
        generation_state: &GenerationState {
            nodes: vec![root.clone()],
            selected_shape_kind: app_config.generation.default_child_shape_kind,
            spawn_placement_mode: app_config.generation.default_spawn_placement_mode,
            spawn_add_mode: SpawnAddMode::default(),
            single_attachment_repeat_count: app_config
                .generation
                .default_single_attachment_repeat_count,
            single_spawn_source_cursor: None,
            parameters: GenerationParameters::from_config(&app_config.generation),
            spawn_hold: HoldRepeatState::default(),
        },
        rendering_config: &app_config.rendering,
        rendering_state: &rendering_state,
        lighting_config: &app_config.lighting,
        lighting_state: &lighting_state,
        material_config: &app_config.materials,
        material_state: &material_state,
        stage_state: &stage_state,
    });
    let runtime_material_config = material_state.runtime_material_config(&app_config.materials);
    let runtime_rendering =
        rendering_state.runtime_rendering_config(&app_config.rendering, &stage_state);
    let runtime_lighting = lighting_state.runtime_lighting_config(&app_config.lighting);
    let initial_opacity = material_state.opacity;

    spawn_scene_lights(&mut commands, &runtime_lighting);
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
        single_attachment_repeat_count: app_config.generation.default_single_attachment_repeat_count,
        single_spawn_source_cursor: None,
        parameters: initial_parameters,
        spawn_hold: HoldRepeatState::default(),
    });
    commands.insert_resource(material_state);
    commands.insert_resource(stage_state);
    commands.insert_resource(rendering_state);
    commands.insert_resource(lighting_state);

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
    println!(
        "{}",
        crate::generation::single_attachment_repeat_count_status_message(
            app_config.generation.default_single_attachment_repeat_count
        )
    );
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

pub(crate) fn spawn_scene_lights(commands: &mut Commands, lighting: &LightingConfig) {
    commands.spawn((
        SceneLightEntity,
        SceneDirectionalLight,
        DirectionalLight {
            color: lighting.directional.color(),
            illuminance: lighting.directional.illuminance,
            shadows_enabled: lighting.directional.shadows_enabled,
            ..default()
        },
        Transform::from_translation(lighting.directional.translation())
            .looking_at(lighting.directional.look_at(), Vec3::Y),
    ));

    commands.spawn((
        SceneLightEntity,
        ScenePointLight,
        PointLight {
            color: lighting.point.color(),
            intensity: lighting.point.intensity,
            range: lighting.point.range,
            shadows_enabled: lighting.point.shadows_enabled,
            ..default()
        },
        Transform::from_translation(lighting.point.translation()),
    ));

    if lighting.accent.enabled {
        commands.spawn((
            SceneLightEntity,
            SceneAccentLight,
            PointLight {
                color: lighting.accent.color(),
                intensity: lighting.accent.intensity,
                range: lighting.accent.range,
                shadows_enabled: lighting.accent.shadows_enabled,
                ..default()
            },
            Transform::from_translation(lighting.accent.translation()),
        ));
    }
}

pub(crate) fn apply_live_rendering_state(
    rendering: &RenderingConfig,
    clear_color: &mut ClearColor,
    ambient_light: &mut GlobalAmbientLight,
) {
    clear_color.0 = rendering.clear_color();
    ambient_light.color = rendering.ambient_light_color();
    ambient_light.brightness = rendering.ambient_light_brightness;
}

pub(crate) fn apply_live_lighting_state<
    DirectionalFilter: bevy::ecs::query::QueryFilter,
    PointFilter: bevy::ecs::query::QueryFilter,
    AccentFilter: bevy::ecs::query::QueryFilter,
>(
    lighting: &LightingConfig,
    directional_lights: &mut Query<
        (&mut DirectionalLight, &mut Transform),
        DirectionalFilter,
    >,
    point_lights: &mut Query<(&mut PointLight, &mut Transform), PointFilter>,
    accent_lights: &mut Query<(&mut PointLight, &mut Transform), AccentFilter>,
) {
    if let Ok((mut light, mut transform)) = directional_lights.single_mut() {
        light.color = lighting.directional.color();
        light.illuminance = lighting.directional.illuminance;
        light.shadows_enabled = lighting.directional.shadows_enabled;
        *transform = Transform::from_translation(lighting.directional.translation())
            .looking_at(lighting.directional.look_at(), Vec3::Y);
    }

    if let Ok((mut light, mut transform)) = point_lights.single_mut() {
        light.color = lighting.point.color();
        light.intensity = lighting.point.intensity;
        light.range = lighting.point.range;
        light.shadows_enabled = lighting.point.shadows_enabled;
        *transform = Transform::from_translation(lighting.point.translation());
    }

    if let Ok((mut light, mut transform)) = accent_lights.single_mut() {
        light.color = lighting.accent.color();
        light.intensity = lighting.accent.intensity;
        light.range = lighting.accent.range;
        light.shadows_enabled = lighting.accent.shadows_enabled;
        *transform = Transform::from_translation(lighting.accent.translation());
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
    stage_entities: &Query<Entity, With<SceneStageEntity>>,
) {
    for entity in stage_entities.iter() {
        commands.entity(entity).despawn();
    }

    spawn_stage_entities(commands, meshes, materials, rendering);
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
            scale: node.combined_scale(),
        },
        Visibility::Visible,
    ));
}

pub(crate) fn sync_shape_transforms<F: bevy::ecs::query::QueryFilter>(
    nodes: &[ShapeNode],
    shape_transforms: &mut Query<(&ShapeEntity, &mut Transform), F>,
) {
    for (shape_entity, mut transform) in shape_transforms.iter_mut() {
        let Some(node) = nodes.get(shape_entity.node_index) else {
            continue;
        };

        transform.translation = node.center;
        transform.rotation = node.rotation;
        transform.scale = node.combined_scale();
    }
}
