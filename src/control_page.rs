use bevy::prelude::*;

use crate::effect_tuner::EffectTunerState;
use crate::presets::PresetBrowserState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ControlPage {
    EffectTuner,
    ScenePresets,
}

impl ControlPage {
    fn captures_keyboard_focus(self) -> bool {
        true
    }

    fn closed_message(self) -> &'static str {
        match self {
            Self::EffectTuner => "FX tuner closed.",
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
            .filter(|page| page.captures_keyboard_focus())
    }

    pub(crate) fn page_has_focus(&self, page: ControlPage) -> bool {
        self.focused_page() == Some(page)
    }

    pub(crate) fn captures_scene_input(&self) -> bool {
        self.focused_page().is_some()
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

pub(crate) fn control_page_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut control_page: ResMut<ControlPageState>,
    mut effect_tuner: ResMut<EffectTunerState>,
    mut preset_browser: ResMut<PresetBrowserState>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if let Some(page) = control_page.close_active_page() {
            close_page(page, &mut effect_tuner, &mut preset_browser);
            println!("{}", page.closed_message());
            return;
        }
    }

    if keys.just_pressed(KeyCode::F2) {
        if control_page.is_active(ControlPage::EffectTuner) {
            control_page.close_active_page();
            effect_tuner.close_page();
            println!("{}", ControlPage::EffectTuner.closed_message());
            return;
        }

        if let Some(previous_page) = control_page.open_page(ControlPage::EffectTuner) {
            close_page(previous_page, &mut effect_tuner, &mut preset_browser);
        }
        effect_tuner.open_page(time.elapsed_secs());
        println!("FX tuner pinned open.");
        return;
    }

    if !keys.just_pressed(KeyCode::F3) {
        return;
    }

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

#[cfg(test)]
mod tests {
    use super::{ControlPage, ControlPageState};

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
        assert!(!state.captures_scene_input());

        state.open_page(ControlPage::EffectTuner);
        assert_eq!(state.active_page(), Some(ControlPage::EffectTuner));
        assert!(state.page_has_focus(ControlPage::EffectTuner));
        assert!(state.captures_scene_input());

        state.open_page(ControlPage::ScenePresets);
        assert_eq!(state.focused_page(), Some(ControlPage::ScenePresets));
        assert!(state.page_has_focus(ControlPage::ScenePresets));
        assert!(state.captures_scene_input());
    }
}
