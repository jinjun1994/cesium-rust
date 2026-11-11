use cesium_renderer::commands::{DrawCommand, FramebufferHandle, Pass, ShaderProgramHandle};
use cesium_renderer::context::{Context, ContextOptions};
use cesium_renderer::point_cloud_eye_dome_lighting::{
    PointCloudCommand, PointCloudEyeDomeLighting, PointCloudEyeDomeLightingFrameState,
};
use cesium_renderer::point_cloud_shading::PointCloudShading;
use cesium_math::{BoundingSphere, Cartesian3, PrimitiveType};

fn create_test_context(draw_buffers: bool, fragment_depth: bool) -> Context {
    let mut ctx = Context::new(ContextOptions {
        width: Some(256),
        height: Some(256),
        ..Default::default()
    })
    .unwrap();
    ctx.draw_buffers = draw_buffers;
    ctx.fragment_depth = fragment_depth;
    ctx
}

#[test]
fn test_is_supported() {
    let supported_ctx = create_test_context(true, true);
    assert!(PointCloudEyeDomeLighting::is_supported(&supported_ctx));

    let unsupported_ctx1 = create_test_context(false, true);
    assert!(!PointCloudEyeDomeLighting::is_supported(&unsupported_ctx1));

    let unsupported_ctx2 = create_test_context(true, false);
    assert!(!PointCloudEyeDomeLighting::is_supported(&unsupported_ctx2));
}

#[test]
fn test_adds_a_clear_command_and_a_post_processing_draw_call() {
    let context = create_test_context(true, true);
    let mut edl = PointCloudEyeDomeLighting::new().unwrap();

    let mut frame_state = PointCloudEyeDomeLightingFrameState::new(&context);

    // Initial point cloud draw command
    let original_point_cmd = DrawCommand {
        primitive_type: PrimitiveType::Points,
        pass: Some(Pass::Cesium3dTile),
        shader_program: Some(ShaderProgramHandle::Name("PointCloudFS".to_string())),
        cast_shadows: true,
        receive_shadows: true,
        ..Default::default()
    };
    frame_state.command_list.push(PointCloudCommand::Draw(Box::new(original_point_cmd)));
    let original_length = frame_state.command_list.len();

    let mut shading = PointCloudShading::new();
    shading.eye_dome_lighting = true;
    shading.attenuation = true;

    let bounding_sphere = BoundingSphere::new(Cartesian3::ZERO, 10.0);
    edl.update(&mut frame_state, 0, &shading, Some(bounding_sphere))
        .unwrap();

    let new_length = frame_state.command_list.len();
    assert_eq!(new_length, original_length + 2);

    // Verify hijacked point command
    if let PointCloudCommand::Draw(ref cmd) = frame_state.command_list[0] {
        assert!(!cmd.cast_shadows);
        assert!(!cmd.receive_shadows);
        assert!(matches!(cmd.framebuffer, Some(FramebufferHandle::Framebuffer(_))));
        assert_eq!(
            cmd.shader_program,
            Some(ShaderProgramHandle::Name("PointCloudFS_EC".to_string()))
        );
    } else {
        panic!("First command should be DrawCommand");
    }

    // Verify appended blend command and clear command
    match &frame_state.command_list[1] {
        PointCloudCommand::Draw(cmd) => {
            assert_eq!(cmd.bounding_volume, Some(bounding_sphere));
            assert_eq!(cmd.primitive_type, PrimitiveType::Triangles);
        }
        _ => panic!("Second command should be DrawCommand"),
    }

    match &frame_state.command_list[2] {
        PointCloudCommand::Clear(cmd) => {
            assert!(cmd.framebuffer.is_some());
        }
        _ => panic!("Third command should be ClearCommand"),
    }

    edl.destroy();
    assert!(edl.is_destroyed());
}

#[test]
fn test_does_not_change_commands_for_pick_calls() {
    let context = create_test_context(true, true);
    let mut edl = PointCloudEyeDomeLighting::new().unwrap();

    let mut frame_state = PointCloudEyeDomeLightingFrameState::new(&context);
    frame_state.is_picking = true;

    let original_point_cmd = DrawCommand {
        primitive_type: PrimitiveType::Points,
        pass: Some(Pass::Cesium3dTile),
        shader_program: Some(ShaderProgramHandle::Name("PointCloudFS".to_string())),
        ..Default::default()
    };
    frame_state.command_list.push(PointCloudCommand::Draw(Box::new(original_point_cmd)));
    let original_length = frame_state.command_list.len();

    let shading = PointCloudShading::new();
    edl.update(&mut frame_state, 0, &shading, None).unwrap();

    let new_length = frame_state.command_list.len();
    assert_eq!(new_length, original_length);

    edl.destroy();
}

#[test]
fn test_works_when_point_cloud_shader_changes() {
    let context = create_test_context(true, true);
    let mut edl = PointCloudEyeDomeLighting::new().unwrap();

    let mut frame_state = PointCloudEyeDomeLightingFrameState::new(&context);

    // Initial shader
    let cmd1 = DrawCommand {
        primitive_type: PrimitiveType::Points,
        pass: Some(Pass::Cesium3dTile),
        shader_program: Some(ShaderProgramHandle::Name("PointCloudInitial".to_string())),
        ..Default::default()
    };
    frame_state.command_list.push(PointCloudCommand::Draw(Box::new(cmd1)));

    let mut shading = PointCloudShading::new();
    shading.eye_dome_lighting = true;

    edl.update(&mut frame_state, 0, &shading, None).unwrap();
    assert_eq!(frame_state.command_list.len(), 3);

    // Now simulate changing point cloud styling / shader program
    frame_state.command_list.truncate(1);
    if let PointCloudCommand::Draw(ref mut cmd) = frame_state.command_list[0] {
        cmd.shader_program = Some(ShaderProgramHandle::Name("PointCloudRedStyle".to_string()));
    }

    edl.update(&mut frame_state, 0, &shading, None).unwrap();
    assert_eq!(frame_state.command_list.len(), 3);

    if let PointCloudCommand::Draw(ref cmd) = frame_state.command_list[0] {
        assert_eq!(
            cmd.shader_program,
            Some(ShaderProgramHandle::Name("PointCloudRedStyle_EC".to_string()))
        );
    } else {
        panic!("First command should be DrawCommand");
    }

    edl.destroy();
}
