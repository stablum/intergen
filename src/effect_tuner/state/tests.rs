use crate::config::{
    CameraConfig, EffectGroup, EffectsConfig, GenerationConfig, LightingConfig, MaterialConfig,
    MaterialSurfaceMode, RenderingConfig, StageConfig,
};
use crate::effect_tuner::lfo::{DEFAULT_LFO_FREQUENCY_HZ, LfoShape};
use crate::effect_tuner::metadata::{EffectEditMode, EffectOverlayField};
use crate::{camera::CameraRig, scene::{GenerationState, LightingState, MaterialState, RenderingState, StageState}};

use super::{
    EffectNumericParameter, EffectRuntimeSnapshot, EffectTunerEditContext, EffectTunerPageMode,
    EffectTunerParameter, EffectTunerSceneParameter, EffectTunerState, EffectTunerViewContext,
    HoldInput,
    lfo_index_for_parameter,
};

fn default_scene_state() -> (
    GenerationConfig,
    GenerationState,
    MaterialConfig,
    MaterialState,
    StageConfig,
    StageState,
) {
    let generation_config = GenerationConfig::default();
    let generation_state = GenerationState::from_config(&generation_config);
    let material_config = MaterialConfig::default();
    let material_state = MaterialState::from_config(&material_config);
    let stage_config = StageConfig::default();
    let stage_state = StageState::from_config(&stage_config);
    (
        generation_config,
        generation_state,
        material_config,
        material_state,
        stage_config,
        stage_state,
    )
}

fn view_context<'a>(
    generation_config: &'a GenerationConfig,
    generation_state: &'a GenerationState,
    material_config: &'a MaterialConfig,
    material_state: &'a MaterialState,
    _stage_config: &'a StageConfig,
    stage_state: &'a StageState,
) -> EffectTunerViewContext<'a> {
    let camera_config: &'a CameraConfig = Box::leak(Box::new(CameraConfig::default()));
    let camera_rig: &'a CameraRig = Box::leak(Box::new(CameraRig::from_config(camera_config)));
    let rendering_config: &'a RenderingConfig = Box::leak(Box::new(RenderingConfig::default()));
    let rendering_state: &'a RenderingState =
        Box::leak(Box::new(RenderingState::from_config(rendering_config)));
    let lighting_config: &'a LightingConfig = Box::leak(Box::new(LightingConfig::default()));
    let lighting_state: &'a LightingState =
        Box::leak(Box::new(LightingState::from_config(lighting_config)));
    EffectTunerViewContext {
        camera_config,
        camera_rig,
        generation_config,
        generation_state,
        rendering_config,
        rendering_state,
        lighting_config,
        lighting_state,
        material_config,
        material_state,
        stage_state,
    }
}

fn edit_context<'a>(
    generation_config: &'a GenerationConfig,
    generation_state: &'a mut GenerationState,
    material_config: &'a MaterialConfig,
    material_state: &'a mut MaterialState,
    _stage_config: &'a StageConfig,
    stage_state: &'a mut StageState,
) -> EffectTunerEditContext<'a> {
    let camera_config: &'a CameraConfig = Box::leak(Box::new(CameraConfig::default()));
    let camera_rig: &'a mut CameraRig = Box::leak(Box::new(CameraRig::from_config(camera_config)));
    let rendering_config: &'a RenderingConfig = Box::leak(Box::new(RenderingConfig::default()));
    let rendering_state: &'a mut RenderingState =
        Box::leak(Box::new(RenderingState::from_config(rendering_config)));
    let lighting_config: &'a LightingConfig = Box::leak(Box::new(LightingConfig::default()));
    let lighting_state: &'a mut LightingState =
        Box::leak(Box::new(LightingState::from_config(lighting_config)));
    EffectTunerEditContext {
        camera_config,
        camera_rig,
        generation_config,
        generation_state,
        rendering_config,
        rendering_state,
        lighting_config,
        lighting_state,
        material_config,
        material_state,
        stage_state,
    }
}

fn select_parameter(effect_tuner: &mut EffectTunerState, parameter: EffectTunerParameter) {
    effect_tuner.selected_index = EffectTunerParameter::all()
        .iter()
        .position(|candidate| *candidate == parameter)
        .expect("parameter should exist in the F2 parameter list");
}

#[test]
fn selected_effect_matches_parameter_group() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    effect_tuner.selected_index = 17;

    assert_eq!(
        effect_tuner.selected_effect_group(),
        Some(EffectGroup::Bloom)
    );
}

#[test]
fn toggle_selected_lfo_updates_enabled_state() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        generation_state,
        material_config,
        material_state,
        stage_config,
        stage_state,
    ) = default_scene_state();
    effect_tuner.selected_index = 13;

    let enabled = effect_tuner.toggle_selected_lfo(
        &view_context(
            &generation_config,
            &generation_state,
            &material_config,
            &material_state,
            &stage_config,
            &stage_state,
        ),
        1.0,
    );

    assert_eq!(enabled, Some(true));
    assert!(effect_tuner.selected_lfo().enabled);
}

#[test]
fn evaluated_effects_apply_lfo_offset() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    effect_tuner.selected_index = 0;
    effect_tuner.current.color_wavefolder.gain = 2.0;
    let lfo = effect_tuner.selected_lfo_mut();
    lfo.enabled = true;
    lfo.shape = LfoShape::Sine;
    lfo.amplitude = 0.5;
    lfo.frequency_hz = 1.0;

    let evaluated = effect_tuner.evaluated_effects(0.25);

    assert!((evaluated.color_wavefolder.gain - 2.5).abs() < 1e-6);
}

#[test]
fn overlay_snapshot_uses_compact_labels_and_active_field() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        generation_state,
        material_config,
        material_state,
        stage_config,
        stage_state,
    ) = default_scene_state();
    effect_tuner.selected_index = 3;
    effect_tuner.edit_mode = EffectEditMode::LfoShape;
    effect_tuner.pinned = true;

    let snapshot = effect_tuner.overlay_snapshot(
        &view_context(
            &generation_config,
            &generation_state,
            &material_config,
            &material_state,
            &stage_config,
            &stage_state,
        ),
        0.0,
    );

    assert_eq!(snapshot.effect_label, "lens");
    assert_eq!(snapshot.parameter_label, "k2");
    assert_eq!(snapshot.active_field, EffectOverlayField::LfoShape);
    assert!(snapshot.pinned);
}

#[test]
fn open_and_close_page_manage_page_modes() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());

    effect_tuner.open_page(1.0);
    assert_eq!(effect_tuner.page_mode(), EffectTunerPageMode::Compact);

    effect_tuner.show_list_page(1.1);
    assert_eq!(effect_tuner.page_mode(), EffectTunerPageMode::List);

    effect_tuner.close_page();
    assert_eq!(effect_tuner.page_mode(), EffectTunerPageMode::Compact);
}

#[test]
fn scroll_selection_moves_without_hold_input() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let initial = effect_tuner.selected_parameter();

    assert!(effect_tuner.scroll_selection(1, 1.0));
    assert_ne!(effect_tuner.selected_parameter(), initial);
}

#[test]
fn list_overlay_snapshot_scrolls_to_keep_selection_visible() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        generation_state,
        material_config,
        material_state,
        stage_config,
        stage_state,
    ) = default_scene_state();
    effect_tuner.selected_index = EffectTunerParameter::all().len() - 1;
    effect_tuner.show_list_page(1.0);

    let snapshot = effect_tuner.list_overlay_snapshot(
        &view_context(
            &generation_config,
            &generation_state,
            &material_config,
            &material_state,
            &stage_config,
            &stage_state,
        ),
        1.0,
        7,
    );

    assert_eq!(snapshot.total_parameters, EffectTunerParameter::all().len());
    assert_eq!(snapshot.rows.len(), 7);
    assert_eq!(
        snapshot.window_start + snapshot.rows.len(),
        snapshot.total_parameters
    );
    assert!(snapshot.rows.last().is_some_and(|row| row.selected));
    assert_eq!(
        snapshot.detail.parameter_label,
        effect_tuner.selected_parameter().short_label()
    );
}

#[test]
fn reset_selected_restores_lfo_frequency_default() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        mut generation_state,
        material_config,
        mut material_state,
        stage_config,
        mut stage_state,
    ) = default_scene_state();
    effect_tuner.selected_index = 16;
    effect_tuner.edit_mode = EffectEditMode::LfoFrequency;
    effect_tuner.selected_lfo_mut().frequency_hz = 3.0;

    effect_tuner.reset_selected(
        &mut edit_context(
            &generation_config,
            &mut generation_state,
            &material_config,
            &mut material_state,
            &stage_config,
            &mut stage_state,
        ),
        1.0,
    );

    assert_eq!(
        effect_tuner.selected_lfo().frequency_hz,
        DEFAULT_LFO_FREQUENCY_HZ
    );
}

#[test]
fn reset_all_restores_effect_enable_defaults_and_disables_lfos() {
    let mut defaults = EffectsConfig::default();
    let (
        generation_config,
        mut generation_state,
        material_config,
        mut material_state,
        stage_config,
        mut stage_state,
    ) = default_scene_state();
    defaults.edge_detection.enabled = true;
    let mut effect_tuner = EffectTunerState::from_config(&defaults);
    effect_tuner.current.edge_detection.enabled = false;
    effect_tuner.selected_index = 22;
    effect_tuner.selected_lfo_mut().enabled = true;
    effect_tuner.selected_lfo_mut().shape = LfoShape::Square;

    effect_tuner.reset_all(
        &mut edit_context(
            &generation_config,
            &mut generation_state,
            &material_config,
            &mut material_state,
            &stage_config,
            &mut stage_state,
        ),
        1.0,
    );

    assert!(effect_tuner.current.edge_detection.enabled);
    assert!(!effect_tuner.selected_lfo().enabled);
    assert_eq!(effect_tuner.selected_lfo().shape, LfoShape::Sine);
}

#[test]
fn runtime_snapshot_round_trips_lfo_state() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    effect_tuner.selected_lfo_mut().enabled = true;
    effect_tuner.selected_lfo_mut().shape = LfoShape::Triangle;
    effect_tuner.selected_lfo_mut().amplitude = 1.25;
    effect_tuner.selected_lfo_mut().frequency_hz = 0.75;

    let snapshot = effect_tuner.runtime_snapshot();
    let encoded = toml::to_string(&snapshot).expect("snapshot should serialize");
    let restored_snapshot: EffectRuntimeSnapshot =
        toml::from_str(&encoded).expect("snapshot should deserialize");

    assert_eq!(restored_snapshot.lfos.len(), snapshot.lfos.len());
    assert_eq!(
        restored_snapshot.current.color_wavefolder.gain,
        snapshot.current.color_wavefolder.gain
    );
    assert!(restored_snapshot.lfos[0].enabled);
    assert_eq!(restored_snapshot.lfos[0].shape, LfoShape::Triangle);
}

#[test]
fn runtime_snapshot_restores_keyed_lfos_independent_of_order() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let effect_index = lfo_index_for_parameter(EffectTunerParameter::Effect(
        EffectNumericParameter::LensStrength,
    ))
    .expect("effect parameter should have an LFO slot");
    effect_tuner.lfos[effect_index].enabled = true;
    effect_tuner.lfos[effect_index].shape = LfoShape::Square;
    effect_tuner.lfos[effect_index].amplitude = 0.42;

    let scene_index = lfo_index_for_parameter(EffectTunerParameter::Scene(
        EffectTunerSceneParameter::GlobalOpacity,
    ))
    .expect("scene parameter should have an LFO slot");
    effect_tuner.lfos[scene_index].enabled = true;
    effect_tuner.lfos[scene_index].shape = LfoShape::BrownianMotion;
    effect_tuner.lfos[scene_index].amplitude = 0.31;
    effect_tuner.lfos[scene_index].frequency_hz = 1.7;

    let snapshot = effect_tuner.runtime_snapshot();
    let encoded = toml::to_string(&snapshot).expect("snapshot should serialize");
    let mut value: toml::Value = toml::from_str(&encoded).expect("snapshot should parse as toml");
    let lfos = value
        .get_mut("lfos")
        .and_then(toml::Value::as_array_mut)
        .expect("snapshot should contain a keyed LFO array");
    lfos.retain(|entry| {
        let parameter = entry
            .get("parameter")
            .and_then(toml::Value::as_str)
            .expect("stored LFOs should include a parameter id");
        matches!(parameter, "lens_distortion.strength" | "materials.opacity")
    });
    lfos.reverse();
    let sparse_encoded = toml::to_string(&value).expect("sparse snapshot should serialize");
    let restored: EffectRuntimeSnapshot =
        toml::from_str(&sparse_encoded).expect("sparse snapshot should deserialize");

    assert!(restored.lfos[effect_index].enabled);
    assert_eq!(restored.lfos[effect_index].shape, LfoShape::Square);
    assert_eq!(restored.lfos[effect_index].amplitude, 0.42);

    assert!(restored.lfos[scene_index].enabled);
    assert_eq!(restored.lfos[scene_index].shape, LfoShape::BrownianMotion);
    assert_eq!(restored.lfos[scene_index].amplitude, 0.31);
    assert_eq!(restored.lfos[scene_index].frequency_hz, 1.7);

    let default_snapshot = EffectTunerState::from_config(&EffectsConfig::default()).runtime_snapshot();
    let missing_index = lfo_index_for_parameter(EffectTunerParameter::Effect(
        EffectNumericParameter::BloomRadius,
    ))
    .expect("missing effect parameter should still have a default LFO slot");
    assert_eq!(
        restored.lfos[missing_index].enabled,
        default_snapshot.lfos[missing_index].enabled
    );
    assert_eq!(
        restored.lfos[missing_index].shape,
        default_snapshot.lfos[missing_index].shape
    );
    assert_eq!(
        restored.lfos[missing_index].amplitude,
        default_snapshot.lfos[missing_index].amplitude
    );
    assert_eq!(
        restored.lfos[missing_index].frequency_hz,
        default_snapshot.lfos[missing_index].frequency_hz
    );
}

#[test]
fn base_states_strip_active_scene_lfo_modulation() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        mut generation_state,
        material_config,
        mut material_state,
        _stage_config,
        mut _stage_state,
    ) = default_scene_state();
    material_state.opacity = 0.6;
    effect_tuner.sync_scene_lfo_bases(&view_context(
        &generation_config,
        &generation_state,
        &material_config,
        &material_state,
        &_stage_config,
        &_stage_state,
    ));

    let twist_base = generation_state.twist_per_vertex_radians_base();
    let opacity_base = material_state.opacity;

    let twist_index = lfo_index_for_parameter(EffectTunerParameter::Scene(
        EffectTunerSceneParameter::ChildTwistPerVertexRadians,
    ))
    .expect("twist parameter should have an LFO slot");
    effect_tuner.lfos[twist_index].enabled = true;
    effect_tuner.lfos[twist_index].shape = LfoShape::Sine;
    effect_tuner.lfos[twist_index].amplitude = 0.1;
    effect_tuner.lfos[twist_index].frequency_hz = 1.0;

    let opacity_index = lfo_index_for_parameter(EffectTunerParameter::Scene(
        EffectTunerSceneParameter::GlobalOpacity,
    ))
    .expect("opacity parameter should have an LFO slot");
    effect_tuner.lfos[opacity_index].enabled = true;
    effect_tuner.lfos[opacity_index].shape = LfoShape::Sine;
    effect_tuner.lfos[opacity_index].amplitude = 0.2;
    effect_tuner.lfos[opacity_index].frequency_hz = 1.0;

    let result = effect_tuner.apply_scene_lfos(
        0.25,
        &mut edit_context(
            &generation_config,
            &mut generation_state,
            &material_config,
            &mut material_state,
            &_stage_config,
            &mut _stage_state,
        ),
    );

    assert!(result.generation_changed);
    assert!(result.materials_changed);
    assert!(generation_state.twist_per_vertex_radians(&generation_config) > twist_base);
    assert!(material_state.opacity > opacity_base);

    let base_generation =
        effect_tuner.base_generation_state(&generation_config, &generation_state);
    let base_material = effect_tuner.base_material_state(&material_config, &material_state);

    assert!((base_generation.twist_per_vertex_radians_base() - twist_base).abs() < 1.0e-6);
    assert!((base_material.opacity - opacity_base).abs() < 1.0e-6);
}

#[test]
fn numeric_entry_updates_selected_value() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        mut generation_state,
        material_config,
        mut material_state,
        stage_config,
        mut stage_state,
    ) = default_scene_state();

    for character in ['0', '.', '1', '5', '7'] {
        assert!(effect_tuner.append_numeric_input(
            character,
            &mut edit_context(
                &generation_config,
                &mut generation_state,
                &material_config,
                &mut material_state,
                &stage_config,
                &mut stage_state,
            ),
            1.0,
        ));
    }

    let snapshot = effect_tuner.overlay_snapshot(
        &view_context(
            &generation_config,
            &generation_state,
            &material_config,
            &material_state,
            &stage_config,
            &stage_state,
        ),
        1.0,
    );
    assert_eq!(snapshot.value_text, "0.157");
    assert!((effect_tuner.current.color_wavefolder.gain - 0.157).abs() < 1.0e-6);
}

#[test]
fn lens_strength_accepts_direct_small_decimal_input() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        mut generation_state,
        material_config,
        mut material_state,
        stage_config,
        mut stage_state,
    ) = default_scene_state();
    select_parameter(
        &mut effect_tuner,
        EffectTunerParameter::Effect(EffectNumericParameter::LensStrength),
    );

    for character in ['0', '.', '0', '1'] {
        assert!(effect_tuner.append_numeric_input(
            character,
            &mut edit_context(
                &generation_config,
                &mut generation_state,
                &material_config,
                &mut material_state,
                &stage_config,
                &mut stage_state,
            ),
            1.0,
        ));
    }

    let snapshot = effect_tuner.overlay_snapshot(
        &view_context(
            &generation_config,
            &generation_state,
            &material_config,
            &material_state,
            &stage_config,
            &stage_state,
        ),
        1.0,
    );
    assert_eq!(snapshot.value_text, "0.01");
    assert!((effect_tuner.current.lens_distortion.strength - 0.01).abs() < 1.0e-6);
}

#[test]
fn lens_strength_accepts_three_decimal_places() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        mut generation_state,
        material_config,
        mut material_state,
        stage_config,
        mut stage_state,
    ) = default_scene_state();
    select_parameter(
        &mut effect_tuner,
        EffectTunerParameter::Effect(EffectNumericParameter::LensStrength),
    );

    for (offset, character) in [(0.0, '0'), (0.1, '.'), (0.2, '0'), (0.3, '0'), (0.4, '1')] {
        assert!(effect_tuner.append_numeric_input(
            character,
            &mut edit_context(
                &generation_config,
                &mut generation_state,
                &material_config,
                &mut material_state,
                &stage_config,
                &mut stage_state,
            ),
            1.0 + offset,
        ));
    }

    let snapshot = effect_tuner.overlay_snapshot(
        &view_context(
            &generation_config,
            &generation_state,
            &material_config,
            &material_state,
            &stage_config,
            &stage_state,
        ),
        1.4,
    );
    assert_eq!(snapshot.value_text, "0.001");
    assert!((effect_tuner.current.lens_distortion.strength - 0.001).abs() < 1.0e-6);
}

#[test]
fn numeric_entry_accepts_comma_decimal_separator() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        mut generation_state,
        material_config,
        mut material_state,
        stage_config,
        mut stage_state,
    ) = default_scene_state();
    select_parameter(
        &mut effect_tuner,
        EffectTunerParameter::Effect(EffectNumericParameter::LensStrength),
    );

    for character in ['0', ',', '0', '1'] {
        assert!(effect_tuner.append_numeric_input(
            character,
            &mut edit_context(
                &generation_config,
                &mut generation_state,
                &material_config,
                &mut material_state,
                &stage_config,
                &mut stage_state,
            ),
            1.0,
        ));
    }

    let snapshot = effect_tuner.overlay_snapshot(
        &view_context(
            &generation_config,
            &generation_state,
            &material_config,
            &material_state,
            &stage_config,
            &stage_state,
        ),
        1.0,
    );
    assert_eq!(snapshot.value_text, "0.01");
    assert!((effect_tuner.current.lens_distortion.strength - 0.01).abs() < 1.0e-6);
}

#[test]
fn paused_numeric_entry_starts_a_fresh_value() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        mut generation_state,
        material_config,
        mut material_state,
        stage_config,
        mut stage_state,
    ) = default_scene_state();
    select_parameter(
        &mut effect_tuner,
        EffectTunerParameter::Effect(EffectNumericParameter::LensStrength),
    );

    for (offset, character) in [(0.0, '0'), (0.1, '.'), (0.2, '0'), (0.3, '1')] {
        assert!(effect_tuner.append_numeric_input(
            character,
            &mut edit_context(
                &generation_config,
                &mut generation_state,
                &material_config,
                &mut material_state,
                &stage_config,
                &mut stage_state,
            ),
            1.0 + offset,
        ));
    }

    for (offset, character) in [(0.0, '0'), (0.1, '.'), (0.2, '0'), (0.3, '0'), (0.4, '1')] {
        assert!(effect_tuner.append_numeric_input(
            character,
            &mut edit_context(
                &generation_config,
                &mut generation_state,
                &material_config,
                &mut material_state,
                &stage_config,
                &mut stage_state,
            ),
            2.5 + offset,
        ));
    }

    let snapshot = effect_tuner.overlay_snapshot(
        &view_context(
            &generation_config,
            &generation_state,
            &material_config,
            &material_state,
            &stage_config,
            &stage_state,
        ),
        2.9,
    );
    assert_eq!(snapshot.value_text, "0.001");
    assert!((effect_tuner.current.lens_distortion.strength - 0.001).abs() < 1.0e-6);
}

#[test]
fn finalizing_numeric_entry_keeps_the_typed_value() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        mut generation_state,
        material_config,
        mut material_state,
        stage_config,
        mut stage_state,
    ) = default_scene_state();
    select_parameter(
        &mut effect_tuner,
        EffectTunerParameter::Effect(EffectNumericParameter::LensStrength),
    );

    for (offset, character) in [(0.0, '0'), (0.1, '.'), (0.2, '0'), (0.3, '0'), (0.4, '1')] {
        assert!(effect_tuner.append_numeric_input(
            character,
            &mut edit_context(
                &generation_config,
                &mut generation_state,
                &material_config,
                &mut material_state,
                &stage_config,
                &mut stage_state,
            ),
            1.0 + offset,
        ));
    }

    assert!(effect_tuner.has_numeric_entry());
    assert!(effect_tuner.finalize_numeric_entry(1.5));
    assert!(!effect_tuner.has_numeric_entry());

    let snapshot = effect_tuner.overlay_snapshot(
        &view_context(
            &generation_config,
            &generation_state,
            &material_config,
            &material_state,
            &stage_config,
            &stage_state,
        ),
        1.5,
    );
    assert_eq!(snapshot.value_text, "0.001");
    assert!((effect_tuner.current.lens_distortion.strength - 0.001).abs() < 1.0e-6);
}

#[test]
fn numeric_entry_updates_lfo_frequency_and_backspace_reparses() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        mut generation_state,
        material_config,
        mut material_state,
        stage_config,
        mut stage_state,
    ) = default_scene_state();
    effect_tuner.edit_mode = EffectEditMode::LfoFrequency;

    for character in ['0', '.', '1', '5', '7'] {
        assert!(effect_tuner.append_numeric_input(
            character,
            &mut edit_context(
                &generation_config,
                &mut generation_state,
                &material_config,
                &mut material_state,
                &stage_config,
                &mut stage_state,
            ),
            1.0,
        ));
    }
    assert!(effect_tuner.backspace_numeric_input(
        &mut edit_context(
            &generation_config,
            &mut generation_state,
            &material_config,
            &mut material_state,
            &stage_config,
            &mut stage_state,
        ),
        1.2,
    ));

    let snapshot = effect_tuner.overlay_snapshot(
        &view_context(
            &generation_config,
            &generation_state,
            &material_config,
            &material_state,
            &stage_config,
            &stage_state,
        ),
        1.2,
    );
    assert_eq!(snapshot.frequency_text, "0.15");
    assert!((effect_tuner.selected_lfo().frequency_hz - 0.15).abs() < 1.0e-6);
}

#[test]
fn switching_field_clears_numeric_entry_highlight_text() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        mut generation_state,
        material_config,
        mut material_state,
        stage_config,
        mut stage_state,
    ) = default_scene_state();
    assert!(effect_tuner.append_numeric_input(
        '0',
        &mut edit_context(
            &generation_config,
            &mut generation_state,
            &material_config,
            &mut material_state,
            &stage_config,
            &mut stage_state,
        ),
        1.0,
    ));

    assert!(effect_tuner.step_edit_mode(1, 1.1));

    let view = view_context(
        &generation_config,
        &generation_state,
        &material_config,
        &material_state,
        &stage_config,
        &stage_state,
    );
    let snapshot = effect_tuner.overlay_snapshot(&view, 1.1);
    assert_eq!(snapshot.active_field, EffectOverlayField::LfoAmplitude);
    assert_eq!(
        snapshot.value_text,
        effect_tuner.parameter_value_text(effect_tuner.selected_parameter(), &view)
    );
}

#[test]
fn selecting_another_parameter_clears_numeric_entry() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        mut generation_state,
        material_config,
        mut material_state,
        stage_config,
        mut stage_state,
    ) = default_scene_state();
    assert!(effect_tuner.append_numeric_input(
        '0',
        &mut edit_context(
            &generation_config,
            &mut generation_state,
            &material_config,
            &mut material_state,
            &stage_config,
            &mut stage_state,
        ),
        1.0,
    ));

    assert!(effect_tuner.step_selection(
        1,
        HoldInput {
            just_pressed: true,
            pressed: true,
            just_released: false,
            delta_secs: 0.0,
        },
        1.1,
    ));

    let view = view_context(
        &generation_config,
        &generation_state,
        &material_config,
        &material_state,
        &stage_config,
        &stage_state,
    );
    let snapshot = effect_tuner.overlay_snapshot(&view, 1.1);
    assert_eq!(snapshot.parameter_label, "mod");
    assert_eq!(
        snapshot.value_text,
        effect_tuner.parameter_value_text(effect_tuner.selected_parameter(), &view)
    );
}

#[test]
fn shape_field_ignores_numeric_entry() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        mut generation_state,
        material_config,
        mut material_state,
        stage_config,
        mut stage_state,
    ) = default_scene_state();
    effect_tuner.edit_mode = EffectEditMode::LfoShape;

    assert!(!effect_tuner.append_numeric_input(
        '1',
        &mut edit_context(
            &generation_config,
            &mut generation_state,
            &material_config,
            &mut material_state,
            &stage_config,
            &mut stage_state,
        ),
        1.0,
    ));
    assert_eq!(effect_tuner.selected_lfo().shape, LfoShape::Sine);
}

#[test]
fn scene_parameters_only_expose_the_value_field() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        generation_state,
        material_config,
        material_state,
        stage_config,
        stage_state,
    ) = default_scene_state();
    select_parameter(
        &mut effect_tuner,
        EffectTunerParameter::Scene(EffectTunerSceneParameter::ChildKind),
    );
    effect_tuner.edit_mode = EffectEditMode::LfoShape;

    let snapshot = effect_tuner.overlay_snapshot(
        &view_context(
            &generation_config,
            &generation_state,
            &material_config,
            &material_state,
            &stage_config,
            &stage_state,
        ),
        0.0,
    );

    assert_eq!(snapshot.effect_label, "scene");
    assert_eq!(snapshot.effect_state_text, "VAL");
    assert!(!snapshot.supports_lfo);
    assert_eq!(snapshot.lfo_state_text, "--");
    assert_eq!(snapshot.amplitude_text, "--");
    assert_eq!(snapshot.active_field, EffectOverlayField::Value);
}

#[test]
fn eligible_scene_numeric_parameters_expose_lfo_fields() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        generation_state,
        material_config,
        material_state,
        stage_config,
        stage_state,
    ) = default_scene_state();
    effect_tuner.sync_scene_lfo_bases(&view_context(
        &generation_config,
        &generation_state,
        &material_config,
        &material_state,
        &stage_config,
        &stage_state,
    ));
    select_parameter(
        &mut effect_tuner,
        EffectTunerParameter::Scene(EffectTunerSceneParameter::GlobalOpacity),
    );
    effect_tuner.edit_mode = EffectEditMode::LfoShape;

    let snapshot = effect_tuner.overlay_snapshot(
        &view_context(
            &generation_config,
            &generation_state,
            &material_config,
            &material_state,
            &stage_config,
            &stage_state,
        ),
        0.0,
    );

    assert!(snapshot.supports_lfo);
    assert_eq!(snapshot.lfo_state_text, "OFF");
    assert_eq!(snapshot.amplitude_text, "0.000");
    assert_eq!(snapshot.active_field, EffectOverlayField::LfoShape);
}

#[test]
fn reset_all_restores_scene_defaults() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        mut generation_state,
        material_config,
        mut material_state,
        stage_config,
        mut stage_state,
    ) = default_scene_state();
    generation_state.selected_shape_kind = crate::shapes::ShapeKind::Cube;
    generation_state.spawn_placement_mode = crate::shapes::SpawnPlacementMode::Face;
    generation_state.spawn_add_mode = crate::shapes::SpawnAddMode::FillLevel;
    generation_state
        .parameter_mut(crate::parameters::GenerationParameter::ChildOutwardOffsetRatio)
        .adjust_clamped_base_value(
            1.5,
            generation_config.parameter_spec(
                crate::parameters::GenerationParameter::ChildOutwardOffsetRatio,
            ),
        );
    material_state.opacity = 0.25;
    material_state.saturation = 0.12;
    material_state.surface_mode = MaterialSurfaceMode::Procedural;
    stage_state.enabled = true;
    stage_state.floor_enabled = true;

    effect_tuner.reset_all(
        &mut edit_context(
            &generation_config,
            &mut generation_state,
            &material_config,
            &mut material_state,
            &stage_config,
            &mut stage_state,
        ),
        1.0,
    );

    assert_eq!(
        generation_state.vertex_offset_ratio_base(),
        EffectTunerSceneParameter::ChildOutwardOffsetRatio.default_value(&view_context(
            &generation_config,
            &generation_state,
            &material_config,
            &material_state,
            &stage_config,
            &stage_state,
        ))
    );
    assert_eq!(
        generation_state.selected_shape_kind,
        generation_config.default_child_shape_kind
    );
    assert_eq!(
        generation_state.spawn_placement_mode,
        generation_config.default_spawn_placement_mode
    );
    assert_eq!(
        generation_state.spawn_add_mode,
        crate::shapes::SpawnAddMode::default()
    );
    assert_eq!(
        material_state.opacity,
        material_config.default_opacity_clamped()
    );
    assert_eq!(material_state.saturation, material_config.saturation);
    assert_eq!(material_state.surface_mode, material_config.surface_mode);
    assert_eq!(stage_state.enabled, stage_config.enabled);
    assert_eq!(stage_state.floor_enabled, stage_config.floor.enabled);
}

#[test]
fn enum_scene_parameter_cycles_with_adjustment() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        mut generation_state,
        material_config,
        mut material_state,
        stage_config,
        mut stage_state,
    ) = default_scene_state();
    select_parameter(
        &mut effect_tuner,
        EffectTunerParameter::Scene(EffectTunerSceneParameter::SpawnAddMode),
    );

    assert!(effect_tuner.step_adjustment(
        1.0,
        HoldInput {
            just_pressed: true,
            pressed: true,
            just_released: false,
            delta_secs: 0.0,
        },
        crate::effect_tuner::state::AdjustmentModifiers {
            shift_pressed: false,
            alt_pressed: false,
        },
        &mut edit_context(
            &generation_config,
            &mut generation_state,
            &material_config,
            &mut material_state,
            &stage_config,
            &mut stage_state,
        ),
        1.0,
    ));

    assert_eq!(
        generation_state.spawn_add_mode,
        crate::shapes::SpawnAddMode::FillLevel
    );
}

#[test]
fn enum_scene_parameter_ignores_numeric_entry() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        mut generation_state,
        material_config,
        mut material_state,
        stage_config,
        mut stage_state,
    ) = default_scene_state();
    select_parameter(
        &mut effect_tuner,
        EffectTunerParameter::Scene(EffectTunerSceneParameter::ChildKind),
    );
    let before = generation_state.selected_shape_kind;

    assert!(!effect_tuner.append_numeric_input(
        '1',
        &mut edit_context(
            &generation_config,
            &mut generation_state,
            &material_config,
            &mut material_state,
            &stage_config,
            &mut stage_state,
        ),
        1.0,
    ));
    assert_eq!(generation_state.selected_shape_kind, before);
}

#[test]
fn material_numeric_parameter_updates_runtime_state() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        mut generation_state,
        material_config,
        mut material_state,
        stage_config,
        mut stage_state,
    ) = default_scene_state();
    select_parameter(
        &mut effect_tuner,
        EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialMetallic),
    );

    assert!(effect_tuner.append_numeric_input(
        '1',
        &mut edit_context(
            &generation_config,
            &mut generation_state,
            &material_config,
            &mut material_state,
            &stage_config,
            &mut stage_state,
        ),
        1.0,
    ));

    assert_eq!(material_state.metallic, 1.0);
}

#[test]
fn material_enum_parameter_cycles_with_adjustment() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        mut generation_state,
        material_config,
        mut material_state,
        stage_config,
        mut stage_state,
    ) = default_scene_state();
    select_parameter(
        &mut effect_tuner,
        EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialSurfaceMode),
    );

    assert!(effect_tuner.step_adjustment(
        1.0,
        HoldInput {
            just_pressed: true,
            pressed: true,
            just_released: false,
            delta_secs: 0.0,
        },
        crate::effect_tuner::state::AdjustmentModifiers {
            shift_pressed: false,
            alt_pressed: false,
        },
        &mut edit_context(
            &generation_config,
            &mut generation_state,
            &material_config,
            &mut material_state,
            &stage_config,
            &mut stage_state,
        ),
        1.0,
    ));

    assert_eq!(material_state.surface_mode, MaterialSurfaceMode::Procedural);
}

#[test]
fn stage_enum_parameter_cycles_with_adjustment() {
    let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
    let (
        generation_config,
        mut generation_state,
        material_config,
        mut material_state,
        stage_config,
        mut stage_state,
    ) = default_scene_state();
    select_parameter(
        &mut effect_tuner,
        EffectTunerParameter::Scene(EffectTunerSceneParameter::StageEnabled),
    );

    assert!(effect_tuner.step_adjustment(
        1.0,
        HoldInput {
            just_pressed: true,
            pressed: true,
            just_released: false,
            delta_secs: 0.0,
        },
        crate::effect_tuner::state::AdjustmentModifiers {
            shift_pressed: false,
            alt_pressed: false,
        },
        &mut edit_context(
            &generation_config,
            &mut generation_state,
            &material_config,
            &mut material_state,
            &stage_config,
            &mut stage_state,
        ),
        1.0,
    ));

    assert!(stage_state.enabled);
}
