fn control_page_bottom(ui_config: &UiConfig) -> f32 {
    ui_config.hint_top
}

fn control_page_secondary_bottom(ui_config: &UiConfig) -> f32 {
    control_page_bottom(ui_config) + ui_config.hint_padding_y * 2.0 + ui_config.hint_font_size + 8.0
}

fn recent_changes_bottom(ui_config: &UiConfig) -> f32 {
    (control_page_bottom(ui_config) * 0.5).max(6.0)
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

fn effect_tuner_panel_fill_color() -> Color {
    Color::srgba(0.0, 0.0, 0.0, 0.72)
}

fn effect_tuner_active_field_background() -> Color {
    Color::srgba(1.0, 1.0, 1.0, 0.9)
}

fn effect_tuner_corner_radius() -> BorderRadius {
    BorderRadius::all(px(0.0))
}
