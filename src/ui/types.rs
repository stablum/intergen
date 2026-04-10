const EFFECT_TUNER_CHAR_WIDTH_FACTOR: f32 = 0.72;
const EFFECT_TUNER_MIN_TEXT_WIDTH: f32 = 28.0;
const EFFECT_TUNER_FIELD_PADDING_X: f32 = 10.0;
const EFFECT_TUNER_FIELD_PADDING_Y: f32 = 4.0;
// F2 values usually render as compact decimals or small integers, so keep the
// slots tight while leaving a bit of headroom for manual numeric entry.
const EFFECT_TUNER_LIVE_VALUE_CHARS: usize = 8;
const EFFECT_TUNER_NUMERIC_INPUT_CHARS: usize = 10;
const EFFECT_TUNER_LIST_VISIBLE_ROWS: usize = 9;
const EFFECT_TUNER_GROUP_PANEL_MAX_WIDTH: f32 = 360.0;
const EFFECT_TUNER_LIST_PANEL_MAX_WIDTH: f32 = 1060.0;
const KEYBOARD_HELP_UNUSED_TEXT: &str = "Unused in neutral mode.";
const KEYBOARD_HELP_KEY_WIDTH: f32 = 44.0;
const KEYBOARD_HELP_KEY_HEIGHT: f32 = 42.0;
const KEYBOARD_HELP_KEY_GAP: f32 = 6.0;
const KEYBOARD_HELP_KEY_BORDER: f32 = 1.5;
const KEYBOARD_HELP_PANEL_MAX_WIDTH: f32 = 980.0;
const HELP_OVERLAY_PANEL_MAX_WIDTH: f32 = 1360.0;
const HELP_OVERLAY_COLUMN_COUNT: usize = 3;
const HELP_OVERLAY_COLUMN_MIN_WIDTH: f32 = 320.0;
const HELP_OVERLAY_BINDING_COLUMN_WIDTH: f32 = 160.0;
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
        "Open parameter groups, second press opens compact controls, third press opens the full list, fourth press closes.",
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
    keyboard_help_key("H", 1.0, false, KEYBOARD_HELP_UNUSED_TEXT),
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
    keyboard_help_key(",", 1.0, true, "Decrease single-spawn source repeat count."),
    keyboard_help_key(".", 1.0, true, "Increase single-spawn source repeat count."),
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

impl HelpOverlayState {
    pub(crate) fn is_visible(&self) -> bool {
        self.mode != HelpOverlayMode::Hidden
    }

    pub(crate) fn cycle(&mut self) {
        self.mode = self.mode.cycle();
    }

    pub(crate) fn hide(&mut self) {
        self.mode = HelpOverlayMode::Hidden;
    }
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
pub(crate) struct EffectTunerGroupOverlay;

#[derive(Component)]
pub(crate) struct EffectTunerGroupPinnedBadge;

#[derive(Component)]
pub(crate) struct EffectTunerGroupWindowText;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EffectTunerGroupRow(usize);

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EffectTunerGroupRowText(usize);

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
pub(crate) struct PresetStripCommandText;

#[derive(Component)]
pub(crate) struct PresetStripTargetText;

#[derive(Component)]
pub(crate) struct PresetStripStatusText;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PresetStripBankText {
    pub(crate) bank: u8,
    pub(crate) kind: PresetStripBankTextKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PresetStripBankTextKind {
    Label,
    Prefix,
    SelectedSlot,
    Suffix,
}

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
