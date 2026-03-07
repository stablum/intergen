use std::path::Path;

use bevy::prelude::*;

use crate::config::{UiConfig, srgb, srgba};
use crate::effect_tuner::EffectTunerState;

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
pub(crate) struct EffectTunerOverlayText;

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
    effect_tuner: Res<EffectTunerState>,
    mut overlay_query: Query<&mut Visibility, With<EffectTunerOverlay>>,
    mut text_query: Query<&mut Text, With<EffectTunerOverlayText>>,
) {
    let Ok(mut visibility) = overlay_query.single_mut() else {
        return;
    };
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    *visibility = if effect_tuner.is_visible(time.elapsed_secs()) {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    *text = Text::new(effect_tuner.overlay_text());
}

pub(crate) fn load_ui_theme(asset_server: &AssetServer, ui_config: &UiConfig) -> UiTheme {
    if let Some(font_asset) = carbon_plus_font_asset(&ui_config.font_candidates) {
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
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(ui_config.hint_top),
                left: px(ui_config.hint_left),
                padding: UiRect::axes(px(ui_config.hint_padding_x), px(ui_config.hint_padding_y)),
                ..default()
            },
            BackgroundColor(srgba(ui_config.hint_background)),
            BorderRadius::MAX,
            GlobalZIndex(20),
            UiTargetCamera(scene_camera),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("F1 / H: controls"),
                ui_theme.text_font(ui_config.hint_font_size),
                TextColor(srgb(ui_config.hint_text)),
            ));
        });

    let tuner_top =
        ui_config.hint_top + ui_config.hint_font_size + ui_config.hint_padding_y * 2.0 + 14.0;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(tuner_top),
                left: px(ui_config.hint_left),
                max_width: px(ui_config.body_max_width + 110.0),
                padding: UiRect::axes(px(ui_config.hint_padding_x), px(ui_config.hint_padding_y)),
                ..default()
            },
            BackgroundColor(srgba(ui_config.panel_background)),
            BorderRadius::all(px(16.0)),
            GlobalZIndex(21),
            Visibility::Hidden,
            EffectTunerOverlay,
            UiTargetCamera(scene_camera),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                ui_theme.text_font((ui_config.body_font_size - 1.0).max(12.0)),
                TextColor(srgb(ui_config.body_text)),
                TextLayout::new_with_justify(Justify::Left),
                Node {
                    max_width: px(ui_config.body_max_width + 110.0),
                    ..default()
                },
                EffectTunerOverlayText,
            ));
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
            "F2: Pin or unpin the FX tuner\n",
            "Ctrl + Up / Down: Select FX parameter\n",
            "Ctrl + Left / Right: Adjust selected FX parameter\n",
            "Shift: Coarse FX adjustment\n",
            "Alt: Fine FX adjustment\n",
            "Enter: Reset selected FX parameter\n",
            "Shift + Enter: Reset all FX parameters\n",
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
            "C: Reset child vertex offset\n",
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
        assert!(text.contains("F2: Pin or unpin the FX tuner"));
        assert!(text.contains("Ctrl + Up / Down: Select FX parameter"));
        assert!(text.contains("Ctrl + Left / Right: Adjust selected FX parameter"));
        assert!(text.contains("Shift + Enter: Reset all FX parameters"));
        assert!(text.contains("Space: Spawn polyhedra (hold to repeat)"));
        assert!(text.contains("Backspace: Stop camera rotation momentum"));
        assert!(text.contains("R: Reset to the selected polyhedron as root"));
        assert!(text.contains("F12: Save a screenshot"));
        assert!(text.contains("4: Select dodecahedron"));
        assert!(text.contains("O / P: Adjust global opacity"));
        assert!(text.contains("I: Reset global opacity"));
        assert!(text.contains("[ / ] or , / .: Adjust child twist angle (hold to repeat)"));
        assert!(text.contains("Z / X: Adjust child vertex offset (hold to repeat)"));
        assert!(text.contains("C: Reset child vertex offset"));
        assert!(text.contains("T: Reset child twist angle"));
    }

    #[test]
    fn fallback_font_status_mentions_assets_directory() {
        let status = font_status_line(UiFontSource::Fallback);

        assert!(status.contains("assets/fonts"));
        assert!(status.contains("Carbon Plus"));
    }
}
