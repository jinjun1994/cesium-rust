use cesium_math::PrimitiveType;
use cesium_renderer::{
    ClassificationPipelineStage, ClassificationStageAttribute, ClassificationStageIndices,
    ClassificationStagePrimitive, ClassificationStageRenderResources,
    ClassificationStageRuntimePrimitive, ShaderBuilder, VertexAttributeSemantic,
};

fn generate_feature_ids(batch_lengths: &[usize]) -> Vec<u32> {
    let mut feature_ids = Vec::new();
    for (id, &batch_length) in batch_lengths.iter().enumerate() {
        for _ in 0..batch_length {
            feature_ids.push(id as u32);
        }
    }
    feature_ids
}

fn mock_primitive(feature_ids: Vec<u32>, indices: Option<ClassificationStageIndices>) -> ClassificationStagePrimitive {
    let count = feature_ids.len();
    let position_attribute = ClassificationStageAttribute {
        semantic: Some(VertexAttributeSemantic::POSITION.to_string()),
        set_index: None,
        count,
        typed_array: None,
    };

    let feature_id_attribute = ClassificationStageAttribute {
        semantic: Some(VertexAttributeSemantic::FEATURE_ID.to_string()),
        set_index: Some(0),
        count,
        typed_array: Some(feature_ids),
    };

    ClassificationStagePrimitive {
        primitive_type: Some(PrimitiveType::Triangles),
        attributes: vec![position_attribute, feature_id_attribute],
        indices,
    }
}

#[test]
fn test_throws_for_primitive_with_no_position_attribute() {
    let input_batch_lengths = [6];
    let feature_ids = generate_feature_ids(&input_batch_lengths);

    let mut primitive = mock_primitive(feature_ids, None);
    primitive.attributes[0].semantic = Some(VertexAttributeSemantic::NORMAL.to_string());

    let mut shader_builder = ShaderBuilder::new();
    let mut runtime_primitive = ClassificationStageRuntimePrimitive::default();
    let mut render_resources = ClassificationStageRenderResources {
        shader_builder: &mut shader_builder,
        runtime_primitive: &mut runtime_primitive,
    };

    let result = ClassificationPipelineStage::process(&mut render_resources, &mut primitive);
    assert!(result.is_err());
}

#[test]
fn test_adds_classification_define_to_the_shader() {
    let input_batch_lengths = [6];
    let feature_ids = generate_feature_ids(&input_batch_lengths);
    let mut primitive = mock_primitive(feature_ids, None);

    let mut shader_builder = ShaderBuilder::new();
    let mut runtime_primitive = ClassificationStageRuntimePrimitive::default();
    let mut render_resources = ClassificationStageRenderResources {
        shader_builder: &mut shader_builder,
        runtime_primitive: &mut runtime_primitive,
    };

    let result = ClassificationPipelineStage::process(&mut render_resources, &mut primitive);
    assert!(result.is_ok());

    let vs = render_resources.shader_builder.create_vertex_shader_source();
    let fs = render_resources.shader_builder.create_fragment_shader_source();

    assert!(vs.defines.iter().any(|d| d == "HAS_CLASSIFICATION"));
    assert!(fs.defines.iter().any(|d| d == "HAS_CLASSIFICATION"));
}

#[test]
fn test_computes_single_batch_for_primitive_without_feature_ids() {
    let input_batch_lengths = [6];
    let feature_ids = generate_feature_ids(&input_batch_lengths);
    let mut primitive = mock_primitive(feature_ids, None);
    primitive.attributes.truncate(1); // Removes the feature ID attribute.

    let mut shader_builder = ShaderBuilder::new();
    let mut runtime_primitive = ClassificationStageRuntimePrimitive::default();
    let mut render_resources = ClassificationStageRenderResources {
        shader_builder: &mut shader_builder,
        runtime_primitive: &mut runtime_primitive,
    };

    let result = ClassificationPipelineStage::process(&mut render_resources, &mut primitive);
    assert!(result.is_ok());

    assert_eq!(render_resources.runtime_primitive.batch_lengths, Some(vec![6]));
    assert_eq!(render_resources.runtime_primitive.batch_offsets, Some(vec![0]));
}

#[test]
fn test_computes_single_batch_for_primitive_with_feature_ids() {
    let input_batch_lengths = [6];
    let feature_ids = generate_feature_ids(&input_batch_lengths);
    let mut primitive = mock_primitive(feature_ids, None);

    let mut shader_builder = ShaderBuilder::new();
    let mut runtime_primitive = ClassificationStageRuntimePrimitive::default();
    let mut render_resources = ClassificationStageRenderResources {
        shader_builder: &mut shader_builder,
        runtime_primitive: &mut runtime_primitive,
    };

    let result = ClassificationPipelineStage::process(&mut render_resources, &mut primitive);
    assert!(result.is_ok());

    assert_eq!(render_resources.runtime_primitive.batch_lengths, Some(vec![6]));
    assert_eq!(render_resources.runtime_primitive.batch_offsets, Some(vec![0]));
}

#[test]
fn test_computes_multiple_batches_for_primitive_with_feature_ids() {
    let input_batch_lengths = [6, 3, 9, 12];
    let feature_ids = generate_feature_ids(&input_batch_lengths);
    let mut primitive = mock_primitive(feature_ids, None);

    let mut shader_builder = ShaderBuilder::new();
    let mut runtime_primitive = ClassificationStageRuntimePrimitive::default();
    let mut render_resources = ClassificationStageRenderResources {
        shader_builder: &mut shader_builder,
        runtime_primitive: &mut runtime_primitive,
    };

    let result = ClassificationPipelineStage::process(&mut render_resources, &mut primitive);
    assert!(result.is_ok());

    assert_eq!(
        render_resources.runtime_primitive.batch_lengths,
        Some(vec![6, 3, 9, 12])
    );
    assert_eq!(
        render_resources.runtime_primitive.batch_offsets,
        Some(vec![0, 6, 9, 18])
    );
}

#[test]
fn test_computes_multiple_batches_for_primitive_with_feature_ids_and_indices() {
    // The feature IDs are intentionally scrambled to demonstrate that only
    // the indices that correspond to the same feature ID need to be contiguous.
    let feature_ids = vec![2, 1, 0, 0, 1, 2, 2, 1, 0, 0, 2];
    let indices = ClassificationStageIndices {
        typed_array: Some(vec![
            2, 3, 8, 2, 8, 9,  // Two triangles with feature ID 0
            1, 4, 7,           // One triangle with feature ID 1
            0, 5, 6, 0, 6, 10, // Two triangles with feature ID 2
        ]),
        count: 15,
    };

    let mut primitive = mock_primitive(feature_ids, Some(indices));

    let mut shader_builder = ShaderBuilder::new();
    let mut runtime_primitive = ClassificationStageRuntimePrimitive::default();
    let mut render_resources = ClassificationStageRenderResources {
        shader_builder: &mut shader_builder,
        runtime_primitive: &mut runtime_primitive,
    };

    let result = ClassificationPipelineStage::process(&mut render_resources, &mut primitive);
    assert!(result.is_ok());

    assert_eq!(
        render_resources.runtime_primitive.batch_lengths,
        Some(vec![6, 3, 6])
    );
    assert_eq!(
        render_resources.runtime_primitive.batch_offsets,
        Some(vec![0, 6, 9])
    );
}

#[test]
fn test_does_not_recompute_batches_for_primitive_if_they_are_already_defined() {
    let input_batch_lengths = [6];
    let feature_ids = generate_feature_ids(&input_batch_lengths);
    let mut primitive = mock_primitive(feature_ids, None);

    let mut shader_builder = ShaderBuilder::new();
    let mut runtime_primitive = ClassificationStageRuntimePrimitive {
        batch_lengths: Some(Vec::new()),
        batch_offsets: Some(Vec::new()),
    };
    let mut render_resources = ClassificationStageRenderResources {
        shader_builder: &mut shader_builder,
        runtime_primitive: &mut runtime_primitive,
    };

    let result = ClassificationPipelineStage::process(&mut render_resources, &mut primitive);
    assert!(result.is_ok());

    assert_eq!(render_resources.runtime_primitive.batch_lengths, Some(vec![]));
    assert_eq!(render_resources.runtime_primitive.batch_offsets, Some(vec![]));
}
