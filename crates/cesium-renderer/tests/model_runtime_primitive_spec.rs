use cesium_math::PrimitiveType;
use cesium_renderer::model_components::{
    Attribute, FeatureId, FeatureIdAttribute, Instances, MorphTarget, Node, Primitive, Quantization,
};
use cesium_renderer::{
    ClassificationType, CustomShaderMode, CustomShaderOptions, ModelRuntimePrimitive,
    ModelRuntimePrimitiveArgs, ModelRuntimePrimitiveModel, ModelType, PointCloudShadingOptions,
    PrimitiveFrameState, PrimitivePipelineStage, SceneMode,
};

#[test]
fn test_throws_for_undefined_arguments() {
    let mock_primitive = Primitive::default();
    let mock_node = Node::default();
    let mock_model = ModelRuntimePrimitiveModel::default();

    // throws for undefined primitive
    let err = ModelRuntimePrimitive::try_new(ModelRuntimePrimitiveArgs {
        primitive: None,
        node: Some(mock_node.clone()),
        model: Some(mock_model.clone()),
    })
    .unwrap_err();
    assert_eq!(err.message, "options.primitive is required.");

    // throws for undefined node
    let err = ModelRuntimePrimitive::try_new(ModelRuntimePrimitiveArgs {
        primitive: Some(mock_primitive.clone()),
        node: None,
        model: Some(mock_model.clone()),
    })
    .unwrap_err();
    assert_eq!(err.message, "options.node is required.");

    // throws for undefined model
    let err = ModelRuntimePrimitive::try_new(ModelRuntimePrimitiveArgs {
        primitive: Some(mock_primitive),
        node: Some(mock_node),
        model: None,
    })
    .unwrap_err();
    assert_eq!(err.message, "options.model is required.");
}

#[test]
fn test_constructs() {
    let mock_primitive = Primitive::default();
    let mock_node = Node::default();
    let mock_model = ModelRuntimePrimitiveModel::default();

    let prim = ModelRuntimePrimitive::try_new(ModelRuntimePrimitiveArgs {
        primitive: Some(mock_primitive),
        node: Some(mock_node),
        model: Some(mock_model),
    })
    .unwrap();

    assert!(prim.primitive.is_some());
    assert!(prim.node.is_some());
    assert!(prim.model.is_some());
}

#[test]
fn test_configures_pipeline_stages_for_model_picking() {
    let mock_primitive = Primitive {
        primitive_type: Some(PrimitiveType::Triangles),
        ..Default::default()
    };
    let mock_node = Node::default();
    let mut model = ModelRuntimePrimitiveModel {
        model_type: ModelType::Gltf,
        allow_picking: true,
        feature_id_label: Some("featureId_0".to_string()),
        ..Default::default()
    };

    let mut primitive = ModelRuntimePrimitive::try_new(ModelRuntimePrimitiveArgs {
        primitive: Some(mock_primitive.clone()),
        node: Some(mock_node.clone()),
        model: Some(model.clone()),
    })
    .unwrap();

    let frame_state = PrimitiveFrameState::default();
    primitive.configure_pipeline(&frame_state);

    let expected_stages = vec![
        PrimitivePipelineStage::Geometry,
        PrimitivePipelineStage::Material,
        PrimitivePipelineStage::FeatureId,
        PrimitivePipelineStage::Metadata,
        PrimitivePipelineStage::MetadataPicking,
        PrimitivePipelineStage::Lighting,
        PrimitivePipelineStage::Picking,
        PrimitivePipelineStage::Alpha,
        PrimitivePipelineStage::PrimitiveStatistics,
    ];
    assert_eq!(primitive.pipeline_stages, expected_stages);

    // When allowPicking is false, PickingPipelineStage is skipped
    model.allow_picking = false;
    let mut primitive_no_pick = ModelRuntimePrimitive::try_new(ModelRuntimePrimitiveArgs {
        primitive: Some(mock_primitive),
        node: Some(mock_node),
        model: Some(model),
    })
    .unwrap();

    primitive_no_pick.configure_pipeline(&frame_state);
    let expected_stages_no_pick = vec![
        PrimitivePipelineStage::Geometry,
        PrimitivePipelineStage::Material,
        PrimitivePipelineStage::FeatureId,
        PrimitivePipelineStage::Metadata,
        PrimitivePipelineStage::MetadataPicking,
        PrimitivePipelineStage::Lighting,
        PrimitivePipelineStage::Alpha,
        PrimitivePipelineStage::PrimitiveStatistics,
    ];
    assert_eq!(primitive_no_pick.pipeline_stages, expected_stages_no_pick);
}

#[test]
fn test_configures_pipeline_stages_for_instance_feature_picking() {
    let mock_primitive = Primitive {
        primitive_type: Some(PrimitiveType::Triangles),
        ..Default::default()
    };
    let mock_node = Node {
        instances: Some(Instances {
            feature_ids: vec![
                FeatureId::Attribute(FeatureIdAttribute {
                    label: Some("defaultIds".to_string()),
                    positional_label: Some("instanceFeatureId_0".to_string()),
                    property_table_id: None,
                    ..Default::default()
                }),
                FeatureId::Attribute(FeatureIdAttribute {
                    label: None,
                    positional_label: Some("instanceFeatureId_1".to_string()),
                    property_table_id: Some(0),
                    ..Default::default()
                }),
            ],
            ..Default::default()
        }),
        ..Default::default()
    };
    let model = ModelRuntimePrimitiveModel {
        model_type: ModelType::Gltf,
        allow_picking: true,
        instance_feature_id_label: Some("instanceFeatureId_1".to_string()),
        ..Default::default()
    };

    let mut primitive = ModelRuntimePrimitive::try_new(ModelRuntimePrimitiveArgs {
        primitive: Some(mock_primitive),
        node: Some(mock_node),
        model: Some(model),
    })
    .unwrap();

    let frame_state = PrimitiveFrameState::default();
    primitive.configure_pipeline(&frame_state);

    let expected_stages = vec![
        PrimitivePipelineStage::Geometry,
        PrimitivePipelineStage::Material,
        PrimitivePipelineStage::FeatureId,
        PrimitivePipelineStage::Metadata,
        PrimitivePipelineStage::MetadataPicking,
        PrimitivePipelineStage::SelectedFeatureId,
        PrimitivePipelineStage::BatchTexture,
        PrimitivePipelineStage::CPUStyling,
        PrimitivePipelineStage::Lighting,
        PrimitivePipelineStage::Picking,
        PrimitivePipelineStage::Alpha,
        PrimitivePipelineStage::PrimitiveStatistics,
    ];
    assert_eq!(primitive.pipeline_stages, expected_stages);
}

#[test]
fn test_configures_dequantization() {
    let mock_primitive = Primitive {
        primitive_type: Some(PrimitiveType::Triangles),
        attributes: vec![
            Attribute {
                semantic: Some("POSITION".to_string()),
                ..Default::default()
            },
            Attribute {
                semantic: Some("NORMAL".to_string()),
                quantization: Some(Quantization::default()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let mut primitive = ModelRuntimePrimitive::try_new(ModelRuntimePrimitiveArgs {
        primitive: Some(mock_primitive),
        node: Some(Node::default()),
        model: Some(ModelRuntimePrimitiveModel::default()),
    })
    .unwrap();

    let frame_state = PrimitiveFrameState::default();
    primitive.configure_pipeline(&frame_state);

    let expected_stages = vec![
        PrimitivePipelineStage::Geometry,
        PrimitivePipelineStage::Dequantization,
        PrimitivePipelineStage::Material,
        PrimitivePipelineStage::FeatureId,
        PrimitivePipelineStage::Metadata,
        PrimitivePipelineStage::MetadataPicking,
        PrimitivePipelineStage::Lighting,
        PrimitivePipelineStage::Picking,
        PrimitivePipelineStage::Alpha,
        PrimitivePipelineStage::PrimitiveStatistics,
    ];
    assert_eq!(primitive.pipeline_stages, expected_stages);
}

#[test]
fn test_configures_custom_shader_pipeline_stages() {
    let mock_primitive = Primitive {
        primitive_type: Some(PrimitiveType::Triangles),
        ..Default::default()
    };
    let model = ModelRuntimePrimitiveModel {
        model_type: ModelType::Gltf,
        allow_picking: false,
        custom_shader: Some(CustomShaderOptions {
            mode: Some(CustomShaderMode::ModifyMaterial),
            vertex_shader_text: Some("void vertexMain() {}".to_string()),
            fragment_shader_text: Some("void fragmentMain() {}".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut primitive = ModelRuntimePrimitive::try_new(ModelRuntimePrimitiveArgs {
        primitive: Some(mock_primitive.clone()),
        node: Some(Node::default()),
        model: Some(model),
    })
    .unwrap();

    let frame_state = PrimitiveFrameState::default();
    primitive.configure_pipeline(&frame_state);

    let expected_stages = vec![
        PrimitivePipelineStage::Geometry,
        PrimitivePipelineStage::Material,
        PrimitivePipelineStage::FeatureId,
        PrimitivePipelineStage::Metadata,
        PrimitivePipelineStage::MetadataPicking,
        PrimitivePipelineStage::CustomShader,
        PrimitivePipelineStage::Lighting,
        PrimitivePipelineStage::Alpha,
        PrimitivePipelineStage::PrimitiveStatistics,
    ];
    assert_eq!(primitive.pipeline_stages, expected_stages);

    // Disables material stage when mode is REPLACE_MATERIAL and fragment shader is present
    let model_replace = ModelRuntimePrimitiveModel {
        model_type: ModelType::Gltf,
        allow_picking: false,
        custom_shader: Some(CustomShaderOptions {
            mode: Some(CustomShaderMode::ReplaceMaterial),
            vertex_shader_text: Some("void vertexMain() {}".to_string()),
            fragment_shader_text: Some("void fragmentMain() {}".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut prim_replace = ModelRuntimePrimitive::try_new(ModelRuntimePrimitiveArgs {
        primitive: Some(mock_primitive.clone()),
        node: Some(Node::default()),
        model: Some(model_replace),
    })
    .unwrap();

    prim_replace.configure_pipeline(&frame_state);
    let expected_replace = vec![
        PrimitivePipelineStage::Geometry,
        PrimitivePipelineStage::FeatureId,
        PrimitivePipelineStage::Metadata,
        PrimitivePipelineStage::MetadataPicking,
        PrimitivePipelineStage::CustomShader,
        PrimitivePipelineStage::Lighting,
        PrimitivePipelineStage::Alpha,
        PrimitivePipelineStage::PrimitiveStatistics,
    ];
    assert_eq!(prim_replace.pipeline_stages, expected_replace);

    // Does not disable material stage if custom shader has no fragment shader
    let model_no_fs = ModelRuntimePrimitiveModel {
        model_type: ModelType::Gltf,
        allow_picking: false,
        custom_shader: Some(CustomShaderOptions {
            mode: Some(CustomShaderMode::ReplaceMaterial),
            vertex_shader_text: Some("void vertexMain() {}".to_string()),
            fragment_shader_text: None,
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut prim_no_fs = ModelRuntimePrimitive::try_new(ModelRuntimePrimitiveArgs {
        primitive: Some(mock_primitive),
        node: Some(Node::default()),
        model: Some(model_no_fs),
    })
    .unwrap();

    prim_no_fs.configure_pipeline(&frame_state);
    assert_eq!(prim_no_fs.pipeline_stages, expected_stages);
}

#[test]
fn test_configures_point_cloud_shading_and_styling() {
    let mock_primitive_points = Primitive {
        primitive_type: Some(PrimitiveType::Points),
        ..Default::default()
    };

    // 1. Attenuation enabled on points
    let model_attenuation = ModelRuntimePrimitiveModel {
        model_type: ModelType::TilePnts,
        allow_picking: false,
        point_cloud_shading: Some(PointCloudShadingOptions {
            attenuation: true,
            back_face_culling: false,
        }),
        ..Default::default()
    };

    let mut prim1 = ModelRuntimePrimitive::try_new(ModelRuntimePrimitiveArgs {
        primitive: Some(mock_primitive_points.clone()),
        node: Some(Node::default()),
        model: Some(model_attenuation),
    })
    .unwrap();

    let frame_state = PrimitiveFrameState::default();
    prim1.configure_pipeline(&frame_state);

    let expected_stages_point_cloud = vec![
        PrimitivePipelineStage::Geometry,
        PrimitivePipelineStage::PointCloudStyling,
        PrimitivePipelineStage::Material,
        PrimitivePipelineStage::FeatureId,
        PrimitivePipelineStage::Metadata,
        PrimitivePipelineStage::MetadataPicking,
        PrimitivePipelineStage::Lighting,
        PrimitivePipelineStage::Alpha,
        PrimitivePipelineStage::PrimitiveStatistics,
    ];
    assert_eq!(prim1.pipeline_stages, expected_stages_point_cloud);

    // 2. Attenuation disabled -> skips PointCloudStyling
    let model_no_attenuation = ModelRuntimePrimitiveModel {
        model_type: ModelType::TilePnts,
        allow_picking: false,
        point_cloud_shading: Some(PointCloudShadingOptions {
            attenuation: false,
            back_face_culling: false,
        }),
        ..Default::default()
    };

    let mut prim2 = ModelRuntimePrimitive::try_new(ModelRuntimePrimitiveArgs {
        primitive: Some(mock_primitive_points.clone()),
        node: Some(Node::default()),
        model: Some(model_no_attenuation),
    })
    .unwrap();

    prim2.configure_pipeline(&frame_state);
    let expected_stages_no_attenuation = vec![
        PrimitivePipelineStage::Geometry,
        PrimitivePipelineStage::Material,
        PrimitivePipelineStage::FeatureId,
        PrimitivePipelineStage::Metadata,
        PrimitivePipelineStage::MetadataPicking,
        PrimitivePipelineStage::Lighting,
        PrimitivePipelineStage::Alpha,
        PrimitivePipelineStage::PrimitiveStatistics,
    ];
    assert_eq!(prim2.pipeline_stages, expected_stages_no_attenuation);

    // 3. Style enabled -> includes PointCloudStyling
    let model_style = ModelRuntimePrimitiveModel {
        model_type: ModelType::TilePnts,
        allow_picking: false,
        has_style: true,
        ..Default::default()
    };

    let mut prim3 = ModelRuntimePrimitive::try_new(ModelRuntimePrimitiveArgs {
        primitive: Some(mock_primitive_points),
        node: Some(Node::default()),
        model: Some(model_style),
    })
    .unwrap();

    prim3.configure_pipeline(&frame_state);
    assert_eq!(prim3.pipeline_stages, expected_stages_point_cloud);
}

#[test]
fn test_configures_morph_targets_and_skinning() {
    let mock_primitive_morph = Primitive {
        primitive_type: Some(PrimitiveType::Triangles),
        morph_targets: vec![MorphTarget::default()],
        ..Default::default()
    };
    let model = ModelRuntimePrimitiveModel {
        allow_picking: false,
        ..Default::default()
    };

    let mut prim_morph = ModelRuntimePrimitive::try_new(ModelRuntimePrimitiveArgs {
        primitive: Some(mock_primitive_morph),
        node: Some(Node::default()),
        model: Some(model.clone()),
    })
    .unwrap();

    let frame_state = PrimitiveFrameState::default();
    prim_morph.configure_pipeline(&frame_state);

    let expected_morph = vec![
        PrimitivePipelineStage::Geometry,
        PrimitivePipelineStage::MorphTargets,
        PrimitivePipelineStage::Material,
        PrimitivePipelineStage::FeatureId,
        PrimitivePipelineStage::Metadata,
        PrimitivePipelineStage::MetadataPicking,
        PrimitivePipelineStage::Lighting,
        PrimitivePipelineStage::Alpha,
        PrimitivePipelineStage::PrimitiveStatistics,
    ];
    assert_eq!(prim_morph.pipeline_stages, expected_morph);

    // Skinning
    let mock_primitive_skin = Primitive {
        primitive_type: Some(PrimitiveType::Triangles),
        ..Default::default()
    };
    let node_skin = Node {
        skin: Some(0),
        ..Default::default()
    };

    let mut prim_skin = ModelRuntimePrimitive::try_new(ModelRuntimePrimitiveArgs {
        primitive: Some(mock_primitive_skin),
        node: Some(node_skin),
        model: Some(model),
    })
    .unwrap();

    prim_skin.configure_pipeline(&frame_state);
    let expected_skin = vec![
        PrimitivePipelineStage::Geometry,
        PrimitivePipelineStage::Skinning,
        PrimitivePipelineStage::Material,
        PrimitivePipelineStage::FeatureId,
        PrimitivePipelineStage::Metadata,
        PrimitivePipelineStage::MetadataPicking,
        PrimitivePipelineStage::Lighting,
        PrimitivePipelineStage::Alpha,
        PrimitivePipelineStage::PrimitiveStatistics,
    ];
    assert_eq!(prim_skin.pipeline_stages, expected_skin);
}

#[test]
fn test_configures_wireframe_and_2d() {
    let mock_primitive_triangles = Primitive {
        primitive_type: Some(PrimitiveType::Triangles),
        ..Default::default()
    };

    // WebGL 1 debugWireframe = true, enableDebugWireframe = true
    let model_wf_webgl1 = ModelRuntimePrimitiveModel {
        allow_picking: false,
        debug_wireframe: true,
        enable_debug_wireframe: true,
        ..Default::default()
    };

    let mut prim_wf1 = ModelRuntimePrimitive::try_new(ModelRuntimePrimitiveArgs {
        primitive: Some(mock_primitive_triangles.clone()),
        node: Some(Node::default()),
        model: Some(model_wf_webgl1),
    })
    .unwrap();

    let frame_state_webgl1 = PrimitiveFrameState {
        webgl2: false,
        ..Default::default()
    };
    prim_wf1.configure_pipeline(&frame_state_webgl1);

    let expected_wf = vec![
        PrimitivePipelineStage::Geometry,
        PrimitivePipelineStage::Wireframe,
        PrimitivePipelineStage::Material,
        PrimitivePipelineStage::FeatureId,
        PrimitivePipelineStage::Metadata,
        PrimitivePipelineStage::MetadataPicking,
        PrimitivePipelineStage::Lighting,
        PrimitivePipelineStage::Alpha,
        PrimitivePipelineStage::PrimitiveStatistics,
    ];
    assert_eq!(prim_wf1.pipeline_stages, expected_wf);

    // 2D Scene Mode
    let model_2d = ModelRuntimePrimitiveModel {
        allow_picking: false,
        project_to_2d: true,
        ..Default::default()
    };

    let mut prim_2d = ModelRuntimePrimitive::try_new(ModelRuntimePrimitiveArgs {
        primitive: Some(mock_primitive_triangles),
        node: Some(Node::default()),
        model: Some(model_2d),
    })
    .unwrap();

    let frame_state_2d = PrimitiveFrameState {
        mode: SceneMode::Scene2D,
        scene_3d_only: false,
        ..Default::default()
    };
    prim_2d.configure_pipeline(&frame_state_2d);

    let expected_2d = vec![
        PrimitivePipelineStage::SceneMode2D,
        PrimitivePipelineStage::Geometry,
        PrimitivePipelineStage::Material,
        PrimitivePipelineStage::FeatureId,
        PrimitivePipelineStage::Metadata,
        PrimitivePipelineStage::MetadataPicking,
        PrimitivePipelineStage::Lighting,
        PrimitivePipelineStage::Alpha,
        PrimitivePipelineStage::PrimitiveStatistics,
    ];
    assert_eq!(prim_2d.pipeline_stages, expected_2d);
}

#[test]
fn test_configures_classification_outlines_and_vertical_exaggeration() {
    let mock_primitive = Primitive {
        primitive_type: Some(PrimitiveType::Triangles),
        outline_coordinates: Some(Attribute::default()),
        ..Default::default()
    };

    let model = ModelRuntimePrimitiveModel {
        allow_picking: false,
        classification_type: Some(ClassificationType::Both),
        enable_show_outline: true,
        has_vertical_exaggeration: true,
        ..Default::default()
    };

    let mut prim = ModelRuntimePrimitive::try_new(ModelRuntimePrimitiveArgs {
        primitive: Some(mock_primitive),
        node: Some(Node::default()),
        model: Some(model),
    })
    .unwrap();

    let frame_state = PrimitiveFrameState {
        vertical_exaggeration: 2.0,
        ..Default::default()
    };
    prim.configure_pipeline(&frame_state);

    let expected_stages = vec![
        PrimitivePipelineStage::Geometry,
        PrimitivePipelineStage::Classification,
        PrimitivePipelineStage::Material,
        PrimitivePipelineStage::FeatureId,
        PrimitivePipelineStage::Metadata,
        PrimitivePipelineStage::MetadataPicking,
        PrimitivePipelineStage::VerticalExaggeration,
        PrimitivePipelineStage::Lighting,
        PrimitivePipelineStage::PrimitiveOutline,
        PrimitivePipelineStage::Alpha,
        PrimitivePipelineStage::PrimitiveStatistics,
    ];
    assert_eq!(prim.pipeline_stages, expected_stages);
}
