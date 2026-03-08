use std::path::Path;

use bevy::prelude::*;

use crate::config::{AppConfig, UiConfig, srgb, srgba};
use crate::effect_tuner::{EffectOverlayField, EffectTunerState};
use crate::presets::PresetBrowserState;

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
    *overlay_visibility = if effect_tuner.is_visible(now_secs) {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    let Ok(mut pinned_badge_visibility) = pinned_badge_query.single_mut() else {
        return;
    };
    *pinned_badge_visibility = if snapshot.pinned {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    for (field, mut background) in field_query.iter_mut() {
        *background = if field.0 == snapshot.active_field {
            BackgroundColor(srgba(ui_config.hint_background))
        } else {
            BackgroundColor(srgba(ui_config.overlay_background))
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
                srgb(ui_config.hint_text)
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
    let Ok(mut strip_visibility) = strip_visibility.single_mut() else {
        return;
    };
    *strip_visibility = if preset_browser.is_visible() {
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
    *chooser_visibility = if preset_browser.chooser_visible() {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    let Ok(mut chooser_text) = chooser_text.single_mut() else {
        return;
    };
    *chooser_text = Text::new(preset_browser.chooser_text().unwrap_or_default());
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
                bottom: px(ui_config.hint_top + 34.0),
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
                bottom: px(ui_config.hint_top + 72.0),
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
                bottom: px(ui_config.hint_top),
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
                        column_gap: px(10.0),
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
                    strip.spawn((
                        Text::new("FX"),
                        ui_theme.text_font(strip_font_size),
                        TextColor(srgb(ui_config.body_text)),
                    ));
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
                                EffectTunerTextKind::Pin,
                            ));
                        });
                    strip.spawn((
                        Text::new(""),
                        ui_theme.text_font(strip_font_size),
                        TextColor(srgb(ui_config.title_text)),
                        EffectTunerTextKind::EffectLabel,
                    ));
                    strip.spawn((
                        Text::new(""),
                        ui_theme.text_font(strip_font_size),
                        TextColor(srgb(ui_config.body_text)),
                        EffectTunerTextKind::EffectState,
                    ));
                    strip.spawn((
                        Text::new(""),
                        ui_theme.text_font(strip_font_size),
                        TextColor(srgb(ui_config.title_text)),
                        EffectTunerTextKind::ParameterLabel,
                    ));
                    strip.spawn((
                        Text::new("val"),
                        ui_theme.text_font(strip_font_size),
                        TextColor(srgb(ui_config.body_text)),
                    ));
                    strip
                        .spawn((
                            Node {
                                padding: UiRect::axes(px(8.0), px(4.0)),
                                ..default()
                            },
                            BackgroundColor(srgba(ui_config.overlay_background)),
                            BorderRadius::all(px(999.0)),
                            EffectTunerEditableField(EffectOverlayField::Value),
                        ))
                        .with_children(|field| {
                            field.spawn((
                                Text::new(""),
                                ui_theme.text_font(strip_font_size),
                                TextColor(srgb(ui_config.body_text)),
                                EffectTunerTextKind::Value,
                                EffectTunerEditableFieldText(EffectOverlayField::Value),
                            ));
                        });
                    strip.spawn((
                        Text::new("live"),
                        ui_theme.text_font(strip_font_size),
                        TextColor(srgb(ui_config.body_text)),
                    ));
                    strip.spawn((
                        Text::new(""),
                        ui_theme.text_font(strip_font_size),
                        TextColor(srgb(ui_config.title_text)),
                        EffectTunerTextKind::LiveValue,
                    ));
                    strip.spawn((
                        Text::new("lfo"),
                        ui_theme.text_font(strip_font_size),
                        TextColor(srgb(ui_config.body_text)),
                    ));
                    strip.spawn((
                        Text::new(""),
                        ui_theme.text_font(strip_font_size),
                        TextColor(srgb(ui_config.body_text)),
                        EffectTunerTextKind::LfoState,
                    ));
                    strip.spawn((
                        Text::new("amp"),
                        ui_theme.text_font(strip_font_size),
                        TextColor(srgb(ui_config.body_text)),
                    ));
                    strip
                        .spawn((
                            Node {
                                padding: UiRect::axes(px(8.0), px(4.0)),
                                ..default()
                            },
                            BackgroundColor(srgba(ui_config.overlay_background)),
                            BorderRadius::all(px(999.0)),
                            EffectTunerEditableField(EffectOverlayField::LfoAmplitude),
                        ))
                        .with_children(|field| {
                            field.spawn((
                                Text::new(""),
                                ui_theme.text_font(strip_font_size),
                                TextColor(srgb(ui_config.body_text)),
                                EffectTunerTextKind::Amplitude,
                                EffectTunerEditableFieldText(EffectOverlayField::LfoAmplitude),
                            ));
                        });
                    strip.spawn((
                        Text::new("freq"),
                        ui_theme.text_font(strip_font_size),
                        TextColor(srgb(ui_config.body_text)),
                    ));
                    strip
                        .spawn((
                            Node {
                                padding: UiRect::axes(px(8.0), px(4.0)),
                                ..default()
                            },
                            BackgroundColor(srgba(ui_config.overlay_background)),
                            BorderRadius::all(px(999.0)),
                            EffectTunerEditableField(EffectOverlayField::LfoFrequency),
                        ))
                        .with_children(|field| {
                            field.spawn((
                                Text::new(""),
                                ui_theme.text_font(strip_font_size),
                                TextColor(srgb(ui_config.body_text)),
                                EffectTunerTextKind::Frequency,
                                EffectTunerEditableFieldText(EffectOverlayField::LfoFrequency),
                            ));
                        });
                    strip.spawn((
                        Text::new("shape"),
                        ui_theme.text_font(strip_font_size),
                        TextColor(srgb(ui_config.body_text)),
                    ));
                    strip
                        .spawn((
                            Node {
                                padding: UiRect::axes(px(8.0), px(4.0)),
                                ..default()
                            },
                            BackgroundColor(srgba(ui_config.overlay_background)),
                            BorderRadius::all(px(999.0)),
                            EffectTunerEditableField(EffectOverlayField::LfoShape),
                        ))
                        .with_children(|field| {
                            field.spawn((
                                Text::new(""),
                                ui_theme.text_font(strip_font_size),
                                TextColor(srgb(ui_config.body_text)),
                                EffectTunerTextKind::Shape,
                                EffectTunerEditableFieldText(EffectOverlayField::LfoShape),
                            ));
                        });
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
    format!(
        concat!(
            "F1 / H: Toggle this overlay\n",
            "F2: Pin or unpin the bottom FX strip\n",
            "F3: Toggle scene preset mode\n",
            "F4: Export the current scene as a Blender .blend\n",
            "Ctrl + Up / Down: Select FX parameter\n",
            "Ctrl + Left / Right: Adjust the selected FX field\n",
            "Tab: Toggle the selected effect on or off\n",
            "L: Toggle the selected parameter LFO on or off\n",
            "M: Cycle which FX value is editable (value / amp / freq / shape)\n",
            "Shift: Coarse FX adjustment\n",
            "Alt: Fine FX adjustment\n",
            "Enter: Reset the selected FX field\n",
            "Shift + Enter: Reset all FX settings and LFOs\n",
            "In preset mode: S save, Del free slot, 00-99 load, Up/Down + Enter resolve collisions\n",
            "Arrow Up / Down: Pitch camera\n",
            "Arrow Left / Right: Yaw camera\n",
            "Q / E: Roll camera\n",
            "W / S: Zoom in / out\n",
            "Backspace: Stop camera rotation momentum\n",
            "Space: Spawn polyhedra (hold to repeat)\n",
            "R: Reset to the selected polyhedron as root\n",
            "1: Select cube\n",
            "2: Select tetrahedron\n",
            "3: Select octahedron\n",
            "4: Select dodecahedron\n",
            "F12: Save a screenshot\n",
            "- / +: Adjust child scale ratio\n",
            "O / P: Adjust global opacity\n",
            "I: Reset global opacity\n",
            "[ / ] or , / .: Adjust child twist angle (hold to repeat)\n",
            "Z / X: Adjust child vertex offset (hold to repeat)\n",
            "V / B: Adjust vertex exclusion probability (hold to repeat)\n",
            "C: Reset child vertex offset\n",
            "N: Reset vertex exclusion probability\n",
            "T: Reset child twist angle\n",
            "\n",
            "{}"
        ),
        font_status_line(font_source)
    )
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
    use super::{UiFontSource, controls_overlay_text, font_status_line};

    #[test]
    fn overlay_text_lists_help_and_spawn_controls() {
        let text = controls_overlay_text(UiFontSource::CarbonPlus);

        assert!(text.contains("F1 / H: Toggle this overlay"));
        assert!(text.contains("F2: Pin or unpin the bottom FX strip"));
        assert!(text.contains("F3: Toggle scene preset mode"));
        assert!(text.contains("F4: Export the current scene as a Blender .blend"));
        assert!(text.contains("Ctrl + Up / Down: Select FX parameter"));
        assert!(text.contains("Ctrl + Left / Right: Adjust the selected FX field"));
        assert!(text.contains("Tab: Toggle the selected effect on or off"));
        assert!(text.contains("L: Toggle the selected parameter LFO on or off"));
        assert!(text.contains("M: Cycle which FX value is editable (value / amp / freq / shape)"));
        assert!(text.contains("Shift + Enter: Reset all FX settings and LFOs"));
        assert!(text.contains(
            "In preset mode: S save, Del free slot, 00-99 load, Up/Down + Enter resolve collisions"
        ));
        assert!(text.contains("Space: Spawn polyhedra (hold to repeat)"));
        assert!(text.contains("Backspace: Stop camera rotation momentum"));
        assert!(text.contains("R: Reset to the selected polyhedron as root"));
        assert!(text.contains("F12: Save a screenshot"));
        assert!(text.contains("4: Select dodecahedron"));
        assert!(text.contains("O / P: Adjust global opacity"));
        assert!(text.contains("I: Reset global opacity"));
        assert!(text.contains("[ / ] or , / .: Adjust child twist angle (hold to repeat)"));
        assert!(text.contains("Z / X: Adjust child vertex offset (hold to repeat)"));
        assert!(text.contains("V / B: Adjust vertex exclusion probability (hold to repeat)"));
        assert!(text.contains("C: Reset child vertex offset"));
        assert!(text.contains("N: Reset vertex exclusion probability"));
        assert!(text.contains("T: Reset child twist angle"));
    }

    #[test]
    fn fallback_font_status_mentions_assets_directory() {
        let status = font_status_line(UiFontSource::Fallback);

        assert!(status.contains("assets/fonts"));
        assert!(status.contains("Carbon Plus"));
    }
}
