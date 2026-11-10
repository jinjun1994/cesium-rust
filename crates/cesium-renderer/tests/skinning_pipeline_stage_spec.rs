use cesium_math::Matrix4;
use cesium_renderer::model_components::{Attribute, Primitive};
use cesium_renderer::skinning_pipeline_stage::{
    MockSkinningRenderResources, MockSkinningRuntimeNode, SkinningPipelineStage,
    SKINNING_STAGE_VS,
};

#[test]
fn test_skinning_pipeline_stage_constants() {
    assert_eq!(SkinningPipelineStage::NAME, "SkinningPipelineStage");
    assert_eq!(
        SkinningPipelineStage::FUNCTION_ID_GET_SKINNING_MATRIX,
        "getSkinningMatrix"
    );
    assert_eq!(
        SkinningPipelineStage::FUNCTION_SIGNATURE_GET_SKINNING_MATRIX,
        "mat4 getSkinningMatrix()"
    );
}

#[test]
fn test_processes_skin_with_two_joints() {
    let mock_joint_matrices = vec![Matrix4::IDENTITY, Matrix4::IDENTITY];
    let mut render_resources = MockSkinningRenderResources {
        runtime_node: MockSkinningRuntimeNode {
            computed_joint_matrices: mock_joint_matrices.clone(),
        },
        ..Default::default()
    };

    let primitive = Primitive {
        attributes: vec![
            Attribute {
                semantic: Some("JOINTS_0".to_string()),
                set_index: Some(0),
                ..Default::default()
            },
            Attribute {
                semantic: Some("WEIGHTS_0".to_string()),
                set_index: Some(0),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    SkinningPipelineStage::process(&mut render_resources, &primitive);

    let sb = &render_resources.shader_builder;

    assert!(sb.vertex_define_lines().contains(&"HAS_SKINNING".to_string()));

    let func = sb
        .get_function(SkinningPipelineStage::FUNCTION_ID_GET_SKINNING_MATRIX)
        .expect("getSkinningMatrix function should be added");
    assert_eq!(
        func.signature,
        SkinningPipelineStage::FUNCTION_SIGNATURE_GET_SKINNING_MATRIX
    );

    let expected_lines = vec![
        "    mat4 skinnedMatrix = mat4(0);",
        "    skinnedMatrix += a_weights_0.x * u_jointMatrices[int(a_joints_0.x)];",
        "    skinnedMatrix += a_weights_0.y * u_jointMatrices[int(a_joints_0.y)];",
        "    skinnedMatrix += a_weights_0.z * u_jointMatrices[int(a_joints_0.z)];",
        "    skinnedMatrix += a_weights_0.w * u_jointMatrices[int(a_joints_0.w)];",
        "    return skinnedMatrix;",
    ];
    for line in expected_lines {
        assert!(
            func.body.iter().any(|l| l.trim() == line.trim()),
            "Missing line: {line}"
        );
    }

    assert!(sb.vertex_lines().contains(&SKINNING_STAGE_VS.to_string()));
    assert!(sb
        .vertex_uniform_lines()
        .contains(&"uniform mat4 u_jointMatrices[2];".to_string()));

    assert_eq!(render_resources.bound_joint_matrices, Some(mock_joint_matrices));
}

#[test]
fn test_processes_skin_with_many_joints() {
    let joint_count = 19;
    let mock_joint_matrices = vec![Matrix4::IDENTITY; joint_count];
    let mut render_resources = MockSkinningRenderResources {
        runtime_node: MockSkinningRuntimeNode {
            computed_joint_matrices: mock_joint_matrices.clone(),
        },
        ..Default::default()
    };

    let primitive = Primitive {
        attributes: vec![
            Attribute {
                semantic: Some("JOINTS_0".to_string()),
                set_index: Some(0),
                ..Default::default()
            },
            Attribute {
                semantic: Some("WEIGHTS_0".to_string()),
                set_index: Some(0),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    SkinningPipelineStage::process(&mut render_resources, &primitive);

    let sb = &render_resources.shader_builder;
    assert!(sb.vertex_define_lines().contains(&"HAS_SKINNING".to_string()));

    assert!(sb
        .vertex_uniform_lines()
        .contains(&format!("uniform mat4 u_jointMatrices[{joint_count}];")));

    assert_eq!(render_resources.bound_joint_matrices, Some(mock_joint_matrices));
}
