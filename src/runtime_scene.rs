use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::camera::{CameraRig, SceneCamera};
use crate::config::AppConfig;
use crate::effect_tuner::EffectTunerState;
use crate::scene::{
    GenerationState, LightingState, MaterialState, RenderingState, SceneAccentLight,
    SceneDirectionalLight, SceneLightEntity, ScenePointLight, SceneStageEntity, ShapeAssets,
    ShapeEntity, StageState,
};

#[derive(SystemParam)]
pub(crate) struct SceneSnapshotAccess<'w, 's> {
    pub(crate) app_config: Res<'w, AppConfig>,
    pub(crate) camera_rig: Res<'w, CameraRig>,
    pub(crate) effect_tuner: Res<'w, EffectTunerState>,
    pub(crate) shape_assets: Res<'w, ShapeAssets>,
    pub(crate) generation_state: Res<'w, GenerationState>,
    pub(crate) material_state: Res<'w, MaterialState>,
    pub(crate) stage_state: Res<'w, StageState>,
    pub(crate) rendering_state: Res<'w, RenderingState>,
    pub(crate) lighting_state: Res<'w, LightingState>,
    _marker: std::marker::PhantomData<&'s ()>,
}

#[derive(SystemParam)]
pub(crate) struct SceneMutationAccess<'w, 's> {
    pub(crate) commands: Commands<'w, 's>,
    pub(crate) app_config: ResMut<'w, AppConfig>,
    pub(crate) clear_color: ResMut<'w, ClearColor>,
    pub(crate) ambient_light: ResMut<'w, GlobalAmbientLight>,
    pub(crate) camera_rig: ResMut<'w, CameraRig>,
    pub(crate) effect_tuner: ResMut<'w, EffectTunerState>,
    pub(crate) shape_assets: Res<'w, ShapeAssets>,
    pub(crate) generation_state: ResMut<'w, GenerationState>,
    pub(crate) material_state: ResMut<'w, MaterialState>,
    pub(crate) stage_state: ResMut<'w, StageState>,
    pub(crate) rendering_state: ResMut<'w, RenderingState>,
    pub(crate) lighting_state: ResMut<'w, LightingState>,
    pub(crate) meshes: ResMut<'w, Assets<Mesh>>,
    pub(crate) materials: ResMut<'w, Assets<StandardMaterial>>,
    pub(crate) shape_entities: Query<'w, 's, Entity, With<ShapeEntity>>,
    pub(crate) light_entities: Query<'w, 's, Entity, With<SceneLightEntity>>,
    pub(crate) stage_entities: Query<'w, 's, Entity, With<SceneStageEntity>>,
    pub(crate) camera_transforms: Query<'w, 's, &'static mut Transform, With<SceneCamera>>,
}

#[derive(SystemParam)]
pub(crate) struct GenerationSceneAccess<'w, 's> {
    pub(crate) commands: Commands<'w, 's>,
    pub(crate) app_config: Res<'w, AppConfig>,
    pub(crate) clear_color: ResMut<'w, ClearColor>,
    pub(crate) ambient_light: ResMut<'w, GlobalAmbientLight>,
    pub(crate) camera_rig: ResMut<'w, CameraRig>,
    pub(crate) shape_assets: Res<'w, ShapeAssets>,
    pub(crate) generation_state: ResMut<'w, GenerationState>,
    pub(crate) material_state: ResMut<'w, MaterialState>,
    pub(crate) stage_state: ResMut<'w, StageState>,
    pub(crate) rendering_state: ResMut<'w, RenderingState>,
    pub(crate) lighting_state: ResMut<'w, LightingState>,
    pub(crate) meshes: ResMut<'w, Assets<Mesh>>,
    pub(crate) materials: ResMut<'w, Assets<StandardMaterial>>,
    pub(crate) shape_entities: Query<'w, 's, Entity, With<ShapeEntity>>,
    pub(crate) stage_entities: Query<'w, 's, Entity, With<SceneStageEntity>>,
    pub(crate) camera_transforms: Query<
        'w,
        's,
        &'static mut Transform,
        (
            With<SceneCamera>,
            Without<SceneDirectionalLight>,
            Without<ScenePointLight>,
            Without<SceneAccentLight>,
            Without<ShapeEntity>,
        ),
    >,
    pub(crate) directional_lights: Query<
        'w,
        's,
        (&'static mut DirectionalLight, &'static mut Transform),
        (
            With<SceneDirectionalLight>,
            Without<SceneCamera>,
            Without<ScenePointLight>,
            Without<SceneAccentLight>,
            Without<ShapeEntity>,
        ),
    >,
    pub(crate) point_lights: Query<
        'w,
        's,
        (&'static mut PointLight, &'static mut Transform),
        (
            With<ScenePointLight>,
            Without<SceneCamera>,
            Without<SceneDirectionalLight>,
            Without<SceneAccentLight>,
            Without<ShapeEntity>,
        ),
    >,
    pub(crate) accent_lights: Query<
        'w,
        's,
        (&'static mut PointLight, &'static mut Transform),
        (
            With<SceneAccentLight>,
            Without<SceneCamera>,
            Without<SceneDirectionalLight>,
            Without<ScenePointLight>,
            Without<ShapeEntity>,
        ),
    >,
    pub(crate) shape_materials: Query<
        'w,
        's,
        (
            &'static ShapeEntity,
            &'static MeshMaterial3d<StandardMaterial>,
        ),
    >,
    pub(crate) shape_transforms: Query<
        'w,
        's,
        (&'static ShapeEntity, &'static mut Transform),
        (
            Without<SceneCamera>,
            Without<SceneDirectionalLight>,
            Without<ScenePointLight>,
            Without<SceneAccentLight>,
        ),
    >,
}
