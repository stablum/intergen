#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EffectTunerOverlaySnapshot {
    pub(crate) pinned: bool,
    pub(crate) effect_label: &'static str,
    pub(crate) effect_state_text: &'static str,
    pub(crate) effect_state_emphasized: bool,
    pub(crate) parameter_label: &'static str,
    pub(crate) value_text: String,
    pub(crate) live_value_text: String,
    pub(crate) supports_lfo: bool,
    pub(crate) lfo_state_text: &'static str,
    pub(crate) lfo_state_emphasized: bool,
    pub(crate) amplitude_text: String,
    pub(crate) frequency_text: String,
    pub(crate) shape_text: &'static str,
    pub(crate) active_field: EffectOverlayField,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EffectTunerPageMode {
    Compact,
    List,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EffectTunerListRowSnapshot {
    pub(crate) effect_label: &'static str,
    pub(crate) effect_state_text: &'static str,
    pub(crate) effect_state_emphasized: bool,
    pub(crate) parameter_label: &'static str,
    pub(crate) value_text: String,
    pub(crate) live_value_text: String,
    pub(crate) supports_lfo: bool,
    pub(crate) lfo_state_text: &'static str,
    pub(crate) lfo_state_emphasized: bool,
    pub(crate) selected: bool,
    pub(crate) active_field: Option<EffectOverlayField>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EffectTunerListOverlaySnapshot {
    pub(crate) pinned: bool,
    pub(crate) total_parameters: usize,
    pub(crate) window_start: usize,
    pub(crate) rows: Vec<EffectTunerListRowSnapshot>,
    pub(crate) detail: EffectTunerOverlaySnapshot,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct EffectRuntimeSnapshot {
    pub(crate) current: EffectsConfig,
    pub(crate) lfos: Vec<ParameterLfo>,
}

#[derive(Clone, Copy)]
pub(crate) struct AdjustmentModifiers {
    pub(crate) shift_pressed: bool,
    pub(crate) alt_pressed: bool,
}

pub(crate) struct EffectTunerViewContext<'a> {
    pub(crate) camera_config: &'a CameraConfig,
    pub(crate) camera_rig: &'a CameraRig,
    pub(crate) generation_config: &'a GenerationConfig,
    pub(crate) generation_state: &'a GenerationState,
    pub(crate) rendering_config: &'a RenderingConfig,
    pub(crate) rendering_state: &'a RenderingState,
    pub(crate) lighting_config: &'a LightingConfig,
    pub(crate) lighting_state: &'a LightingState,
    pub(crate) material_config: &'a MaterialConfig,
    pub(crate) material_state: &'a MaterialState,
    pub(crate) stage_state: &'a StageState,
}

pub(crate) struct EffectTunerEditContext<'a> {
    pub(crate) camera_config: &'a CameraConfig,
    pub(crate) camera_rig: &'a mut CameraRig,
    pub(crate) generation_config: &'a GenerationConfig,
    pub(crate) generation_state: &'a mut GenerationState,
    pub(crate) rendering_config: &'a RenderingConfig,
    pub(crate) rendering_state: &'a mut RenderingState,
    pub(crate) lighting_config: &'a LightingConfig,
    pub(crate) lighting_state: &'a mut LightingState,
    pub(crate) material_config: &'a MaterialConfig,
    pub(crate) material_state: &'a mut MaterialState,
    pub(crate) stage_state: &'a mut StageState,
}

impl EffectTunerEditContext<'_> {
    fn view(&self) -> EffectTunerViewContext<'_> {
        EffectTunerViewContext {
            camera_config: self.camera_config,
            camera_rig: &*self.camera_rig,
            generation_config: self.generation_config,
            generation_state: &*self.generation_state,
            rendering_config: self.rendering_config,
            rendering_state: &*self.rendering_state,
            lighting_config: self.lighting_config,
            lighting_state: &*self.lighting_state,
            material_config: self.material_config,
            material_state: &*self.material_state,
            stage_state: &*self.stage_state,
        }
    }
}

fn mark_scene_lfo_change(
    result: &mut SceneLfoApplicationResult,
    parameter: EffectTunerSceneParameter,
) {
    match parameter {
        EffectTunerSceneParameter::ChildTwistPerVertexRadians
        | EffectTunerSceneParameter::ChildOutwardOffsetRatio => {
            result.generation_changed = true;
        }
        EffectTunerSceneParameter::GlobalOpacity
        | EffectTunerSceneParameter::MaterialHueStepPerLevel
        | EffectTunerSceneParameter::MaterialSaturation
        | EffectTunerSceneParameter::MaterialLightness
        | EffectTunerSceneParameter::MaterialMetallic
        | EffectTunerSceneParameter::MaterialPerceptualRoughness
        | EffectTunerSceneParameter::MaterialReflectance
        | EffectTunerSceneParameter::MaterialCubeHueBias
        | EffectTunerSceneParameter::MaterialTetrahedronHueBias
        | EffectTunerSceneParameter::MaterialOctahedronHueBias
        | EffectTunerSceneParameter::MaterialDodecahedronHueBias
        | EffectTunerSceneParameter::MaterialAccentEveryNLevels
        | EffectTunerSceneParameter::MaterialLevelLightnessShift
        | EffectTunerSceneParameter::MaterialLevelSaturationShift
        | EffectTunerSceneParameter::MaterialLevelMetallicShift
        | EffectTunerSceneParameter::MaterialLevelRoughnessShift
        | EffectTunerSceneParameter::MaterialLevelReflectanceShift => {
            result.materials_changed = true;
        }
        EffectTunerSceneParameter::CameraDistance
        | EffectTunerSceneParameter::CameraAngularVelocityX
        | EffectTunerSceneParameter::CameraAngularVelocityY
        | EffectTunerSceneParameter::CameraAngularVelocityZ
        | EffectTunerSceneParameter::CameraZoomVelocity => {
            result.camera_changed = true;
        }
        EffectTunerSceneParameter::RenderingClearColorR
        | EffectTunerSceneParameter::RenderingClearColorG
        | EffectTunerSceneParameter::RenderingClearColorB
        | EffectTunerSceneParameter::RenderingAmbientColorR
        | EffectTunerSceneParameter::RenderingAmbientColorG
        | EffectTunerSceneParameter::RenderingAmbientColorB
        | EffectTunerSceneParameter::RenderingAmbientBrightness => {
            result.rendering_changed = true;
        }
        EffectTunerSceneParameter::StageFloorColorR
        | EffectTunerSceneParameter::StageFloorColorG
        | EffectTunerSceneParameter::StageFloorColorB
        | EffectTunerSceneParameter::StageFloorTranslationX
        | EffectTunerSceneParameter::StageFloorTranslationY
        | EffectTunerSceneParameter::StageFloorTranslationZ
        | EffectTunerSceneParameter::StageFloorRotationX
        | EffectTunerSceneParameter::StageFloorRotationY
        | EffectTunerSceneParameter::StageFloorRotationZ
        | EffectTunerSceneParameter::StageFloorSizeX
        | EffectTunerSceneParameter::StageFloorSizeY
        | EffectTunerSceneParameter::StageFloorThickness
        | EffectTunerSceneParameter::StageFloorMetallic
        | EffectTunerSceneParameter::StageFloorPerceptualRoughness
        | EffectTunerSceneParameter::StageFloorReflectance
        | EffectTunerSceneParameter::StageBackdropColorR
        | EffectTunerSceneParameter::StageBackdropColorG
        | EffectTunerSceneParameter::StageBackdropColorB
        | EffectTunerSceneParameter::StageBackdropTranslationX
        | EffectTunerSceneParameter::StageBackdropTranslationY
        | EffectTunerSceneParameter::StageBackdropTranslationZ
        | EffectTunerSceneParameter::StageBackdropRotationX
        | EffectTunerSceneParameter::StageBackdropRotationY
        | EffectTunerSceneParameter::StageBackdropRotationZ
        | EffectTunerSceneParameter::StageBackdropSizeX
        | EffectTunerSceneParameter::StageBackdropSizeY
        | EffectTunerSceneParameter::StageBackdropThickness
        | EffectTunerSceneParameter::StageBackdropMetallic
        | EffectTunerSceneParameter::StageBackdropPerceptualRoughness
        | EffectTunerSceneParameter::StageBackdropReflectance => {
            result.stage_changed = true;
        }
        EffectTunerSceneParameter::LightingDirectionalColorR
        | EffectTunerSceneParameter::LightingDirectionalColorG
        | EffectTunerSceneParameter::LightingDirectionalColorB
        | EffectTunerSceneParameter::LightingDirectionalIlluminance
        | EffectTunerSceneParameter::LightingDirectionalTranslationX
        | EffectTunerSceneParameter::LightingDirectionalTranslationY
        | EffectTunerSceneParameter::LightingDirectionalTranslationZ
        | EffectTunerSceneParameter::LightingDirectionalLookAtX
        | EffectTunerSceneParameter::LightingDirectionalLookAtY
        | EffectTunerSceneParameter::LightingDirectionalLookAtZ
        | EffectTunerSceneParameter::LightingPointColorR
        | EffectTunerSceneParameter::LightingPointColorG
        | EffectTunerSceneParameter::LightingPointColorB
        | EffectTunerSceneParameter::LightingPointIntensity
        | EffectTunerSceneParameter::LightingPointRange
        | EffectTunerSceneParameter::LightingPointTranslationX
        | EffectTunerSceneParameter::LightingPointTranslationY
        | EffectTunerSceneParameter::LightingPointTranslationZ
        | EffectTunerSceneParameter::LightingAccentColorR
        | EffectTunerSceneParameter::LightingAccentColorG
        | EffectTunerSceneParameter::LightingAccentColorB
        | EffectTunerSceneParameter::LightingAccentIntensity
        | EffectTunerSceneParameter::LightingAccentRange
        | EffectTunerSceneParameter::LightingAccentTranslationX
        | EffectTunerSceneParameter::LightingAccentTranslationY
        | EffectTunerSceneParameter::LightingAccentTranslationZ => {
            result.lighting_changed = true;
        }
        EffectTunerSceneParameter::ChildKind
        | EffectTunerSceneParameter::SpawnPlacementMode
        | EffectTunerSceneParameter::SpawnAddMode
        | EffectTunerSceneParameter::ChildScaleRatio
        | EffectTunerSceneParameter::ChildSpawnExclusionProbability
        | EffectTunerSceneParameter::StageEnabled
        | EffectTunerSceneParameter::StageFloorEnabled
        | EffectTunerSceneParameter::StageBackdropEnabled
        | EffectTunerSceneParameter::MaterialSurfaceMode
        | EffectTunerSceneParameter::MaterialBaseSurface
        | EffectTunerSceneParameter::MaterialRootSurface
        | EffectTunerSceneParameter::MaterialAccentSurface => {}
    }
}

#[derive(Default)]
pub(crate) struct SceneLfoApplicationResult {
    pub(crate) generation_changed: bool,
    pub(crate) materials_changed: bool,
    pub(crate) rendering_changed: bool,
    pub(crate) stage_changed: bool,
    pub(crate) lighting_changed: bool,
    pub(crate) camera_changed: bool,
}

#[derive(Resource, Clone)]
pub(crate) struct EffectTunerState {
    defaults: EffectsConfig,
    current: EffectsConfig,
    lfos: Vec<ParameterLfo>,
    scene_lfo_bases: Vec<f32>,
    generation_scene_lfo_applied: bool,
    scene_numeric_lfo_applied: bool,
    selected_index: usize,
    page_mode: EffectTunerPageMode,
    edit_mode: EffectEditMode,
    numeric_entry: NumericEntryBuffer,
    last_numeric_entry_edit_secs: Option<f32>,
    pinned: bool,
    visible_until_secs: f32,
    select_previous_hold: HoldRepeatState,
    select_next_hold: HoldRepeatState,
    decrease_hold: HoldRepeatState,
    increase_hold: HoldRepeatState,
}

impl EffectTunerState {
    pub(crate) fn from_config(effects_config: &EffectsConfig) -> Self {
        Self {
            defaults: effects_config.clone(),
            current: effects_config.clone(),
            lfos: default_lfos(),
            scene_lfo_bases: default_scene_lfo_bases(),
            generation_scene_lfo_applied: false,
            scene_numeric_lfo_applied: false,
            selected_index: 0,
            page_mode: EffectTunerPageMode::Compact,
            edit_mode: EffectEditMode::Value,
            numeric_entry: NumericEntryBuffer::default(),
            last_numeric_entry_edit_secs: None,
            pinned: false,
            visible_until_secs: 0.0,
            select_previous_hold: HoldRepeatState::default(),
            select_next_hold: HoldRepeatState::default(),
            decrease_hold: HoldRepeatState::default(),
            increase_hold: HoldRepeatState::default(),
        }
    }

    pub(crate) fn selected_parameter(&self) -> EffectTunerParameter {
        EffectTunerParameter::all()[self.selected_index]
    }

    pub(crate) fn page_mode(&self) -> EffectTunerPageMode {
        self.page_mode
    }

    pub(crate) fn selected_effect_group(&self) -> Option<EffectGroup> {
        self.selected_parameter().effect_group()
    }

    pub(crate) fn active_field(&self) -> EffectOverlayField {
        self.displayed_edit_mode().overlay_field()
    }

    pub(crate) fn is_visible(&self, now_secs: f32) -> bool {
        self.pinned || now_secs <= self.visible_until_secs
    }

    pub(crate) fn has_active_effect_lfos(&self) -> bool {
        let effect_count = EffectNumericParameter::all().len();
        self.lfos[..effect_count]
            .iter()
            .copied()
            .any(ParameterLfo::is_active)
    }

    pub(crate) fn has_active_scene_lfos(&self) -> bool {
        let effect_count = EffectNumericParameter::all().len();
        self.lfos[effect_count..]
            .iter()
            .copied()
            .any(ParameterLfo::is_active)
    }

    pub(crate) fn needs_scene_lfo_application(&self) -> bool {
        self.has_active_scene_lfos()
            || self.generation_scene_lfo_applied
            || self.scene_numeric_lfo_applied
    }

    pub(crate) fn runtime_snapshot(&self) -> EffectRuntimeSnapshot {
        EffectRuntimeSnapshot {
            current: self.current.clone(),
            lfos: self.lfos.clone(),
        }
    }

    pub(crate) fn apply_runtime_snapshot(&mut self, snapshot: &EffectRuntimeSnapshot) {
        self.current = snapshot.current.clone();
        self.lfos = default_lfos();
        self.scene_lfo_bases = default_scene_lfo_bases();
        self.generation_scene_lfo_applied = false;
        self.scene_numeric_lfo_applied = false;
        for (target, source) in self.lfos.iter_mut().zip(snapshot.lfos.iter().copied()) {
            *target = source;
        }
        self.selected_index = 0;
        self.page_mode = EffectTunerPageMode::Compact;
        self.edit_mode = EffectEditMode::Value;
        self.clear_numeric_entry();
        self.reset_hold_states();
    }

    pub(crate) fn sync_scene_lfo_bases(&mut self, context: &EffectTunerViewContext<'_>) {
        for parameter in EffectTunerSceneParameter::lfo_capable() {
            self.sync_scene_parameter_base_if_needed(EffectTunerParameter::Scene(*parameter), context);
        }
    }

    pub(crate) fn restore_scene_parameter_base_if_needed(
        &mut self,
        parameter: EffectTunerParameter,
        context: &mut EffectTunerEditContext<'_>,
    ) -> bool {
        let EffectTunerParameter::Scene(parameter) = parameter else {
            return false;
        };
        if parameter.is_generation_lfo_parameter() {
            return false;
        }
        let Some(base_index) = parameter.lfo_scene_index() else {
            return false;
        };
        let base_value = self.scene_lfo_bases[base_index];
        let current_value = parameter.value(&context.view());
        if (current_value - base_value).abs() <= 1.0e-6 {
            return false;
        }
        let applied_value = parameter.set_value(context, base_value);
        (applied_value - current_value).abs() > 1.0e-6
    }

    pub(crate) fn sync_scene_parameter_base_if_needed(
        &mut self,
        parameter: EffectTunerParameter,
        context: &EffectTunerViewContext<'_>,
    ) {
        let EffectTunerParameter::Scene(parameter) = parameter else {
            return;
        };
        let Some(base_index) = parameter.lfo_scene_index() else {
            return;
        };
        if parameter.is_numeric() {
            self.scene_lfo_bases[base_index] = parameter.value(context);
        }
    }

    pub(crate) fn base_material_state(
        &self,
        material_config: &MaterialConfig,
        material_state: &MaterialState,
    ) -> MaterialState {
        let mut base_state = material_state.clone();
        for parameter in EffectTunerSceneParameter::material_lfo_capable() {
            let Some(base_index) = parameter.lfo_scene_index() else {
                continue;
            };
            parameter.apply_material_numeric_value(
                material_config,
                &mut base_state,
                self.scene_lfo_bases[base_index],
            );
        }
        base_state
    }

    pub(crate) fn base_stage_state(&self, stage_state: &StageState) -> StageState {
        let mut base_state = stage_state.clone();
        for parameter in EffectTunerSceneParameter::lfo_capable() {
            let Some(base_index) = parameter.lfo_scene_index() else {
                continue;
            };
            let _ = parameter.apply_stage_numeric_value(&mut base_state, self.scene_lfo_bases[base_index]);
        }
        base_state
    }

    pub(crate) fn base_rendering_state(&self, rendering_state: &RenderingState) -> RenderingState {
        let mut base_state = rendering_state.clone();
        for parameter in EffectTunerSceneParameter::lfo_capable() {
            let Some(base_index) = parameter.lfo_scene_index() else {
                continue;
            };
            let _ = parameter.apply_rendering_numeric_value(&mut base_state, self.scene_lfo_bases[base_index]);
        }
        base_state
    }

    pub(crate) fn base_lighting_state(&self, lighting_state: &LightingState) -> LightingState {
        let mut base_state = lighting_state.clone();
        for parameter in EffectTunerSceneParameter::lfo_capable() {
            let Some(base_index) = parameter.lfo_scene_index() else {
                continue;
            };
            let _ = parameter.apply_lighting_numeric_value(&mut base_state, self.scene_lfo_bases[base_index]);
        }
        base_state
    }

    pub(crate) fn base_camera_rig(
        &self,
        camera_config: &CameraConfig,
        camera_rig: &CameraRig,
    ) -> CameraRig {
        let mut base_state = CameraRig {
            orientation: camera_rig.orientation,
            angular_velocity: camera_rig.angular_velocity,
            distance: camera_rig.distance,
            zoom_velocity: camera_rig.zoom_velocity,
        };
        for parameter in EffectTunerSceneParameter::lfo_capable() {
            let Some(base_index) = parameter.lfo_scene_index() else {
                continue;
            };
            let _ = parameter.apply_camera_numeric_value(
                camera_config,
                &mut base_state,
                self.scene_lfo_bases[base_index],
            );
        }
        base_state
    }

    pub(crate) fn base_generation_state(
        &self,
        generation_config: &GenerationConfig,
        generation_state: &GenerationState,
    ) -> GenerationState {
        let mut base_state = generation_state.clone();
        for parameter in [
            GenerationParameter::ChildTwistPerVertexRadians,
            GenerationParameter::ChildOutwardOffsetRatio,
        ] {
            base_state.parameter_mut(parameter).set_additive_offset(0.0);
        }
        let twist_per_vertex_radians = base_state.twist_per_vertex_radians(generation_config);
        let vertex_offset_ratio = base_state.vertex_offset_ratio(generation_config);
        crate::shapes::recompute_spawn_tree(
            &mut base_state.nodes,
            &crate::shapes::ShapeCatalog::new(),
            twist_per_vertex_radians,
            vertex_offset_ratio,
        );
        base_state
    }

    pub(crate) fn apply_scene_lfos(
        &mut self,
        now_secs: f32,
        context: &mut EffectTunerEditContext<'_>,
    ) -> SceneLfoApplicationResult {
        let mut result = SceneLfoApplicationResult::default();

        let mut generation_lfo_active = false;
        for parameter in [
            EffectTunerSceneParameter::ChildTwistPerVertexRadians,
            EffectTunerSceneParameter::ChildOutwardOffsetRatio,
        ] {
            let Some(lfo_index) = lfo_index_for_parameter(EffectTunerParameter::Scene(parameter))
            else {
                continue;
            };
            let lfo = self.lfos[lfo_index];
            let offset = if lfo.is_active() {
                generation_lfo_active = true;
                lfo.amplitude
                    * lfo
                        .shape
                        .sample(now_secs * lfo.frequency_hz, lfo_seed_for_index(lfo_index))
            } else {
                0.0
            };
            let generation_parameter = parameter
                .generation_parameter()
                .expect("generation LFO parameter should map to generation state");
            context
                .generation_state
                .parameter_mut(generation_parameter)
                .set_additive_offset(offset);
        }
        if generation_lfo_active || self.generation_scene_lfo_applied {
            result.generation_changed = true;
        }
        self.generation_scene_lfo_applied = generation_lfo_active;

        let mut scene_numeric_lfo_active = false;
        for parameter in EffectTunerSceneParameter::lfo_capable() {
            if parameter.is_generation_lfo_parameter() {
                continue;
            }
            let Some(lfo_index) = lfo_index_for_parameter(EffectTunerParameter::Scene(*parameter))
            else {
                continue;
            };
            let Some(base_index) = parameter.lfo_scene_index() else {
                continue;
            };
            let base_value = self.scene_lfo_bases[base_index];
            let lfo = self.lfos[lfo_index];
            let next_value = if lfo.is_active() {
                scene_numeric_lfo_active = true;
                base_value
                    + lfo.amplitude
                        * lfo
                            .shape
                            .sample(now_secs * lfo.frequency_hz, lfo_seed_for_index(lfo_index))
            } else {
                base_value
            };
            let previous_value = parameter.value(&context.view());
            let applied_value = parameter.set_value(context, next_value);
            if (previous_value - applied_value).abs() > 1.0e-6 {
                mark_scene_lfo_change(&mut result, *parameter);
            }
        }
        self.scene_numeric_lfo_applied = scene_numeric_lfo_active;

        result
    }

    pub(crate) fn evaluated_effects(&self, now_secs: f32) -> EffectsConfig {
        let mut effects = self.current.clone();

        for (index, parameter) in EffectNumericParameter::all().iter().copied().enumerate() {
            let lfo = self.lfos[index];
            if !lfo.is_active() {
                continue;
            }

            let base_value = parameter.value(&self.current);
            let lfo_offset = lfo.amplitude
                * lfo
                    .shape
                    .sample(now_secs * lfo.frequency_hz, index as u32 + 1);
            parameter.set_value(&mut effects, base_value + lfo_offset);
        }

        effects
    }

    fn scene_parameter_base_numeric_value(
        &self,
        parameter: EffectTunerSceneParameter,
        context: &EffectTunerViewContext<'_>,
    ) -> Option<f32> {
        if let Some(base_index) = parameter.lfo_scene_index() {
            return self.scene_lfo_bases.get(base_index).copied();
        }
        parameter.is_numeric().then(|| parameter.value(context))
    }

    fn parameter_value_text(
        &self,
        parameter: EffectTunerParameter,
        context: &EffectTunerViewContext<'_>,
    ) -> String {
        match parameter {
            EffectTunerParameter::Effect(parameter) => parameter.display_value(&self.current),
            EffectTunerParameter::Scene(parameter) => self
                .scene_parameter_base_numeric_value(parameter, context)
                .and_then(|value| parameter.display_numeric_value(value))
                .unwrap_or_else(|| parameter.display_value(context)),
        }
    }

    fn parameter_live_value_text(
        &self,
        parameter: EffectTunerParameter,
        live_effects: &EffectsConfig,
        context: &EffectTunerViewContext<'_>,
    ) -> String {
        match parameter {
            EffectTunerParameter::Effect(parameter) => parameter.display_value(live_effects),
            EffectTunerParameter::Scene(parameter) => parameter
                .live_value(context)
                .and_then(|value| parameter.display_numeric_value(value))
                .unwrap_or_else(|| parameter.display_value(context)),
        }
    }

    pub(crate) fn overlay_snapshot(
        &self,
        context: &EffectTunerViewContext<'_>,
        now_secs: f32,
    ) -> EffectTunerOverlaySnapshot {
        let parameter = self.selected_parameter();
        let live_effects = self.evaluated_effects(now_secs);
        let supports_lfo = parameter.supports_lfo();
        let (effect_state_text, effect_state_emphasized) = match parameter.effect_group() {
            Some(effect) => {
                let enabled = effect.is_enabled(&self.current);
                (if enabled { "ON" } else { "OFF" }, enabled)
            }
            None => ("VAL", false),
        };
        let (lfo_state_text, lfo_state_emphasized, amplitude_text, frequency_text, shape_text) =
            if supports_lfo {
                let lfo = self.selected_lfo();
                (
                    if lfo.enabled { "ON" } else { "OFF" },
                    lfo.enabled,
                    self.overlay_numeric_text(
                        EffectOverlayField::LfoAmplitude,
                        format!("{:.3}", lfo.amplitude),
                    ),
                    self.overlay_numeric_text(
                        EffectOverlayField::LfoFrequency,
                        format!("{:.3}", lfo.frequency_hz),
                    ),
                    lfo.shape.label(),
                )
            } else {
                ("--", false, "--".to_string(), "--".to_string(), "--")
            };

        EffectTunerOverlaySnapshot {
            pinned: self.pinned,
            effect_label: parameter.group_label(),
            effect_state_text,
            effect_state_emphasized,
            parameter_label: parameter.short_label(),
            value_text: self.overlay_numeric_text(
                EffectOverlayField::Value,
                self.parameter_value_text(parameter, context),
            ),
            live_value_text: self.parameter_live_value_text(parameter, &live_effects, context),
            supports_lfo,
            lfo_state_text,
            lfo_state_emphasized,
            amplitude_text,
            frequency_text,
            shape_text,
            active_field: self.active_field(),
        }
    }

    pub(crate) fn list_overlay_snapshot(
        &self,
        context: &EffectTunerViewContext<'_>,
        now_secs: f32,
        visible_rows: usize,
    ) -> EffectTunerListOverlaySnapshot {
        let live_effects = self.evaluated_effects(now_secs);
        let (window_start, window_end) = self.selection_window_bounds(visible_rows.max(1));
        let rows = (window_start..window_end)
            .map(|index| {
                self.list_row_snapshot(EffectTunerParameter::all()[index], context, &live_effects)
            })
            .collect();

        EffectTunerListOverlaySnapshot {
            pinned: self.pinned,
            total_parameters: EffectTunerParameter::all().len(),
            window_start,
            rows,
            detail: self.overlay_snapshot(context, now_secs),
        }
    }

    pub(crate) fn edit_mode_label(&self) -> &'static str {
        self.displayed_edit_mode().label()
    }

    pub(crate) fn open_page(&mut self, now_secs: f32) {
        self.page_mode = EffectTunerPageMode::Compact;
        self.pinned = true;
        self.note_interaction(now_secs);
    }

    pub(crate) fn show_list_page(&mut self, now_secs: f32) {
        self.page_mode = EffectTunerPageMode::List;
        self.note_interaction(now_secs);
    }

    pub(crate) fn close_page(&mut self) {
        self.page_mode = EffectTunerPageMode::Compact;
        self.pinned = false;
        self.visible_until_secs = 0.0;
        self.clear_numeric_entry();
        self.reset_hold_states();
    }

    pub(crate) fn toggle_selected_effect(&mut self, now_secs: f32) -> Option<bool> {
        self.clear_numeric_entry();
        let effect = self.selected_effect_group()?;
        let next_enabled = !effect.is_enabled(&self.current);
        effect.set_enabled(&mut self.current, next_enabled);
        self.note_interaction(now_secs);
        Some(next_enabled)
    }

    pub(crate) fn toggle_selected_lfo(
        &mut self,
        context: &EffectTunerViewContext<'_>,
        now_secs: f32,
    ) -> Option<bool> {
        if !self.selected_parameter().supports_lfo() {
            return None;
        }

        self.clear_numeric_entry();
        if !self.selected_lfo().enabled && self.selected_lfo().amplitude <= f32::EPSILON {
            self.selected_lfo_mut().amplitude =
                self.selected_parameter().default_lfo_amplitude(context);
        }
        let lfo = self.selected_lfo_mut();
        lfo.enabled = !lfo.enabled;
        let enabled = lfo.enabled;
        self.note_interaction(now_secs);
        Some(enabled)
    }

    pub(crate) fn step_edit_mode(&mut self, direction: isize, now_secs: f32) -> bool {
        self.clear_numeric_entry();
        let previous_mode = self.edit_mode;
        let mut next_mode = self.edit_mode;
        for _ in 0..4 {
            next_mode = next_mode.step(direction);
            if self.mode_supported_for_parameter(next_mode, self.selected_parameter()) {
                break;
            }
        }
        self.edit_mode = next_mode;
        self.note_interaction(now_secs);
        self.edit_mode != previous_mode
    }

    pub(crate) fn step_selection(
        &mut self,
        direction: isize,
        input: HoldInput,
        now_secs: f32,
    ) -> bool {
        let hold = if direction < 0 {
            &mut self.select_previous_hold
        } else {
            &mut self.select_next_hold
        };

        if hold.update_with_input(input, HOLD_DELAY_SECS, REPEAT_INTERVAL_SECS) {
            self.cycle_selection(direction, now_secs);
            true
        } else {
            false
        }
    }

    pub(crate) fn scroll_selection(&mut self, direction: isize, now_secs: f32) -> bool {
        let direction = direction.signum();
        if direction == 0 {
            return false;
        }

        if direction < 0 {
            self.select_previous_hold.reset();
        } else {
            self.select_next_hold.reset();
        }
        self.cycle_selection(direction, now_secs);
        true
    }

    pub(crate) fn step_adjustment(
        &mut self,
        direction: f32,
        input: HoldInput,
        modifiers: AdjustmentModifiers,
        context: &mut EffectTunerEditContext<'_>,
        now_secs: f32,
    ) -> bool {
        let hold = if direction < 0.0 {
            &mut self.decrease_hold
        } else {
            &mut self.increase_hold
        };

        if hold.update_with_input(input, HOLD_DELAY_SECS, REPEAT_INTERVAL_SECS) {
            self.adjust_selected(direction, modifiers, context, now_secs);
            true
        } else {
            false
        }
    }

    pub(crate) fn append_numeric_input(
        &mut self,
        character: char,
        context: &mut EffectTunerEditContext<'_>,
        now_secs: f32,
    ) -> bool {
        if !self.active_field_accepts_numeric_entry() {
            return false;
        }

        if self.should_restart_numeric_entry(now_secs) {
            self.clear_numeric_entry();
        }

        if !self.numeric_entry.push(character) {
            return false;
        }

        self.last_numeric_entry_edit_secs = Some(now_secs);
        self.apply_numeric_entry_to_selected(context);
        self.note_interaction(now_secs);
        true
    }

    pub(crate) fn backspace_numeric_input(
        &mut self,
        context: &mut EffectTunerEditContext<'_>,
        now_secs: f32,
    ) -> bool {
        if !self.numeric_entry.backspace() {
            return false;
        }

        self.last_numeric_entry_edit_secs = self.numeric_entry.displayed_text().map(|_| now_secs);
        self.apply_numeric_entry_to_selected(context);
        self.note_interaction(now_secs);
        true
    }

    pub(crate) fn has_numeric_entry(&self) -> bool {
        self.numeric_entry.displayed_text().is_some()
    }

    pub(crate) fn finalize_numeric_entry(&mut self, now_secs: f32) -> bool {
        if !self.has_numeric_entry() {
            return false;
        }

        self.clear_numeric_entry();
        self.note_interaction(now_secs);
        true
    }

    pub(crate) fn reset_selected(
        &mut self,
        context: &mut EffectTunerEditContext<'_>,
        now_secs: f32,
    ) {
        self.clear_numeric_entry();
        let parameter = self.selected_parameter();
        match self.displayed_edit_mode() {
            EffectEditMode::Value => {
                parameter.reset_value(&self.defaults, &mut self.current, context);
            }
            EffectEditMode::LfoAmplitude => {
                self.selected_lfo_mut().amplitude =
                    parameter.default_lfo_amplitude(&context.view());
            }
            EffectEditMode::LfoFrequency => {
                self.selected_lfo_mut().frequency_hz = DEFAULT_LFO_FREQUENCY_HZ;
            }
            EffectEditMode::LfoShape => {
                self.selected_lfo_mut().shape = LfoShape::Sine;
            }
        }
        self.note_interaction(now_secs);
    }

    pub(crate) fn reset_all(&mut self, context: &mut EffectTunerEditContext<'_>, now_secs: f32) {
        self.current = self.defaults.clone();
        self.lfos = default_lfos();
        self.edit_mode = EffectEditMode::Value;
        self.clear_numeric_entry();
        for parameter in EffectTunerSceneParameter::all() {
            parameter.reset_value(context);
        }
        self.note_interaction(now_secs);
    }

    pub(crate) fn selected_status_message(
        &self,
        context: &EffectTunerViewContext<'_>,
        now_secs: f32,
    ) -> String {
        let parameter = self.selected_parameter();
        let live_effects = self.evaluated_effects(now_secs);
        match self.displayed_edit_mode() {
            EffectEditMode::Value => match parameter {
                EffectTunerParameter::Effect(effect_parameter) => format!(
                    "{} = {} (live {})",
                    effect_parameter.label(),
                    effect_parameter.display_value(&self.current),
                    effect_parameter.display_value(&live_effects)
                ),
                EffectTunerParameter::Scene(scene_parameter) => format!(
                    "{} = {} (live {})",
                    scene_parameter.label(),
                    self.parameter_value_text(parameter, context),
                    self.parameter_live_value_text(parameter, &live_effects, context)
                ),
            },
            EffectEditMode::LfoAmplitude => {
                let lfo = self.selected_lfo();
                format!("{} lfo amplitude = {:.3}", parameter.label(), lfo.amplitude)
            }
            EffectEditMode::LfoFrequency => {
                let lfo = self.selected_lfo();
                format!(
                    "{} lfo frequency = {:.3}Hz",
                    parameter.label(),
                    lfo.frequency_hz
                )
            }
            EffectEditMode::LfoShape => {
                let lfo = self.selected_lfo();
                format!("{} lfo shape = {}", parameter.label(), lfo.shape.label())
            }
        }
    }

    fn note_interaction(&mut self, now_secs: f32) {
        self.visible_until_secs = now_secs + OVERLAY_HOLD_SECS;
    }

    fn cycle_selection(&mut self, direction: isize, now_secs: f32) {
        self.clear_numeric_entry();
        let parameter_count = EffectTunerParameter::all().len() as isize;
        let next_index =
            (self.selected_index as isize + direction).rem_euclid(parameter_count) as usize;
        self.selected_index = next_index;
        self.coerce_edit_mode_for_selected();
        self.note_interaction(now_secs);
    }

    fn adjust_selected(
        &mut self,
        direction: f32,
        modifiers: AdjustmentModifiers,
        context: &mut EffectTunerEditContext<'_>,
        now_secs: f32,
    ) {
        self.clear_numeric_entry();
        let parameter = self.selected_parameter();
        match self.displayed_edit_mode() {
            EffectEditMode::Value => {
                parameter.adjust_value(&mut self.current, context, direction, modifiers);
            }
            EffectEditMode::LfoAmplitude => {
                let step = parameter.adjustment_step(
                    &context.view(),
                    modifiers.shift_pressed,
                    modifiers.alt_pressed,
                );
                let lfo = self.selected_lfo_mut();
                lfo.amplitude = (lfo.amplitude + direction * step).max(0.0);
            }
            EffectEditMode::LfoFrequency => {
                let mut step = LFO_FREQUENCY_STEP_HZ;
                if modifiers.shift_pressed {
                    step *= 10.0;
                }
                if modifiers.alt_pressed {
                    step *= 0.1;
                }
                let lfo = self.selected_lfo_mut();
                lfo.frequency_hz = (lfo.frequency_hz + direction * step).max(0.0);
            }
            EffectEditMode::LfoShape => {
                let lfo = self.selected_lfo_mut();
                lfo.shape = lfo.shape.cycle(direction);
            }
        }
        self.note_interaction(now_secs);
    }

    fn selected_lfo(&self) -> ParameterLfo {
        let index = lfo_index_for_parameter(self.selected_parameter())
            .expect("selected parameter should support LFOs");
        self.lfos[index]
    }

    fn selected_lfo_mut(&mut self) -> &mut ParameterLfo {
        let index = lfo_index_for_parameter(self.selected_parameter())
            .expect("selected parameter should support LFOs");
        &mut self.lfos[index]
    }

    fn overlay_numeric_text(&self, field: EffectOverlayField, fallback: String) -> String {
        if self.active_field() == field {
            if let Some(buffer) = self.numeric_entry.displayed_text() {
                return buffer.to_string();
            }
        }

        fallback
    }

    fn apply_numeric_entry_to_selected(
        &mut self,
        context: &mut EffectTunerEditContext<'_>,
    ) -> bool {
        let Some(value) = self.numeric_entry.parsed_value() else {
            return false;
        };

        let parameter = self.selected_parameter();
        match self.displayed_edit_mode() {
            EffectEditMode::Value => {
                return parameter.apply_numeric_value_input(&mut self.current, context, value);
            }
            EffectEditMode::LfoAmplitude => self.selected_lfo_mut().amplitude = value.max(0.0),
            EffectEditMode::LfoFrequency => self.selected_lfo_mut().frequency_hz = value.max(0.0),
            EffectEditMode::LfoShape => return false,
        }

        true
    }

    fn lfo_for_parameter(&self, parameter: EffectTunerParameter) -> Option<ParameterLfo> {
        let index = lfo_index_for_parameter(parameter)?;
        self.lfos.get(index).copied()
    }

    fn mode_supported_for_parameter(
        &self,
        edit_mode: EffectEditMode,
        parameter: EffectTunerParameter,
    ) -> bool {
        matches!(edit_mode, EffectEditMode::Value) || parameter.supports_lfo()
    }

    fn displayed_edit_mode(&self) -> EffectEditMode {
        if self.mode_supported_for_parameter(self.edit_mode, self.selected_parameter()) {
            self.edit_mode
        } else {
            EffectEditMode::Value
        }
    }

    fn active_field_accepts_numeric_entry(&self) -> bool {
        match self.displayed_edit_mode() {
            EffectEditMode::Value => self.selected_parameter().value_accepts_numeric_input(),
            EffectEditMode::LfoAmplitude | EffectEditMode::LfoFrequency => true,
            EffectEditMode::LfoShape => false,
        }
    }

    fn coerce_edit_mode_for_selected(&mut self) {
        if !self.mode_supported_for_parameter(self.edit_mode, self.selected_parameter()) {
            self.edit_mode = EffectEditMode::Value;
        }
    }

    fn clear_numeric_entry(&mut self) {
        self.numeric_entry.clear();
        self.last_numeric_entry_edit_secs = None;
    }

    fn should_restart_numeric_entry(&self, now_secs: f32) -> bool {
        self.numeric_entry.should_restart_after_idle()
            && self
                .last_numeric_entry_edit_secs
                .is_some_and(|last_edit| now_secs - last_edit >= NUMERIC_ENTRY_RESTART_SECS)
    }

    fn reset_hold_states(&mut self) {
        self.select_previous_hold.reset();
        self.select_next_hold.reset();
        self.decrease_hold.reset();
        self.increase_hold.reset();
    }

    fn selection_window_bounds(&self, visible_rows: usize) -> (usize, usize) {
        let total = EffectTunerParameter::all().len();
        if total <= visible_rows {
            return (0, total);
        }

        let half_window = visible_rows / 2;
        let max_start = total - visible_rows;
        let window_start = self
            .selected_index
            .saturating_sub(half_window)
            .min(max_start);
        let window_end = (window_start + visible_rows).min(total);
        (window_start, window_end)
    }

    fn list_row_snapshot(
        &self,
        parameter: EffectTunerParameter,
        context: &EffectTunerViewContext<'_>,
        live_effects: &EffectsConfig,
    ) -> EffectTunerListRowSnapshot {
        let selected = parameter == self.selected_parameter();
        let supports_lfo = parameter.supports_lfo();
        let (effect_state_text, effect_state_emphasized) = match parameter.effect_group() {
            Some(effect) => {
                let enabled = effect.is_enabled(&self.current);
                (if enabled { "ON" } else { "OFF" }, enabled)
            }
            None => ("VAL", false),
        };
        let (lfo_state_text, lfo_state_emphasized) = if supports_lfo {
            let lfo = self
                .lfo_for_parameter(parameter)
                .expect("LFO-capable parameter should have an LFO slot");
            (if lfo.enabled { "ON" } else { "OFF" }, lfo.enabled)
        } else {
            ("--", false)
        };

        EffectTunerListRowSnapshot {
            effect_label: parameter.group_label(),
            effect_state_text,
            effect_state_emphasized,
            parameter_label: parameter.short_label(),
            value_text: if selected {
                self.overlay_numeric_text(
                    EffectOverlayField::Value,
                    self.parameter_value_text(parameter, context),
                )
            } else {
                self.parameter_value_text(parameter, context)
            },
            live_value_text: self.parameter_live_value_text(parameter, live_effects, context),
            supports_lfo,
            lfo_state_text,
            lfo_state_emphasized,
            selected,
            active_field: selected.then_some(self.active_field()),
        }
    }
}
