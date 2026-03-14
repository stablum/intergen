use std::path::Path;

use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};

use crate::config::{AppConfig, EffectNumericParameter, UiConfig, srgb, srgba};
use crate::control_page::{ControlPage, ControlPageState};
use crate::effect_tuner::{EffectOverlayField, EffectTunerState};
use crate::help_text::overlay_controls_text as shared_overlay_controls_text;
use crate::presets::PresetBrowserState;

const EFFECT_TUNER_CHAR_WIDTH_FACTOR: f32 = 0.72;
const EFFECT_TUNER_MIN_TEXT_WIDTH: f32 = 28.0;
const EFFECT_TUNER_FIELD_PADDING_X: f32 = 10.0;
const EFFECT_TUNER_FIELD_PADDING_Y: f32 = 4.0;
const EFFECT_TUNER_NUMERIC_SLOT_CHARS: usize = 12;

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
    visible: bool,
}

#[derive(Component)]
pub(crate) struct HelpOverlay;

#[derive(Component)]
pub(crate) struct EffectTunerOverlay;

#[derive(Component)]
pub(crate) struct EffectTunerPinnedBadge;

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

pub(crate) fn toggle_help_overlay_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut help_overlay: ResMut<HelpOverlayState>,
    mut overlay_query: Query<&mut Visibility, With<HelpOverlay>>,
) {
    if !(keys.just_pressed(KeyCode::F1) || keys.just_pressed(KeyCode::KeyH)) {
        return;
    }

    help_overlay.visible = !help_overlay.visible;

    let Ok(mut visibility) = overlay_query.single_mut() else {
        return;
    };

    *visibility = if help_overlay.visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

pub(crate) fn update_effect_tuner_overlay_system(
    time: Res<Time>,
    app_config: Res<AppConfig>,
    control_page: Res<ControlPageState>,
    effect_tuner: Res<EffectTunerState>,
    mut overlay_query: Query<&mut Visibility, With<EffectTunerOverlay>>,
    mut pinned_badge_query: Query<
        &mut Visibility,
        (With<EffectTunerPinnedBadge>, Without<EffectTunerOverlay>),
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
    let snapshot = effect_tuner.overlay_snapshot(now_secs);
    let ui_config = &app_config.ui;

    let Ok(mut overlay_visibility) = overlay_query.single_mut() else {
        return;
    };
    *overlay_visibility = if control_page.page_has_focus(ControlPage::EffectTuner)
        && effect_tuner.is_visible(now_secs)
    {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    let Ok(mut pinned_badge_visibility) = pinned_badge_query.single_mut() else {
        return;
    };
    *pinned_badge_visibility =
        if control_page.page_has_focus(ControlPage::EffectTuner) && snapshot.pinned {
            Visibility::Visible
        } else {
            Visibility::Hidden
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
            EffectTunerTextKind::EffectState => {
                if snapshot.effect_enabled { "ON" } else { "OFF" }.to_string()
            }
            EffectTunerTextKind::ParameterLabel => snapshot.parameter_label.to_string(),
            EffectTunerTextKind::Value => snapshot.value_text.clone(),
            EffectTunerTextKind::LiveValue => snapshot.live_value_text.clone(),
            EffectTunerTextKind::LfoState => {
                if snapshot.lfo_enabled { "ON" } else { "OFF" }.to_string()
            }
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
                    if snapshot.effect_enabled {
                        srgb(ui_config.title_text)
                    } else {
                        srgb(ui_config.body_text)
                    }
                }
                EffectTunerTextKind::LfoState => {
                    if snapshot.lfo_enabled {
                        srgb(ui_config.title_text)
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
    ["wavefolder", "lens", "blur", "bloom", "edge"]
        .into_iter()
        .map(str::len)
        .max()
        .unwrap_or(1)
}

fn effect_tuner_parameter_label_chars() -> usize {
    EffectNumericParameter::all()
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

fn effect_tuner_live_value_width(font_size: f32) -> f32 {
    effect_tuner_text_width(EFFECT_TUNER_NUMERIC_SLOT_CHARS, font_size)
}

fn effect_tuner_numeric_field_width(font_size: f32) -> f32 {
    effect_tuner_live_value_width(font_size) + EFFECT_TUNER_FIELD_PADDING_X * 2.0
}

fn effect_tuner_shape_field_width(font_size: f32) -> f32 {
    effect_tuner_text_width(effect_tuner_shape_label_chars(), font_size)
        + EFFECT_TUNER_FIELD_PADDING_X * 2.0
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
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderRadius::all(px(999.0)),
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
                        ..default()
                    },
                    BackgroundColor(srgba(ui_config.panel_background)),
                    BorderRadius::all(px(999.0)),
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
                        ..default()
                    },
                    BackgroundColor(srgba(ui_config.panel_background)),
                    BorderRadius::all(px(ui_config.panel_radius)),
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
                        ..default()
                    },
                    BackgroundColor(srgba(ui_config.panel_background)),
                    BorderRadius::all(px(999.0)),
                ))
                .with_children(|strip| {
                    spawn_effect_tuner_label(
                        strip,
                        ui_theme,
                        strip_font_size,
                        "FX",
                        srgb(ui_config.body_text),
                    );
                    strip
                        .spawn((
                            Node {
                                padding: UiRect::axes(px(7.0), px(3.0)),
                                ..default()
                            },
                            BackgroundColor(srgba(ui_config.hint_background)),
                            BorderRadius::all(px(999.0)),
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
                    spawn_effect_tuner_label(
                        strip,
                        ui_theme,
                        strip_font_size,
                        "lfo",
                        srgb(ui_config.body_text),
                    );
                    spawn_effect_tuner_text_slot(
                        strip,
                        ui_theme,
                        strip_font_size,
                        EffectTunerTextKind::LfoState,
                        effect_tuner_state_width(strip_font_size),
                        Justify::Center,
                        srgb(ui_config.body_text),
                    );
                    spawn_effect_tuner_label(
                        strip,
                        ui_theme,
                        strip_font_size,
                        "amp",
                        srgb(ui_config.body_text),
                    );
                    spawn_effect_tuner_editable_slot(
                        strip,
                        ui_theme,
                        strip_font_size,
                        EffectOverlayField::LfoAmplitude,
                        EffectTunerTextKind::Amplitude,
                        effect_tuner_numeric_field_width(strip_font_size),
                        Justify::Right,
                        srgb(ui_config.body_text),
                    );
                    spawn_effect_tuner_label(
                        strip,
                        ui_theme,
                        strip_font_size,
                        "freq",
                        srgb(ui_config.body_text),
                    );
                    spawn_effect_tuner_editable_slot(
                        strip,
                        ui_theme,
                        strip_font_size,
                        EffectOverlayField::LfoFrequency,
                        EffectTunerTextKind::Frequency,
                        effect_tuner_numeric_field_width(strip_font_size),
                        Justify::Right,
                        srgb(ui_config.body_text),
                    );
                    spawn_effect_tuner_label(
                        strip,
                        ui_theme,
                        strip_font_size,
                        "shape",
                        srgb(ui_config.body_text),
                    );
                    spawn_effect_tuner_editable_slot(
                        strip,
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
            BackgroundColor(srgba(ui_config.overlay_background)),
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
                        row_gap: px(ui_config.panel_row_gap),
                        padding: UiRect::all(px(ui_config.panel_padding)),
                        ..default()
                    },
                    BackgroundColor(srgba(ui_config.panel_background)),
                    BorderRadius::all(px(ui_config.panel_radius)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Keybindings"),
                        ui_theme.text_font(ui_config.title_font_size),
                        TextColor(srgb(ui_config.title_text)),
                    ));
                    panel.spawn((
                        Text::new(controls_overlay_text(ui_theme.source)),
                        ui_theme.text_font(ui_config.body_font_size),
                        TextColor(srgb(ui_config.body_text)),
                        TextLayout::new_with_justify(Justify::Left),
                        Node {
                            max_width: px(ui_config.body_max_width),
                            ..default()
                        },
                    ));
                });
        });
}
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
        UiFontSource, control_page_bottom, control_page_secondary_bottom, controls_overlay_text,
        effect_tuner_parameter_label_chars, effect_tuner_shape_label_chars, font_status_line,
    };

    #[test]
    fn overlay_text_lists_help_and_spawn_controls() {
        let text = controls_overlay_text(UiFontSource::CarbonPlus);

        assert!(text.contains("F1 / H: Toggle this overlay"));
        assert!(text.contains("F2: Toggle the FX control page"));
        assert!(text.contains("F3: Toggle the scene preset page"));
        assert!(text.contains("Esc: Close the current control page"));
        assert!(text.contains("F4: Export the current scene as a Blender .blend"));
        assert!(text.contains("In FX page: Ctrl + Up / Down select parameter"));
        assert!(
            text.contains("In FX page: Left / Right or Tab / Shift+Tab switch the active field")
        );
        assert!(text.contains("In FX page: Up / Down adjust the active field"));
        assert!(text.contains("In FX page: Space toggles the selected effect"));
        assert!(text.contains("In FX page: L toggles the selected parameter LFO"));
        assert!(text.contains("In FX page: Type digits / . / - / +"));
        assert!(text.contains("In FX page: Backspace erases the typed numeric input"));
        assert!(text.contains("Shift + Enter: Reset all FX settings and LFOs"));
        assert!(text.contains(
            "In preset page: S save, Del free slot, 00-99 load, Up/Down + Enter resolve collisions"
        ));
        assert!(text.contains("Space: Add polyhedra using the current add mode (hold to repeat)"));
        assert!(text.contains("Ctrl + Space: Cycle add mode (single / fill current level)"));
        assert!(text.contains("G: Cycle spawn placement mode (vertex / edge / face)"));
        assert!(text.contains("Backspace: Stop camera rotation momentum"));
        assert!(text.contains("R: Reset to the selected polyhedron as root"));
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
        assert_eq!(effect_tuner_parameter_label_chars(), 9);
        assert_eq!(effect_tuner_shape_label_chars(), "brownian motion".len());
    }

    #[test]
    fn fallback_font_status_mentions_assets_directory() {
        let status = font_status_line(UiFontSource::Fallback);

        assert!(status.contains("assets/fonts"));
        assert!(status.contains("Carbon Plus"));
    }
}
