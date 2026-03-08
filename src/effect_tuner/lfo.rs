use std::f32::consts::TAU;

use serde::{Deserialize, Serialize};

use super::metadata::EffectNumericParameter;

pub(crate) const DEFAULT_LFO_FREQUENCY_HZ: f32 = 0.25;
pub(crate) const LFO_FREQUENCY_STEP_HZ: f32 = 0.05;
const BROWNIAN_OCTAVES: usize = 5;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LfoShape {
    Sine,
    Triangle,
    Saw,
    Square,
    SteppedRandom,
    BrownianMotion,
}

impl LfoShape {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Sine => "sine",
            Self::Triangle => "triangle",
            Self::Saw => "saw",
            Self::Square => "square",
            Self::SteppedRandom => "stepped random",
            Self::BrownianMotion => "brownian motion",
        }
    }

    pub(crate) fn sample(self, phase_cycles: f32, seed: u32) -> f32 {
        let phase = phase_cycles.rem_euclid(1.0);
        match self {
            Self::Sine => (phase * TAU).sin(),
            Self::Triangle => {
                if phase < 0.25 {
                    phase * 4.0
                } else if phase < 0.75 {
                    2.0 - phase * 4.0
                } else {
                    phase * 4.0 - 4.0
                }
            }
            Self::Saw => phase * 2.0 - 1.0,
            Self::Square => {
                if phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Self::SteppedRandom => sample_stepped_random(phase_cycles, seed),
            Self::BrownianMotion => sample_brownian_motion(phase_cycles, seed),
        }
    }

    pub(crate) fn cycle(self, direction: f32) -> Self {
        let all = [
            Self::Sine,
            Self::Triangle,
            Self::Saw,
            Self::Square,
            Self::SteppedRandom,
            Self::BrownianMotion,
        ];
        let index = all.iter().position(|shape| *shape == self).unwrap_or(0) as isize;
        let delta = if direction < 0.0 { -1 } else { 1 };
        let next_index = (index + delta).rem_euclid(all.len() as isize) as usize;
        all[next_index]
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub(crate) struct ParameterLfo {
    pub(crate) enabled: bool,
    pub(crate) shape: LfoShape,
    pub(crate) amplitude: f32,
    pub(crate) frequency_hz: f32,
}

impl ParameterLfo {
    pub(crate) fn default_for(parameter: EffectNumericParameter) -> Self {
        Self {
            enabled: false,
            shape: LfoShape::Sine,
            amplitude: parameter.default_lfo_amplitude(),
            frequency_hz: DEFAULT_LFO_FREQUENCY_HZ,
        }
    }

    pub(crate) fn is_active(self) -> bool {
        self.enabled && self.amplitude > 0.0 && self.frequency_hz > 0.0
    }
}

fn sample_stepped_random(phase_cycles: f32, seed: u32) -> f32 {
    signed_hash(phase_cycles.floor() as i32, seed)
}

fn sample_brownian_motion(phase_cycles: f32, seed: u32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut amplitude_sum = 0.0;

    for octave in 0..BROWNIAN_OCTAVES {
        let octave_seed = seed.wrapping_add((octave as u32).wrapping_mul(0x9E37_79B9));
        value += amplitude * smooth_value_noise_1d(phase_cycles * frequency, octave_seed);
        amplitude_sum += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    if amplitude_sum <= f32::EPSILON {
        0.0
    } else {
        (value / amplitude_sum).clamp(-1.0, 1.0)
    }
}

fn smooth_value_noise_1d(position: f32, seed: u32) -> f32 {
    let left_index = position.floor() as i32;
    let local_t = position - left_index as f32;
    let blend = local_t * local_t * (3.0 - 2.0 * local_t);
    let left_value = signed_hash(left_index, seed);
    let right_value = signed_hash(left_index + 1, seed);
    left_value + (right_value - left_value) * blend
}

fn signed_hash(index: i32, seed: u32) -> f32 {
    let mut state = (index as u32).wrapping_add(seed.wrapping_mul(0x85EB_CA6B));
    state ^= state >> 16;
    state = state.wrapping_mul(0x7FEB_352D);
    state ^= state >> 15;
    state = state.wrapping_mul(0x846C_A68B);
    state ^= state >> 16;
    (state as f32 / u32::MAX as f32) * 2.0 - 1.0
}

#[cfg(test)]
mod tests {
    use super::LfoShape;

    #[test]
    fn stepped_random_holds_its_value_within_a_cycle() {
        let first = LfoShape::SteppedRandom.sample(0.10, 17);
        let second = LfoShape::SteppedRandom.sample(0.90, 17);

        assert_eq!(first, second);
    }

    #[test]
    fn brownian_motion_is_bounded_and_smooth() {
        let first = LfoShape::BrownianMotion.sample(0.50, 23);
        let second = LfoShape::BrownianMotion.sample(0.52, 23);

        assert!(first.abs() <= 1.0);
        assert!(second.abs() <= 1.0);
        assert!((second - first).abs() < 0.2);
    }

    #[test]
    fn lfo_shape_cycle_includes_random_shapes() {
        assert_eq!(LfoShape::Square.cycle(1.0), LfoShape::SteppedRandom);
        assert_eq!(LfoShape::SteppedRandom.cycle(1.0), LfoShape::BrownianMotion);
        assert_eq!(LfoShape::BrownianMotion.cycle(1.0), LfoShape::Sine);
    }
}
