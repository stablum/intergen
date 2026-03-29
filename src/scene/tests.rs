use bevy::prelude::{AlphaMode, Quat, Vec3};

use crate::config::GenerationConfig;
use crate::parameters::{GenerationParameter, HoldRepeatState};
use crate::shapes::{
    AttachmentOccupancy, NodeOrigin, ShapeCatalog, ShapeKind, ShapeNode, SpawnAddMode,
    SpawnAttachment, SpawnPlacementMode,
};

use super::{
    GenerationParameters, GenerationState, alpha_mode_for_opacity, reset_generation_state,
    root_generation_node,
};

#[test]
fn reset_generation_state_restores_root_only() {
    let shape_catalog = ShapeCatalog::new();
    let generation_config = GenerationConfig::default();
    let mut root = root_generation_node(&shape_catalog, &generation_config);
    root.occupied_attachments.vertices[0] = true;

    let child = ShapeNode {
        kind: ShapeKind::Tetrahedron,
        level: 1,
        center: Vec3::new(2.0, -1.0, 0.5),
        rotation: Quat::IDENTITY,
        scale: 0.4,
        radius: 0.7,
        occupied_attachments: AttachmentOccupancy::default(),
        origin: NodeOrigin::Child {
            parent_index: 0,
            attachment: SpawnAttachment {
                mode: SpawnPlacementMode::Vertex,
                index: 0,
            },
        },
    };

    let mut generation_state = GenerationState {
        nodes: vec![root, child],
        selected_shape_kind: ShapeKind::Octahedron,
        spawn_placement_mode: SpawnPlacementMode::Face,
        spawn_add_mode: SpawnAddMode::FillLevel,
        parameters: GenerationParameters::from_base_values(0.42, 0.3, 0.6, 0.2),
        spawn_hold: HoldRepeatState {
            elapsed_secs: 1.0,
            repeating: true,
        },
    };
    let twist_spec =
        generation_config.parameter_spec(GenerationParameter::ChildTwistPerVertexRadians);
    generation_state
        .parameter_mut(GenerationParameter::ChildTwistPerVertexRadians)
        .input_mut()
        .request_decrease(
            false,
            true,
            false,
            twist_spec.hold_delay_secs() * 0.5,
            twist_spec,
        );
    generation_state
        .parameter_mut(GenerationParameter::ChildTwistPerVertexRadians)
        .input_mut()
        .request_increase(
            false,
            true,
            false,
            twist_spec.hold_delay_secs() * 0.5,
            twist_spec,
        );
    let offset_spec =
        generation_config.parameter_spec(GenerationParameter::ChildOutwardOffsetRatio);
    generation_state
        .parameter_mut(GenerationParameter::ChildOutwardOffsetRatio)
        .input_mut()
        .request_decrease(
            false,
            true,
            false,
            offset_spec.hold_delay_secs() * 0.5,
            offset_spec,
        );
    generation_state
        .parameter_mut(GenerationParameter::ChildOutwardOffsetRatio)
        .input_mut()
        .request_increase(
            false,
            true,
            false,
            offset_spec.hold_delay_secs() * 0.5,
            offset_spec,
        );
    let exclusion_spec =
        generation_config.parameter_spec(GenerationParameter::ChildSpawnExclusionProbability);
    generation_state
        .parameter_mut(GenerationParameter::ChildSpawnExclusionProbability)
        .input_mut()
        .request_decrease(
            false,
            true,
            false,
            exclusion_spec.hold_delay_secs() * 0.5,
            exclusion_spec,
        );
    generation_state
        .parameter_mut(GenerationParameter::ChildSpawnExclusionProbability)
        .input_mut()
        .request_increase(
            false,
            true,
            false,
            exclusion_spec.hold_delay_secs() * 0.5,
            exclusion_spec,
        );

    let reset_root =
        reset_generation_state(&mut generation_state, &shape_catalog, &generation_config);

    assert_eq!(generation_state.nodes.len(), 1);
    assert_eq!(generation_state.nodes[0].kind, ShapeKind::Octahedron);
    assert_eq!(generation_state.nodes[0].level, 0);
    assert_eq!(generation_state.nodes[0].center, Vec3::ZERO);
    assert_eq!(generation_state.selected_shape_kind, ShapeKind::Octahedron);
    assert_eq!(generation_state.scale_ratio_base(), 0.42);
    assert_eq!(generation_state.twist_per_vertex_radians_base(), 0.3);
    assert_eq!(generation_state.vertex_offset_ratio_base(), 0.6);
    assert_eq!(
        generation_state.vertex_spawn_exclusion_probability_base(),
        0.2
    );
    assert_eq!(
        generation_state.spawn_placement_mode,
        SpawnPlacementMode::Face
    );
    assert_eq!(generation_state.spawn_add_mode, SpawnAddMode::FillLevel);
    assert!(
        generation_state.nodes[0]
            .occupied_attachments
            .vertices
            .iter()
            .all(|occupied| !occupied)
    );
    assert!(
        generation_state.nodes[0]
            .occupied_attachments
            .edges
            .iter()
            .all(|occupied| !occupied)
    );
    assert!(
        generation_state.nodes[0]
            .occupied_attachments
            .faces
            .iter()
            .all(|occupied| !occupied)
    );
    assert_eq!(reset_root.center, Vec3::ZERO);
    assert_eq!(reset_root.kind, ShapeKind::Octahedron);
    assert_eq!(generation_state.spawn_hold.elapsed_secs, 0.0);
    assert!(!generation_state.spawn_hold.repeating);
    let twist_input = generation_state
        .parameter(GenerationParameter::ChildTwistPerVertexRadians)
        .input();
    assert_eq!(twist_input.decrease_hold().elapsed_secs, 0.0);
    assert!(!twist_input.decrease_hold().repeating);
    assert_eq!(twist_input.increase_hold().elapsed_secs, 0.0);
    assert!(!twist_input.increase_hold().repeating);
    let offset_input = generation_state
        .parameter(GenerationParameter::ChildOutwardOffsetRatio)
        .input();
    assert_eq!(offset_input.decrease_hold().elapsed_secs, 0.0);
    assert!(!offset_input.decrease_hold().repeating);
    assert_eq!(offset_input.increase_hold().elapsed_secs, 0.0);
    assert!(!offset_input.increase_hold().repeating);
    let exclusion_input = generation_state
        .parameter(GenerationParameter::ChildSpawnExclusionProbability)
        .input();
    assert_eq!(exclusion_input.decrease_hold().elapsed_secs, 0.0);
    assert!(!exclusion_input.decrease_hold().repeating);
    assert_eq!(exclusion_input.increase_hold().elapsed_secs, 0.0);
    assert!(!exclusion_input.increase_hold().repeating);
}

#[test]
fn transparent_materials_use_blend_mode() {
    assert!(matches!(alpha_mode_for_opacity(0.6), AlphaMode::Blend));
    assert!(matches!(alpha_mode_for_opacity(1.0), AlphaMode::Opaque));
}
