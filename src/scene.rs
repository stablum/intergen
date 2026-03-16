use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::math::primitives::Cuboid;
use bevy::prelude::*;

use crate::camera::{CameraRig, SceneCamera};
use crate::config::{
    AppConfig, GenerationConfig, MaterialConfig, MaterialSurfaceFamily, RenderingConfig,
    StageSurfaceConfig,
};
use crate::effects::{camera_effects_from_config, effects_status_messages};
use crate::generation::{
    spawn_add_mode_status_message, spawn_placement_mode_status_message, twist_status_message,
    vertex_exclusion_status_message, vertex_offset_status_message,
};
use crate::help_text::{startup_controls_message, startup_fx_message};
use crate::parameters::{GenerationParameter, HoldRepeatState, ScalarParameterState};
use crate::polyhedra::{
    PolyhedronKind, PolyhedronNode, ShapeCatalog, ShapeGeometry, SpawnAddMode, SpawnPlacementMode,
    SpawnTuning, build_mesh, root_node,
};
use crate::ui::{UiFontSource, load_ui_theme, spawn_help_ui};

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
            cube: ShapeRuntime::new(catalog.geometry(PolyhedronKind::Cube), meshes),
            tetrahedron: ShapeRuntime::new(catalog.geometry(PolyhedronKind::Tetrahedron), meshes),
            octahedron: ShapeRuntime::new(catalog.geometry(PolyhedronKind::Octahedron), meshes),
            dodecahedron: ShapeRuntime::new(catalog.geometry(PolyhedronKind::Dodecahedron), meshes),
            catalog,
        }
    }

    pub(crate) fn mesh(&self, kind: PolyhedronKind) -> &Handle<Mesh> {
        match kind {
            PolyhedronKind::Cube => &self.cube.mesh,
            PolyhedronKind::Tetrahedron => &self.tetrahedron.mesh,
            PolyhedronKind::Octahedron => &self.octahedron.mesh,
            PolyhedronKind::Dodecahedron => &self.dodecahedron.mesh,
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

#[derive(Resource)]
pub(crate) struct GenerationState {
    pub(crate) nodes: Vec<PolyhedronNode>,
    pub(crate) selected_kind: PolyhedronKind,
    pub(crate) spawn_placement_mode: SpawnPlacementMode,
    pub(crate) spawn_add_mode: SpawnAddMode,
    pub(crate) parameters: GenerationParameters,
    pub(crate) spawn_hold: HoldRepeatState,
}

#[cfg_attr(not(test), allow(dead_code))]
impl GenerationState {
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

#[derive(Resource)]
pub(crate) struct MaterialState {
    pub(crate) opacity: f32,
}

#[derive(Component)]
pub(crate) struct PolyhedronEntity {
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

pub(crate) fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    app_config: Res<AppConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_rig: Res<CameraRig>,
) {
    let ui_theme = load_ui_theme(&asset_server, &app_config.ui);
    let shape_assets = ShapeAssets::new(&mut meshes);
    let root = root_generation_node(&shape_assets.catalog, &app_config.generation);
    let initial_opacity = app_config.materials.default_opacity_clamped();

    spawn_scene_lights(&mut commands, &app_config);
    spawn_stage_entities(
        &mut commands,
        &mut meshes,
        &mut materials,
        &app_config.rendering,
    );

    spawn_polyhedron_entity(
        &mut commands,
        &mut materials,
        shape_assets.mesh(root.kind),
        &root,
        &app_config.materials,
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
        selected_kind: app_config.generation.default_child_kind,
        spawn_placement_mode: initial_spawn_placement_mode,
        spawn_add_mode: SpawnAddMode::default(),
        parameters: initial_parameters,
        spawn_hold: HoldRepeatState::default(),
    });
    commands.insert_resource(MaterialState {
        opacity: initial_opacity,
    });

    println!("{}", startup_controls_message());
    println!("{}", startup_fx_message());
    println!(
        "Selected child shape: {:?}, ratio: {:.2}",
        app_config.generation.default_child_kind, initial_scale_ratio
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

pub(crate) fn root_generation_node(
    shape_catalog: &ShapeCatalog,
    generation_config: &GenerationConfig,
) -> PolyhedronNode {
    root_node(
        generation_config.root_kind,
        generation_config.root_scale,
        shape_catalog,
    )
}

pub(crate) fn reset_generation_state(
    generation_state: &mut GenerationState,
    shape_catalog: &ShapeCatalog,
    generation_config: &GenerationConfig,
) -> PolyhedronNode {
    let root = root_node(
        generation_state.selected_kind,
        generation_config.root_scale,
        shape_catalog,
    );
    generation_state.nodes = vec![root.clone()];
    generation_state.spawn_hold.reset();
    generation_state.parameters.clear_runtime_state();
    root
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

pub(crate) fn spawn_polyhedron_entity(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    mesh: &Handle<Mesh>,
    node: &PolyhedronNode,
    material_config: &MaterialConfig,
    opacity: f32,
    node_index: usize,
) {
    let material = materials.add(polyhedron_material(node, material_config, opacity));

    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material),
        PolyhedronEntity { node_index },
        Transform {
            translation: node.center,
            rotation: node.rotation,
            scale: Vec3::splat(node.scale),
        },
        Visibility::Visible,
    ));
}

pub(crate) fn sync_polyhedron_transforms(
    nodes: &[PolyhedronNode],
    polyhedron_transforms: &mut Query<(&PolyhedronEntity, &mut Transform)>,
) {
    for (polyhedron_entity, mut transform) in polyhedron_transforms.iter_mut() {
        let Some(node) = nodes.get(polyhedron_entity.node_index) else {
            continue;
        };

        transform.translation = node.center;
        transform.rotation = node.rotation;
        transform.scale = Vec3::splat(node.scale);
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct MaterialAppearance {
    pub(crate) base_color: [f32; 4],
    pub(crate) metallic: f32,
    pub(crate) perceptual_roughness: f32,
    pub(crate) reflectance: f32,
}

#[derive(Clone, Copy)]
struct SurfaceProperties {
    metallic: f32,
    perceptual_roughness: f32,
    reflectance: f32,
    saturation_bias: f32,
    lightness_bias: f32,
}

pub(crate) fn material_appearance(
    node: &PolyhedronNode,
    material_config: &MaterialConfig,
    opacity: f32,
) -> MaterialAppearance {
    let family = material_config.surface_family(node.level);
    let surface = resolved_surface(material_config, family);
    let level = node.level as f32;
    let hue = (node.level as f32 * material_config.hue_step_per_level
        + material_config.hue_bias(node.kind))
    .rem_euclid(360.0);
    let saturation = (material_config.saturation
        + surface.saturation_bias
        + material_config.level_saturation_shift * level)
        .clamp(0.0, 1.0);
    let lightness = (material_config.lightness
        + surface.lightness_bias
        + material_config.level_lightness_shift * level)
        .clamp(0.0, 1.0);
    let rgb = hsl_to_rgb(hue, saturation, lightness);
    let opacity = opacity.clamp(0.0, 1.0);

    MaterialAppearance {
        base_color: [rgb[0], rgb[1], rgb[2], opacity],
        metallic: (surface.metallic + material_config.level_metallic_shift * level).clamp(0.0, 1.0),
        perceptual_roughness: (surface.perceptual_roughness
            + material_config.level_roughness_shift * level)
            .clamp(0.02, 1.0),
        reflectance: (surface.reflectance + material_config.level_reflectance_shift * level)
            .clamp(0.0, 1.0),
    }
}

fn resolved_surface(
    material_config: &MaterialConfig,
    family: MaterialSurfaceFamily,
) -> SurfaceProperties {
    match family {
        MaterialSurfaceFamily::Legacy => SurfaceProperties {
            metallic: material_config.metallic,
            perceptual_roughness: material_config.perceptual_roughness,
            reflectance: material_config.reflectance,
            saturation_bias: 0.0,
            lightness_bias: 0.0,
        },
        MaterialSurfaceFamily::Matte => SurfaceProperties {
            metallic: 0.0,
            perceptual_roughness: 0.92,
            reflectance: 0.22,
            saturation_bias: -0.06,
            lightness_bias: 0.02,
        },
        MaterialSurfaceFamily::Satin => SurfaceProperties {
            metallic: 0.08,
            perceptual_roughness: 0.5,
            reflectance: 0.32,
            saturation_bias: 0.0,
            lightness_bias: 0.0,
        },
        MaterialSurfaceFamily::Glossy => SurfaceProperties {
            metallic: 0.02,
            perceptual_roughness: 0.18,
            reflectance: 0.56,
            saturation_bias: 0.02,
            lightness_bias: 0.04,
        },
        MaterialSurfaceFamily::Metal => SurfaceProperties {
            metallic: 0.92,
            perceptual_roughness: 0.28,
            reflectance: 0.82,
            saturation_bias: -0.22,
            lightness_bias: -0.08,
        },
        MaterialSurfaceFamily::Frosted => SurfaceProperties {
            metallic: 0.0,
            perceptual_roughness: 0.38,
            reflectance: 0.7,
            saturation_bias: -0.12,
            lightness_bias: 0.06,
        },
    }
}

fn polyhedron_material(
    node: &PolyhedronNode,
    material_config: &MaterialConfig,
    opacity: f32,
) -> StandardMaterial {
    let appearance = material_appearance(node, material_config, opacity);

    StandardMaterial {
        base_color: Color::srgba(
            appearance.base_color[0],
            appearance.base_color[1],
            appearance.base_color[2],
            appearance.base_color[3],
        ),
        alpha_mode: alpha_mode_for_opacity(appearance.base_color[3]),
        metallic: appearance.metallic,
        perceptual_roughness: appearance.perceptual_roughness,
        reflectance: appearance.reflectance,
        ..default()
    }
}

pub(crate) fn hsl_to_rgb(hue_degrees: f32, saturation: f32, lightness: f32) -> [f32; 3] {
    if saturation <= f32::EPSILON {
        return [lightness, lightness, lightness];
    }

    let hue = hue_degrees.rem_euclid(360.0) / 360.0;
    let q = if lightness < 0.5 {
        lightness * (1.0 + saturation)
    } else {
        lightness + saturation - lightness * saturation
    };
    let p = 2.0 * lightness - q;

    [
        hue_to_rgb(p, q, hue + 1.0 / 3.0),
        hue_to_rgb(p, q, hue),
        hue_to_rgb(p, q, hue - 1.0 / 3.0),
    ]
}

fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {
    let mut t = t;
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 0.5 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

pub(crate) fn alpha_mode_for_opacity(opacity: f32) -> AlphaMode {
    if opacity < 0.999 {
        AlphaMode::Blend
    } else {
        AlphaMode::Opaque
    }
}

pub(crate) fn opacity_status_message(opacity: f32) -> String {
    format!(
        "Global object opacity: {:.0}%",
        opacity.clamp(0.0, 1.0) * 100.0
    )
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{AlphaMode, Quat, Vec3};

    use crate::config::GenerationConfig;
    use crate::parameters::{GenerationParameter, HoldRepeatState};
    use crate::polyhedra::{
        AttachmentOccupancy, NodeOrigin, PolyhedronKind, PolyhedronNode, ShapeCatalog,
        SpawnAddMode, SpawnAttachment, SpawnPlacementMode,
    };

    use super::{
        GenerationParameters, GenerationState, alpha_mode_for_opacity, reset_generation_state,
        root_generation_node,
    };

    #[test]
    fn reset_generation_state_restores_root_only() {
        let shape_catalog = ShapeCatalog::new();
        let generation_config = GenerationConfig::default();
        let mut root = root_generation_node(&shape_catalog, &generation_config);
        root.occupied_attachments.vertices[0] = true;

        let child = PolyhedronNode {
            kind: PolyhedronKind::Tetrahedron,
            level: 1,
            center: Vec3::new(2.0, -1.0, 0.5),
            rotation: Quat::IDENTITY,
            scale: 0.4,
            radius: 0.7,
            occupied_attachments: AttachmentOccupancy::default(),
            origin: NodeOrigin::Child {
                parent_index: 0,
                attachment: SpawnAttachment {
                    mode: SpawnPlacementMode::Vertex,
                    index: 0,
                },
            },
        };

        let mut generation_state = GenerationState {
            nodes: vec![root, child],
            selected_kind: PolyhedronKind::Octahedron,
            spawn_placement_mode: SpawnPlacementMode::Face,
            spawn_add_mode: SpawnAddMode::FillLevel,
            parameters: GenerationParameters::from_base_values(0.42, 0.3, 0.6, 0.2),
            spawn_hold: HoldRepeatState {
                elapsed_secs: 1.0,
                repeating: true,
            },
        };
        let twist_spec =
            generation_config.parameter_spec(GenerationParameter::ChildTwistPerVertexRadians);
        generation_state
            .parameter_mut(GenerationParameter::ChildTwistPerVertexRadians)
            .input_mut()
            .request_decrease(
                false,
                true,
                false,
                twist_spec.hold_delay_secs() * 0.5,
                twist_spec,
            );
        generation_state
            .parameter_mut(GenerationParameter::ChildTwistPerVertexRadians)
            .input_mut()
            .request_increase(
                false,
                true,
                false,
                twist_spec.hold_delay_secs() * 0.5,
                twist_spec,
            );
        let offset_spec =
            generation_config.parameter_spec(GenerationParameter::ChildOutwardOffsetRatio);
        generation_state
            .parameter_mut(GenerationParameter::ChildOutwardOffsetRatio)
            .input_mut()
            .request_decrease(
                false,
                true,
                false,
                offset_spec.hold_delay_secs() * 0.5,
                offset_spec,
            );
        generation_state
            .parameter_mut(GenerationParameter::ChildOutwardOffsetRatio)
            .input_mut()
            .request_increase(
                false,
                true,
                false,
                offset_spec.hold_delay_secs() * 0.5,
                offset_spec,
            );
        let exclusion_spec =
            generation_config.parameter_spec(GenerationParameter::ChildSpawnExclusionProbability);
        generation_state
            .parameter_mut(GenerationParameter::ChildSpawnExclusionProbability)
            .input_mut()
            .request_decrease(
                false,
                true,
                false,
                exclusion_spec.hold_delay_secs() * 0.5,
                exclusion_spec,
            );
        generation_state
            .parameter_mut(GenerationParameter::ChildSpawnExclusionProbability)
            .input_mut()
            .request_increase(
                false,
                true,
                false,
                exclusion_spec.hold_delay_secs() * 0.5,
                exclusion_spec,
            );

        let reset_root =
            reset_generation_state(&mut generation_state, &shape_catalog, &generation_config);

        assert_eq!(generation_state.nodes.len(), 1);
        assert_eq!(generation_state.nodes[0].kind, PolyhedronKind::Octahedron);
        assert_eq!(generation_state.nodes[0].level, 0);
        assert_eq!(generation_state.nodes[0].center, Vec3::ZERO);
        assert_eq!(generation_state.selected_kind, PolyhedronKind::Octahedron);
        assert_eq!(generation_state.scale_ratio_base(), 0.42);
        assert_eq!(generation_state.twist_per_vertex_radians_base(), 0.3);
        assert_eq!(generation_state.vertex_offset_ratio_base(), 0.6);
        assert_eq!(
            generation_state.vertex_spawn_exclusion_probability_base(),
            0.2
        );
        assert_eq!(
            generation_state.spawn_placement_mode,
            SpawnPlacementMode::Face
        );
        assert_eq!(generation_state.spawn_add_mode, SpawnAddMode::FillLevel);
        assert!(
            generation_state.nodes[0]
                .occupied_attachments
                .vertices
                .iter()
                .all(|occupied| !occupied)
        );
        assert!(
            generation_state.nodes[0]
                .occupied_attachments
                .edges
                .iter()
                .all(|occupied| !occupied)
        );
        assert!(
            generation_state.nodes[0]
                .occupied_attachments
                .faces
                .iter()
                .all(|occupied| !occupied)
        );
        assert_eq!(reset_root.center, Vec3::ZERO);
        assert_eq!(reset_root.kind, PolyhedronKind::Octahedron);
        assert_eq!(generation_state.spawn_hold.elapsed_secs, 0.0);
        assert!(!generation_state.spawn_hold.repeating);
        let twist_input = generation_state
            .parameter(GenerationParameter::ChildTwistPerVertexRadians)
            .input();
        assert_eq!(twist_input.decrease_hold().elapsed_secs, 0.0);
        assert!(!twist_input.decrease_hold().repeating);
        assert_eq!(twist_input.increase_hold().elapsed_secs, 0.0);
        assert!(!twist_input.increase_hold().repeating);
        let offset_input = generation_state
            .parameter(GenerationParameter::ChildOutwardOffsetRatio)
            .input();
        assert_eq!(offset_input.decrease_hold().elapsed_secs, 0.0);
        assert!(!offset_input.decrease_hold().repeating);
        assert_eq!(offset_input.increase_hold().elapsed_secs, 0.0);
        assert!(!offset_input.increase_hold().repeating);
        let exclusion_input = generation_state
            .parameter(GenerationParameter::ChildSpawnExclusionProbability)
            .input();
        assert_eq!(exclusion_input.decrease_hold().elapsed_secs, 0.0);
        assert!(!exclusion_input.decrease_hold().repeating);
        assert_eq!(exclusion_input.increase_hold().elapsed_secs, 0.0);
        assert!(!exclusion_input.increase_hold().repeating);
    }

    #[test]
    fn transparent_materials_use_blend_mode() {
        assert!(matches!(alpha_mode_for_opacity(0.6), AlphaMode::Blend));
        assert!(matches!(alpha_mode_for_opacity(1.0), AlphaMode::Opaque));
    }
}
