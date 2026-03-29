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

const EFFECT_TUNER_CHAR_WIDTH_FACTOR: f32 = 0.72;
const EFFECT_TUNER_MIN_TEXT_WIDTH: f32 = 28.0;
const EFFECT_TUNER_FIELD_PADDING_X: f32 = 10.0;
const EFFECT_TUNER_FIELD_PADDING_Y: f32 = 4.0;
// F2 values usually render as compact decimals or small integers, so keep the
// slots tight while leaving a bit of headroom for manual numeric entry.
const EFFECT_TUNER_LIVE_VALUE_CHARS: usize = 8;
const EFFECT_TUNER_NUMERIC_INPUT_CHARS: usize = 10;
const EFFECT_TUNER_LIST_VISIBLE_ROWS: usize = 9;
const EFFECT_TUNER_LIST_PANEL_MAX_WIDTH: f32 = 1060.0;
const KEYBOARD_HELP_UNUSED_TEXT: &str = "Unused in neutral mode.";
const KEYBOARD_HELP_KEY_WIDTH: f32 = 44.0;
const KEYBOARD_HELP_KEY_HEIGHT: f32 = 42.0;
const KEYBOARD_HELP_KEY_GAP: f32 = 6.0;
const KEYBOARD_HELP_KEY_BORDER: f32 = 1.5;
const KEYBOARD_HELP_PANEL_MAX_WIDTH: f32 = 980.0;
const HELP_OVERLAY_BINDING_COLUMN_WIDTH: f32 = 240.0;
const HELP_OVERLAY_ROW_BORDER: f32 = 1.0;

#[derive(Clone, Copy)]
struct KeyboardHelpKeySpec {
    label: &'static str,
    width_units: f32,
    used: bool,
    message: &'static str,
}

const KEYBOARD_FUNCTION_ROW: [KeyboardHelpKeySpec; 13] = [
    keyboard_help_key("Esc", 1.2, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("F1", 1.0, true, "Cycle the help overlay views."),
    keyboard_help_key(
        "F2",
        1.0,
        true,
        "Open compact controls, second press opens the list, third press closes.",
    ),
    keyboard_help_key("F3", 1.0, true, "Toggle the scene preset page."),
    keyboard_help_key(
        "F4",
        1.0,
        true,
        "Export the current scene as a Blender .blend.",
    ),
    keyboard_help_key("F5", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("F6", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("F7", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("F8", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("F9", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("F10", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("F11", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("F12", 1.2, true, "Save a screenshot."),
];

const KEYBOARD_NUMBER_ROW: [KeyboardHelpKeySpec; 13] = [
    keyboard_help_key("1", 1.0, true, "Select cube as the child shape."),
    keyboard_help_key("2", 1.0, true, "Select tetrahedron as the child shape."),
    keyboard_help_key("3", 1.0, true, "Select octahedron as the child shape."),
    keyboard_help_key("4", 1.0, true, "Select dodecahedron as the child shape."),
    keyboard_help_key("5", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("6", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("7", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("8", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("9", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("0", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("-", 1.0, true, "Decrease the child scale ratio."),
    keyboard_help_key("+", 1.0, true, "Increase the child scale ratio."),
    keyboard_help_key("Backspace", 2.2, true, "Stop camera rotation momentum."),
];

const KEYBOARD_TOP_LETTER_ROW: [KeyboardHelpKeySpec; 14] = [
    keyboard_help_key("Tab", 1.5, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("Q", 1.0, true, "Roll the camera left."),
    keyboard_help_key("W", 1.0, true, "Zoom in."),
    keyboard_help_key("E", 1.0, true, "Roll the camera right."),
    keyboard_help_key("R", 1.0, true, "Reset to the selected shape as root."),
    keyboard_help_key("T", 1.0, true, "Reset the child twist angle."),
    keyboard_help_key("Y", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("U", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("I", 1.0, true, "Reset global opacity."),
    keyboard_help_key("O", 1.0, true, "Decrease global opacity."),
    keyboard_help_key("P", 1.0, true, "Increase global opacity."),
    keyboard_help_key("[", 1.0, true, "Decrease the child twist angle."),
    keyboard_help_key("]", 1.0, true, "Increase the child twist angle."),
    keyboard_help_key("\\", 1.5, false, KEYBOARD_HELP_UNUSED_TEXT),
];

const KEYBOARD_HOME_ROW: [KeyboardHelpKeySpec; 13] = [
    keyboard_help_key("Caps", 1.8, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("A", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("S", 1.0, true, "Zoom out."),
    keyboard_help_key("D", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("F", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("G", 1.0, true, "Cycle the spawn placement mode."),
    keyboard_help_key("H", 1.0, true, "Cycle the help overlay views."),
    keyboard_help_key("J", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("K", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("L", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key(";", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("'", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("Enter", 2.4, false, KEYBOARD_HELP_UNUSED_TEXT),
];

const KEYBOARD_BOTTOM_LETTER_ROW: [KeyboardHelpKeySpec; 12] = [
    keyboard_help_key("Shift", 2.3, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("Z", 1.0, true, "Decrease the child outward offset."),
    keyboard_help_key("X", 1.0, true, "Increase the child outward offset."),
    keyboard_help_key("C", 1.0, true, "Reset the child outward offset."),
    keyboard_help_key("V", 1.0, true, "Decrease spawn exclusion probability."),
    keyboard_help_key("B", 1.0, true, "Increase spawn exclusion probability."),
    keyboard_help_key("N", 1.0, true, "Reset spawn exclusion probability."),
    keyboard_help_key("M", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key(",", 1.0, true, "Decrease the child twist angle."),
    keyboard_help_key(".", 1.0, true, "Increase the child twist angle."),
    keyboard_help_key("/", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key("Shift", 2.3, false, KEYBOARD_HELP_UNUSED_TEXT),
];

const KEYBOARD_SPACE_ROW: [KeyboardHelpKeySpec; 7] = [
    keyboard_help_key("Ctrl", 1.5, true, "Hold with Space to cycle the add mode."),
    keyboard_help_key("Alt", 1.2, false, KEYBOARD_HELP_UNUSED_TEXT),
    keyboard_help_key(
        "Space",
        5.8,
        true,
        "Add shapes; hold to repeat. With Ctrl, cycle the add mode.",
    ),
    keyboard_help_key("Left", 1.1, true, "Yaw the camera left."),
    keyboard_help_key("Down", 1.1, true, "Pitch the camera down."),
    keyboard_help_key("Up", 1.1, true, "Pitch the camera up."),
    keyboard_help_key("Right", 1.2, true, "Yaw the camera right."),
];

const KEYBOARD_HELP_ROWS: [&[KeyboardHelpKeySpec]; 6] = [
    &KEYBOARD_FUNCTION_ROW,
    &KEYBOARD_NUMBER_ROW,
    &KEYBOARD_TOP_LETTER_ROW,
    &KEYBOARD_HOME_ROW,
    &KEYBOARD_BOTTOM_LETTER_ROW,
    &KEYBOARD_SPACE_ROW,
];

const fn keyboard_help_key(
    label: &'static str,
    width_units: f32,
    used: bool,
    message: &'static str,
) -> KeyboardHelpKeySpec {
    KeyboardHelpKeySpec {
        label,
        width_units,
        used,
        message,
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum HelpOverlayMode {
    #[default]
    Hidden,
    Text,
    Keyboard,
}

impl HelpOverlayMode {
    fn cycle(self) -> Self {
        match self {
            Self::Hidden => Self::Text,
            Self::Text => Self::Keyboard,
            Self::Keyboard => Self::Hidden,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiFontSource {
    CarbonPlus,
    Fallback,
}

#[derive(Clone, Resource)]
pub(crate) struct UiTheme {
    font: Handle<Font>,
    pub(crate) source: UiFontSource,
}

impl UiTheme {
    pub(crate) fn text_font(&self, font_size: f32) -> TextFont {
        TextFont {
            font: self.font.clone(),
            font_size,
            ..default()
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct HelpOverlayState {
    mode: HelpOverlayMode,
}

#[derive(Component)]
pub(crate) struct HelpOverlay;

#[derive(Component)]
pub(crate) struct KeyboardHelpOverlay;

#[derive(Component)]
pub(crate) struct KeyboardHelpTooltipText;

#[derive(Component, Clone, Copy)]
pub(crate) struct KeyboardHelpKey {
    message: &'static str,
}

#[derive(Component)]
pub(crate) struct EffectTunerOverlay;

#[derive(Component)]
pub(crate) struct EffectTunerPinnedBadge;

#[derive(Component)]
pub(crate) struct EffectTunerListOverlay;

#[derive(Component)]
pub(crate) struct EffectTunerLfoSection;

#[derive(Component)]
pub(crate) struct EffectTunerListPinnedBadge;

#[derive(Component)]
pub(crate) struct EffectTunerListWindowText;

#[derive(Component)]
pub(crate) struct PresetStripOverlay;

#[derive(Component)]
pub(crate) struct PresetStripText;

#[derive(Component)]
pub(crate) struct PresetChooserOverlay;

#[derive(Component)]
pub(crate) struct PresetChooserText;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EffectTunerTextKind {
    Pin,
    EffectLabel,
    EffectState,
    ParameterLabel,
    Value,
    LiveValue,
    LfoState,
    Amplitude,
    Frequency,
    Shape,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EffectTunerEditableField(EffectOverlayField);

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EffectTunerEditableFieldText(EffectOverlayField);

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EffectTunerListRow(usize);

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EffectTunerListRowText {
    slot: usize,
    kind: EffectTunerListRowTextKind,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EffectTunerListValueField(usize);

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EffectTunerListDetailPanel(usize);

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EffectTunerListDetailField {
    slot: usize,
    field: EffectOverlayField,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EffectTunerListDetailText {
    slot: usize,
    kind: EffectTunerListDetailTextKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EffectTunerListRowTextKind {
    EffectLabel,
    EffectState,
    ParameterLabel,
    Value,
    LiveValue,
    LfoState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EffectTunerListDetailTextKind {
    State,
    Amplitude,
    Frequency,
    Shape,
}

pub(crate) fn toggle_help_overlay_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut help_overlay: ResMut<HelpOverlayState>,
    mut text_overlay_query: Query<
        &mut Visibility,
        (With<HelpOverlay>, Without<KeyboardHelpOverlay>),
    >,
    mut keyboard_overlay_query: Query<
        &mut Visibility,
        (With<KeyboardHelpOverlay>, Without<HelpOverlay>),
    >,
) {
    if !(keys.just_pressed(KeyCode::F1) || keys.just_pressed(KeyCode::KeyH)) {
        return;
    }

    help_overlay.mode = help_overlay.mode.cycle();

    let Ok(mut text_visibility) = text_overlay_query.single_mut() else {
        return;
    };
    let Ok(mut keyboard_visibility) = keyboard_overlay_query.single_mut() else {
        return;
    };

    *text_visibility = if help_overlay.mode == HelpOverlayMode::Text {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    *keyboard_visibility = if help_overlay.mode == HelpOverlayMode::Keyboard {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

pub(crate) fn update_keyboard_help_overlay_system(
    help_overlay: Res<HelpOverlayState>,
    mut tooltip_query: Query<&mut Text, With<KeyboardHelpTooltipText>>,
    mut key_query: Query<(&Interaction, &KeyboardHelpKey, &mut BackgroundColor)>,
) {
    let Ok(mut tooltip_text) = tooltip_query.single_mut() else {
        return;
    };

    let keyboard_visible = help_overlay.mode == HelpOverlayMode::Keyboard;
    let mut hovered_message = None;

    for (interaction, key, mut background) in key_query.iter_mut() {
        let hovered =
            keyboard_visible && matches!(*interaction, Interaction::Hovered | Interaction::Pressed);
        *background = if hovered {
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.14))
        } else {
            BackgroundColor(Color::NONE)
        };

        if hovered && hovered_message.is_none() {
            hovered_message = Some(key.message);
        }
    }

    *tooltip_text = Text::new(hovered_message.unwrap_or(""));
}

pub(crate) fn update_effect_tuner_overlay_system(
    time: Res<Time>,
    app_config: Res<AppConfig>,
    control_page: Res<ControlPageState>,
    effect_tuner: Res<EffectTunerState>,
    generation_state: Res<GenerationState>,
    material_state: Res<MaterialState>,
    stage_state: Res<crate::scene::StageState>,
    mut overlay_query: Query<(&mut Visibility, &mut Node), With<EffectTunerOverlay>>,
    mut pinned_badge_query: Query<
        &mut Visibility,
        (With<EffectTunerPinnedBadge>, Without<EffectTunerOverlay>),
    >,
    mut lfo_section_query: Query<
        (&mut Visibility, &mut Node),
        (
            With<EffectTunerLfoSection>,
            Without<EffectTunerOverlay>,
            Without<EffectTunerPinnedBadge>,
        ),
    >,
    mut text_query: Query<(
        &EffectTunerTextKind,
        Option<&EffectTunerEditableFieldText>,
        &mut Text,
        &mut TextColor,
    )>,
    mut field_query: Query<(&EffectTunerEditableField, &mut BackgroundColor)>,
) {
    let now_secs = time.elapsed_secs();
    let snapshot = effect_tuner.overlay_snapshot(
        &EffectTunerViewContext {
            generation_config: &app_config.generation,
            generation_state: &generation_state,
            material_config: &app_config.materials,
            material_state: &material_state,
            stage_state: &stage_state,
        },
        now_secs,
    );
    let ui_config = &app_config.ui;

    let Ok((mut overlay_visibility, mut overlay_node)) = overlay_query.single_mut() else {
        return;
    };
    let compact_visible = control_page.page_has_focus(ControlPage::EffectTuner)
        && effect_tuner.page_mode() == EffectTunerPageMode::Compact
        && effect_tuner.is_visible(now_secs);
    *overlay_visibility = if compact_visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    overlay_node.display = if compact_visible {
        Display::Flex
    } else {
        Display::None
    };

    let Ok(mut pinned_badge_visibility) = pinned_badge_query.single_mut() else {
        return;
    };
    *pinned_badge_visibility = if compact_visible && snapshot.pinned {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    let Ok((mut lfo_section_visibility, mut lfo_section_node)) = lfo_section_query.single_mut()
    else {
        return;
    };
    let lfo_visible = compact_visible && snapshot.supports_lfo;
    *lfo_section_visibility = if lfo_visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    lfo_section_node.display = if lfo_visible {
        Display::Flex
    } else {
        Display::None
    };

    for (field, mut background) in field_query.iter_mut() {
        *background = if field.0 == snapshot.active_field {
            BackgroundColor(srgba(ui_config.focus_background))
        } else {
            BackgroundColor(Color::NONE)
        };
    }

    for (text_kind, editable_field, mut text, mut text_color) in text_query.iter_mut() {
        let value = match text_kind {
            EffectTunerTextKind::Pin => "PIN".to_string(),
            EffectTunerTextKind::EffectLabel => snapshot.effect_label.to_string(),
            EffectTunerTextKind::EffectState => snapshot.effect_state_text.to_string(),
            EffectTunerTextKind::ParameterLabel => snapshot.parameter_label.to_string(),
            EffectTunerTextKind::Value => snapshot.value_text.clone(),
            EffectTunerTextKind::LiveValue => snapshot.live_value_text.clone(),
            EffectTunerTextKind::LfoState => format!("LFO {}", snapshot.lfo_state_text),
            EffectTunerTextKind::Amplitude => snapshot.amplitude_text.clone(),
            EffectTunerTextKind::Frequency => snapshot.frequency_text.clone(),
            EffectTunerTextKind::Shape => snapshot.shape_text.to_string(),
        };
        *text = Text::new(value);

        let color = if let Some(editable_field) = editable_field {
            if editable_field.0 == snapshot.active_field {
                srgb(ui_config.focus_text)
            } else {
                srgb(ui_config.body_text)
            }
        } else {
            match text_kind {
                EffectTunerTextKind::Pin => srgb(ui_config.hint_text),
                EffectTunerTextKind::EffectLabel | EffectTunerTextKind::ParameterLabel => {
                    srgb(ui_config.title_text)
                }
                EffectTunerTextKind::EffectState => {
                    if snapshot.effect_state_emphasized {
                        srgb(ui_config.title_text)
                    } else {
                        srgb(ui_config.body_text)
                    }
                }
                EffectTunerTextKind::LfoState => {
                    if snapshot.lfo_state_emphasized {
                        lfo_enabled_text_color()
                    } else {
                        srgb(ui_config.body_text)
                    }
                }
                EffectTunerTextKind::LiveValue => srgb(ui_config.title_text),
                _ => srgb(ui_config.body_text),
            }
        };
        *text_color = TextColor(color);
    }
}

pub(crate) fn update_effect_tuner_list_overlay_system(
    time: Res<Time>,
    app_config: Res<AppConfig>,
    control_page: Res<ControlPageState>,
    effect_tuner: Res<EffectTunerState>,
    generation_state: Res<GenerationState>,
    material_state: Res<MaterialState>,
    stage_state: Res<crate::scene::StageState>,
    mut overlay_query: Query<(&mut Visibility, &mut Node), With<EffectTunerListOverlay>>,
    mut pinned_badge_query: Query<
        &mut Visibility,
        (
            With<EffectTunerListPinnedBadge>,
            Without<EffectTunerListOverlay>,
        ),
    >,
    mut window_text_query: Query<
        &mut Text,
        (
            With<EffectTunerListWindowText>,
            Without<EffectTunerListRowText>,
            Without<EffectTunerListDetailText>,
        ),
    >,
    mut row_query: Query<
        (&EffectTunerListRow, &mut Visibility, &mut BackgroundColor),
        (
            Without<EffectTunerListOverlay>,
            Without<EffectTunerListPinnedBadge>,
            Without<EffectTunerListValueField>,
            Without<EffectTunerListDetailPanel>,
            Without<EffectTunerListDetailField>,
        ),
    >,
    mut row_text_query: Query<
        (&EffectTunerListRowText, &mut Text, &mut TextColor),
        (
            Without<EffectTunerListWindowText>,
            Without<EffectTunerListDetailText>,
        ),
    >,
    mut value_field_query: Query<
        (&EffectTunerListValueField, &mut BackgroundColor),
        (
            Without<EffectTunerListOverlay>,
            Without<EffectTunerListPinnedBadge>,
            Without<EffectTunerListRow>,
            Without<EffectTunerListDetailField>,
        ),
    >,
    mut detail_panel_query: Query<
        (&EffectTunerListDetailPanel, &mut Visibility, &mut Node),
        (
            Without<EffectTunerListOverlay>,
            Without<EffectTunerListPinnedBadge>,
            Without<EffectTunerListRow>,
        ),
    >,
    mut detail_field_query: Query<
        (&EffectTunerListDetailField, &mut BackgroundColor),
        (
            Without<EffectTunerListOverlay>,
            Without<EffectTunerListPinnedBadge>,
            Without<EffectTunerListRow>,
            Without<EffectTunerListValueField>,
        ),
    >,
    mut detail_text_query: Query<
        (&EffectTunerListDetailText, &mut Text, &mut TextColor),
        (
            Without<EffectTunerListWindowText>,
            Without<EffectTunerListRowText>,
        ),
    >,
) {
    let now_secs = time.elapsed_secs();
    let snapshot = effect_tuner.list_overlay_snapshot(
        &EffectTunerViewContext {
            generation_config: &app_config.generation,
            generation_state: &generation_state,
            material_config: &app_config.materials,
            material_state: &material_state,
            stage_state: &stage_state,
        },
        now_secs,
        EFFECT_TUNER_LIST_VISIBLE_ROWS,
    );
    let ui_config = &app_config.ui;
    let visible = control_page.page_has_focus(ControlPage::EffectTuner)
        && effect_tuner.page_mode() == EffectTunerPageMode::List
        && effect_tuner.is_visible(now_secs);

    let Ok((mut overlay_visibility, mut overlay_node)) = overlay_query.single_mut() else {
        return;
    };
    *overlay_visibility = if visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    overlay_node.display = if visible {
        Display::Flex
    } else {
        Display::None
    };

    let Ok(mut pinned_badge_visibility) = pinned_badge_query.single_mut() else {
        return;
    };
    *pinned_badge_visibility = if visible && snapshot.pinned {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    let Ok(mut window_text) = window_text_query.single_mut() else {
        return;
    };
    let visible_end = snapshot.window_start + snapshot.rows.len();
    *window_text = Text::new(format!(
        "LIST {}-{} / {}",
        snapshot.window_start + 1,
        visible_end,
        snapshot.total_parameters
    ));

    for (row, mut visibility, mut background) in row_query.iter_mut() {
        if let Some(row_snapshot) = snapshot.rows.get(row.0) {
            *visibility = Visibility::Visible;
            *background = if row_snapshot.selected {
                BackgroundColor(srgba(ui_config.hint_background))
            } else {
                BackgroundColor(Color::NONE)
            };
        } else {
            *visibility = Visibility::Hidden;
            *background = BackgroundColor(Color::NONE);
        }
    }

    for (text_meta, mut text, mut text_color) in row_text_query.iter_mut() {
        let Some(row_snapshot) = snapshot.rows.get(text_meta.slot) else {
            *text = Text::new("");
            *text_color = TextColor(srgb(ui_config.body_text));
            continue;
        };

        let value = match text_meta.kind {
            EffectTunerListRowTextKind::EffectLabel => row_snapshot.effect_label.to_string(),
            EffectTunerListRowTextKind::EffectState => row_snapshot.effect_state_text.to_string(),
            EffectTunerListRowTextKind::ParameterLabel => row_snapshot.parameter_label.to_string(),
            EffectTunerListRowTextKind::Value => row_snapshot.value_text.clone(),
            EffectTunerListRowTextKind::LiveValue => row_snapshot.live_value_text.clone(),
            EffectTunerListRowTextKind::LfoState => {
                if row_snapshot.supports_lfo {
                    format!("LFO {}", row_snapshot.lfo_state_text)
                } else {
                    String::new()
                }
            }
        };
        *text = Text::new(value);

        let color = match text_meta.kind {
            EffectTunerListRowTextKind::EffectLabel
            | EffectTunerListRowTextKind::ParameterLabel => srgb(ui_config.title_text),
            EffectTunerListRowTextKind::EffectState => {
                if row_snapshot.effect_state_emphasized {
                    srgb(ui_config.title_text)
                } else {
                    srgb(ui_config.body_text)
                }
            }
            EffectTunerListRowTextKind::Value => {
                if row_snapshot.active_field == Some(EffectOverlayField::Value) {
                    srgb(ui_config.focus_text)
                } else {
                    srgb(ui_config.body_text)
                }
            }
            EffectTunerListRowTextKind::LiveValue => srgb(ui_config.title_text),
            EffectTunerListRowTextKind::LfoState => {
                if row_snapshot.lfo_state_emphasized {
                    lfo_enabled_text_color()
                } else {
                    srgb(ui_config.body_text)
                }
            }
        };
        *text_color = TextColor(color);
    }

    for (field, mut background) in value_field_query.iter_mut() {
        let active = snapshot
            .rows
            .get(field.0)
            .and_then(|row| row.active_field)
            .is_some_and(|active_field| active_field == EffectOverlayField::Value);
        *background = if active {
            BackgroundColor(srgba(ui_config.focus_background))
        } else {
            BackgroundColor(Color::NONE)
        };
    }

    for (panel, mut visibility, mut node) in detail_panel_query.iter_mut() {
        let lfo_detail_visible = snapshot
            .rows
            .get(panel.0)
            .is_some_and(|row| row.selected && row.supports_lfo);
        *visibility = if lfo_detail_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        node.display = if lfo_detail_visible {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (field, mut background) in detail_field_query.iter_mut() {
        let selected_slot = snapshot
            .rows
            .get(field.slot)
            .is_some_and(|row| row.selected && row.supports_lfo);
        let active = selected_slot && snapshot.detail.active_field == field.field;
        *background = if active {
            BackgroundColor(srgba(ui_config.focus_background))
        } else {
            BackgroundColor(Color::NONE)
        };
    }

    for (text_meta, mut text, mut text_color) in detail_text_query.iter_mut() {
        let selected_slot = snapshot
            .rows
            .get(text_meta.slot)
            .is_some_and(|row| row.selected && row.supports_lfo);
        if !selected_slot {
            *text = Text::new("");
            *text_color = TextColor(srgb(ui_config.body_text));
            continue;
        }

        let value = match text_meta.kind {
            EffectTunerListDetailTextKind::State => String::new(),
            EffectTunerListDetailTextKind::Amplitude => snapshot.detail.amplitude_text.clone(),
            EffectTunerListDetailTextKind::Frequency => snapshot.detail.frequency_text.clone(),
            EffectTunerListDetailTextKind::Shape => snapshot.detail.shape_text.to_string(),
        };
        *text = Text::new(value);

        let color = match text_meta.kind {
            EffectTunerListDetailTextKind::State => srgb(ui_config.body_text),
            EffectTunerListDetailTextKind::Amplitude => {
                if snapshot.detail.active_field == EffectOverlayField::LfoAmplitude {
                    srgb(ui_config.focus_text)
                } else {
                    srgb(ui_config.body_text)
                }
            }
            EffectTunerListDetailTextKind::Frequency => {
                if snapshot.detail.active_field == EffectOverlayField::LfoFrequency {
                    srgb(ui_config.focus_text)
                } else {
                    srgb(ui_config.body_text)
                }
            }
            EffectTunerListDetailTextKind::Shape => {
                if snapshot.detail.active_field == EffectOverlayField::LfoShape {
                    srgb(ui_config.focus_text)
                } else {
                    srgb(ui_config.body_text)
                }
            }
        };
        *text_color = TextColor(color);
    }
}

pub(crate) fn update_preset_overlay_system(
    control_page: Res<ControlPageState>,
    preset_browser: Res<PresetBrowserState>,
    mut strip_visibility: Query<
        &mut Visibility,
        (With<PresetStripOverlay>, Without<PresetChooserOverlay>),
    >,
    mut strip_text: Query<&mut Text, (With<PresetStripText>, Without<PresetChooserText>)>,
    mut chooser_visibility: Query<
        &mut Visibility,
        (With<PresetChooserOverlay>, Without<PresetStripOverlay>),
    >,
    mut chooser_text: Query<&mut Text, (With<PresetChooserText>, Without<PresetStripText>)>,
) {
    let preset_page_visible = control_page.is_active(ControlPage::ScenePresets);

    let Ok(mut strip_visibility) = strip_visibility.single_mut() else {
        return;
    };
    *strip_visibility = if preset_page_visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    let Ok(mut strip_text) = strip_text.single_mut() else {
        return;
    };
    *strip_text = Text::new(preset_browser.strip_text());

    let Ok(mut chooser_visibility) = chooser_visibility.single_mut() else {
        return;
    };
    *chooser_visibility = if preset_page_visible && preset_browser.chooser_visible() {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    let Ok(mut chooser_text) = chooser_text.single_mut() else {
        return;
    };
    *chooser_text = Text::new(preset_browser.chooser_text().unwrap_or_default());
}

fn control_page_bottom(ui_config: &UiConfig) -> f32 {
    ui_config.hint_top
}

fn control_page_secondary_bottom(ui_config: &UiConfig) -> f32 {
    control_page_bottom(ui_config) + ui_config.hint_padding_y * 2.0 + ui_config.hint_font_size + 8.0
}

fn effect_tuner_text_layout(justify: Justify) -> TextLayout {
    TextLayout::new_with_justify(justify).with_no_wrap()
}

fn effect_tuner_text_width(chars: usize, font_size: f32) -> f32 {
    (chars.max(1) as f32 * font_size * EFFECT_TUNER_CHAR_WIDTH_FACTOR)
        .max(EFFECT_TUNER_MIN_TEXT_WIDTH)
}

fn effect_tuner_effect_label_chars() -> usize {
    EffectTunerParameter::all()
        .iter()
        .map(|parameter| parameter.group_label().chars().count())
        .max()
        .unwrap_or(1)
}

fn effect_tuner_parameter_label_chars() -> usize {
    EffectTunerParameter::all()
        .iter()
        .map(|parameter| parameter.short_label().chars().count())
        .max()
        .unwrap_or(1)
}

fn effect_tuner_shape_label_chars() -> usize {
    [
        "sine",
        "triangle",
        "saw",
        "square",
        "stepped random",
        "brownian motion",
    ]
    .into_iter()
    .map(str::len)
    .max()
    .unwrap_or(1)
}

fn effect_tuner_effect_label_width(font_size: f32) -> f32 {
    effect_tuner_text_width(effect_tuner_effect_label_chars(), font_size)
}

fn effect_tuner_parameter_label_width(font_size: f32) -> f32 {
    effect_tuner_text_width(effect_tuner_parameter_label_chars(), font_size)
}

fn effect_tuner_state_width(font_size: f32) -> f32 {
    effect_tuner_text_width(3, font_size)
}

fn effect_tuner_lfo_state_width(font_size: f32) -> f32 {
    effect_tuner_text_width(7, font_size)
}

fn effect_tuner_live_value_width(font_size: f32) -> f32 {
    effect_tuner_text_width(EFFECT_TUNER_LIVE_VALUE_CHARS, font_size)
}

fn effect_tuner_numeric_field_width(font_size: f32) -> f32 {
    effect_tuner_text_width(EFFECT_TUNER_NUMERIC_INPUT_CHARS, font_size)
        + EFFECT_TUNER_FIELD_PADDING_X * 2.0
}

fn effect_tuner_shape_field_width(font_size: f32) -> f32 {
    effect_tuner_text_width(effect_tuner_shape_label_chars(), font_size)
        + EFFECT_TUNER_FIELD_PADDING_X * 2.0
}

fn keyboard_help_key_width(width_units: f32) -> f32 {
    KEYBOARD_HELP_KEY_WIDTH * width_units + KEYBOARD_HELP_KEY_GAP * (width_units - 1.0).max(0.0)
}

fn keyboard_help_row_width(row: &[KeyboardHelpKeySpec]) -> f32 {
    row.iter()
        .map(|spec| keyboard_help_key_width(spec.width_units))
        .sum::<f32>()
        + KEYBOARD_HELP_KEY_GAP * row.len().saturating_sub(1) as f32
}

fn keyboard_help_block_width() -> f32 {
    KEYBOARD_HELP_ROWS
        .iter()
        .map(|row| keyboard_help_row_width(row))
        .fold(0.0, f32::max)
}

fn keyboard_help_outline_color() -> Color {
    Color::srgba(1.0, 1.0, 1.0, 0.92)
}

fn keyboard_help_active_text_color() -> Color {
    Color::srgb(1.0, 1.0, 1.0)
}

fn keyboard_help_inactive_text_color() -> Color {
    Color::srgb(0.32, 0.32, 0.32)
}

fn help_overlay_badge_border_color() -> Color {
    Color::srgba(1.0, 1.0, 1.0, 0.52)
}

fn help_overlay_row_divider_color() -> Color {
    Color::srgba(1.0, 1.0, 1.0, 0.12)
}

fn lfo_enabled_text_color() -> Color {
    Color::srgb(0.95, 0.34, 0.34)
}

fn spawn_effect_tuner_label(
    parent: &mut ChildSpawnerCommands,
    ui_theme: &UiTheme,
    font_size: f32,
    label: &'static str,
    color: Color,
) {
    parent.spawn((
        Text::new(label),
        ui_theme.text_font(font_size),
        TextColor(color),
        effect_tuner_text_layout(Justify::Left),
    ));
}

fn spawn_effect_tuner_text_slot(
    parent: &mut ChildSpawnerCommands,
    ui_theme: &UiTheme,
    font_size: f32,
    text_kind: EffectTunerTextKind,
    width: f32,
    justify: Justify,
    color: Color,
) {
    parent
        .spawn(Node {
            width: px(width),
            min_width: px(width),
            max_width: px(width),
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|slot| {
            slot.spawn((
                Text::new(""),
                ui_theme.text_font(font_size),
                TextColor(color),
                effect_tuner_text_layout(justify),
                Node {
                    width: percent(100),
                    ..default()
                },
                text_kind,
            ));
        });
}

fn spawn_effect_tuner_editable_slot(
    parent: &mut ChildSpawnerCommands,
    ui_theme: &UiTheme,
    font_size: f32,
    field: EffectOverlayField,
    text_kind: EffectTunerTextKind,
    width: f32,
    justify: Justify,
    color: Color,
) {
    parent
        .spawn((
            Node {
                width: px(width),
                min_width: px(width),
                max_width: px(width),
                padding: UiRect::axes(
                    px(EFFECT_TUNER_FIELD_PADDING_X),
                    px(EFFECT_TUNER_FIELD_PADDING_Y),
                ),
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(px(999.0)),
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(Color::NONE),
            EffectTunerEditableField(field),
        ))
        .with_children(|slot| {
            slot.spawn((
                Text::new(""),
                ui_theme.text_font(font_size),
                TextColor(color),
                effect_tuner_text_layout(justify),
                Node {
                    width: percent(100),
                    ..default()
                },
                text_kind,
                EffectTunerEditableFieldText(field),
            ));
        });
}

fn spawn_effect_tuner_list_text_slot(
    parent: &mut ChildSpawnerCommands,
    ui_theme: &UiTheme,
    font_size: f32,
    slot: usize,
    kind: EffectTunerListRowTextKind,
    width: f32,
    justify: Justify,
    color: Color,
) {
    parent
        .spawn(Node {
            width: px(width),
            min_width: px(width),
            max_width: px(width),
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|text_parent| {
            text_parent.spawn((
                Text::new(""),
                ui_theme.text_font(font_size),
                TextColor(color),
                effect_tuner_text_layout(justify),
                Node {
                    width: percent(100),
                    ..default()
                },
                EffectTunerListRowText { slot, kind },
            ));
        });
}

fn spawn_effect_tuner_list_value_slot(
    parent: &mut ChildSpawnerCommands,
    ui_theme: &UiTheme,
    font_size: f32,
    slot: usize,
    width: f32,
    color: Color,
) {
    parent
        .spawn((
            Node {
                width: px(width),
                min_width: px(width),
                max_width: px(width),
                padding: UiRect::axes(
                    px(EFFECT_TUNER_FIELD_PADDING_X),
                    px(EFFECT_TUNER_FIELD_PADDING_Y),
                ),
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(px(999.0)),
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(Color::NONE),
            EffectTunerListValueField(slot),
        ))
        .with_children(|slot_parent| {
            slot_parent.spawn((
                Text::new(""),
                ui_theme.text_font(font_size),
                TextColor(color),
                effect_tuner_text_layout(Justify::Right),
                Node {
                    width: percent(100),
                    ..default()
                },
                EffectTunerListRowText {
                    slot,
                    kind: EffectTunerListRowTextKind::Value,
                },
            ));
        });
}

fn spawn_effect_tuner_list_detail_slot(
    parent: &mut ChildSpawnerCommands,
    ui_theme: &UiTheme,
    font_size: f32,
    slot: usize,
    field: EffectOverlayField,
    kind: EffectTunerListDetailTextKind,
    width: f32,
    justify: Justify,
    color: Color,
) {
    parent
        .spawn((
            Node {
                width: px(width),
                min_width: px(width),
                max_width: px(width),
                padding: UiRect::axes(
                    px(EFFECT_TUNER_FIELD_PADDING_X),
                    px(EFFECT_TUNER_FIELD_PADDING_Y),
                ),
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(px(999.0)),
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(Color::NONE),
            EffectTunerListDetailField { slot, field },
        ))
        .with_children(|slot_parent| {
            slot_parent.spawn((
                Text::new(""),
                ui_theme.text_font(font_size),
                TextColor(color),
                effect_tuner_text_layout(justify),
                Node {
                    width: percent(100),
                    ..default()
                },
                EffectTunerListDetailText { slot, kind },
            ));
        });
}

fn spawn_help_overlay_row(
    parent: &mut ChildSpawnerCommands,
    ui_theme: &UiTheme,
    font_size: f32,
    entry: &HelpEntry,
    row_border: bool,
    ui_config: &UiConfig,
) {
    parent
        .spawn((
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                column_gap: px(18.0),
                padding: UiRect::vertical(px(7.0)),
                border: if row_border {
                    UiRect::bottom(px(HELP_OVERLAY_ROW_BORDER))
                } else {
                    UiRect::default()
                },
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(help_overlay_row_divider_color()),
        ))
        .with_children(|row| {
            row.spawn(Node {
                width: px(HELP_OVERLAY_BINDING_COLUMN_WIDTH),
                min_width: px(HELP_OVERLAY_BINDING_COLUMN_WIDTH),
                max_width: px(HELP_OVERLAY_BINDING_COLUMN_WIDTH),
                flex_shrink: 0.0,
                ..default()
            })
            .with_children(|binding_cell| {
                binding_cell
                    .spawn((
                        Node {
                            padding: UiRect::axes(px(10.0), px(5.0)),
                            border: UiRect::all(px(1.0)),
                            border_radius: BorderRadius::all(px(999.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        BorderColor::all(help_overlay_badge_border_color()),
                    ))
                    .with_children(|badge| {
                        badge.spawn((
                            Text::new(entry.binding),
                            ui_theme.text_font(font_size),
                            TextColor(srgb(ui_config.title_text)),
                            effect_tuner_text_layout(Justify::Center),
                        ));
                    });
            });

            row.spawn(Node {
                flex_grow: 1.0,
                min_width: px(0.0),
                padding: UiRect::top(px(5.0)),
                ..default()
            })
            .with_children(|explanation_cell| {
                explanation_cell.spawn((
                    Text::new(entry.explanation),
                    ui_theme.text_font(font_size),
                    TextColor(srgb(ui_config.body_text)),
                    TextLayout::new_with_justify(Justify::Left),
                    Node {
                        width: percent(100),
                        ..default()
                    },
                ));
            });
        });
}

fn spawn_keyboard_help_key(
    parent: &mut ChildSpawnerCommands,
    ui_theme: &UiTheme,
    font_size: f32,
    spec: KeyboardHelpKeySpec,
) {
    let width = keyboard_help_key_width(spec.width_units);
    let text_color = if spec.used {
        keyboard_help_active_text_color()
    } else {
        keyboard_help_inactive_text_color()
    };

    parent
        .spawn((
            Button,
            Node {
                width: px(width),
                min_width: px(width),
                max_width: px(width),
                height: px(KEYBOARD_HELP_KEY_HEIGHT),
                padding: UiRect::horizontal(px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(KEYBOARD_HELP_KEY_BORDER)),
                border_radius: BorderRadius::all(px(8.0)),
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(keyboard_help_outline_color()),
            KeyboardHelpKey {
                message: spec.message,
            },
        ))
        .with_children(|key| {
            key.spawn((
                Text::new(spec.label),
                ui_theme.text_font(font_size),
                TextColor(text_color),
                effect_tuner_text_layout(Justify::Center),
            ));
        });
}

fn spawn_keyboard_help_overlay(
    commands: &mut Commands,
    ui_theme: &UiTheme,
    scene_camera: Entity,
    ui_config: &UiConfig,
) {
    let title_color = keyboard_help_active_text_color();
    let keyboard_font_size = (ui_config.body_font_size - 1.0).max(14.0);
    let keyboard_block_width = keyboard_help_block_width();

    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                position_type: PositionType::Absolute,
                padding: UiRect::all(px(ui_config.overlay_padding)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
            GlobalZIndex(30),
            Visibility::Hidden,
            KeyboardHelpOverlay,
            UiTargetCamera(scene_camera),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: percent(100),
                        max_width: px(KEYBOARD_HELP_PANEL_MAX_WIDTH.max(ui_config.panel_max_width)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(18.0),
                        padding: UiRect::all(px(ui_config.panel_padding)),
                        border_radius: BorderRadius::all(px(ui_config.panel_radius)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Neutral Mode Keyboard"),
                        ui_theme.text_font(ui_config.title_font_size),
                        TextColor(title_color),
                    ));
                    panel.spawn((
                        Text::new("Second F1/H press opens this view. Hover a key to see what it does when no F-page is active."),
                        ui_theme.text_font((ui_config.body_font_size - 1.0).max(14.0)),
                        TextColor(srgb(ui_config.body_text)),
                        TextLayout::new_with_justify(Justify::Center),
                        Node {
                            max_width: px((ui_config.body_max_width + 220.0).max(760.0)),
                            ..default()
                        },
                    ));
                    panel
                        .spawn(Node {
                            width: px(keyboard_block_width),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexStart,
                            row_gap: px(KEYBOARD_HELP_KEY_GAP),
                            ..default()
                        })
                        .with_children(|keyboard| {
                            for row in KEYBOARD_HELP_ROWS {
                                keyboard
                                    .spawn(Node {
                                        width: percent(100),
                                        flex_direction: FlexDirection::Row,
                                        justify_content: JustifyContent::FlexStart,
                                        align_items: AlignItems::Center,
                                        column_gap: px(KEYBOARD_HELP_KEY_GAP),
                                        ..default()
                                    })
                                    .with_children(|row_parent| {
                                        for spec in row {
                                            spawn_keyboard_help_key(
                                                row_parent,
                                                ui_theme,
                                                keyboard_font_size,
                                                *spec,
                                            );
                                        }
                                    });
                            }
                        });

                    panel
                        .spawn((
                            Node {
                                width: percent(100),
                                min_height: px(78.0),
                                padding: UiRect::all(px(ui_config.hint_padding_x)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(px(12.0)),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                        ))
                        .with_children(|tooltip| {
                            tooltip.spawn((
                                Text::new(""),
                                ui_theme.text_font(ui_config.body_font_size),
                                TextColor(srgb(ui_config.body_text)),
                                TextLayout::new_with_justify(Justify::Center),
                                KeyboardHelpTooltipText,
                                Node {
                                    max_width: px((ui_config.body_max_width + 220.0).max(760.0)),
                                    ..default()
                                },
                            ));
                        });
                });
        });
}

fn spawn_effect_tuner_list_overlay(
    commands: &mut Commands,
    ui_theme: &UiTheme,
    scene_camera: Entity,
    ui_config: &UiConfig,
) {
    let header_font_size = (ui_config.hint_font_size - 1.0).max(12.0);
    let row_font_size = (ui_config.body_font_size - 1.0).max(14.0);

    commands
        .spawn((
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                left: px(ui_config.hint_left),
                right: px(ui_config.hint_left),
                bottom: px(control_page_bottom(ui_config)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            GlobalZIndex(22),
            Visibility::Hidden,
            EffectTunerListOverlay,
            UiTargetCamera(scene_camera),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: percent(100),
                        max_width: px(
                            EFFECT_TUNER_LIST_PANEL_MAX_WIDTH.max(ui_config.panel_max_width)
                        ),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8.0),
                        padding: UiRect::all(px(ui_config.panel_padding * 0.7)),
                        border_radius: BorderRadius::all(px(ui_config.panel_radius)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|panel| {
                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: px(8.0),
                            ..default()
                        })
                        .with_children(|header| {
                            spawn_effect_tuner_label(
                                header,
                                ui_theme,
                                header_font_size,
                                "CTL",
                                srgb(ui_config.body_text),
                            );
                            header
                                .spawn((
                                    Node {
                                        padding: UiRect::axes(px(7.0), px(3.0)),
                                        border_radius: BorderRadius::all(px(999.0)),
                                        ..default()
                                    },
                                    BackgroundColor(srgba(ui_config.hint_background)),
                                    Visibility::Hidden,
                                    EffectTunerListPinnedBadge,
                                ))
                                .with_children(|badge| {
                                    badge.spawn((
                                        Text::new("PIN"),
                                        ui_theme.text_font(header_font_size),
                                        TextColor(srgb(ui_config.hint_text)),
                                        effect_tuner_text_layout(Justify::Center),
                                    ));
                                });
                            header.spawn((
                                Text::new(""),
                                ui_theme.text_font(header_font_size),
                                TextColor(srgb(ui_config.title_text)),
                                effect_tuner_text_layout(Justify::Left),
                                EffectTunerListWindowText,
                            ));
                        });

                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: px(6.0),
                            ..default()
                        })
                        .with_children(|rows| {
                            for slot in 0..EFFECT_TUNER_LIST_VISIBLE_ROWS {
                                rows.spawn((
                                    Node {
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        column_gap: px(8.0),
                                        padding: UiRect::axes(px(8.0), px(4.0)),
                                        border_radius: BorderRadius::all(px(14.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::NONE),
                                    EffectTunerListRow(slot),
                                ))
                                .with_children(|row| {
                                    spawn_effect_tuner_list_text_slot(
                                        row,
                                        ui_theme,
                                        row_font_size,
                                        slot,
                                        EffectTunerListRowTextKind::EffectLabel,
                                        effect_tuner_effect_label_width(row_font_size),
                                        Justify::Left,
                                        srgb(ui_config.title_text),
                                    );
                                    spawn_effect_tuner_list_text_slot(
                                        row,
                                        ui_theme,
                                        row_font_size,
                                        slot,
                                        EffectTunerListRowTextKind::EffectState,
                                        effect_tuner_state_width(row_font_size),
                                        Justify::Center,
                                        srgb(ui_config.body_text),
                                    );
                                    spawn_effect_tuner_list_text_slot(
                                        row,
                                        ui_theme,
                                        row_font_size,
                                        slot,
                                        EffectTunerListRowTextKind::ParameterLabel,
                                        effect_tuner_parameter_label_width(row_font_size),
                                        Justify::Left,
                                        srgb(ui_config.title_text),
                                    );
                                    spawn_effect_tuner_list_value_slot(
                                        row,
                                        ui_theme,
                                        row_font_size,
                                        slot,
                                        effect_tuner_numeric_field_width(row_font_size),
                                        srgb(ui_config.body_text),
                                    );
                                    spawn_effect_tuner_list_text_slot(
                                        row,
                                        ui_theme,
                                        row_font_size,
                                        slot,
                                        EffectTunerListRowTextKind::LiveValue,
                                        effect_tuner_live_value_width(row_font_size),
                                        Justify::Right,
                                        srgb(ui_config.title_text),
                                    );
                                    spawn_effect_tuner_list_text_slot(
                                        row,
                                        ui_theme,
                                        row_font_size,
                                        slot,
                                        EffectTunerListRowTextKind::LfoState,
                                        effect_tuner_lfo_state_width(row_font_size),
                                        Justify::Center,
                                        srgb(ui_config.body_text),
                                    );

                                    row.spawn((
                                        Node {
                                            display: Display::None,
                                            flex_direction: FlexDirection::Row,
                                            align_items: AlignItems::Center,
                                            column_gap: px(6.0),
                                            margin: UiRect::left(px(4.0)),
                                            padding: UiRect::axes(px(8.0), px(4.0)),
                                            border_radius: BorderRadius::all(px(999.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::NONE),
                                        Visibility::Hidden,
                                        EffectTunerListDetailPanel(slot),
                                    ))
                                    .with_children(|detail| {
                                        detail.spawn((
                                            Text::new(""),
                                            ui_theme.text_font(header_font_size),
                                            TextColor(srgb(ui_config.body_text)),
                                            effect_tuner_text_layout(Justify::Left),
                                            EffectTunerListDetailText {
                                                slot,
                                                kind: EffectTunerListDetailTextKind::State,
                                            },
                                        ));
                                        spawn_effect_tuner_label(
                                            detail,
                                            ui_theme,
                                            header_font_size,
                                            "amp",
                                            srgb(ui_config.body_text),
                                        );
                                        spawn_effect_tuner_list_detail_slot(
                                            detail,
                                            ui_theme,
                                            header_font_size,
                                            slot,
                                            EffectOverlayField::LfoAmplitude,
                                            EffectTunerListDetailTextKind::Amplitude,
                                            effect_tuner_numeric_field_width(header_font_size),
                                            Justify::Right,
                                            srgb(ui_config.body_text),
                                        );
                                        spawn_effect_tuner_label(
                                            detail,
                                            ui_theme,
                                            header_font_size,
                                            "freq",
                                            srgb(ui_config.body_text),
                                        );
                                        spawn_effect_tuner_list_detail_slot(
                                            detail,
                                            ui_theme,
                                            header_font_size,
                                            slot,
                                            EffectOverlayField::LfoFrequency,
                                            EffectTunerListDetailTextKind::Frequency,
                                            effect_tuner_numeric_field_width(header_font_size),
                                            Justify::Right,
                                            srgb(ui_config.body_text),
                                        );
                                        spawn_effect_tuner_label(
                                            detail,
                                            ui_theme,
                                            header_font_size,
                                            "shape",
                                            srgb(ui_config.body_text),
                                        );
                                        spawn_effect_tuner_list_detail_slot(
                                            detail,
                                            ui_theme,
                                            header_font_size,
                                            slot,
                                            EffectOverlayField::LfoShape,
                                            EffectTunerListDetailTextKind::Shape,
                                            effect_tuner_shape_field_width(header_font_size),
                                            Justify::Left,
                                            srgb(ui_config.body_text),
                                        );
                                    });
                                });
                            }
                        });
                });
        });
}

fn spawn_preset_ui(
    commands: &mut Commands,
    ui_theme: &UiTheme,
    scene_camera: Entity,
    ui_config: &UiConfig,
    strip_font_size: f32,
) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(ui_config.hint_left),
                right: px(ui_config.hint_left),
                bottom: px(control_page_bottom(ui_config)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            GlobalZIndex(22),
            Visibility::Hidden,
            PresetStripOverlay,
            UiTargetCamera(scene_camera),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        padding: UiRect::axes(
                            px(ui_config.hint_padding_x),
                            px((ui_config.hint_padding_y - 1.0).max(4.0)),
                        ),
                        border_radius: BorderRadius::all(px(999.0)),
                        ..default()
                    },
                    BackgroundColor(srgba(ui_config.panel_background)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new(""),
                        ui_theme.text_font(strip_font_size),
                        TextColor(srgb(ui_config.body_text)),
                        PresetStripText,
                    ));
                });
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(ui_config.hint_left),
                right: px(ui_config.hint_left),
                bottom: px(control_page_secondary_bottom(ui_config)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            GlobalZIndex(23),
            Visibility::Hidden,
            PresetChooserOverlay,
            UiTargetCamera(scene_camera),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        max_width: px(ui_config.panel_max_width),
                        padding: UiRect::all(px(ui_config.panel_padding * 0.7)),
                        border_radius: BorderRadius::all(px(ui_config.panel_radius)),
                        ..default()
                    },
                    BackgroundColor(srgba(ui_config.panel_background)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new(""),
                        ui_theme.text_font(ui_config.body_font_size),
                        TextColor(srgb(ui_config.body_text)),
                        TextLayout::new_with_justify(Justify::Left),
                        PresetChooserText,
                    ));
                });
        });
}

pub(crate) fn load_ui_theme(asset_server: &AssetServer, ui_config: &UiConfig) -> UiTheme {
    let mut font_candidates = ui_config.font_candidates.clone();
    for fallback_candidate in UiConfig::default().font_candidates {
        if !font_candidates.contains(&fallback_candidate) {
            font_candidates.push(fallback_candidate);
        }
    }

    if let Some(font_asset) = carbon_plus_font_asset(&font_candidates) {
        return UiTheme {
            font: asset_server.load(font_asset),
            source: UiFontSource::CarbonPlus,
        };
    }

    UiTheme {
        font: default(),
        source: UiFontSource::Fallback,
    }
}

fn carbon_plus_font_asset(font_candidates: &[String]) -> Option<String> {
    font_candidates
        .iter()
        .find(|path| Path::new("assets").join(path).is_file())
        .cloned()
}

pub(crate) fn spawn_help_ui(
    commands: &mut Commands,
    ui_theme: &UiTheme,
    scene_camera: Entity,
    ui_config: &UiConfig,
) {
    let strip_font_size = (ui_config.hint_font_size - 1.0).max(12.0);

    spawn_preset_ui(commands, ui_theme, scene_camera, ui_config, strip_font_size);
    spawn_keyboard_help_overlay(commands, ui_theme, scene_camera, ui_config);
    spawn_effect_tuner_list_overlay(commands, ui_theme, scene_camera, ui_config);

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(ui_config.hint_left),
                right: px(ui_config.hint_left),
                bottom: px(control_page_bottom(ui_config)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            GlobalZIndex(21),
            Visibility::Hidden,
            EffectTunerOverlay,
            UiTargetCamera(scene_camera),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: px(8.0),
                        padding: UiRect::axes(
                            px(ui_config.hint_padding_x),
                            px((ui_config.hint_padding_y - 1.0).max(4.0)),
                        ),
                        border_radius: BorderRadius::all(px(999.0)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|strip| {
                    spawn_effect_tuner_label(
                        strip,
                        ui_theme,
                        strip_font_size,
                        "CTL",
                        srgb(ui_config.body_text),
                    );
                    strip
                        .spawn((
                            Node {
                                padding: UiRect::axes(px(7.0), px(3.0)),
                                border_radius: BorderRadius::all(px(999.0)),
                                ..default()
                            },
                            BackgroundColor(srgba(ui_config.hint_background)),
                            Visibility::Hidden,
                            EffectTunerPinnedBadge,
                        ))
                        .with_children(|badge| {
                            badge.spawn((
                                Text::new("PIN"),
                                ui_theme.text_font(strip_font_size),
                                TextColor(srgb(ui_config.hint_text)),
                                effect_tuner_text_layout(Justify::Center),
                                EffectTunerTextKind::Pin,
                            ));
                        });
                    spawn_effect_tuner_text_slot(
                        strip,
                        ui_theme,
                        strip_font_size,
                        EffectTunerTextKind::EffectLabel,
                        effect_tuner_effect_label_width(strip_font_size),
                        Justify::Left,
                        srgb(ui_config.title_text),
                    );
                    spawn_effect_tuner_text_slot(
                        strip,
                        ui_theme,
                        strip_font_size,
                        EffectTunerTextKind::EffectState,
                        effect_tuner_state_width(strip_font_size),
                        Justify::Center,
                        srgb(ui_config.body_text),
                    );
                    spawn_effect_tuner_text_slot(
                        strip,
                        ui_theme,
                        strip_font_size,
                        EffectTunerTextKind::ParameterLabel,
                        effect_tuner_parameter_label_width(strip_font_size),
                        Justify::Left,
                        srgb(ui_config.title_text),
                    );
                    spawn_effect_tuner_label(
                        strip,
                        ui_theme,
                        strip_font_size,
                        "val",
                        srgb(ui_config.body_text),
                    );
                    spawn_effect_tuner_editable_slot(
                        strip,
                        ui_theme,
                        strip_font_size,
                        EffectOverlayField::Value,
                        EffectTunerTextKind::Value,
                        effect_tuner_numeric_field_width(strip_font_size),
                        Justify::Right,
                        srgb(ui_config.body_text),
                    );
                    spawn_effect_tuner_label(
                        strip,
                        ui_theme,
                        strip_font_size,
                        "live",
                        srgb(ui_config.body_text),
                    );
                    spawn_effect_tuner_text_slot(
                        strip,
                        ui_theme,
                        strip_font_size,
                        EffectTunerTextKind::LiveValue,
                        effect_tuner_live_value_width(strip_font_size),
                        Justify::Right,
                        srgb(ui_config.title_text),
                    );
                    strip
                        .spawn((
                            Node {
                                display: Display::None,
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: px(8.0),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            Visibility::Hidden,
                            EffectTunerLfoSection,
                        ))
                        .with_children(|lfo_section| {
                            spawn_effect_tuner_text_slot(
                                lfo_section,
                                ui_theme,
                                strip_font_size,
                                EffectTunerTextKind::LfoState,
                                effect_tuner_lfo_state_width(strip_font_size),
                                Justify::Center,
                                srgb(ui_config.body_text),
                            );
                            spawn_effect_tuner_label(
                                lfo_section,
                                ui_theme,
                                strip_font_size,
                                "amp",
                                srgb(ui_config.body_text),
                            );
                            spawn_effect_tuner_editable_slot(
                                lfo_section,
                                ui_theme,
                                strip_font_size,
                                EffectOverlayField::LfoAmplitude,
                                EffectTunerTextKind::Amplitude,
                                effect_tuner_numeric_field_width(strip_font_size),
                                Justify::Right,
                                srgb(ui_config.body_text),
                            );
                            spawn_effect_tuner_label(
                                lfo_section,
                                ui_theme,
                                strip_font_size,
                                "freq",
                                srgb(ui_config.body_text),
                            );
                            spawn_effect_tuner_editable_slot(
                                lfo_section,
                                ui_theme,
                                strip_font_size,
                                EffectOverlayField::LfoFrequency,
                                EffectTunerTextKind::Frequency,
                                effect_tuner_numeric_field_width(strip_font_size),
                                Justify::Right,
                                srgb(ui_config.body_text),
                            );
                            spawn_effect_tuner_label(
                                lfo_section,
                                ui_theme,
                                strip_font_size,
                                "shape",
                                srgb(ui_config.body_text),
                            );
                            spawn_effect_tuner_editable_slot(
                                lfo_section,
                                ui_theme,
                                strip_font_size,
                                EffectOverlayField::LfoShape,
                                EffectTunerTextKind::Shape,
                                effect_tuner_shape_field_width(strip_font_size),
                                Justify::Left,
                                srgb(ui_config.body_text),
                            );
                        });
                });
        });

    let help_entries = overlay_help_entries().collect::<Vec<_>>();

    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                position_type: PositionType::Absolute,
                padding: UiRect::all(px(ui_config.overlay_padding)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            BackgroundColor(Color::NONE),
            GlobalZIndex(30),
            Visibility::Hidden,
            HelpOverlay,
            UiTargetCamera(scene_camera),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: percent(100),
                        max_width: px(ui_config.panel_max_width),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(14.0),
                        padding: UiRect::all(px(ui_config.panel_padding)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Keybindings"),
                        ui_theme.text_font(ui_config.title_font_size),
                        TextColor(srgb(ui_config.title_text)),
                    ));
                    panel.spawn((
                        Text::new(
                            "First F1/H press opens this quick reference. The left column shows the keybinding, and the right column explains what it does.",
                        ),
                        ui_theme.text_font((ui_config.body_font_size - 1.0).max(14.0)),
                        TextColor(srgb(ui_config.body_text)),
                        TextLayout::new_with_justify(Justify::Center),
                        Node {
                            max_width: px((ui_config.body_max_width + 220.0).max(760.0)),
                            ..default()
                        },
                    ));
                    panel
                        .spawn((
                            Node {
                                width: percent(100),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: px(18.0),
                                padding: UiRect::bottom(px(8.0)),
                                border: UiRect::bottom(px(HELP_OVERLAY_ROW_BORDER)),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            BorderColor::all(help_overlay_row_divider_color()),
                        ))
                        .with_children(|header| {
                            header.spawn((
                                Text::new("KEYBINDING"),
                                ui_theme.text_font((ui_config.hint_font_size - 0.5).max(12.0)),
                                TextColor(srgb(ui_config.hint_text)),
                                effect_tuner_text_layout(Justify::Left),
                                Node {
                                    width: px(HELP_OVERLAY_BINDING_COLUMN_WIDTH),
                                    min_width: px(HELP_OVERLAY_BINDING_COLUMN_WIDTH),
                                    max_width: px(HELP_OVERLAY_BINDING_COLUMN_WIDTH),
                                    flex_shrink: 0.0,
                                    ..default()
                                },
                            ));
                            header.spawn((
                                Text::new("EXPLANATION"),
                                ui_theme.text_font((ui_config.hint_font_size - 0.5).max(12.0)),
                                TextColor(srgb(ui_config.hint_text)),
                                effect_tuner_text_layout(Justify::Left),
                                Node {
                                    flex_grow: 1.0,
                                    min_width: px(0.0),
                                    ..default()
                                },
                            ));
                        });
                    panel
                        .spawn(Node {
                            width: percent(100),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Stretch,
                            row_gap: px(0.0),
                            ..default()
                        })
                        .with_children(|table| {
                            for (index, entry) in help_entries.iter().enumerate() {
                                spawn_help_overlay_row(
                                    table,
                                    ui_theme,
                                    (ui_config.body_font_size - 1.5).max(14.0),
                                    entry,
                                    index + 1 < help_entries.len(),
                                    ui_config,
                                );
                            }
                        });
                    panel.spawn((
                        Text::new(font_status_line(ui_theme.source)),
                        ui_theme.text_font((ui_config.hint_font_size - 0.5).max(12.0)),
                        TextColor(srgb(ui_config.hint_text)),
                        TextLayout::new_with_justify(Justify::Left),
                        Node {
                            width: percent(100),
                            margin: UiRect::top(px(10.0)),
                            ..default()
                        },
                    ));
                });
        });
}
#[cfg(test)]
pub(crate) fn controls_overlay_text(font_source: UiFontSource) -> String {
    shared_overlay_controls_text(font_status_line(font_source))
}

pub(crate) fn font_status_line(font_source: UiFontSource) -> &'static str {
    match font_source {
        UiFontSource::CarbonPlus => "Font: Carbon Plus",
        UiFontSource::Fallback => {
            "Font: fallback active. Add a Carbon Plus .ttf or .otf under assets/fonts/."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        HelpOverlayMode, KEYBOARD_HELP_ROWS, KEYBOARD_HOME_ROW, KEYBOARD_TOP_LETTER_ROW,
        UiFontSource, control_page_bottom, control_page_secondary_bottom, controls_overlay_text,
        effect_tuner_lfo_state_width, effect_tuner_live_value_width,
        effect_tuner_numeric_field_width, effect_tuner_parameter_label_chars,
        effect_tuner_shape_label_chars, effect_tuner_text_width, font_status_line,
    };

    #[test]
    fn overlay_text_lists_help_and_spawn_controls() {
        let text = controls_overlay_text(UiFontSource::CarbonPlus);

        assert!(text.contains("F1 / H: Cycle help views"));
        assert!(text.contains(
            "F2: Open compact controls, second press opens the list, third press closes"
        ));
        assert!(text.contains("F3: Toggle the scene preset page"));
        assert!(text.contains("Esc: Close the current control page"));
        assert!(text.contains("F4: Export the current scene as a Blender .blend"));
        assert!(text.contains("In F2 page: Ctrl + Up / Down select control"));
        assert!(text.contains("In F2 page: Second F2 press opens the scrolling parameter list"));
        assert!(
            text.contains("In F2 page: Left / Right or Tab / Shift+Tab switch the active field")
        );
        assert!(text.contains("In F2 page: Up / Down adjust the active field"));
        assert!(text.contains("In F2 page: Space toggles the selected shader effect"));
        assert!(text.contains("In F2 page: L toggles the selected parameter LFO when supported"));
        assert!(text.contains("In F2 page: Type digits / . / - / +"));
        assert!(text.contains("In F2 page: Backspace erases the typed numeric input"));
        assert!(text.contains("Shift + Enter: Reset all F2 controls"));
        assert!(text.contains(
            "In preset page: S save, Del free slot, 00-99 load, Up/Down + Enter resolve collisions"
        ));
        assert!(text.contains("Space: Add shapes using the current add mode (hold to repeat)"));
        assert!(text.contains("Ctrl + Space: Cycle add mode (single / fill current level)"));
        assert!(text.contains("G: Cycle spawn placement mode (vertex / edge / face)"));
        assert!(text.contains("Backspace: Stop camera rotation momentum"));
        assert!(text.contains("R: Reset to the selected shape as root"));
        assert!(text.contains("F12: Save a screenshot"));
        assert!(text.contains("4: Select dodecahedron"));
        assert!(text.contains("O / P: Adjust global opacity"));
        assert!(text.contains("I: Reset global opacity"));
        assert!(text.contains("[ / ] or , / .: Adjust child twist angle (hold to repeat)"));
        assert!(text.contains("Z / X: Adjust child outward offset (hold to repeat)"));
        assert!(text.contains("V / B: Adjust spawn exclusion probability (hold to repeat)"));
        assert!(text.contains("C: Reset child outward offset"));
        assert!(text.contains("N: Reset spawn exclusion probability"));
        assert!(text.contains("T: Reset child twist angle"));
    }

    #[test]
    fn help_overlay_modes_cycle_in_order() {
        assert_eq!(HelpOverlayMode::Hidden.cycle(), HelpOverlayMode::Text);
        assert_eq!(HelpOverlayMode::Text.cycle(), HelpOverlayMode::Keyboard);
        assert_eq!(HelpOverlayMode::Keyboard.cycle(), HelpOverlayMode::Hidden);
    }

    #[test]
    fn keyboard_help_rows_include_active_and_inactive_keys() {
        let specs = KEYBOARD_HELP_ROWS
            .iter()
            .flat_map(|row| row.iter())
            .collect::<Vec<_>>();

        assert!(specs.iter().any(|spec| spec.label == "F1" && spec.used));
        assert!(specs.iter().any(|spec| spec.label == "A" && !spec.used));
        assert!(specs.iter().any(|spec| spec.label == "F11" && !spec.used));
        assert!(
            KEYBOARD_TOP_LETTER_ROW
                .iter()
                .any(|spec| spec.label == "\\")
        );
        assert!(
            !KEYBOARD_TOP_LETTER_ROW
                .iter()
                .any(|spec| spec.label == "Enter")
        );
        assert!(KEYBOARD_HOME_ROW.iter().any(|spec| spec.label == "Enter"));
    }

    #[test]
    fn help_overlay_table_rows_cover_primary_bindings() {
        let entries = crate::help_text::overlay_help_entries().collect::<Vec<_>>();

        assert!(
            entries
                .iter()
                .any(|entry| entry.binding == "F1 / H" && entry.explanation.contains("Cycle"))
        );
        assert!(
            entries
                .iter()
                .any(|entry| entry.binding == "Ctrl + Up / Down"
                    && entry.explanation.contains("F2 page"))
        );
        assert!(
            entries
                .iter()
                .any(|entry| entry.binding == "S / Del / 00-99 / Up / Down + Enter")
        );
        assert!(
            entries
                .iter()
                .any(|entry| entry.binding == "Arrow Up / Down")
        );
        assert!(
            entries
                .iter()
                .any(|entry| entry.binding == "V / B"
                    && entry.explanation.contains("spawn exclusion"))
        );
    }

    #[test]
    fn control_pages_share_the_same_bottom_anchor() {
        let ui_config = crate::config::UiConfig::default();

        assert_eq!(control_page_bottom(&ui_config), ui_config.hint_top);
        assert_eq!(
            control_page_secondary_bottom(&ui_config),
            ui_config.hint_top + 38.0
        );
    }

    #[test]
    fn effect_tuner_slot_helpers_cover_the_longest_labels() {
        assert_eq!(effect_tuner_parameter_label_chars(), 10);
        assert_eq!(effect_tuner_shape_label_chars(), "brownian motion".len());
        assert!(
            effect_tuner_lfo_state_width(13.0) >= effect_tuner_text_width("LFO OFF".len(), 13.0)
        );
    }

    #[test]
    fn effect_tuner_numeric_slots_stay_compact() {
        let live_width = effect_tuner_live_value_width(13.0);
        let input_width = effect_tuner_numeric_field_width(13.0);

        assert!(live_width < 80.0);
        assert!(input_width < 120.0);
        assert!(input_width > live_width);
    }

    #[test]
    fn fallback_font_status_mentions_assets_directory() {
        let status = font_status_line(UiFontSource::Fallback);

        assert!(status.contains("assets/fonts"));
        assert!(status.contains("Carbon Plus"));
    }
}
