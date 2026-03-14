#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum GenerationParameter {
    ChildScaleRatio,
    ChildTwistPerVertexRadians,
    ChildOutwardOffsetRatio,
    ChildSpawnExclusionProbability,
}

impl GenerationParameter {
    pub(crate) const ALL: [Self; 4] = [
        Self::ChildScaleRatio,
        Self::ChildTwistPerVertexRadians,
        Self::ChildOutwardOffsetRatio,
        Self::ChildSpawnExclusionProbability,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScalarParameterSpec {
    default_value: f32,
    min_value: f32,
    max_value: f32,
    step: f32,
    hold_delay_secs: f32,
    repeat_interval_secs: f32,
}

impl ScalarParameterSpec {
    pub(crate) fn new(
        default_value: f32,
        min_value: f32,
        max_value: f32,
        step: f32,
        hold_delay_secs: f32,
        repeat_interval_secs: f32,
    ) -> Self {
        let (min_value, max_value) = ordered_pair(min_value, max_value);
        Self {
            default_value,
            min_value,
            max_value,
            step: step.abs(),
            hold_delay_secs: hold_delay_secs.max(0.0),
            repeat_interval_secs: repeat_interval_secs.max(0.0),
        }
    }

    pub(crate) fn new_nonnegative(
        default_value: f32,
        min_value: f32,
        max_value: f32,
        step: f32,
        hold_delay_secs: f32,
        repeat_interval_secs: f32,
    ) -> Self {
        let min_value = min_value.max(0.0);
        let max_value = max_value.max(min_value);
        Self::new(
            default_value,
            min_value,
            max_value,
            step,
            hold_delay_secs,
            repeat_interval_secs,
        )
    }

    pub(crate) fn new_probability(
        default_value: f32,
        min_value: f32,
        max_value: f32,
        step: f32,
        hold_delay_secs: f32,
        repeat_interval_secs: f32,
    ) -> Self {
        let min_value = min_value.clamp(0.0, 1.0);
        let max_value = max_value.clamp(min_value, 1.0);
        Self::new(
            default_value,
            min_value,
            max_value,
            step,
            hold_delay_secs,
            repeat_interval_secs,
        )
    }

    pub(crate) fn bounds(self) -> (f32, f32) {
        (self.min_value, self.max_value)
    }

    pub(crate) fn clamp(self, value: f32) -> f32 {
        value.clamp(self.min_value, self.max_value)
    }

    pub(crate) fn default_value(self) -> f32 {
        self.clamp(self.default_value)
    }

    pub(crate) fn step(self) -> f32 {
        self.step
    }

    pub(crate) fn hold_delay_secs(self) -> f32 {
        self.hold_delay_secs
    }

    pub(crate) fn repeat_interval_secs(self) -> f32 {
        self.repeat_interval_secs
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct ScalarParameterModulation {
    additive_offset: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScalarParameterValue {
    base_value: f32,
    modulation: ScalarParameterModulation,
}

impl ScalarParameterValue {
    pub(crate) fn new(spec: ScalarParameterSpec) -> Self {
        Self {
            base_value: spec.default_value(),
            modulation: ScalarParameterModulation::default(),
        }
    }

    pub(crate) fn from_base(base_value: f32) -> Self {
        Self {
            base_value,
            modulation: ScalarParameterModulation::default(),
        }
    }

    pub(crate) fn base_value(self) -> f32 {
        self.base_value
    }

    pub(crate) fn evaluated(self, spec: ScalarParameterSpec) -> f32 {
        spec.clamp(self.base_value + self.modulation.additive_offset)
    }

    pub(crate) fn set_clamped_base_value(
        &mut self,
        base_value: f32,
        spec: ScalarParameterSpec,
    ) -> f32 {
        self.base_value = spec.clamp(base_value);
        self.base_value
    }

    pub(crate) fn adjust_clamped_base_value(
        &mut self,
        delta: f32,
        spec: ScalarParameterSpec,
    ) -> f32 {
        self.set_clamped_base_value(self.base_value + delta, spec)
    }

    pub(crate) fn reset_to_default(&mut self, spec: ScalarParameterSpec) -> f32 {
        self.set_clamped_base_value(spec.default_value(), spec)
    }

    pub(crate) fn clear_modulation(&mut self) {
        self.modulation = ScalarParameterModulation::default();
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct HoldInput {
    pub(crate) just_pressed: bool,
    pub(crate) pressed: bool,
    pub(crate) just_released: bool,
    pub(crate) delta_secs: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HoldRepeatState {
    pub(crate) elapsed_secs: f32,
    pub(crate) repeating: bool,
}

impl HoldRepeatState {
    pub(crate) fn update(
        &mut self,
        just_pressed: bool,
        pressed: bool,
        just_released: bool,
        delta_secs: f32,
        hold_delay_secs: f32,
        repeat_interval_secs: f32,
    ) -> bool {
        self.update_with_input(
            HoldInput {
                just_pressed,
                pressed,
                just_released,
                delta_secs,
            },
            hold_delay_secs,
            repeat_interval_secs,
        )
    }

    pub(crate) fn update_with_input(
        &mut self,
        input: HoldInput,
        hold_delay_secs: f32,
        repeat_interval_secs: f32,
    ) -> bool {
        if input.just_released || !input.pressed {
            self.reset();
            return false;
        }

        if input.just_pressed {
            self.reset();
            return true;
        }

        self.elapsed_secs += input.delta_secs;
        let threshold = if self.repeating {
            repeat_interval_secs
        } else {
            hold_delay_secs
        };

        if self.elapsed_secs < threshold {
            return false;
        }

        self.elapsed_secs = 0.0;
        self.repeating = true;
        true
    }

    pub(crate) fn reset(&mut self) {
        self.elapsed_secs = 0.0;
        self.repeating = false;
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ScalarParameterInputState {
    decrease_hold: HoldRepeatState,
    increase_hold: HoldRepeatState,
}

#[cfg_attr(not(test), allow(dead_code))]
impl ScalarParameterInputState {
    pub(crate) fn request_decrease(
        &mut self,
        just_pressed: bool,
        pressed: bool,
        just_released: bool,
        delta_secs: f32,
        spec: ScalarParameterSpec,
    ) -> bool {
        self.decrease_hold.update(
            just_pressed,
            pressed,
            just_released,
            delta_secs,
            spec.hold_delay_secs(),
            spec.repeat_interval_secs(),
        )
    }

    pub(crate) fn request_increase(
        &mut self,
        just_pressed: bool,
        pressed: bool,
        just_released: bool,
        delta_secs: f32,
        spec: ScalarParameterSpec,
    ) -> bool {
        self.increase_hold.update(
            just_pressed,
            pressed,
            just_released,
            delta_secs,
            spec.hold_delay_secs(),
            spec.repeat_interval_secs(),
        )
    }

    pub(crate) fn reset(&mut self) {
        self.decrease_hold.reset();
        self.increase_hold.reset();
    }

    pub(crate) fn decrease_hold(&self) -> &HoldRepeatState {
        &self.decrease_hold
    }

    pub(crate) fn increase_hold(&self) -> &HoldRepeatState {
        &self.increase_hold
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScalarParameterState {
    value: ScalarParameterValue,
    input: ScalarParameterInputState,
}

#[cfg_attr(not(test), allow(dead_code))]
impl ScalarParameterState {
    pub(crate) fn new(spec: ScalarParameterSpec) -> Self {
        Self {
            value: ScalarParameterValue::new(spec),
            input: ScalarParameterInputState::default(),
        }
    }

    pub(crate) fn from_base(base_value: f32) -> Self {
        Self {
            value: ScalarParameterValue::from_base(base_value),
            input: ScalarParameterInputState::default(),
        }
    }

    pub(crate) fn base_value(&self) -> f32 {
        self.value.base_value()
    }

    pub(crate) fn evaluated(&self, spec: ScalarParameterSpec) -> f32 {
        self.value.evaluated(spec)
    }

    pub(crate) fn adjust_clamped_base_value(
        &mut self,
        delta: f32,
        spec: ScalarParameterSpec,
    ) -> f32 {
        self.value.adjust_clamped_base_value(delta, spec)
    }

    pub(crate) fn reset_to_default(&mut self, spec: ScalarParameterSpec) -> f32 {
        self.value.reset_to_default(spec)
    }

    pub(crate) fn input_mut(&mut self) -> &mut ScalarParameterInputState {
        &mut self.input
    }

    pub(crate) fn input(&self) -> &ScalarParameterInputState {
        &self.input
    }

    pub(crate) fn clear_runtime_state(&mut self) {
        self.value.clear_modulation();
        self.input.reset();
    }
}

fn ordered_pair(left: f32, right: f32) -> (f32, f32) {
    if left <= right {
        (left, right)
    } else {
        (right, left)
    }
}

#[cfg(test)]
mod tests {
    use super::{HoldRepeatState, ScalarParameterSpec, ScalarParameterState};

    #[test]
    fn scalar_parameter_evaluation_separates_base_from_clamped_value() {
        let spec = ScalarParameterSpec::new_nonnegative(0.5, 0.0, 1.0, 0.1, 0.2, 0.05);
        let parameter = ScalarParameterState::from_base(1.4);

        assert_eq!(parameter.base_value(), 1.4);
        assert_eq!(parameter.evaluated(spec), 1.0);
    }

    #[test]
    fn hold_repeat_state_repeats_after_threshold() {
        let mut hold = HoldRepeatState::default();

        assert!(hold.update(true, true, false, 0.0, 0.2, 0.05));
        assert!(!hold.update(false, true, false, 0.1, 0.2, 0.05));
        assert!(hold.update(false, true, false, 0.1, 0.2, 0.05));
        assert!(!hold.update(false, true, false, 0.02, 0.2, 0.05));
        assert!(hold.update(false, true, false, 0.05, 0.2, 0.05));
    }
}
