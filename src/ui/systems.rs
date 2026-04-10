#[derive(SystemParam)]
pub(crate) struct EffectTunerUiViewAccess<'w, 's> {
    pub(crate) app_config: Res<'w, AppConfig>,
    pub(crate) camera_rig: Res<'w, CameraRig>,
    pub(crate) generation_state: Res<'w, GenerationState>,
    pub(crate) rendering_state: Res<'w, RenderingState>,
    pub(crate) lighting_state: Res<'w, LightingState>,
    pub(crate) material_state: Res<'w, MaterialState>,
    pub(crate) stage_state: Res<'w, StageState>,
    _marker: std::marker::PhantomData<&'s ()>,
}

impl<'w, 's> EffectTunerUiViewAccess<'w, 's> {
    fn effect_tuner_view_context(&self) -> EffectTunerViewContext<'_> {
        EffectTunerViewContext {
            camera_config: &self.app_config.camera,
            camera_rig: &self.camera_rig,
            generation_config: &self.app_config.generation,
            generation_state: &self.generation_state,
            rendering_config: &self.app_config.rendering,
            rendering_state: &self.rendering_state,
            lighting_config: &self.app_config.lighting,
            lighting_state: &self.lighting_state,
            material_config: &self.app_config.materials,
            material_state: &self.material_state,
            stage_state: &self.stage_state,
        }
    }
}

pub(crate) fn toggle_help_overlay_system(
    help_overlay: Res<HelpOverlayState>,
    mut text_overlay_query: Query<
        &mut Visibility,
        (With<HelpOverlay>, Without<KeyboardHelpOverlay>),
    >,
    mut keyboard_overlay_query: Query<
        &mut Visibility,
        (With<KeyboardHelpOverlay>, Without<HelpOverlay>),
    >,
) {
    let Ok(mut text_visibility) = text_overlay_query.single_mut() else {
        return;
    };
    let Ok(mut keyboard_visibility) = keyboard_overlay_query.single_mut() else {
        return;
    };

    *text_visibility = if help_overlay.mode == HelpOverlayMode::Text {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    *keyboard_visibility = if help_overlay.mode == HelpOverlayMode::Keyboard {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

pub(crate) fn update_keyboard_help_overlay_system(
    help_overlay: Res<HelpOverlayState>,
    mut tooltip_query: Query<&mut Text, With<KeyboardHelpTooltipText>>,
    mut key_query: Query<(&Interaction, &KeyboardHelpKey, &mut BackgroundColor)>,
) {
    let Ok(mut tooltip_text) = tooltip_query.single_mut() else {
        return;
    };

    let keyboard_visible = help_overlay.mode == HelpOverlayMode::Keyboard;
    let mut hovered_message = None;

    for (interaction, key, mut background) in key_query.iter_mut() {
        let hovered =
            keyboard_visible && matches!(*interaction, Interaction::Hovered | Interaction::Pressed);
        *background = if hovered {
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.14))
        } else {
            BackgroundColor(Color::NONE)
        };

        if hovered && hovered_message.is_none() {
            hovered_message = Some(key.message);
        }
    }

    *tooltip_text = Text::new(hovered_message.unwrap_or(""));
}

pub(crate) fn update_effect_tuner_overlay_system(
    time: Res<Time>,
    control_page: Res<ControlPageState>,
    effect_tuner: Res<EffectTunerState>,
    view: EffectTunerUiViewAccess,
    mut overlay_query: Query<(&mut Visibility, &mut Node), With<EffectTunerOverlay>>,
    mut pinned_badge_query: Query<
        &mut Visibility,
        (With<EffectTunerPinnedBadge>, Without<EffectTunerOverlay>),
    >,
    mut lfo_section_query: Query<
        (&mut Visibility, &mut Node),
        (
            With<EffectTunerLfoSection>,
            Without<EffectTunerOverlay>,
            Without<EffectTunerPinnedBadge>,
        ),
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
    let snapshot = effect_tuner.overlay_snapshot(&view.effect_tuner_view_context(), now_secs);
    let ui_config = &view.app_config.ui;

    let Ok((mut overlay_visibility, mut overlay_node)) = overlay_query.single_mut() else {
        return;
    };
    let compact_visible = control_page.page_has_focus(ControlPage::EffectTuner)
        && effect_tuner.page_mode() == EffectTunerPageMode::Compact
        && effect_tuner.is_visible(now_secs);
    *overlay_visibility = if compact_visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    overlay_node.display = if compact_visible {
        Display::Flex
    } else {
        Display::None
    };

    let Ok(mut pinned_badge_visibility) = pinned_badge_query.single_mut() else {
        return;
    };
    *pinned_badge_visibility = if compact_visible && snapshot.pinned {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    let Ok((mut lfo_section_visibility, mut lfo_section_node)) = lfo_section_query.single_mut()
    else {
        return;
    };
    let lfo_visible = compact_visible && snapshot.supports_lfo;
    *lfo_section_visibility = if lfo_visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    lfo_section_node.display = if lfo_visible {
        Display::Flex
    } else {
        Display::None
    };

    for (field, mut background) in field_query.iter_mut() {
        *background = if field.0 == snapshot.active_field {
            BackgroundColor(effect_tuner_active_field_background())
        } else {
            BackgroundColor(Color::NONE)
        };
    }

    for (text_kind, editable_field, mut text, mut text_color) in text_query.iter_mut() {
        let value = match text_kind {
            EffectTunerTextKind::Pin => "PIN".to_string(),
            EffectTunerTextKind::EffectLabel => snapshot.effect_label.to_string(),
            EffectTunerTextKind::EffectState => snapshot.effect_state_text.to_string(),
            EffectTunerTextKind::ParameterLabel => snapshot.parameter_label.to_string(),
            EffectTunerTextKind::Value => snapshot.value_text.clone(),
            EffectTunerTextKind::LiveValue => snapshot.live_value_text.clone(),
            EffectTunerTextKind::LfoState => format!("LFO {}", snapshot.lfo_state_text),
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
                    if snapshot.effect_state_emphasized {
                        srgb(ui_config.title_text)
                    } else {
                        srgb(ui_config.body_text)
                    }
                }
                EffectTunerTextKind::LfoState => {
                    if snapshot.lfo_state_emphasized {
                        lfo_enabled_text_color()
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

pub(crate) fn update_effect_tuner_group_overlay_system(
    time: Res<Time>,
    control_page: Res<ControlPageState>,
    effect_tuner: Res<EffectTunerState>,
    view: EffectTunerUiViewAccess,
    mut overlay_query: Query<(&mut Visibility, &mut Node), With<EffectTunerGroupOverlay>>,
    mut pinned_badge_query: Query<
        &mut Visibility,
        (
            With<EffectTunerGroupPinnedBadge>,
            Without<EffectTunerGroupOverlay>,
        ),
    >,
    mut window_text_query: Query<
        &mut Text,
        (
            With<EffectTunerGroupWindowText>,
            Without<EffectTunerGroupRowText>,
        ),
    >,
    mut row_query: Query<
        (&EffectTunerGroupRow, &mut Visibility, &mut BackgroundColor),
        (
            Without<EffectTunerGroupOverlay>,
            Without<EffectTunerGroupPinnedBadge>,
        ),
    >,
    mut row_text_query: Query<
        (&EffectTunerGroupRowText, &mut Text, &mut TextColor),
        (
            Without<EffectTunerGroupWindowText>,
            Without<EffectTunerGroupRow>,
        ),
    >,
) {
    let now_secs = time.elapsed_secs();
    let snapshot = effect_tuner.group_overlay_snapshot(EFFECT_TUNER_LIST_VISIBLE_ROWS);
    let ui_config = &view.app_config.ui;
    let visible = control_page.page_has_focus(ControlPage::EffectTuner)
        && effect_tuner.page_mode() == EffectTunerPageMode::GroupSelect
        && effect_tuner.is_visible(now_secs);

    let Ok((mut overlay_visibility, mut overlay_node)) = overlay_query.single_mut() else {
        return;
    };
    *overlay_visibility = if visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    overlay_node.display = if visible {
        Display::Flex
    } else {
        Display::None
    };

    let Ok(mut pinned_badge_visibility) = pinned_badge_query.single_mut() else {
        return;
    };
    *pinned_badge_visibility = if visible && snapshot.pinned {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    let Ok(mut window_text) = window_text_query.single_mut() else {
        return;
    };
    let visible_end = snapshot.window_start + snapshot.rows.len();
    *window_text = Text::new(format!(
        "GROUPS {}-{} / {}",
        snapshot.window_start + 1,
        visible_end,
        snapshot.total_groups
    ));

    for (row, mut visibility, mut background) in row_query.iter_mut() {
        if let Some(row_snapshot) = snapshot.rows.get(row.0) {
            *visibility = Visibility::Visible;
            *background = if row_snapshot.selected {
                BackgroundColor(effect_tuner_panel_fill_color())
            } else {
                BackgroundColor(Color::NONE)
            };
        } else {
            *visibility = Visibility::Hidden;
            *background = BackgroundColor(Color::NONE);
        }
    }

    for (text_meta, mut text, mut text_color) in row_text_query.iter_mut() {
        let Some(row_snapshot) = snapshot.rows.get(text_meta.0) else {
            *text = Text::new("");
            *text_color = TextColor(srgb(ui_config.body_text));
            continue;
        };

        *text = Text::new(row_snapshot.group_label);
        *text_color = TextColor(srgb(ui_config.title_text));
    }
}

pub(crate) fn update_effect_tuner_list_overlay_system(
    time: Res<Time>,
    control_page: Res<ControlPageState>,
    effect_tuner: Res<EffectTunerState>,
    view: EffectTunerUiViewAccess,
    mut overlay_query: Query<(&mut Visibility, &mut Node), With<EffectTunerListOverlay>>,
    mut pinned_badge_query: Query<
        &mut Visibility,
        (
            With<EffectTunerListPinnedBadge>,
            Without<EffectTunerListOverlay>,
        ),
    >,
    mut window_text_query: Query<
        &mut Text,
        (
            With<EffectTunerListWindowText>,
            Without<EffectTunerListRowText>,
            Without<EffectTunerListDetailText>,
        ),
    >,
    mut row_query: Query<
        (&EffectTunerListRow, &mut Visibility, &mut BackgroundColor),
        (
            Without<EffectTunerListOverlay>,
            Without<EffectTunerListPinnedBadge>,
            Without<EffectTunerListValueField>,
            Without<EffectTunerListDetailPanel>,
            Without<EffectTunerListDetailField>,
        ),
    >,
    mut row_text_query: Query<
        (&EffectTunerListRowText, &mut Text, &mut TextColor),
        (
            Without<EffectTunerListWindowText>,
            Without<EffectTunerListDetailText>,
        ),
    >,
    mut value_field_query: Query<
        (&EffectTunerListValueField, &mut BackgroundColor),
        (
            Without<EffectTunerListOverlay>,
            Without<EffectTunerListPinnedBadge>,
            Without<EffectTunerListRow>,
            Without<EffectTunerListDetailField>,
        ),
    >,
    mut detail_panel_query: Query<
        (&EffectTunerListDetailPanel, &mut Visibility, &mut Node),
        (
            Without<EffectTunerListOverlay>,
            Without<EffectTunerListPinnedBadge>,
            Without<EffectTunerListRow>,
        ),
    >,
    mut detail_field_query: Query<
        (&EffectTunerListDetailField, &mut BackgroundColor),
        (
            Without<EffectTunerListOverlay>,
            Without<EffectTunerListPinnedBadge>,
            Without<EffectTunerListRow>,
            Without<EffectTunerListValueField>,
        ),
    >,
    mut detail_text_query: Query<
        (&EffectTunerListDetailText, &mut Text, &mut TextColor),
        (
            Without<EffectTunerListWindowText>,
            Without<EffectTunerListRowText>,
        ),
    >,
) {
    let now_secs = time.elapsed_secs();
    let snapshot = effect_tuner.list_overlay_snapshot(
        &view.effect_tuner_view_context(),
        now_secs,
        EFFECT_TUNER_LIST_VISIBLE_ROWS,
    );
    let ui_config = &view.app_config.ui;
    let visible = control_page.page_has_focus(ControlPage::EffectTuner)
        && matches!(
            effect_tuner.page_mode(),
            EffectTunerPageMode::List | EffectTunerPageMode::GroupList
        )
        && effect_tuner.is_visible(now_secs);

    let Ok((mut overlay_visibility, mut overlay_node)) = overlay_query.single_mut() else {
        return;
    };
    *overlay_visibility = if visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    overlay_node.display = if visible {
        Display::Flex
    } else {
        Display::None
    };

    let Ok(mut pinned_badge_visibility) = pinned_badge_query.single_mut() else {
        return;
    };
    *pinned_badge_visibility = if visible && snapshot.pinned {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    let Ok(mut window_text) = window_text_query.single_mut() else {
        return;
    };
    *window_text = Text::new(snapshot.window_text.clone());

    for (row, mut visibility, mut background) in row_query.iter_mut() {
        if let Some(row_snapshot) = snapshot.rows.get(row.0) {
            *visibility = Visibility::Visible;
            *background = if row_snapshot.selected {
                BackgroundColor(effect_tuner_panel_fill_color())
            } else {
                BackgroundColor(Color::NONE)
            };
        } else {
            *visibility = Visibility::Hidden;
            *background = BackgroundColor(Color::NONE);
        }
    }

    for (text_meta, mut text, mut text_color) in row_text_query.iter_mut() {
        let Some(row_snapshot) = snapshot.rows.get(text_meta.slot) else {
            *text = Text::new("");
            *text_color = TextColor(srgb(ui_config.body_text));
            continue;
        };

        let value = match text_meta.kind {
            EffectTunerListRowTextKind::EffectLabel => row_snapshot.effect_label.to_string(),
            EffectTunerListRowTextKind::EffectState => row_snapshot.effect_state_text.to_string(),
            EffectTunerListRowTextKind::ParameterLabel => row_snapshot.parameter_label.to_string(),
            EffectTunerListRowTextKind::Value => row_snapshot.value_text.clone(),
            EffectTunerListRowTextKind::LiveValue => row_snapshot.live_value_text.clone(),
            EffectTunerListRowTextKind::LfoState => {
                if row_snapshot.supports_lfo {
                    format!("LFO {}", row_snapshot.lfo_state_text)
                } else {
                    String::new()
                }
            }
        };
        *text = Text::new(value);

        let color = match text_meta.kind {
            EffectTunerListRowTextKind::EffectLabel
            | EffectTunerListRowTextKind::ParameterLabel => srgb(ui_config.title_text),
            EffectTunerListRowTextKind::EffectState => {
                if row_snapshot.effect_state_emphasized {
                    srgb(ui_config.title_text)
                } else {
                    srgb(ui_config.body_text)
                }
            }
            EffectTunerListRowTextKind::Value => {
                if row_snapshot.active_field == Some(EffectOverlayField::Value) {
                    srgb(ui_config.focus_text)
                } else {
                    srgb(ui_config.body_text)
                }
            }
            EffectTunerListRowTextKind::LiveValue => srgb(ui_config.title_text),
            EffectTunerListRowTextKind::LfoState => {
                if row_snapshot.lfo_state_emphasized {
                    lfo_enabled_text_color()
                } else {
                    srgb(ui_config.body_text)
                }
            }
        };
        *text_color = TextColor(color);
    }

    for (field, mut background) in value_field_query.iter_mut() {
        let active = snapshot
            .rows
            .get(field.0)
            .and_then(|row| row.active_field)
            .is_some_and(|active_field| active_field == EffectOverlayField::Value);
        *background = if active {
            BackgroundColor(effect_tuner_active_field_background())
        } else {
            BackgroundColor(Color::NONE)
        };
    }

    for (panel, mut visibility, mut node) in detail_panel_query.iter_mut() {
        let lfo_detail_visible = snapshot
            .rows
            .get(panel.0)
            .is_some_and(|row| row.selected && row.supports_lfo);
        *visibility = if lfo_detail_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        node.display = if lfo_detail_visible {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (field, mut background) in detail_field_query.iter_mut() {
        let selected_slot = snapshot
            .rows
            .get(field.slot)
            .is_some_and(|row| row.selected && row.supports_lfo);
        let active = selected_slot && snapshot.detail.active_field == field.field;
        *background = if active {
            BackgroundColor(effect_tuner_active_field_background())
        } else {
            BackgroundColor(Color::NONE)
        };
    }

    for (text_meta, mut text, mut text_color) in detail_text_query.iter_mut() {
        let selected_slot = snapshot
            .rows
            .get(text_meta.slot)
            .is_some_and(|row| row.selected && row.supports_lfo);
        if !selected_slot {
            *text = Text::new("");
            *text_color = TextColor(srgb(ui_config.body_text));
            continue;
        }

        let value = match text_meta.kind {
            EffectTunerListDetailTextKind::State => String::new(),
            EffectTunerListDetailTextKind::Amplitude => snapshot.detail.amplitude_text.clone(),
            EffectTunerListDetailTextKind::Frequency => snapshot.detail.frequency_text.clone(),
            EffectTunerListDetailTextKind::Shape => snapshot.detail.shape_text.to_string(),
        };
        *text = Text::new(value);

        let color = match text_meta.kind {
            EffectTunerListDetailTextKind::State => srgb(ui_config.body_text),
            EffectTunerListDetailTextKind::Amplitude => {
                if snapshot.detail.active_field == EffectOverlayField::LfoAmplitude {
                    srgb(ui_config.focus_text)
                } else {
                    srgb(ui_config.body_text)
                }
            }
            EffectTunerListDetailTextKind::Frequency => {
                if snapshot.detail.active_field == EffectOverlayField::LfoFrequency {
                    srgb(ui_config.focus_text)
                } else {
                    srgb(ui_config.body_text)
                }
            }
            EffectTunerListDetailTextKind::Shape => {
                if snapshot.detail.active_field == EffectOverlayField::LfoShape {
                    srgb(ui_config.focus_text)
                } else {
                    srgb(ui_config.body_text)
                }
            }
        };
        *text_color = TextColor(color);
    }
}

pub(crate) fn update_preset_overlay_system(
    app_config: Res<AppConfig>,
    control_page: Res<ControlPageState>,
    preset_browser: Res<PresetBrowserState>,
    mut strip_visibility: Query<
        &mut Visibility,
        (With<PresetStripOverlay>, Without<PresetChooserOverlay>),
    >,
    mut strip_command_text: Query<
        (&mut Text, &mut TextColor),
        (
            With<PresetStripCommandText>,
            Without<PresetStripTargetText>,
            Without<PresetStripStatusText>,
            Without<PresetChooserText>,
        ),
    >,
    mut strip_target_text: Query<
        (&mut Text, &mut TextColor),
        (
            With<PresetStripTargetText>,
            Without<PresetStripCommandText>,
            Without<PresetStripStatusText>,
            Without<PresetChooserText>,
        ),
    >,
    mut strip_bank_text: Query<
        (&PresetStripBankText, &mut Text, &mut TextColor),
        (
            With<PresetStripBankText>,
            Without<PresetStripCommandText>,
            Without<PresetStripTargetText>,
            Without<PresetStripStatusText>,
            Without<PresetChooserText>,
        ),
    >,
    mut strip_status_text: Query<
        &mut Text,
        (
            With<PresetStripStatusText>,
            Without<PresetStripCommandText>,
            Without<PresetStripTargetText>,
            Without<PresetChooserText>,
        ),
    >,
    mut chooser_visibility: Query<
        &mut Visibility,
        (With<PresetChooserOverlay>, Without<PresetStripOverlay>),
    >,
    mut chooser_text: Query<&mut Text, (With<PresetChooserText>, Without<PresetStripText>)>,
) {
    let ui_config = &app_config.ui;
    let preset_page_visible = control_page.is_active(ControlPage::ScenePresets);
    let strip_segments = preset_browser.strip_segments();

    let Ok(mut strip_visibility) = strip_visibility.single_mut() else {
        return;
    };
    *strip_visibility = if preset_page_visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    let Ok((mut strip_command_text, mut strip_command_color)) = strip_command_text.single_mut()
    else {
        return;
    };
    *strip_command_text = Text::new(strip_segments.command);
    *strip_command_color = if strip_segments.emphasize_command {
        TextColor(lfo_enabled_text_color())
    } else {
        TextColor(srgb(ui_config.body_text))
    };

    let Ok((mut strip_target_text, mut strip_target_color)) = strip_target_text.single_mut()
    else {
        return;
    };
    *strip_target_text = Text::new(strip_segments.target);
    *strip_target_color = if strip_segments.emphasize_target {
        TextColor(lfo_enabled_text_color())
    } else {
        TextColor(srgb(ui_config.body_text))
    };

    for (bank_text_meta, mut bank_text, mut bank_text_color) in strip_bank_text.iter_mut() {
        let Some(bank_segment) = strip_segments
            .banks
            .iter()
            .find(|segment| segment.bank == bank_text_meta.bank)
        else {
            *bank_text = Text::new("");
            *bank_text_color = TextColor(srgb(ui_config.body_text));
            continue;
        };

        let (value, emphasized) = match bank_text_meta.kind {
            PresetStripBankTextKind::Label => {
                (bank_segment.label.as_str(), bank_segment.emphasize_bank)
            }
            PresetStripBankTextKind::Prefix => (bank_segment.prefix.as_str(), false),
            PresetStripBankTextKind::SelectedSlot => (
                bank_segment.selected_slot.as_str(),
                bank_segment.emphasize_selected_slot,
            ),
            PresetStripBankTextKind::Suffix => (bank_segment.suffix.as_str(), false),
        };
        *bank_text = Text::new(value);
        *bank_text_color = if emphasized {
            TextColor(lfo_enabled_text_color())
        } else {
            TextColor(srgb(ui_config.body_text))
        };
    }

    let Ok(mut strip_status_text) = strip_status_text.single_mut() else {
        return;
    };
    *strip_status_text = Text::new(strip_segments.status);

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
