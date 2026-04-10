use super::{
    HELP_OVERLAY_BINDING_COLUMN_WIDTH, HELP_OVERLAY_COLUMN_COUNT,
    HELP_OVERLAY_COLUMN_MIN_WIDTH, HelpOverlayMode, KEYBOARD_HELP_ROWS, KEYBOARD_HOME_ROW,
    KEYBOARD_TOP_LETTER_ROW, UiFontSource, control_page_bottom, control_page_secondary_bottom,
    controls_overlay_text, effect_tuner_lfo_state_width, effect_tuner_live_value_width,
    effect_tuner_numeric_field_width, effect_tuner_parameter_label_chars,
    effect_tuner_shape_label_chars, effect_tuner_text_width, font_status_line,
    help_overlay_binding_fragments,
};

#[test]
fn overlay_text_lists_help_and_spawn_controls() {
    let text = controls_overlay_text(UiFontSource::CarbonPlus);

    assert!(text.contains("F1: Cycle help views"));
    assert!(text.contains(
        "F2: Open parameter groups, second press opens compact controls, third press opens the full list, fourth press closes"
    ));
    assert!(text.contains("F3: Toggle the scene preset page"));
    assert!(text.contains("Esc: Close the current F-page"));
    assert!(text.contains("F4: Export the current scene as a Blender .blend"));
    assert!(text.contains("In first F2 page: Up / Down select parameter group"));
    assert!(text.contains(
        "In first F2 page: Enter or Space opens the selected group parameter list"
    ));
    assert!(text.contains("In F2 compact or list page: Ctrl + Up / Down select control"));
    assert!(text.contains(
        "In F2 page: Second F2 press opens compact controls, third press opens the full list"
    ));
    assert!(
        text.contains(
            "In F2 compact or list page: Left / Right or Tab / Shift+Tab switch the active field"
        )
    );
    assert!(text.contains("In F2 compact or list page: Up / Down adjust the active field"));
    assert!(text.contains(
        "In F2 compact or list page: Space toggles the selected shader effect"
    ));
    assert!(text.contains(
        "In F2 compact or list page: L toggles the selected parameter LFO when supported"
    ));
    assert!(text.contains("In F2 compact or list page: Type digits / . / , / - / +"));
    assert!(text.contains(
        "In F2 compact or list page: Backspace erases the typed numeric input"
    ));
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
    assert!(text.contains("[ / ]: Adjust child twist angle (hold to repeat)"));
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
fn help_overlay_binding_fragments_split_composite_shortcuts() {
    assert_eq!(
        help_overlay_binding_fragments("F1 / H"),
        vec![
            ("F1".to_string(), true),
            ("/".to_string(), false),
            ("H".to_string(), true),
        ]
    );
    assert_eq!(
        help_overlay_binding_fragments("Left / Right or Tab / Shift+Tab"),
        vec![
            ("Left".to_string(), true),
            ("/".to_string(), false),
            ("Right".to_string(), true),
            ("or".to_string(), false),
            ("Tab".to_string(), true),
            ("/".to_string(), false),
            ("Shift".to_string(), true),
            ("+".to_string(), false),
            ("Tab".to_string(), true),
        ]
    );
}

#[test]
fn help_overlay_first_panel_uses_three_balanced_columns() {
    let entries = crate::help_text::overlay_help_entries().collect::<Vec<_>>();
    let entries_per_column =
        (entries.len() + HELP_OVERLAY_COLUMN_COUNT - 1) / HELP_OVERLAY_COLUMN_COUNT;
    let column_count = entries.chunks(entries_per_column.max(1)).count();

    assert_eq!(HELP_OVERLAY_COLUMN_COUNT, 3);
    assert_eq!(column_count, HELP_OVERLAY_COLUMN_COUNT);
    assert!(HELP_OVERLAY_COLUMN_MIN_WIDTH > HELP_OVERLAY_BINDING_COLUMN_WIDTH);
}

#[test]
fn keyboard_help_rows_include_active_and_inactive_keys() {
    let specs = KEYBOARD_HELP_ROWS
        .iter()
        .flat_map(|row| row.iter())
        .collect::<Vec<_>>();

    assert!(specs.iter().any(|spec| spec.label == "F1" && spec.used));
    assert!(specs.iter().any(|spec| spec.label == "A" && !spec.used));
    assert!(specs.iter().any(|spec| spec.label == "H" && !spec.used));
    assert!(specs.iter().any(|spec| spec.label == "," && !spec.used));
    assert!(specs.iter().any(|spec| spec.label == "." && !spec.used));
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
            .any(|entry| entry.binding == "F1" && entry.explanation.contains("Cycle"))
    );
    assert!(
        entries
            .iter()
            .any(|entry| entry.binding == "Ctrl + Up / Down"
                && entry.explanation.contains("F2 compact or list page"))
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
