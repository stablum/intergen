use std::path::Path;

use bevy::prelude::*;

const CARBON_PLUS_FONT_CANDIDATES: &[&str] = &[
    "fonts/CarbonPlus-Regular.ttf",
    "fonts/CarbonPlus-Regular.otf",
    "fonts/Carbon Plus Regular.ttf",
    "fonts/Carbon Plus Regular.otf",
    "fonts/CarbonPlus.ttf",
    "fonts/Carbon Plus.ttf",
];

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

pub(crate) fn load_ui_theme(asset_server: &AssetServer) -> UiTheme {
    if let Some(font_asset) = carbon_plus_font_asset() {
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

fn carbon_plus_font_asset() -> Option<&'static str> {
    CARBON_PLUS_FONT_CANDIDATES
        .iter()
        .copied()
        .find(|path| Path::new("assets").join(path).is_file())
}

pub(crate) fn spawn_help_ui(commands: &mut Commands, ui_theme: &UiTheme, scene_camera: Entity) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(18),
                left: px(18),
                padding: UiRect::axes(px(12), px(8)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.08, 0.13, 0.86)),
            BorderRadius::MAX,
            GlobalZIndex(20),
            UiTargetCamera(scene_camera),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("F1 / H: controls"),
                ui_theme.text_font(14.0),
                TextColor(Color::srgb(0.93, 0.95, 0.99)),
            ));
        });

    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                position_type: PositionType::Absolute,
                padding: UiRect::all(px(24)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            BackgroundColor(Color::srgba(0.01, 0.02, 0.04, 0.72)),
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
                        max_width: px(460),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(12),
                        padding: UiRect::all(px(20)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.07, 0.1, 0.16, 0.95)),
                    BorderRadius::all(px(20)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Keybindings"),
                        ui_theme.text_font(28.0),
                        TextColor(Color::srgb(0.98, 0.99, 1.0)),
                    ));
                    panel.spawn((
                        Text::new(controls_overlay_text(ui_theme.source)),
                        ui_theme.text_font(16.0),
                        TextColor(Color::srgb(0.89, 0.92, 0.96)),
                        TextLayout::new_with_justify(Justify::Left),
                        Node {
                            max_width: px(420),
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
            "Arrow Up / Down: Pitch camera\n",
            "Arrow Left / Right: Yaw camera\n",
            "Q / E: Roll camera\n",
            "W / S: Zoom in / out\n",
            "Space: Spawn polyhedra (hold to repeat)\n",
            "R: Reset to the root polyhedron\n",
            "1: Select cube\n",
            "2: Select tetrahedron\n",
            "3: Select octahedron\n",
            "4: Select dodecahedron\n",
            "F12: Save a screenshot\n",
            "- / +: Adjust child scale ratio\n",
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
        assert!(text.contains("Space: Spawn polyhedra (hold to repeat)"));
        assert!(text.contains("R: Reset to the root polyhedron"));
        assert!(text.contains("F12: Save a screenshot"));
        assert!(text.contains("4: Select dodecahedron"));
    }

    #[test]
    fn fallback_font_status_mentions_assets_directory() {
        let status = font_status_line(UiFontSource::Fallback);

        assert!(status.contains("assets/fonts"));
        assert!(status.contains("Carbon Plus"));
    }
}
