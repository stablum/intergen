use bevy::app::AppExit;
use bevy::prelude::*;

use super::browser::{AutomatedScenePresetLoad, PresetBrowserState, PresetCommand, PresetIndex};
use super::storage::{ScenePresetFile, read_preset_file, unique_preset_path, write_preset_file};
use crate::camera::CameraRig;
use crate::config::AppConfig;
use crate::control_page::{ControlPage, ControlPageState};
use crate::effect_tuner::EffectTunerState;
use crate::runtime_scene::SceneMutationAccess;
use crate::scene::{GenerationState, MaterialState, spawn_polyhedron_entity};
use crate::scene_snapshot::SceneStateSnapshot;

pub(crate) fn automated_scene_preset_load_system(
    preset_load: Option<Res<AutomatedScenePresetLoad>>,
    mut scene: SceneMutationAccess,
    mut app_exit: MessageWriter<AppExit>,
) {
    let Some(preset_load) = preset_load else {
        return;
    };

    let result = read_preset_file(&preset_load.path)
        .and_then(|file| {
            let summary = file.summary.clone();
            apply_scene_preset(&file.scene, &mut scene)?;
            Ok(summary)
        })
        .map(|summary| {
            println!(
                "Loaded scene preset from {}: {}",
                preset_load.path.display(),
                summary
            );
        });

    if let Err(error) = result {
        eprintln!(
            "Could not load scene preset {}: {error}",
            preset_load.path.display()
        );
        app_exit.write(AppExit::error());
    }
}

pub(crate) fn preset_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    control_page: Res<ControlPageState>,
    mut preset_browser: ResMut<PresetBrowserState>,
    mut scene: SceneMutationAccess,
) {
    if !control_page.is_active(ControlPage::ScenePresets) {
        return;
    }

    if let Some(chooser) = preset_browser.chooser.as_mut() {
        if keys.just_pressed(KeyCode::ArrowUp) && chooser.selected > 0 {
            chooser.selected -= 1;
        }
        if keys.just_pressed(KeyCode::ArrowDown) && chooser.selected + 1 < chooser.candidates.len()
        {
            chooser.selected += 1;
        }
        if keys.just_pressed(KeyCode::Enter) {
            match resolve_collision(&mut preset_browser, &mut scene) {
                Ok(Some(message)) => println!("{message}"),
                Ok(None) => {}
                Err(error) => eprintln!("{error}"),
            }
        }
        return;
    }

    if keys.just_pressed(KeyCode::KeyS) {
        preset_browser.arm_save();
        return;
    }

    if keys.just_pressed(KeyCode::Delete) {
        preset_browser.arm_free();
        return;
    }

    let Some(digit) = just_pressed_digit(&keys) else {
        return;
    };
    let Some(index) = preset_browser.push_digit(digit) else {
        return;
    };

    let result = match preset_browser.command {
        PresetCommand::Load => load_assigned_preset(&mut preset_browser, index, &mut scene),
        PresetCommand::Save => save_scene_preset(
            &mut preset_browser,
            index,
            &scene.app_config,
            &scene.camera_rig,
            &scene.generation_state,
            &scene.material_state,
            &scene.effect_tuner,
        ),
        PresetCommand::Free => free_assigned_slot(&mut preset_browser, index),
    };

    match result {
        Ok(Some(message)) => println!("{message}"),
        Ok(None) => {}
        Err(error) => eprintln!("{error}"),
    }
}

fn load_assigned_preset(
    preset_browser: &mut PresetBrowserState,
    index: PresetIndex,
    scene: &mut SceneMutationAccess<'_, '_>,
) -> Result<Option<String>, String> {
    let records = preset_browser.records_for_index(index);
    if records.is_empty() {
        set_status(preset_browser, format!("slot {} is empty", index.code()));
        return Ok(None);
    }

    if records.len() > 1 {
        reset_command_state(preset_browser);
        preset_browser.start_collision_resolution(index, true);
        return Ok(Some(format!(
            "Slot {} has multiple assigned presets. Resolve the chooser.",
            index.code()
        )));
    }

    let record = &records[0];
    apply_scene_preset(&record.file.scene, scene)?;
    Ok(finish_with_status(
        preset_browser,
        format!(
            "Loaded scene preset {}: {}",
            index.code(),
            record.file.summary
        ),
    ))
}

fn save_scene_preset(
    preset_browser: &mut PresetBrowserState,
    index: PresetIndex,
    app_config: &AppConfig,
    camera_rig: &CameraRig,
    generation_state: &GenerationState,
    material_state: &MaterialState,
    effect_tuner: &EffectTunerState,
) -> Result<Option<String>, String> {
    let scene = SceneStateSnapshot::capture(
        app_config,
        camera_rig,
        generation_state,
        material_state,
        effect_tuner,
    );
    let file = ScenePresetFile::new(index, scene.clone());
    let path = unique_preset_path(scene.file_slug().as_str())?;
    write_preset_file(&path, &file)?;
    preset_browser.refresh()?;

    if preset_browser.records_for_index(index).len() > 1 {
        reset_command_state(preset_browser);
        preset_browser.start_collision_resolution(index, false);
        return Ok(Some(format!(
            "Stored a new scene preset in slot {}. Resolve which preset stays assigned.",
            index.code()
        )));
    }

    Ok(finish_with_status(
        preset_browser,
        format!("Stored scene preset {}: {}", index.code(), file.summary),
    ))
}

fn free_assigned_slot(
    preset_browser: &mut PresetBrowserState,
    index: PresetIndex,
) -> Result<Option<String>, String> {
    let records = preset_browser.records_for_index(index);
    if records.is_empty() {
        set_status(
            preset_browser,
            format!("slot {} is already empty", index.code()),
        );
        return Ok(None);
    }

    for mut record in records {
        record.file.assignment = None;
        write_preset_file(&record.path, &record.file)?;
    }

    preset_browser.refresh()?;
    Ok(finish_with_status(
        preset_browser,
        format!("Freed scene preset slot {}.", index.code()),
    ))
}

fn resolve_collision(
    preset_browser: &mut PresetBrowserState,
    scene: &mut SceneMutationAccess<'_, '_>,
) -> Result<Option<String>, String> {
    let Some(chooser) = preset_browser.chooser.take() else {
        return Ok(None);
    };
    let Some(chosen) = chooser.candidates.get(chooser.selected).cloned() else {
        preset_browser.status_message = "chooser selection was invalid".to_string();
        return Ok(None);
    };

    for (candidate_index, mut candidate) in chooser.candidates.into_iter().enumerate() {
        candidate.file.assignment = if candidate_index == chooser.selected {
            Some(chooser.index)
        } else {
            None
        };
        write_preset_file(&candidate.path, &candidate.file)?;
    }

    preset_browser.refresh()?;

    if chooser.load_after_resolution {
        apply_scene_preset(&chosen.file.scene, scene)?;
    }

    Ok(finish_with_status(
        preset_browser,
        format!(
            "Slot {} now points to {}.",
            chooser.index.code(),
            chosen.file.summary
        ),
    ))
}

fn apply_scene_preset(
    scene: &SceneStateSnapshot,
    runtime: &mut SceneMutationAccess<'_, '_>,
) -> Result<(), String> {
    let prepared = scene.prepare_runtime()?;

    runtime.app_config.rendering = prepared.rendering;
    runtime.app_config.lighting = prepared.lighting;
    runtime.app_config.materials = prepared.materials;

    runtime.clear_color.0 = runtime.app_config.rendering.clear_color();
    runtime.ambient_light.color = runtime.app_config.rendering.ambient_light_color();
    runtime.ambient_light.brightness = runtime.app_config.rendering.ambient_light_brightness;

    for (mut light, mut transform) in runtime.directional_lights.iter_mut() {
        light.color = runtime.app_config.lighting.directional.color();
        light.illuminance = runtime.app_config.lighting.directional.illuminance;
        light.shadows_enabled = runtime.app_config.lighting.directional.shadows_enabled;
        *transform =
            Transform::from_translation(runtime.app_config.lighting.directional.translation())
                .looking_at(runtime.app_config.lighting.directional.look_at(), Vec3::Y);
    }

    for (mut light, mut transform) in runtime.point_lights.iter_mut() {
        light.color = runtime.app_config.lighting.point.color();
        light.intensity = runtime.app_config.lighting.point.intensity;
        light.range = runtime.app_config.lighting.point.range;
        light.shadows_enabled = runtime.app_config.lighting.point.shadows_enabled;
        *transform = Transform::from_translation(runtime.app_config.lighting.point.translation());
    }

    *runtime.camera_rig = prepared.camera_rig;
    runtime
        .effect_tuner
        .apply_runtime_snapshot(&prepared.effects);
    *runtime.generation_state = prepared.generation;
    runtime.material_state.opacity = prepared.material_opacity;

    for entity in runtime.polyhedron_entities.iter() {
        runtime.commands.entity(entity).despawn();
    }

    for (node_index, node) in runtime.generation_state.nodes.iter().enumerate() {
        spawn_polyhedron_entity(
            &mut runtime.commands,
            &mut runtime.materials,
            runtime.shape_assets.mesh(node.kind),
            node,
            &runtime.app_config.materials,
            runtime.material_state.opacity,
            node_index,
        );
    }

    Ok(())
}

fn reset_command_state(preset_browser: &mut PresetBrowserState) {
    preset_browser.command = PresetCommand::Load;
    preset_browser.first_digit = None;
}

fn set_status(preset_browser: &mut PresetBrowserState, status_message: String) {
    reset_command_state(preset_browser);
    preset_browser.status_message = status_message;
}

fn finish_with_status(
    preset_browser: &mut PresetBrowserState,
    status_message: String,
) -> Option<String> {
    set_status(preset_browser, status_message.clone());
    Some(status_message)
}

fn just_pressed_digit(keys: &ButtonInput<KeyCode>) -> Option<u8> {
    const DIGIT_KEYS: [(KeyCode, u8); 10] = [
        (KeyCode::Digit0, 0),
        (KeyCode::Digit1, 1),
        (KeyCode::Digit2, 2),
        (KeyCode::Digit3, 3),
        (KeyCode::Digit4, 4),
        (KeyCode::Digit5, 5),
        (KeyCode::Digit6, 6),
        (KeyCode::Digit7, 7),
        (KeyCode::Digit8, 8),
        (KeyCode::Digit9, 9),
    ];
    const NUMPAD_KEYS: [(KeyCode, u8); 10] = [
        (KeyCode::Numpad0, 0),
        (KeyCode::Numpad1, 1),
        (KeyCode::Numpad2, 2),
        (KeyCode::Numpad3, 3),
        (KeyCode::Numpad4, 4),
        (KeyCode::Numpad5, 5),
        (KeyCode::Numpad6, 6),
        (KeyCode::Numpad7, 7),
        (KeyCode::Numpad8, 8),
        (KeyCode::Numpad9, 9),
    ];

    DIGIT_KEYS
        .into_iter()
        .chain(NUMPAD_KEYS)
        .find_map(|(key_code, digit)| keys.just_pressed(key_code).then_some(digit))
}
