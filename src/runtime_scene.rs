use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::camera::CameraRig;
use crate::config::AppConfig;
use crate::effect_tuner::EffectTunerState;
use crate::scene::{
    GenerationState, MaterialState, PolyhedronEntity, SceneLightEntity, SceneStageEntity,
    ShapeAssets,
};

#[derive(SystemParam)]
pub(crate) struct SceneSnapshotAccess<'w, 's> {
    pub(crate) app_config: Res<'w, AppConfig>,
    pub(crate) camera_rig: Res<'w, CameraRig>,
    pub(crate) effect_tuner: Res<'w, EffectTunerState>,
    pub(crate) shape_assets: Res<'w, ShapeAssets>,
    pub(crate) generation_state: Res<'w, GenerationState>,
    pub(crate) material_state: Res<'w, MaterialState>,
    _marker: std::marker::PhantomData<&'s ()>,
}

#[derive(SystemParam)]
pub(crate) struct SceneMutationAccess<'w, 's> {
    pub(crate) commands: Commands<'w, 's>,
    pub(crate) app_config: ResMut<'w, AppConfig>,
    pub(crate) clear_color: ResMut<'w, ClearColor>,
    pub(crate) ambient_light: ResMut<'w, AmbientLight>,
    pub(crate) camera_rig: ResMut<'w, CameraRig>,
    pub(crate) effect_tuner: ResMut<'w, EffectTunerState>,
    pub(crate) shape_assets: Res<'w, ShapeAssets>,
    pub(crate) generation_state: ResMut<'w, GenerationState>,
    pub(crate) material_state: ResMut<'w, MaterialState>,
    pub(crate) meshes: ResMut<'w, Assets<Mesh>>,
    pub(crate) materials: ResMut<'w, Assets<StandardMaterial>>,
    pub(crate) polyhedron_entities: Query<'w, 's, Entity, With<PolyhedronEntity>>,
    pub(crate) light_entities: Query<'w, 's, Entity, With<SceneLightEntity>>,
    pub(crate) stage_entities: Query<'w, 's, Entity, With<SceneStageEntity>>,
}

#[derive(SystemParam)]
pub(crate) struct GenerationSceneAccess<'w, 's> {
    pub(crate) commands: Commands<'w, 's>,
    pub(crate) app_config: Res<'w, AppConfig>,
    pub(crate) shape_assets: Res<'w, ShapeAssets>,
    pub(crate) generation_state: ResMut<'w, GenerationState>,
    pub(crate) material_state: ResMut<'w, MaterialState>,
    pub(crate) materials: ResMut<'w, Assets<StandardMaterial>>,
    pub(crate) polyhedron_entities: Query<'w, 's, Entity, With<PolyhedronEntity>>,
    pub(crate) polyhedron_materials: Query<
        'w,
        's,
        (
            &'static PolyhedronEntity,
            &'static MeshMaterial3d<StandardMaterial>,
        ),
    >,
    pub(crate) polyhedron_transforms:
        Query<'w, 's, (&'static PolyhedronEntity, &'static mut Transform)>,
}
