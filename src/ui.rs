use std::path::Path;

use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};

use crate::config::{AppConfig, UiConfig, srgb, srgba};
use crate::control_page::{ControlPage, ControlPageState};
use crate::effect_tuner::{
    EffectOverlayField, EffectTunerPageMode, EffectTunerParameter, EffectTunerState,
    EffectTunerViewContext,
};
#[cfg(test)]
use crate::help_text::overlay_controls_text as shared_overlay_controls_text;
use crate::help_text::{HelpEntry, overlay_help_entries};
use crate::presets::PresetBrowserState;
use crate::scene::{GenerationState, MaterialState};

include!("ui/types.rs");
include!("ui/layout.rs");
include!("ui/systems.rs");
include!("ui/spawn.rs");
include!("ui/theme.rs");

#[cfg(test)]
mod tests {
    use super::*;

    include!("ui/tests.rs");
}
