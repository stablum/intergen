#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EffectEditMode {
    Value,
    LfoAmplitude,
    LfoFrequency,
    LfoShape,
}

impl EffectEditMode {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Value => "value",
            Self::LfoAmplitude => "lfo amp",
            Self::LfoFrequency => "lfo freq",
            Self::LfoShape => "lfo shape",
        }
    }

    pub(crate) fn next(self) -> Self {
        match self {
            Self::Value => Self::LfoAmplitude,
            Self::LfoAmplitude => Self::LfoFrequency,
            Self::LfoFrequency => Self::LfoShape,
            Self::LfoShape => Self::Value,
        }
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
