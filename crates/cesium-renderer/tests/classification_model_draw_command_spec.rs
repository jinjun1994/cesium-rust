use cesium_math::{BoundingSphere, Cartesian3, ClassificationType};
use glam::{DMat4, DVec3};
use cesium_renderer::{
    BlendingState, ClassificationFramePasses, ClassificationFrameState,
    ClassificationModelDrawCommand, ClassificationModelOptions,
    ClassificationPrimitiveRenderResources, ClassificationRuntimePrimitive, ColorMaskState,
    DepthFunction, DepthTestState, DrawCommand, DrawCommandOptions, Pass, RenderState,
    RenderStateHandle, StencilConstants, StencilFunction, StencilOperation, StencilOperationState,
    StencilTestState,
};
use std::sync::Arc;

fn batch_lengths() -> Vec<usize> {
    vec![6, 6, 3]
}

fn batch_offsets() -> Vec<usize> {
    vec![0, 6, 12]
}

const MOCK_PICK_ID: &str = "mockPickId";

fn mock_render_resources(
    classification_type: ClassificationType,
    debug_wireframe: bool,
    allow_picking: bool,
    pick_id: Option<String>,
) -> ClassificationPrimitiveRenderResources {
    ClassificationPrimitiveRenderResources {
        model: ClassificationModelOptions {
            classification_type,
            enable_debug_wireframe: debug_wireframe,
            debug_wireframe,
            allow_picking,
        },
        runtime_primitive: ClassificationRuntimePrimitive {
            bounding_sphere: BoundingSphere::new(DVec3::ZERO, 1.0),
            batch_lengths: batch_lengths(),
            batch_offsets: batch_offsets(),
        },
        pick_id,
    }
}

fn create_draw_command() -> DrawCommand {
    let bounding_sphere = BoundingSphere::new(DVec3::ZERO, 1.0);
    DrawCommand::new(DrawCommandOptions {
        bounding_volume: Some(bounding_sphere),
        model_matrix: Some(DMat4::IDENTITY),
        render_state: Some(RenderStateHandle::State(Arc::new(RenderState::default()))),
        pass: Some(Pass::Opaque),
        cull: Some(true),
        ..Default::default()
    })
}

fn verify_stencil_depth_command(
    command: &DrawCommand,
    expected_pass: Pass,
    expected_stencil_function: StencilFunction,
    test_for_picking: bool,
) {
    assert_eq!(command.pass, Some(expected_pass));
    assert_eq!(command.pick_id, None);
    assert_eq!(command.cull, test_for_picking);
    assert_eq!(command.pick_only, test_for_picking);

    let rs = match command.render_state.as_ref().unwrap() {
        RenderStateHandle::State(rs) => rs,
        _ => panic!("Expected State handle"),
    };

    assert_eq!(
        rs.color_mask,
        ColorMaskState {
            red: false,
            green: false,
            blue: false,
            alpha: false,
        }
    );

    let expected_stencil_test = StencilTestState {
        enabled: true,
        front_function: expected_stencil_function,
        front_operation: StencilOperationState {
            fail: StencilOperation::Keep,
            z_fail: StencilOperation::DecrWrap,
            z_pass: StencilOperation::Keep,
        },
        back_function: expected_stencil_function,
        back_operation: StencilOperationState {
            fail: StencilOperation::Keep,
            z_fail: StencilOperation::IncrWrap,
            z_pass: StencilOperation::Keep,
        },
        reference: StencilConstants::CESIUM_3D_TILE_MASK as i32,
        mask: StencilConstants::CESIUM_3D_TILE_MASK,
    };
    assert_eq!(rs.stencil_test, expected_stencil_test);
    assert_eq!(rs.stencil_mask, StencilConstants::CLASSIFICATION_MASK);
    assert_eq!(
        rs.depth_test,
        DepthTestState {
            enabled: true,
            func: DepthFunction::LEqual,
        }
    );
    assert!(!rs.depth_mask);
}

fn verify_color_command(command: &DrawCommand, expected_pass: Pass, test_for_picking: bool) {
    assert_eq!(command.pass, Some(expected_pass));

    if test_for_picking {
        assert!(command.cull);
        assert!(command.pick_only);
        assert!(command.pick_id.is_some());
    } else {
        assert!(!command.cull);
        assert!(!command.pick_only);
        assert!(command.pick_id.is_none());
    }

    let rs = match command.render_state.as_ref().unwrap() {
        RenderStateHandle::State(rs) => rs,
        _ => panic!("Expected State handle"),
    };

    let expected_stencil_test = StencilTestState {
        enabled: true,
        front_function: StencilFunction::NotEqual,
        front_operation: StencilOperationState {
            fail: StencilOperation::Zero,
            z_fail: StencilOperation::Zero,
            z_pass: StencilOperation::Zero,
        },
        back_function: StencilFunction::NotEqual,
        back_operation: StencilOperationState {
            fail: StencilOperation::Zero,
            z_fail: StencilOperation::Zero,
            z_pass: StencilOperation::Zero,
        },
        reference: 0,
        mask: StencilConstants::CLASSIFICATION_MASK,
    };
    assert_eq!(rs.stencil_test, expected_stencil_test);
    assert_eq!(rs.stencil_mask, StencilConstants::CLASSIFICATION_MASK);
    assert!(!rs.depth_test.enabled);
    assert!(!rs.depth_mask);

    if test_for_picking {
        assert!(!rs.blending.enabled);
    } else {
        assert_eq!(rs.blending, BlendingState::PRE_MULTIPLIED_ALPHA_BLEND);
    }
}

fn verify_batched_stencil_and_color_commands(
    draw_command: &ClassificationModelDrawCommand,
    command_list: &[DrawCommand],
    expected_pass: Pass,
    expected_stencil_function: StencilFunction,
    test_for_picking: bool,
) {
    let lengths = draw_command.batch_lengths();
    let offsets = draw_command.batch_offsets();
    let num_batches = lengths.len();

    assert_eq!(command_list.len(), num_batches * 2);

    for i in 0..num_batches {
        let stencil_cmd = &command_list[i * 2];
        assert_eq!(stencil_cmd.count, Some(lengths[i]));
        assert_eq!(stencil_cmd.offset, offsets[i]);
        verify_stencil_depth_command(
            stencil_cmd,
            expected_pass,
            expected_stencil_function,
            test_for_picking,
        );

        let color_cmd = &command_list[i * 2 + 1];
        assert_eq!(color_cmd.count, Some(lengths[i]));
        assert_eq!(color_cmd.offset, offsets[i]);
        verify_color_command(color_cmd, expected_pass, test_for_picking);
    }
}

#[test]
fn test_throws_for_undefined_command() {
    let res = ClassificationModelDrawCommand::new(None, Some(mock_render_resources(ClassificationType::Terrain, false, false, None)));
    assert!(res.is_err());
}

#[test]
fn test_throws_for_undefined_primitive_render_resources() {
    let command = create_draw_command();
    let res = ClassificationModelDrawCommand::new(Some(command), None);
    assert!(res.is_err());
}

#[test]
fn test_constructs_for_terrain_classification() {
    let resources = mock_render_resources(ClassificationType::Terrain, false, false, None);
    let command = create_draw_command();
    let draw_command = ClassificationModelDrawCommand::new(Some(command.clone()), Some(resources.clone())).unwrap();

    assert_eq!(draw_command.command(), &command);
    assert_eq!(draw_command.classification_type(), ClassificationType::Terrain);
    assert!(draw_command.classifies_terrain());
    assert!(!draw_command.classifies_3d_tiles());

    let command_list = draw_command.command_list_terrain();
    verify_batched_stencil_and_color_commands(
        &draw_command,
        command_list,
        Pass::TerrainClassification,
        StencilFunction::Always,
        false,
    );

    assert_eq!(draw_command.command_list_3d_tiles().len(), 0);
    assert_eq!(draw_command.command_list_ignore_show().len(), 0);
    assert_eq!(draw_command.command_list_debug_wireframe().len(), 0);
}

#[test]
fn test_constructs_for_3d_tiles_classification() {
    let resources = mock_render_resources(ClassificationType::Cesium3DTile, false, false, None);
    let command = create_draw_command();
    let draw_command = ClassificationModelDrawCommand::new(Some(command.clone()), Some(resources.clone())).unwrap();

    assert_eq!(draw_command.command(), &command);
    assert_eq!(draw_command.classification_type(), ClassificationType::Cesium3DTile);
    assert!(!draw_command.classifies_terrain());
    assert!(draw_command.classifies_3d_tiles());

    let command_list = draw_command.command_list_3d_tiles();
    verify_batched_stencil_and_color_commands(
        &draw_command,
        command_list,
        Pass::Cesium3dTileClassification,
        StencilFunction::Equal,
        false,
    );

    assert_eq!(draw_command.command_list_terrain().len(), 0);
    assert_eq!(draw_command.command_list_ignore_show().len(), 0);
    assert_eq!(draw_command.command_list_debug_wireframe().len(), 0);
}

#[test]
fn test_constructs_for_both_classifications() {
    let resources = mock_render_resources(ClassificationType::Both, false, false, None);
    let command = create_draw_command();
    let draw_command = ClassificationModelDrawCommand::new(Some(command.clone()), Some(resources.clone())).unwrap();

    assert_eq!(draw_command.command(), &command);
    assert_eq!(draw_command.classification_type(), ClassificationType::Both);
    assert!(draw_command.classifies_terrain());
    assert!(draw_command.classifies_3d_tiles());

    let terrain_list = draw_command.command_list_terrain();
    verify_batched_stencil_and_color_commands(
        &draw_command,
        terrain_list,
        Pass::TerrainClassification,
        StencilFunction::Always,
        false,
    );

    let tiles_list = draw_command.command_list_3d_tiles();
    verify_batched_stencil_and_color_commands(
        &draw_command,
        tiles_list,
        Pass::Cesium3dTileClassification,
        StencilFunction::Equal,
        false,
    );

    assert_eq!(draw_command.command_list_ignore_show().len(), 0);
    assert_eq!(draw_command.command_list_debug_wireframe().len(), 0);
}

#[test]
fn test_constructs_for_debug_wireframe() {
    let resources = mock_render_resources(ClassificationType::Both, true, false, None);
    let command = create_draw_command();
    let draw_command = ClassificationModelDrawCommand::new(Some(command.clone()), Some(resources.clone())).unwrap();

    assert!(draw_command.use_debug_wireframe());
    let command_list = draw_command.command_list_debug_wireframe();
    let lengths = draw_command.batch_lengths();
    let offsets = draw_command.batch_offsets();

    assert_eq!(command_list.len(), lengths.len());
    for i in 0..lengths.len() {
        let wireframe_cmd = &command_list[i];
        assert_eq!(wireframe_cmd.count, Some(lengths[i] * 2));
        assert_eq!(wireframe_cmd.offset, offsets[i] * 2);
        assert_eq!(wireframe_cmd.pass, Some(Pass::Opaque));
    }

    assert_eq!(draw_command.command_list_terrain().len(), 0);
    assert_eq!(draw_command.command_list_3d_tiles().len(), 0);
    assert_eq!(draw_command.command_list_ignore_show().len(), 0);
}

#[test]
fn test_constructs_pick_commands_for_terrain() {
    let resources = mock_render_resources(ClassificationType::Terrain, false, true, Some(MOCK_PICK_ID.to_string()));
    let command = create_draw_command();
    let draw_command = ClassificationModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    assert_eq!(draw_command.pick_id(), Some(MOCK_PICK_ID));
    let command_list = draw_command.command_list_terrain_picking();
    verify_batched_stencil_and_color_commands(
        &draw_command,
        command_list,
        Pass::TerrainClassification,
        StencilFunction::Always,
        true,
    );

    assert_eq!(draw_command.command_list_terrain().len(), 6);
    assert_eq!(draw_command.command_list_3d_tiles().len(), 0);
    assert_eq!(draw_command.command_list_3d_tiles_picking().len(), 0);
    assert_eq!(draw_command.command_list_ignore_show().len(), 0);
    assert_eq!(draw_command.command_list_debug_wireframe().len(), 0);
}

#[test]
fn test_constructs_pick_commands_for_3d_tiles() {
    let resources = mock_render_resources(ClassificationType::Cesium3DTile, false, true, Some(MOCK_PICK_ID.to_string()));
    let command = create_draw_command();
    let draw_command = ClassificationModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    assert_eq!(draw_command.pick_id(), Some(MOCK_PICK_ID));
    let command_list = draw_command.command_list_3d_tiles_picking();
    verify_batched_stencil_and_color_commands(
        &draw_command,
        command_list,
        Pass::Cesium3dTileClassification,
        StencilFunction::Equal,
        true,
    );

    assert_eq!(draw_command.command_list_3d_tiles().len(), 6);
    assert_eq!(draw_command.command_list_terrain().len(), 0);
    assert_eq!(draw_command.command_list_terrain_picking().len(), 0);
}

#[test]
fn test_constructs_pick_commands_for_both_classifications() {
    let resources = mock_render_resources(ClassificationType::Both, false, true, Some(MOCK_PICK_ID.to_string()));
    let command = create_draw_command();
    let draw_command = ClassificationModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    let terrain_picking = draw_command.command_list_terrain_picking();
    verify_batched_stencil_and_color_commands(
        &draw_command,
        terrain_picking,
        Pass::TerrainClassification,
        StencilFunction::Always,
        true,
    );

    let tiles_picking = draw_command.command_list_3d_tiles_picking();
    verify_batched_stencil_and_color_commands(
        &draw_command,
        tiles_picking,
        Pass::Cesium3dTileClassification,
        StencilFunction::Equal,
        true,
    );

    assert_eq!(draw_command.command_list_terrain().len(), 6);
    assert_eq!(draw_command.command_list_3d_tiles().len(), 6);
}

#[test]
fn test_does_not_construct_pick_commands_if_allow_picking_is_false() {
    let resources = mock_render_resources(ClassificationType::Both, false, false, Some(MOCK_PICK_ID.to_string()));
    let command = create_draw_command();
    let draw_command = ClassificationModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    assert_eq!(draw_command.command_list_terrain().len(), 6);
    assert_eq!(draw_command.command_list_3d_tiles().len(), 6);
    assert_eq!(draw_command.command_list_terrain_picking().len(), 0);
    assert_eq!(draw_command.command_list_3d_tiles_picking().len(), 0);
}

#[test]
fn test_push_commands_works_for_render_pass() {
    let resources = mock_render_resources(ClassificationType::Both, false, false, None);
    let command = create_draw_command();
    let mut draw_command = ClassificationModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    let mut command_list = Vec::new();
    let frame_state = ClassificationFrameState {
        passes: ClassificationFramePasses {
            render: true,
            pick: false,
        },
        invert_classification: false,
    };

    draw_command.push_commands(&frame_state, &mut command_list);
    assert_eq!(command_list.len(), 12);

    let terrain_slice = &command_list[0..6];
    verify_batched_stencil_and_color_commands(
        &draw_command,
        terrain_slice,
        Pass::TerrainClassification,
        StencilFunction::Always,
        false,
    );

    let tiles_slice = &command_list[6..12];
    verify_batched_stencil_and_color_commands(
        &draw_command,
        tiles_slice,
        Pass::Cesium3dTileClassification,
        StencilFunction::Equal,
        false,
    );
}

#[test]
fn test_push_commands_works_for_pick_pass() {
    let resources = mock_render_resources(ClassificationType::Both, false, true, Some(MOCK_PICK_ID.to_string()));
    let command = create_draw_command();
    let mut draw_command = ClassificationModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    let mut command_list = Vec::new();
    let frame_state = ClassificationFrameState {
        passes: ClassificationFramePasses {
            render: false,
            pick: true,
        },
        invert_classification: false,
    };

    draw_command.push_commands(&frame_state, &mut command_list);
    assert_eq!(command_list.len(), 12);

    let terrain_slice = &command_list[0..6];
    verify_batched_stencil_and_color_commands(
        &draw_command,
        terrain_slice,
        Pass::TerrainClassification,
        StencilFunction::Always,
        true,
    );

    let tiles_slice = &command_list[6..12];
    verify_batched_stencil_and_color_commands(
        &draw_command,
        tiles_slice,
        Pass::Cesium3dTileClassification,
        StencilFunction::Equal,
        true,
    );
}

#[test]
fn test_push_commands_derives_ignore_show_commands_for_3d_tiles() {
    let resources = mock_render_resources(ClassificationType::Cesium3DTile, false, false, None);
    let command = create_draw_command();
    let mut draw_command = ClassificationModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    assert_eq!(draw_command.command_list_ignore_show().len(), 0);

    let frame_state = ClassificationFrameState {
        passes: ClassificationFramePasses {
            render: true,
            pick: false,
        },
        invert_classification: true,
    };

    let mut command_list = Vec::new();
    draw_command.push_commands(&frame_state, &mut command_list);
    assert_eq!(command_list.len(), 9);

    let tiles_slice = &command_list[0..6];
    verify_batched_stencil_and_color_commands(
        &draw_command,
        tiles_slice,
        Pass::Cesium3dTileClassification,
        StencilFunction::Equal,
        false,
    );

    let ignore_show_commands = draw_command.command_list_ignore_show();
    assert_eq!(ignore_show_commands.len(), 3);

    for (i, ignore_show_cmd) in ignore_show_commands.iter().enumerate() {
        assert_eq!(&command_list[6 + i], ignore_show_cmd);
        verify_stencil_depth_command(
            ignore_show_cmd,
            Pass::Cesium3dTileClassificationIgnoreShow,
            StencilFunction::Equal,
            false,
        );
    }
}

#[test]
fn test_push_commands_does_not_derive_ignore_show_for_terrain() {
    let resources = mock_render_resources(ClassificationType::Terrain, false, false, None);
    let command = create_draw_command();
    let mut draw_command = ClassificationModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    assert_eq!(draw_command.command_list_ignore_show().len(), 0);

    let frame_state = ClassificationFrameState {
        passes: ClassificationFramePasses {
            render: true,
            pick: false,
        },
        invert_classification: true,
    };

    let mut command_list = Vec::new();
    draw_command.push_commands(&frame_state, &mut command_list);

    assert_eq!(command_list.len(), 6);
    assert_eq!(draw_command.command_list_ignore_show().len(), 0);
}

#[test]
fn test_updates_model_matrix() {
    let resources = mock_render_resources(ClassificationType::Both, false, false, None);
    let command = create_draw_command();
    let mut draw_command = ClassificationModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    assert_eq!(draw_command.model_matrix(), DMat4::IDENTITY);

    let translation_cart = Cartesian3::from_degrees(100.0, 25.0, Some(0.0), None).unwrap();
    let translation = DVec3::new(translation_cart.x, translation_cart.y, translation_cart.z);
    let model_matrix = DMat4::from_translation(translation);

    draw_command.set_model_matrix(model_matrix);
    assert_eq!(draw_command.model_matrix(), model_matrix);

    let mut command_list = Vec::new();
    command_list.extend(draw_command.command_list_terrain().iter().cloned());
    command_list.extend(draw_command.command_list_3d_tiles().iter().cloned());

    assert_eq!(command_list.len(), 12);
    for cmd in command_list {
        assert_eq!(cmd.model_matrix, Some(model_matrix));
        assert_eq!(cmd.bounding_volume.unwrap().center, translation);
    }
}
