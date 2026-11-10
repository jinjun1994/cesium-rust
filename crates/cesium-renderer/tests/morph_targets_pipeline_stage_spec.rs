use cesium_math::{AttributeType, ComponentDatatype};
use cesium_renderer::buffer::{Buffer, BufferOptions};
use cesium_renderer::classification_pipeline_stage::VertexAttributeSemantic;
use cesium_renderer::model_components::{Attribute, MorphTarget, Primitive};
use cesium_renderer::morph_targets_pipeline_stage::{
    MockMorphTargetsRenderResources, MorphTargetsPipelineStage, MORPH_TARGETS_STAGE_VS,
};

#[test]
fn test_processes_morph_target_with_position() {
    let mut render_resources = MockMorphTargetsRenderResources {
        attributes: Vec::new(),
        attribute_index: 1,
        morph_weights: vec![1.0],
        ..Default::default()
    };

    let vertex_buffer = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 48,
        ..Default::default()
    })
    .expect("Buffer creation");

    let position_attr = Attribute {
        semantic: Some(VertexAttributeSemantic::POSITION.to_string()),
        attribute_type: Some(AttributeType::Vec3),
        component_datatype: Some(ComponentDatatype::Float),
        buffer: Some(vertex_buffer),
        byte_offset: 0,
        byte_stride: Some(12),
        normalized: false,
        ..Default::default()
    };

    let primitive = Primitive {
        morph_targets: vec![MorphTarget {
            attributes: vec![position_attr],
        }],
        ..Default::default()
    };

    MorphTargetsPipelineStage::process(&mut render_resources, &primitive);

    let attributes = &render_resources.attributes;
    assert_eq!(attributes.len(), 1);

    let pos = &attributes[0];
    assert_eq!(pos.index, 1);
    assert!(pos.vertex_buffer.is_some());
    assert_eq!(pos.components_per_attribute, 3);
    assert_eq!(pos.component_datatype, ComponentDatatype::Float);
    assert_eq!(pos.offset_in_bytes, 0);
    assert_eq!(pos.stride_in_bytes, 12);

    let sb = &render_resources.shader_builder;

    // Check vertex function getMorphedPosition
    let func = sb.get_function(MorphTargetsPipelineStage::FUNCTION_ID_GET_MORPHED_POSITION);
    assert!(func.is_some());
    let func = func.unwrap();
    assert_eq!(
        func.signature,
        MorphTargetsPipelineStage::FUNCTION_SIGNATURE_GET_MORPHED_POSITION
    );
    assert_eq!(
        func.body,
        vec![
            "    vec3 morphedPosition = position;",
            "    morphedPosition += u_morphWeights[0] * a_targetPosition_0;",
            "    return morphedPosition;",
        ]
    );

    // Check vertex defines
    assert_eq!(sb.vertex_define_lines(), &["HAS_MORPH_TARGETS".to_string()]);

    // Check attributes
    assert_eq!(sb.attribute_lines(), &["in vec3 a_targetPosition_0;".to_string()]);

    // Check vertex lines equal MORPH_TARGETS_STAGE_VS
    assert_eq!(sb.vertex_lines(), &[MORPH_TARGETS_STAGE_VS.to_string()]);

    // Check vertex uniforms
    assert_eq!(
        sb.vertex_uniform_lines(),
        &["uniform float u_morphWeights[1];".to_string()]
    );

    // Check uniform map
    assert_eq!(render_resources.uniform_morph_weights, Some(vec![1.0]));
}

#[test]
fn test_processes_morph_target_with_position_normal_and_tangent() {
    let mut render_resources = MockMorphTargetsRenderResources {
        attributes: Vec::new(),
        attribute_index: 1,
        morph_weights: vec![0.5, 0.5],
        ..Default::default()
    };

    let vertex_buffer = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 48,
        ..Default::default()
    })
    .expect("Buffer creation");

    let create_target = |buffer: Buffer| -> MorphTarget {
        let pos = Attribute {
            semantic: Some(VertexAttributeSemantic::POSITION.to_string()),
            attribute_type: Some(AttributeType::Vec3),
            component_datatype: Some(ComponentDatatype::Float),
            buffer: Some(buffer.clone()),
            byte_offset: 0,
            byte_stride: Some(12),
            ..Default::default()
        };
        let norm = Attribute {
            semantic: Some(VertexAttributeSemantic::NORMAL.to_string()),
            attribute_type: Some(AttributeType::Vec3),
            component_datatype: Some(ComponentDatatype::Float),
            buffer: Some(buffer.clone()),
            byte_offset: 0,
            byte_stride: Some(12),
            ..Default::default()
        };
        let tan = Attribute {
            semantic: Some(VertexAttributeSemantic::TANGENT.to_string()),
            attribute_type: Some(AttributeType::Vec3),
            component_datatype: Some(ComponentDatatype::Float),
            buffer: Some(buffer),
            byte_offset: 0,
            byte_stride: Some(12),
            ..Default::default()
        };
        MorphTarget {
            attributes: vec![pos, norm, tan],
        }
    };

    let primitive = Primitive {
        morph_targets: vec![
            create_target(vertex_buffer.clone()),
            create_target(vertex_buffer),
        ],
        ..Default::default()
    };

    MorphTargetsPipelineStage::process(&mut render_resources, &primitive);

    let attributes = &render_resources.attributes;
    assert_eq!(attributes.len(), 6);

    for (i, attr) in attributes.iter().enumerate() {
        assert_eq!(attr.index, (i + 1) as u32);
        assert!(attr.vertex_buffer.is_some());
        assert_eq!(attr.components_per_attribute, 3);
        assert_eq!(attr.component_datatype, ComponentDatatype::Float);
        assert_eq!(attr.offset_in_bytes, 0);
        assert_eq!(attr.stride_in_bytes, 12);
    }

    let sb = &render_resources.shader_builder;

    let func_pos = sb
        .get_function(MorphTargetsPipelineStage::FUNCTION_ID_GET_MORPHED_POSITION)
        .unwrap();
    assert_eq!(
        func_pos.signature,
        MorphTargetsPipelineStage::FUNCTION_SIGNATURE_GET_MORPHED_POSITION
    );
    assert_eq!(
        func_pos.body,
        vec![
            "    vec3 morphedPosition = position;",
            "    morphedPosition += u_morphWeights[0] * a_targetPosition_0;",
            "    morphedPosition += u_morphWeights[1] * a_targetPosition_1;",
            "    return morphedPosition;",
        ]
    );

    let func_norm = sb
        .get_function(MorphTargetsPipelineStage::FUNCTION_ID_GET_MORPHED_NORMAL)
        .unwrap();
    assert_eq!(
        func_norm.signature,
        MorphTargetsPipelineStage::FUNCTION_SIGNATURE_GET_MORPHED_NORMAL
    );
    assert_eq!(
        func_norm.body,
        vec![
            "    vec3 morphedNormal = normal;",
            "    morphedNormal += u_morphWeights[0] * a_targetNormal_0;",
            "    morphedNormal += u_morphWeights[1] * a_targetNormal_1;",
            "    return morphedNormal;",
        ]
    );

    let func_tan = sb
        .get_function(MorphTargetsPipelineStage::FUNCTION_ID_GET_MORPHED_TANGENT)
        .unwrap();
    assert_eq!(
        func_tan.signature,
        MorphTargetsPipelineStage::FUNCTION_SIGNATURE_GET_MORPHED_TANGENT
    );
    assert_eq!(
        func_tan.body,
        vec![
            "    vec3 morphedTangent = tangent;",
            "    morphedTangent += u_morphWeights[0] * a_targetTangent_0;",
            "    morphedTangent += u_morphWeights[1] * a_targetTangent_1;",
            "    return morphedTangent;",
        ]
    );

    assert_eq!(sb.vertex_define_lines(), &["HAS_MORPH_TARGETS".to_string()]);

    assert_eq!(
        sb.attribute_lines(),
        &[
            "in vec3 a_targetPosition_0;".to_string(),
            "in vec3 a_targetNormal_0;".to_string(),
            "in vec3 a_targetTangent_0;".to_string(),
            "in vec3 a_targetPosition_1;".to_string(),
            "in vec3 a_targetNormal_1;".to_string(),
            "in vec3 a_targetTangent_1;".to_string(),
        ]
    );

    assert_eq!(sb.vertex_lines(), &[MORPH_TARGETS_STAGE_VS.to_string()]);

    assert_eq!(
        sb.vertex_uniform_lines(),
        &["uniform float u_morphWeights[2];".to_string()]
    );

    assert_eq!(
        render_resources.uniform_morph_weights,
        Some(vec![0.5, 0.5])
    );
}
