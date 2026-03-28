#[derive(Clone, Copy)]
struct HelpEntry {
    startup: &'static str,
    overlay: &'static str,
}

const TOP_LEVEL_HELP: [HelpEntry; 5] = [
    HelpEntry {
        startup: "F1/H cycles help views",
        overlay: "F1 / H: Cycle help views",
    },
    HelpEntry {
        startup: "F2 cycles compact/list/close for the live control page",
        overlay: "F2: Open compact controls, second press opens the list, third press closes",
    },
    HelpEntry {
        startup: "F3 scene preset page",
        overlay: "F3: Toggle the scene preset page",
    },
    HelpEntry {
        startup: "Esc closes the active control page",
        overlay: "Esc: Close the current control page",
    },
    HelpEntry {
        startup: "F4 export Blender .blend",
        overlay: "F4: Export the current scene as a Blender .blend",
    },
];

const FX_HELP: [HelpEntry; 12] = [
    HelpEntry {
        startup: "Ctrl+Up/Down selects a control",
        overlay: "In F2 page: Ctrl + Up / Down select control",
    },
    HelpEntry {
        startup: "second F2 press opens the scrolling parameter list",
        overlay: "In F2 page: Second F2 press opens the scrolling parameter list",
    },
    HelpEntry {
        startup: "Left/Right or Tab/Shift+Tab switch the active field",
        overlay: "In F2 page: Left / Right or Tab / Shift+Tab switch the active field",
    },
    HelpEntry {
        startup: "Up/Down adjust the active field",
        overlay: "In F2 page: Up / Down adjust the active field",
    },
    HelpEntry {
        startup: "Space toggles the selected shader effect",
        overlay: "In F2 page: Space toggles the selected shader effect",
    },
    HelpEntry {
        startup: "L toggles the selected shader-effect parameter LFO",
        overlay: "In F2 page: L toggles the selected shader-effect parameter LFO",
    },
    HelpEntry {
        startup: "typing a number sets the active numeric field",
        overlay: "In F2 page: Type digits / . / - / + (for example 0.157) to set the active numeric field",
    },
    HelpEntry {
        startup: "Backspace erases typed F2 input",
        overlay: "In F2 page: Backspace erases the typed numeric input",
    },
    HelpEntry {
        startup: "Shift is coarse",
        overlay: "Shift: Coarse F2 adjustment",
    },
    HelpEntry {
        startup: "Alt is fine",
        overlay: "Alt: Fine F2 adjustment",
    },
    HelpEntry {
        startup: "Enter resets the field",
        overlay: "Enter: Reset the selected F2 field",
    },
    HelpEntry {
        startup: "Shift+Enter resets all F2 controls",
        overlay: "Shift + Enter: Reset all F2 controls",
    },
];

const PRESET_HELP: [HelpEntry; 1] = [HelpEntry {
    startup: "scene preset page supports save/free/load/collision resolution",
    overlay:
        "In preset page: S save, Del free slot, 00-99 load, Up/Down + Enter resolve collisions",
}];

const SCENE_HELP: [HelpEntry; 23] = [
    HelpEntry {
        startup: "Arrow Up/Down pitch camera",
        overlay: "Arrow Up / Down: Pitch camera",
    },
    HelpEntry {
        startup: "Arrow Left/Right yaw camera",
        overlay: "Arrow Left / Right: Yaw camera",
    },
    HelpEntry {
        startup: "Q/E roll camera",
        overlay: "Q / E: Roll camera",
    },
    HelpEntry {
        startup: "W/S zoom",
        overlay: "W / S: Zoom in / out",
    },
    HelpEntry {
        startup: "Backspace stops camera rotation",
        overlay: "Backspace: Stop camera rotation momentum",
    },
    HelpEntry {
        startup: "hold Space to add objects",
        overlay: "Space: Add shapes using the current add mode (hold to repeat)",
    },
    HelpEntry {
        startup: "Ctrl+Space cycles add mode",
        overlay: "Ctrl + Space: Cycle add mode (single / fill current level)",
    },
    HelpEntry {
        startup: "G cycles spawn placement mode",
        overlay: "G: Cycle spawn placement mode (vertex / edge / face)",
    },
    HelpEntry {
        startup: "R reset scene",
        overlay: "R: Reset to the selected shape as root",
    },
    HelpEntry {
        startup: "1 selects cube",
        overlay: "1: Select cube",
    },
    HelpEntry {
        startup: "2 selects tetrahedron",
        overlay: "2: Select tetrahedron",
    },
    HelpEntry {
        startup: "3 selects octahedron",
        overlay: "3: Select octahedron",
    },
    HelpEntry {
        startup: "4 selects dodecahedron",
        overlay: "4: Select dodecahedron",
    },
    HelpEntry {
        startup: "F12 screenshot",
        overlay: "F12: Save a screenshot",
    },
    HelpEntry {
        startup: "-/+ adjust child scale ratio",
        overlay: "- / +: Adjust child scale ratio",
    },
    HelpEntry {
        startup: "O/P adjust opacity",
        overlay: "O / P: Adjust global opacity",
    },
    HelpEntry {
        startup: "I reset opacity",
        overlay: "I: Reset global opacity",
    },
    HelpEntry {
        startup: "hold [/] or ,/. to adjust child twist",
        overlay: "[ / ] or , / .: Adjust child twist angle (hold to repeat)",
    },
    HelpEntry {
        startup: "hold Z/X to adjust child offset",
        overlay: "Z / X: Adjust child outward offset (hold to repeat)",
    },
    HelpEntry {
        startup: "hold V/B to adjust spawn exclusion probability",
        overlay: "V / B: Adjust spawn exclusion probability (hold to repeat)",
    },
    HelpEntry {
        startup: "C resets child outward offset",
        overlay: "C: Reset child outward offset",
    },
    HelpEntry {
        startup: "N resets spawn exclusion probability",
        overlay: "N: Reset spawn exclusion probability",
    },
    HelpEntry {
        startup: "T resets child twist angle",
        overlay: "T: Reset child twist angle",
    },
];

pub(crate) fn startup_controls_message() -> String {
    startup_message("Controls", &[&TOP_LEVEL_HELP, &SCENE_HELP])
}

pub(crate) fn startup_fx_message() -> String {
    startup_message("F2 page", &[&FX_HELP])
}

pub(crate) fn overlay_controls_text(font_status_line: &str) -> String {
    let mut lines = Vec::new();
    append_overlay_lines(&mut lines, &TOP_LEVEL_HELP);
    append_overlay_lines(&mut lines, &FX_HELP);
    append_overlay_lines(&mut lines, &PRESET_HELP);
    append_overlay_lines(&mut lines, &SCENE_HELP);
    lines.push(String::new());
    lines.push(font_status_line.to_string());
    lines.join("\n")
}

fn startup_message(prefix: &str, groups: &[&[HelpEntry]]) -> String {
    let fragments = groups
        .iter()
        .flat_map(|entries| entries.iter().map(|entry| entry.startup))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{prefix}: {fragments}")
}

fn append_overlay_lines(lines: &mut Vec<String>, entries: &[HelpEntry]) {
    lines.extend(entries.iter().map(|entry| entry.overlay.to_string()));
}
