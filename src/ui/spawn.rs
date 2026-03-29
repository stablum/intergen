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
                border_radius: effect_tuner_corner_radius(),
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
                border_radius: effect_tuner_corner_radius(),
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
                border_radius: effect_tuner_corner_radius(),
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
            spawn_help_overlay_binding_cell(row, ui_theme, font_size, entry.binding, ui_config);

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

fn spawn_help_overlay_binding_cell(
    parent: &mut ChildSpawnerCommands,
    ui_theme: &UiTheme,
    font_size: f32,
    binding: &str,
    ui_config: &UiConfig,
) {
    parent
        .spawn(Node {
            width: px(HELP_OVERLAY_BINDING_COLUMN_WIDTH),
            min_width: px(HELP_OVERLAY_BINDING_COLUMN_WIDTH),
            max_width: px(HELP_OVERLAY_BINDING_COLUMN_WIDTH),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Center,
            align_content: AlignContent::FlexStart,
            column_gap: px(6.0),
            row_gap: px(6.0),
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|binding_cell| {
            for (text, is_badge) in help_overlay_binding_fragments(binding) {
                if is_badge {
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
                                Text::new(text),
                                ui_theme.text_font(font_size),
                                TextColor(srgb(ui_config.title_text)),
                                effect_tuner_text_layout(Justify::Center),
                            ));
                        });
                } else {
                    binding_cell.spawn((
                        Text::new(text),
                        ui_theme.text_font((font_size - 1.0).max(12.0)),
                        TextColor(srgb(ui_config.hint_text)),
                        effect_tuner_text_layout(Justify::Center),
                        Node {
                            padding: UiRect::horizontal(px(1.0)),
                            ..default()
                        },
                    ));
                }
            }
        });
}

fn help_overlay_binding_fragments(binding: &str) -> Vec<(String, bool)> {
    let mut fragments = Vec::new();

    for (or_index, option) in binding.split(" or ").enumerate() {
        if or_index > 0 {
            fragments.push(("or".to_string(), false));
        }

        for (slash_index, slash_part) in option.split(" / ").enumerate() {
            if slash_index > 0 {
                fragments.push(("/".to_string(), false));
            }

            for (plus_index, plus_part) in slash_part.split(" + ").enumerate() {
                if plus_index > 0 {
                    fragments.push(("+".to_string(), false));
                }
                push_help_overlay_binding_badges(&mut fragments, plus_part.trim());
            }
        }
    }

    fragments
}

fn push_help_overlay_binding_badges(fragments: &mut Vec<(String, bool)>, token: &str) {
    let token = token.trim();
    if token.is_empty() {
        return;
    }

    if token == "+" {
        fragments.push((token.to_string(), true));
        return;
    }

    if token.contains('+') {
        let mut parts = token.split('+').peekable();
        while let Some(part) = parts.next() {
            let part = part.trim();
            if !part.is_empty() {
                fragments.push((part.to_string(), true));
            }
            if parts.peek().is_some() {
                fragments.push(("+".to_string(), false));
            }
        }
        return;
    }

    fragments.push((token.to_string(), true));
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
                        Text::new("Second F1 press opens this view. Hover a key to see what it does when no F-page is active."),
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
                        border_radius: effect_tuner_corner_radius(),
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
                                        border_radius: effect_tuner_corner_radius(),
                                        ..default()
                                    },
                                    BackgroundColor(effect_tuner_panel_fill_color()),
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
                                        border_radius: effect_tuner_corner_radius(),
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
                                            border_radius: effect_tuner_corner_radius(),
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
                        border_radius: effect_tuner_corner_radius(),
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
                                border_radius: effect_tuner_corner_radius(),
                                ..default()
                            },
                            BackgroundColor(effect_tuner_panel_fill_color()),
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
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
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
                        max_width: px(ui_config.panel_max_width.max(HELP_OVERLAY_PANEL_MAX_WIDTH)),
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
                            "First F1 press opens this quick reference. Each column pairs keybinding pills with a short explanation.",
                        ),
                        ui_theme.text_font((ui_config.body_font_size - 1.0).max(14.0)),
                        TextColor(srgb(ui_config.body_text)),
                        TextLayout::new_with_justify(Justify::Center),
                        Node {
                            max_width: px((ui_config.body_max_width + 520.0).max(1120.0)),
                            ..default()
                        },
                    ));
                    panel
                        .spawn(Node {
                            width: percent(100),
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            align_items: AlignItems::FlexStart,
                            align_content: AlignContent::FlexStart,
                            column_gap: px(22.0),
                            row_gap: px(16.0),
                            ..default()
                        })
                        .with_children(|columns| {
                            let column_count = HELP_OVERLAY_COLUMN_COUNT.min(help_entries.len().max(1));
                            let entries_per_column =
                                (help_entries.len() + column_count - 1) / column_count;
                            for column_entries in help_entries.chunks(entries_per_column.max(1)) {
                                columns
                                    .spawn(Node {
                                        min_width: px(HELP_OVERLAY_COLUMN_MIN_WIDTH),
                                        flex_basis: px(0.0),
                                        flex_grow: 1.0,
                                        flex_direction: FlexDirection::Column,
                                        row_gap: px(0.0),
                                        ..default()
                                    })
                                    .with_children(|column| {
                                        column
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
                                                    ui_theme.text_font(
                                                        (ui_config.hint_font_size - 0.5).max(12.0),
                                                    ),
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
                                                    ui_theme.text_font(
                                                        (ui_config.hint_font_size - 0.5).max(12.0),
                                                    ),
                                                    TextColor(srgb(ui_config.hint_text)),
                                                    effect_tuner_text_layout(Justify::Left),
                                                    Node {
                                                        flex_grow: 1.0,
                                                        min_width: px(0.0),
                                                        ..default()
                                                    },
                                                ));
                                            });
                                        column
                                            .spawn(Node {
                                                width: percent(100),
                                                flex_direction: FlexDirection::Column,
                                                align_items: AlignItems::Stretch,
                                                row_gap: px(0.0),
                                                ..default()
                                            })
                                            .with_children(|table| {
                                                for (index, entry) in column_entries.iter().enumerate() {
                                                    spawn_help_overlay_row(
                                                        table,
                                                        ui_theme,
                                                        (ui_config.body_font_size - 1.5).max(14.0),
                                                        entry,
                                                        index + 1 < column_entries.len(),
                                                        ui_config,
                                                    );
                                                }
                                            });
                                    });
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
