#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EffectEditMode {
    Value,
    LfoAmplitude,
    LfoFrequency,
    LfoShape,
}

impl EffectEditMode {
    const ALL: [Self; 4] = [
        Self::Value,
        Self::LfoAmplitude,
        Self::LfoFrequency,
        Self::LfoShape,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Value => "value",
            Self::LfoAmplitude => "lfo amp",
            Self::LfoFrequency => "lfo freq",
            Self::LfoShape => "lfo shape",
        }
    }

    pub(crate) fn step(self, direction: isize) -> Self {
        let current_index = Self::ALL.iter().position(|mode| *mode == self).unwrap_or(0) as isize;
        let next_index = (current_index + direction).rem_euclid(Self::ALL.len() as isize) as usize;
        Self::ALL[next_index]
    }

    pub(crate) fn overlay_field(self) -> EffectOverlayField {
        match self {
            Self::Value => EffectOverlayField::Value,
            Self::LfoAmplitude => EffectOverlayField::LfoAmplitude,
            Self::LfoFrequency => EffectOverlayField::LfoFrequency,
            Self::LfoShape => EffectOverlayField::LfoShape,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EffectOverlayField {
    Value,
    LfoAmplitude,
    LfoFrequency,
    LfoShape,
}
