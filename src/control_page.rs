use bevy::prelude::*;

use crate::effect_tuner::{EffectTunerPageMode, EffectTunerState};
use crate::presets::PresetBrowserState;
use crate::ui::HelpOverlayState;

const EFFECT_TUNER_BINDINGS: [KeyBindingPattern; 10] = [
    KeyBindingPattern::any_modifiers(KeyCode::Space),
    KeyBindingPattern::any_modifiers(KeyCode::KeyL),
    KeyBindingPattern::any_modifiers(KeyCode::Tab),
    KeyBindingPattern::any_modifiers(KeyCode::ArrowLeft),
    KeyBindingPattern::any_modifiers(KeyCode::ArrowRight),
    KeyBindingPattern::any_modifiers(KeyCode::ArrowUp),
    KeyBindingPattern::any_modifiers(KeyCode::ArrowDown),
    KeyBindingPattern::any_modifiers(KeyCode::Enter),
    KeyBindingPattern::any_modifiers(KeyCode::NumpadEnter),
    KeyBindingPattern::any_modifiers(KeyCode::Backspace),
];
const EFFECT_TUNER_NUMERIC_BINDINGS: [KeyBindingPattern; 27] = [
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::Digit0),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::Digit1),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::Digit2),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::Digit3),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::Digit4),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::Digit5),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::Digit6),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::Digit7),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::Digit8),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::Digit9),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::Numpad0),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::Numpad1),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::Numpad2),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::Numpad3),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::Numpad4),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::Numpad5),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::Numpad6),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::Numpad7),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::Numpad8),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::Numpad9),
    KeyBindingPattern::unmodified(KeyCode::Period),
    KeyBindingPattern::unmodified(KeyCode::Comma),
    KeyBindingPattern::unmodified(KeyCode::Minus),
    KeyBindingPattern::shifted(KeyCode::Equal),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::NumpadDecimal),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::NumpadSubtract),
    KeyBindingPattern::no_ctrl_or_alt(KeyCode::NumpadAdd),
];
const PRESET_PAGE_BINDINGS: [KeyBindingPattern; 22] = [
    KeyBindingPattern::any_modifiers(KeyCode::KeyS),
    KeyBindingPattern::any_modifiers(KeyCode::Delete),
    KeyBindingPattern::any_modifiers(KeyCode::Digit0),
    KeyBindingPattern::any_modifiers(KeyCode::Digit1),
    KeyBindingPattern::any_modifiers(KeyCode::Digit2),
    KeyBindingPattern::any_modifiers(KeyCode::Digit3),
    KeyBindingPattern::any_modifiers(KeyCode::Digit4),
    KeyBindingPattern::any_modifiers(KeyCode::Digit5),
    KeyBindingPattern::any_modifiers(KeyCode::Digit6),
    KeyBindingPattern::any_modifiers(KeyCode::Digit7),
    KeyBindingPattern::any_modifiers(KeyCode::Digit8),
    KeyBindingPattern::any_modifiers(KeyCode::Digit9),
    KeyBindingPattern::any_modifiers(KeyCode::Numpad0),
    KeyBindingPattern::any_modifiers(KeyCode::Numpad1),
    KeyBindingPattern::any_modifiers(KeyCode::Numpad2),
    KeyBindingPattern::any_modifiers(KeyCode::Numpad3),
    KeyBindingPattern::any_modifiers(KeyCode::Numpad4),
    KeyBindingPattern::any_modifiers(KeyCode::Numpad5),
    KeyBindingPattern::any_modifiers(KeyCode::Numpad6),
    KeyBindingPattern::any_modifiers(KeyCode::Numpad7),
    KeyBindingPattern::any_modifiers(KeyCode::Numpad8),
    KeyBindingPattern::any_modifiers(KeyCode::Numpad9),
];
const PRESET_CHOOSER_BINDINGS: [KeyBindingPattern; 3] = [
    KeyBindingPattern::any_modifiers(KeyCode::ArrowUp),
    KeyBindingPattern::any_modifiers(KeyCode::ArrowDown),
    KeyBindingPattern::any_modifiers(KeyCode::Enter),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ControlPage {
    EffectTuner,
    ScenePresets,
}

impl ControlPage {
    fn closed_message(self) -> &'static str {
        match self {
            Self::EffectTuner => "F2 control page closed.",
            Self::ScenePresets => "Scene preset page closed.",
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct ControlPageState {
    active_page: Option<ControlPage>,
}

impl ControlPageState {
    pub(crate) fn active_page(&self) -> Option<ControlPage> {
        self.active_page
    }

    pub(crate) fn is_active(&self, page: ControlPage) -> bool {
        self.active_page() == Some(page)
    }

    pub(crate) fn focused_page(&self) -> Option<ControlPage> {
        self.active_page()
    }

    pub(crate) fn page_has_focus(&self, page: ControlPage) -> bool {
        self.focused_page() == Some(page)
    }

    fn open_page(&mut self, page: ControlPage) -> Option<ControlPage> {
        if self.active_page == Some(page) {
            return None;
        }

        self.active_page.replace(page)
    }

    fn close_active_page(&mut self) -> Option<ControlPage> {
        self.active_page.take()
    }
}

#[derive(Resource, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ControlPageInputMask {
    active_page: Option<ControlPage>,
    preset_chooser_visible: bool,
}

impl ControlPageInputMask {
    fn from_states(control_page: &ControlPageState, preset_browser: &PresetBrowserState) -> Self {
        Self {
            active_page: control_page.active_page(),
            preset_chooser_visible: preset_browser.chooser_visible(),
        }
    }

    fn captures_binding(self, binding: KeyBinding) -> bool {
        match self.active_page {
            Some(ControlPage::EffectTuner) => {
                binding_captured(binding, &EFFECT_TUNER_BINDINGS)
                    || binding_captured(binding, &EFFECT_TUNER_NUMERIC_BINDINGS)
            }
            Some(ControlPage::ScenePresets) => {
                binding_captured(binding, &PRESET_PAGE_BINDINGS)
                    || (self.preset_chooser_visible
                        && binding_captured(binding, &PRESET_CHOOSER_BINDINGS))
            }
            None => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct KeyModifiers {
    shift: bool,
    control: bool,
    alt: bool,
}

impl KeyModifiers {
    fn from_keys(keys: &ButtonInput<KeyCode>) -> Self {
        Self {
            shift: modifier_pressed(keys, &[KeyCode::ShiftLeft, KeyCode::ShiftRight]),
            control: modifier_pressed(keys, &[KeyCode::ControlLeft, KeyCode::ControlRight]),
            alt: modifier_pressed(keys, &[KeyCode::AltLeft, KeyCode::AltRight]),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct KeyBinding {
    key_code: KeyCode,
    modifiers: KeyModifiers,
}

impl KeyBinding {
    fn from_keys(keys: &ButtonInput<KeyCode>, key_code: KeyCode) -> Self {
        Self {
            key_code,
            modifiers: KeyModifiers::from_keys(keys),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModifierRequirement {
    Any,
    Required,
    Forbidden,
}

impl ModifierRequirement {
    const fn matches(self, pressed: bool) -> bool {
        match self {
            Self::Any => true,
            Self::Required => pressed,
            Self::Forbidden => !pressed,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct KeyBindingPattern {
    key_code: KeyCode,
    shift: ModifierRequirement,
    control: ModifierRequirement,
    alt: ModifierRequirement,
}

impl KeyBindingPattern {
    const fn any_modifiers(key_code: KeyCode) -> Self {
        Self {
            key_code,
            shift: ModifierRequirement::Any,
            control: ModifierRequirement::Any,
            alt: ModifierRequirement::Any,
        }
    }

    const fn unmodified(key_code: KeyCode) -> Self {
        Self {
            key_code,
            shift: ModifierRequirement::Forbidden,
            control: ModifierRequirement::Forbidden,
            alt: ModifierRequirement::Forbidden,
        }
    }

    const fn shifted(key_code: KeyCode) -> Self {
        Self {
            key_code,
            shift: ModifierRequirement::Required,
            control: ModifierRequirement::Forbidden,
            alt: ModifierRequirement::Forbidden,
        }
    }

    const fn no_ctrl_or_alt(key_code: KeyCode) -> Self {
        Self {
            key_code,
            shift: ModifierRequirement::Any,
            control: ModifierRequirement::Forbidden,
            alt: ModifierRequirement::Forbidden,
        }
    }

    fn matches(self, binding: KeyBinding) -> bool {
        self.key_code == binding.key_code
            && self.shift.matches(binding.modifiers.shift)
            && self.control.matches(binding.modifiers.control)
            && self.alt.matches(binding.modifiers.alt)
    }
}

pub(crate) fn sync_control_page_input_mask_system(
    control_page: Res<ControlPageState>,
    preset_browser: Res<PresetBrowserState>,
    mut input_mask: ResMut<ControlPageInputMask>,
) {
    *input_mask = ControlPageInputMask::from_states(&control_page, &preset_browser);
}

pub(crate) fn just_pressed_unmasked(
    keys: &ButtonInput<KeyCode>,
    input_mask: ControlPageInputMask,
    key_code: KeyCode,
) -> bool {
    keys.just_pressed(key_code)
        && !input_mask.captures_binding(KeyBinding::from_keys(keys, key_code))
}

pub(crate) fn pressed_unmasked(
    keys: &ButtonInput<KeyCode>,
    input_mask: ControlPageInputMask,
    key_code: KeyCode,
) -> bool {
    keys.pressed(key_code) && !input_mask.captures_binding(KeyBinding::from_keys(keys, key_code))
}

pub(crate) fn just_released_unmasked(
    keys: &ButtonInput<KeyCode>,
    input_mask: ControlPageInputMask,
    key_code: KeyCode,
) -> bool {
    keys.just_released(key_code)
        && !input_mask.captures_binding(KeyBinding::from_keys(keys, key_code))
}

pub(crate) fn control_page_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut control_page: ResMut<ControlPageState>,
    mut help_overlay: ResMut<HelpOverlayState>,
    mut effect_tuner: ResMut<EffectTunerState>,
    mut preset_browser: ResMut<PresetBrowserState>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if let Some(page) = control_page.close_active_page() {
            close_page(page, &mut effect_tuner, &mut preset_browser);
            println!("{}", page.closed_message());
            return;
        }

        if help_overlay.is_visible() {
            help_overlay.hide();
            println!("F1 help page closed.");
            return;
        }
    }

    if keys.just_pressed(KeyCode::F1) {
        if let Some(page) = control_page.close_active_page() {
            close_page(page, &mut effect_tuner, &mut preset_browser);
            println!("{}", page.closed_message());
        }
        help_overlay.cycle();
        return;
    }

    if keys.just_pressed(KeyCode::F2) {
        help_overlay.hide();

        if control_page.is_active(ControlPage::EffectTuner) {
            match effect_tuner.page_mode() {
                EffectTunerPageMode::Compact => {
                    effect_tuner.show_list_page(time.elapsed_secs());
                    println!("F2 parameter list page pinned open.");
                }
                EffectTunerPageMode::List => {
                    control_page.close_active_page();
                    effect_tuner.close_page();
                    println!("{}", ControlPage::EffectTuner.closed_message());
                }
            }
            return;
        }

        if let Some(previous_page) = control_page.open_page(ControlPage::EffectTuner) {
            close_page(previous_page, &mut effect_tuner, &mut preset_browser);
        }
        effect_tuner.open_page(time.elapsed_secs());
        println!("F2 control page pinned open.");
        return;
    }

    if !keys.just_pressed(KeyCode::F3) {
        return;
    }

    help_overlay.hide();

    if control_page.is_active(ControlPage::ScenePresets) {
        control_page.close_active_page();
        preset_browser.close_page();
        println!("{}", ControlPage::ScenePresets.closed_message());
        return;
    }

    match preset_browser.open_page() {
        Ok(()) => {
            if let Some(previous_page) = control_page.open_page(ControlPage::ScenePresets) {
                close_page(previous_page, &mut effect_tuner, &mut preset_browser);
            }
            println!("Scene preset mode open. Type two digits to recall a slot.");
        }
        Err(error) => eprintln!("{error}"),
    }
}

fn close_page(
    page: ControlPage,
    effect_tuner: &mut EffectTunerState,
    preset_browser: &mut PresetBrowserState,
) {
    match page {
        ControlPage::EffectTuner => effect_tuner.close_page(),
        ControlPage::ScenePresets => preset_browser.close_page(),
    }
}

fn binding_captured(binding: KeyBinding, patterns: &[KeyBindingPattern]) -> bool {
    patterns
        .iter()
        .copied()
        .any(|pattern| pattern.matches(binding))
}

fn modifier_pressed(keys: &ButtonInput<KeyCode>, key_codes: &[KeyCode]) -> bool {
    key_codes
        .iter()
        .copied()
        .any(|key_code| keys.pressed(key_code))
}

#[cfg(test)]
mod tests {
    use bevy::{
        ecs::system::SystemState,
        prelude::{ButtonInput, KeyCode, Res, ResMut, Time, World},
    };

    use super::{ControlPage, ControlPageInputMask, ControlPageState, KeyBinding, KeyModifiers};
    use crate::config::EffectsConfig;
    use crate::effect_tuner::EffectTunerState;
    use crate::presets::PresetBrowserState;
    use crate::ui::HelpOverlayState;

    #[test]
    fn opening_a_page_replaces_the_previous_page() {
        let mut state = ControlPageState::default();

        assert_eq!(state.open_page(ControlPage::EffectTuner), None);
        assert!(state.is_active(ControlPage::EffectTuner));

        assert_eq!(
            state.open_page(ControlPage::ScenePresets),
            Some(ControlPage::EffectTuner)
        );
        assert!(state.is_active(ControlPage::ScenePresets));
    }

    #[test]
    fn active_page_is_the_focused_page() {
        let mut state = ControlPageState::default();

        assert_eq!(state.active_page(), None);
        assert_eq!(state.focused_page(), None);

        state.open_page(ControlPage::EffectTuner);
        assert_eq!(state.active_page(), Some(ControlPage::EffectTuner));
        assert!(state.page_has_focus(ControlPage::EffectTuner));

        state.open_page(ControlPage::ScenePresets);
        assert_eq!(state.focused_page(), Some(ControlPage::ScenePresets));
        assert!(state.page_has_focus(ControlPage::ScenePresets));
    }

    #[test]
    fn effect_tuner_mask_only_captures_its_own_bindings() {
        let mask = ControlPageInputMask {
            active_page: Some(ControlPage::EffectTuner),
            preset_chooser_visible: false,
        };

        assert!(mask.captures_binding(binding(KeyCode::Space)));
        assert!(mask.captures_binding(binding(KeyCode::ArrowUp)));
        assert!(mask.captures_binding(binding(KeyCode::Digit1)));
        assert!(mask.captures_binding(shifted_binding(KeyCode::Digit1)));
        assert!(mask.captures_binding(binding(KeyCode::Comma)));
        assert!(mask.captures_binding(binding(KeyCode::Minus)));
        assert!(mask.captures_binding(shifted_binding(KeyCode::Equal)));
        assert!(mask.captures_binding(binding(KeyCode::NumpadAdd)));

        assert!(!mask.captures_binding(binding(KeyCode::KeyG)));
        assert!(!mask.captures_binding(binding(KeyCode::KeyW)));
        assert!(!mask.captures_binding(binding(KeyCode::Equal)));
    }

    #[test]
    fn preset_page_mask_tracks_collision_chooser_bindings() {
        let preset_page_mask = ControlPageInputMask {
            active_page: Some(ControlPage::ScenePresets),
            preset_chooser_visible: false,
        };

        assert!(preset_page_mask.captures_binding(binding(KeyCode::KeyS)));
        assert!(preset_page_mask.captures_binding(binding(KeyCode::Digit4)));
        assert!(!preset_page_mask.captures_binding(binding(KeyCode::ArrowUp)));
        assert!(!preset_page_mask.captures_binding(binding(KeyCode::KeyW)));

        let chooser_mask = ControlPageInputMask {
            preset_chooser_visible: true,
            ..preset_page_mask
        };

        assert!(chooser_mask.captures_binding(binding(KeyCode::ArrowUp)));
        assert!(chooser_mask.captures_binding(binding(KeyCode::ArrowDown)));
        assert!(chooser_mask.captures_binding(binding(KeyCode::Enter)));
    }

    #[test]
    fn help_shortcut_closes_effect_tuner_before_showing_help() {
        let mut world = input_world();
        world
            .resource_mut::<ControlPageState>()
            .open_page(ControlPage::EffectTuner);
        world.resource_mut::<EffectTunerState>().open_page(0.0);
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::F1);

        run_control_page_input(&mut world);

        assert_eq!(world.resource::<ControlPageState>().active_page(), None);
        assert!(world.resource::<HelpOverlayState>().is_visible());
        assert!(!world.resource::<EffectTunerState>().is_visible(1.0));
    }

    #[test]
    fn h_key_does_not_open_help_overlay() {
        let mut world = input_world();
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyH);

        run_control_page_input(&mut world);

        assert!(!world.resource::<HelpOverlayState>().is_visible());
        assert_eq!(world.resource::<ControlPageState>().active_page(), None);
    }

    #[test]
    fn effect_tuner_shortcut_hides_help_overlay() {
        let mut world = input_world();
        world.resource_mut::<HelpOverlayState>().cycle();
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::F2);

        run_control_page_input(&mut world);

        assert!(!world.resource::<HelpOverlayState>().is_visible());
        assert_eq!(
            world.resource::<ControlPageState>().active_page(),
            Some(ControlPage::EffectTuner)
        );
        assert!(world.resource::<EffectTunerState>().is_visible(0.0));
    }

    #[test]
    fn escape_closes_help_overlay_when_no_control_page_is_active() {
        let mut world = input_world();
        world.resource_mut::<HelpOverlayState>().cycle();
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);

        run_control_page_input(&mut world);

        assert!(!world.resource::<HelpOverlayState>().is_visible());
        assert_eq!(world.resource::<ControlPageState>().active_page(), None);
    }

    fn binding(key_code: KeyCode) -> KeyBinding {
        KeyBinding {
            key_code,
            modifiers: KeyModifiers::default(),
        }
    }

    fn shifted_binding(key_code: KeyCode) -> KeyBinding {
        KeyBinding {
            key_code,
            modifiers: KeyModifiers {
                shift: true,
                ..KeyModifiers::default()
            },
        }
    }

    fn input_world() -> World {
        let mut world = World::default();
        world.insert_resource(ButtonInput::<KeyCode>::default());
        world.insert_resource::<Time>(Time::default());
        world.insert_resource(ControlPageState::default());
        world.insert_resource(HelpOverlayState::default());
        world.insert_resource(EffectTunerState::from_config(&EffectsConfig::default()));
        world.insert_resource(PresetBrowserState::default());
        world
    }

    fn run_control_page_input(world: &mut World) {
        let mut system_state = SystemState::<(
            Res<ButtonInput<KeyCode>>,
            Res<Time>,
            ResMut<ControlPageState>,
            ResMut<HelpOverlayState>,
            ResMut<EffectTunerState>,
            ResMut<PresetBrowserState>,
        )>::new(world);
        let (keys, time, control_page, help_overlay, effect_tuner, preset_browser) =
            system_state.get_mut(world);
        super::control_page_input_system(
            keys,
            time,
            control_page,
            help_overlay,
            effect_tuner,
            preset_browser,
        );
        system_state.apply(world);
    }
}
