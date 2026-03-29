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

#[cfg(test)]
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
