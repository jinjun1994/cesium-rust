use cesium_math::{AttributeType, ComponentDatatype, PrimitiveType, SceneMode};
use cesium_renderer::buffer::{Buffer, BufferOptions, BufferUsage};
use cesium_renderer::geometry_pipeline_stage::{
    GeometryFrameState, GeometryModelOptions, GeometryPipelineStage, GeometryRenderResources,
    GeometryRuntimeNode, GeometryRuntimePrimitive,
};
use cesium_renderer::model_components::{Attribute, Primitive, Quantization};
use cesium_renderer::shader_builder::ShaderBuilder;

fn create_test_buffer(bytes: &[u8]) -> Buffer {
    Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 0,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: Some(bytes.to_vec()),
    })
    .unwrap()
}

fn create_mock_resources(
    shader_builder: &mut ShaderBuilder,
) -> GeometryRenderResources<'_> {
    GeometryRenderResources {
        shader_builder,
        attributes: Vec::new(),
        attribute_index: 1,
        count: 2,
        model: GeometryModelOptions::default(),
        runtime_node: GeometryRuntimeNode::default(),
        runtime_primitive: GeometryRuntimePrimitive::default(),
        feature_id_vertex_attribute_set_index: 0,
    }
}

#[test]
fn test_processes_position_attribute_from_primitive() {
    let mut sb = ShaderBuilder::default();
    let mut rr = create_mock_resources(&mut sb);

    let pos_buf = create_test_buffer(&[0u8; 24]);
    let primitive = Primitive {
        attributes: vec![Attribute {
            semantic: Some("POSITION".to_string()),
            count: 2,
            attribute_type: Some(AttributeType::Vec3),
            component_datatype: Some(ComponentDatatype::Float),
            byte_offset: 0,
            byte_stride: Some(12),
            buffer: Some(pos_buf),
            ..Default::default()
        }],
        ..Default::default()
    };

    let frame_state = GeometryFrameState::default();
    GeometryPipelineStage::process(&mut rr, &primitive, &frame_state).unwrap();

    assert_eq!(rr.attributes.len(), 1);
    let pos_attr = &rr.attributes[0];
    assert_eq!(pos_attr.index, 0);
    assert!(pos_attr.vertex_buffer.is_some());
    assert_eq!(pos_attr.components_per_attribute, 3);
    assert_eq!(pos_attr.component_datatype, ComponentDatatype::Float);
    assert_eq!(pos_attr.offset_in_bytes, 0);
    assert_eq!(pos_attr.stride_in_bytes, Some(12));

    let vs = rr.shader_builder.create_vertex_shader_source().sources.join("\n");
    let fs = rr.shader_builder.create_fragment_shader_source().sources.join("\n");

    assert!(vs.contains("vec3 positionMC;"));
    assert!(fs.contains("vec3 positionMC;"));
    assert!(fs.contains("vec3 positionWC;"));
    assert!(fs.contains("vec3 positionEC;"));

    assert!(vs.contains("struct SelectedFeature"));
    assert!(fs.contains("struct SelectedFeature"));

    assert!(vs.contains("attributes.positionMC = a_positionMC;"));
    assert!(vs.contains("in vec3 a_positionMC;"));

    assert!(vs.contains("out vec3 v_positionMC;"));
    assert!(vs.contains("out vec3 v_positionEC;"));
    assert!(vs.contains("out vec3 v_positionWC;"));
    assert!(fs.contains("in vec3 v_positionMC;"));
    assert!(fs.contains("in vec3 v_positionEC;"));
    assert!(fs.contains("in vec3 v_positionWC;"));
}

#[test]
fn test_processes_position_normal_and_texcoord_attributes_from_primitive() {
    let mut sb = ShaderBuilder::default();
    let mut rr = create_mock_resources(&mut sb);

    let norm_buf = create_test_buffer(&[0u8; 24]);
    let pos_buf = create_test_buffer(&[0u8; 300]);
    let tex_buf = create_test_buffer(&[0u8; 16]);

    let primitive = Primitive {
        attributes: vec![
            Attribute {
                semantic: Some("NORMAL".to_string()),
                count: 2,
                attribute_type: Some(AttributeType::Vec3),
                component_datatype: Some(ComponentDatatype::Float),
                byte_offset: 0,
                byte_stride: Some(12),
                buffer: Some(norm_buf),
                ..Default::default()
            },
            Attribute {
                semantic: Some("POSITION".to_string()),
                count: 2,
                attribute_type: Some(AttributeType::Vec3),
                component_datatype: Some(ComponentDatatype::Float),
                byte_offset: 288,
                byte_stride: Some(12),
                buffer: Some(pos_buf),
                ..Default::default()
            },
            Attribute {
                semantic: Some("TEXCOORD".to_string()),
                set_index: Some(0),
                count: 2,
                attribute_type: Some(AttributeType::Vec2),
                component_datatype: Some(ComponentDatatype::Float),
                byte_offset: 0,
                byte_stride: Some(8),
                buffer: Some(tex_buf),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let frame_state = GeometryFrameState::default();
    GeometryPipelineStage::process(&mut rr, &primitive, &frame_state).unwrap();

    assert_eq!(rr.attributes.len(), 3);
    assert_eq!(rr.attributes[0].index, 1);
    assert_eq!(rr.attributes[1].index, 0); // Position is 0
    assert_eq!(rr.attributes[2].index, 2);

    let vs_source = rr.shader_builder.create_vertex_shader_source();
    let fs_source = rr.shader_builder.create_fragment_shader_source();
    let vs = vs_source.sources.join("\n");
    let fs = fs_source.sources.join("\n");

    assert!(vs_source.defines.contains(&"HAS_NORMALS".to_string()));
    assert!(vs_source.defines.contains(&"HAS_TEXCOORD_0".to_string()));
    assert!(fs_source.defines.contains(&"HAS_NORMALS".to_string()));
    assert!(fs_source.defines.contains(&"HAS_TEXCOORD_0".to_string()));

    assert!(vs.contains("vec3 normalMC;"));
    assert!(vs.contains("vec2 texCoord_0;"));
    assert!(fs.contains("vec3 normalEC;"));
    assert!(fs.contains("vec2 texCoord_0;"));

    assert!(vs.contains("attributes.normalMC = a_normalMC;"));
    assert!(vs.contains("attributes.texCoord_0 = a_texCoord_0;"));
    assert!(vs.contains("v_texCoord_0 = attributes.texCoord_0;"));
    assert!(fs.contains("attributes.texCoord_0 = v_texCoord_0;"));
}

#[test]
fn test_processes_position_attribute_from_primitive_for_2d() {
    let mut sb = ShaderBuilder::default();
    let mut rr = create_mock_resources(&mut sb);
    rr.model.project_to_2d = true;

    let pos2d_buf = create_test_buffer(&[0u8; 24]);
    rr.runtime_primitive.position_buffer_2d = Some(pos2d_buf);

    let norm_buf = create_test_buffer(&[0u8; 24]);
    let pos_buf = create_test_buffer(&[0u8; 300]);
    let tex_buf = create_test_buffer(&[0u8; 16]);

    let primitive = Primitive {
        attributes: vec![
            Attribute {
                semantic: Some("NORMAL".to_string()),
                count: 2,
                attribute_type: Some(AttributeType::Vec3),
                component_datatype: Some(ComponentDatatype::Float),
                byte_offset: 0,
                byte_stride: Some(12),
                buffer: Some(norm_buf),
                ..Default::default()
            },
            Attribute {
                semantic: Some("POSITION".to_string()),
                count: 2,
                attribute_type: Some(AttributeType::Vec3),
                component_datatype: Some(ComponentDatatype::Float),
                byte_offset: 288,
                byte_stride: Some(12),
                buffer: Some(pos_buf),
                ..Default::default()
            },
            Attribute {
                semantic: Some("TEXCOORD".to_string()),
                set_index: Some(0),
                count: 2,
                attribute_type: Some(AttributeType::Vec2),
                component_datatype: Some(ComponentDatatype::Float),
                byte_offset: 0,
                byte_stride: Some(8),
                buffer: Some(tex_buf),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let frame_state = GeometryFrameState {
        mode: SceneMode::Scene2D,
        scene_3d_only: false,
        webgl2: true,
    };
    GeometryPipelineStage::process(&mut rr, &primitive, &frame_state).unwrap();

    assert_eq!(rr.attributes.len(), 4);
    assert_eq!(rr.attributes[0].index, 1);
    assert_eq!(rr.attributes[1].index, 0); // 3D position
    assert_eq!(rr.attributes[2].index, 2); // 2D position
    assert_eq!(rr.attributes[3].index, 3); // texCoord

    let vs = rr.shader_builder.create_vertex_shader_source().sources.join("\n");
    assert!(vs.contains("vec3 position2D;"));
    assert!(vs.contains("attributes.position2D = a_position2D;"));
    assert!(vs.contains("in vec3 a_position2D;"));
}

#[test]
fn test_processes_position_normal_texcoord_and_tangent_attributes() {
    let mut sb = ShaderBuilder::default();
    let mut rr = create_mock_resources(&mut sb);

    let tex_buf = create_test_buffer(&[0u8; 16]);
    let norm_buf = create_test_buffer(&[0u8; 24]);
    let tan_buf = create_test_buffer(&[0u8; 32]);
    let pos_buf = create_test_buffer(&[0u8; 24]);

    let primitive = Primitive {
        attributes: vec![
            Attribute {
                semantic: Some("TEXCOORD".to_string()),
                set_index: Some(0),
                count: 2,
                attribute_type: Some(AttributeType::Vec2),
                component_datatype: Some(ComponentDatatype::Float),
                byte_offset: 0,
                byte_stride: Some(8),
                buffer: Some(tex_buf),
                ..Default::default()
            },
            Attribute {
                semantic: Some("NORMAL".to_string()),
                count: 2,
                attribute_type: Some(AttributeType::Vec3),
                component_datatype: Some(ComponentDatatype::Float),
                byte_offset: 0,
                byte_stride: Some(12),
                buffer: Some(norm_buf),
                ..Default::default()
            },
            Attribute {
                semantic: Some("TANGENT".to_string()),
                count: 2,
                attribute_type: Some(AttributeType::Vec4),
                component_datatype: Some(ComponentDatatype::Float),
                byte_offset: 0,
                byte_stride: Some(16),
                buffer: Some(tan_buf),
                ..Default::default()
            },
            Attribute {
                semantic: Some("POSITION".to_string()),
                count: 2,
                attribute_type: Some(AttributeType::Vec3),
                component_datatype: Some(ComponentDatatype::Float),
                byte_offset: 0,
                byte_stride: Some(12),
                buffer: Some(pos_buf),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let frame_state = GeometryFrameState::default();
    GeometryPipelineStage::process(&mut rr, &primitive, &frame_state).unwrap();

    assert_eq!(rr.attributes.len(), 4);
    assert_eq!(rr.attributes[0].index, 1);
    assert_eq!(rr.attributes[1].index, 2);
    assert_eq!(rr.attributes[2].index, 3);
    assert_eq!(rr.attributes[3].index, 0);

    let vs_source = rr.shader_builder.create_vertex_shader_source();
    let fs_source = rr.shader_builder.create_fragment_shader_source();
    let vs = vs_source.sources.join("\n");
    let fs = fs_source.sources.join("\n");

    assert!(vs_source.defines.contains(&"HAS_BITANGENTS".to_string()));
    assert!(vs_source.defines.contains(&"HAS_NORMALS".to_string()));
    assert!(vs_source.defines.contains(&"HAS_TANGENTS".to_string()));
    assert!(vs_source.defines.contains(&"HAS_TEXCOORD_0".to_string()));

    assert!(vs.contains("vec3 tangentMC;"));
    assert!(vs.contains("float tangentSignMC;"));
    assert!(vs.contains("vec3 bitangentMC;"));
    assert!(fs.contains("vec3 tangentEC;"));
    assert!(fs.contains("vec3 bitangentEC;"));

    assert!(vs.contains("attributes.tangentMC = a_tangentMC.xyz;"));
    assert!(vs.contains("attributes.tangentSignMC = a_tangentMC.w;"));
    assert!(vs.contains("out vec3 v_bitangentEC;"));
    assert!(fs.contains("in vec3 v_bitangentEC;"));
}

#[test]
fn test_processes_multiple_texcoord_attributes() {
    let mut sb = ShaderBuilder::default();
    let mut rr = create_mock_resources(&mut sb);

    let pos_buf = create_test_buffer(&[0u8; 24]);
    let tex0_buf = create_test_buffer(&[0u8; 16]);
    let tex1_buf = create_test_buffer(&[0u8; 16]);

    let primitive = Primitive {
        attributes: vec![
            Attribute {
                semantic: Some("POSITION".to_string()),
                count: 2,
                attribute_type: Some(AttributeType::Vec3),
                component_datatype: Some(ComponentDatatype::Float),
                buffer: Some(pos_buf),
                ..Default::default()
            },
            Attribute {
                semantic: Some("TEXCOORD".to_string()),
                set_index: Some(0),
                count: 2,
                attribute_type: Some(AttributeType::Vec2),
                component_datatype: Some(ComponentDatatype::Float),
                buffer: Some(tex0_buf),
                ..Default::default()
            },
            Attribute {
                semantic: Some("TEXCOORD".to_string()),
                set_index: Some(1),
                count: 2,
                attribute_type: Some(AttributeType::Vec2),
                component_datatype: Some(ComponentDatatype::Float),
                buffer: Some(tex1_buf),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let frame_state = GeometryFrameState::default();
    GeometryPipelineStage::process(&mut rr, &primitive, &frame_state).unwrap();

    let vs_source = rr.shader_builder.create_vertex_shader_source();
    let vs = vs_source.sources.join("\n");
    let fs = rr.shader_builder.create_fragment_shader_source().sources.join("\n");

    assert!(vs_source.defines.contains(&"HAS_TEXCOORD_0".to_string()));
    assert!(vs_source.defines.contains(&"HAS_TEXCOORD_1".to_string()));

    assert!(vs.contains("v_texCoord_0 = attributes.texCoord_0;"));
    assert!(vs.contains("v_texCoord_1 = attributes.texCoord_1;"));
    assert!(fs.contains("attributes.texCoord_0 = v_texCoord_0;"));
    assert!(fs.contains("attributes.texCoord_1 = v_texCoord_1;"));
}

#[test]
fn test_processes_position_normal_texcoord_and_color_attributes() {
    let mut sb = ShaderBuilder::default();
    let mut rr = create_mock_resources(&mut sb);

    let pos_buf = create_test_buffer(&[0u8; 24]);
    let col_buf = create_test_buffer(&[0u8; 16]);

    let primitive = Primitive {
        attributes: vec![
            Attribute {
                semantic: Some("POSITION".to_string()),
                count: 2,
                attribute_type: Some(AttributeType::Vec3),
                component_datatype: Some(ComponentDatatype::Float),
                buffer: Some(pos_buf),
                ..Default::default()
            },
            Attribute {
                semantic: Some("COLOR".to_string()),
                set_index: Some(0),
                count: 2,
                attribute_type: Some(AttributeType::Vec4),
                component_datatype: Some(ComponentDatatype::UnsignedByte),
                normalized: true,
                buffer: Some(col_buf),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let frame_state = GeometryFrameState::default();
    GeometryPipelineStage::process(&mut rr, &primitive, &frame_state).unwrap();

    let vs_source = rr.shader_builder.create_vertex_shader_source();
    let vs = vs_source.sources.join("\n");
    let fs = rr.shader_builder.create_fragment_shader_source().sources.join("\n");

    assert!(vs_source.defines.contains(&"HAS_COLOR_0".to_string()));
    assert!(vs.contains("vec4 color_0;"));
    assert!(fs.contains("vec4 color_0;"));
    assert!(vs.contains("v_color_0 = attributes.color_0;"));
    assert!(fs.contains("attributes.color_0 = v_color_0;"));
}

#[test]
fn test_promotes_vec3_vertex_colors_to_vec4_in_shader() {
    let mut sb = ShaderBuilder::default();
    let mut rr = create_mock_resources(&mut sb);

    let pos_buf = create_test_buffer(&[0u8; 24]);
    let col_buf = create_test_buffer(&[0u8; 24]);

    let primitive = Primitive {
        attributes: vec![
            Attribute {
                semantic: Some("POSITION".to_string()),
                count: 2,
                attribute_type: Some(AttributeType::Vec3),
                component_datatype: Some(ComponentDatatype::Float),
                buffer: Some(pos_buf),
                ..Default::default()
            },
            Attribute {
                semantic: Some("COLOR".to_string()),
                set_index: Some(0),
                count: 2,
                attribute_type: Some(AttributeType::Vec3), // vec3 color!
                component_datatype: Some(ComponentDatatype::Float),
                buffer: Some(col_buf),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let frame_state = GeometryFrameState::default();
    GeometryPipelineStage::process(&mut rr, &primitive, &frame_state).unwrap();

    // Render resources attribute should still have 3 components per attribute
    assert_eq!(rr.attributes[1].components_per_attribute, 3);

    // But shader should promote it to vec4!
    let vs = rr.shader_builder.create_vertex_shader_source().sources.join("\n");
    let fs = rr.shader_builder.create_fragment_shader_source().sources.join("\n");

    assert!(vs.contains("in vec4 a_color_0;"));
    assert!(vs.contains("vec4 color_0;"));
    assert!(fs.contains("vec4 color_0;"));
    assert!(vs.contains("out vec4 v_color_0;"));
    assert!(fs.contains("in vec4 v_color_0;"));
}

#[test]
fn test_processes_custom_vertex_attribute_from_primitive() {
    let mut sb = ShaderBuilder::default();
    let mut rr = create_mock_resources(&mut sb);

    let pos_buf = create_test_buffer(&[0u8; 24]);
    let temp_buf = create_test_buffer(&[0u8; 12]);

    let primitive = Primitive {
        attributes: vec![
            Attribute {
                semantic: Some("POSITION".to_string()),
                count: 2,
                attribute_type: Some(AttributeType::Vec3),
                component_datatype: Some(ComponentDatatype::Float),
                buffer: Some(pos_buf),
                ..Default::default()
            },
            Attribute {
                name: Some("_TEMPERATURE".to_string()),
                count: 2,
                attribute_type: Some(AttributeType::Vec2),
                component_datatype: Some(ComponentDatatype::UnsignedShort),
                buffer: Some(temp_buf),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let frame_state = GeometryFrameState::default();
    GeometryPipelineStage::process(&mut rr, &primitive, &frame_state).unwrap();

    let vs = rr.shader_builder.create_vertex_shader_source().sources.join("\n");
    let fs = rr.shader_builder.create_fragment_shader_source().sources.join("\n");

    assert!(vs.contains("in vec2 a_temperature;"));
    assert!(vs.contains("vec2 temperature;"));
    assert!(fs.contains("vec2 temperature;"));
    assert!(vs.contains("attributes.temperature = a_temperature;"));
    assert!(vs.contains("v_temperature = attributes.temperature;"));
    assert!(fs.contains("attributes.temperature = v_temperature;"));
}

#[test]
fn test_processes_position_normal_and_feature_id_attributes() {
    let mut sb = ShaderBuilder::default();
    let mut rr = create_mock_resources(&mut sb);

    let pos_buf = create_test_buffer(&[0u8; 24]);
    let fid0_buf = create_test_buffer(&[0u8; 8]);
    let fid1_buf = create_test_buffer(&[0u8; 8]);

    let primitive = Primitive {
        attributes: vec![
            Attribute {
                semantic: Some("POSITION".to_string()),
                count: 2,
                attribute_type: Some(AttributeType::Vec3),
                component_datatype: Some(ComponentDatatype::Float),
                buffer: Some(pos_buf),
                ..Default::default()
            },
            Attribute {
                semantic: Some("_FEATURE_ID".to_string()),
                set_index: Some(0),
                count: 2,
                attribute_type: Some(AttributeType::Scalar),
                component_datatype: Some(ComponentDatatype::Float),
                buffer: Some(fid0_buf),
                ..Default::default()
            },
            Attribute {
                semantic: Some("_FEATURE_ID".to_string()),
                set_index: Some(1),
                count: 2,
                attribute_type: Some(AttributeType::Scalar),
                component_datatype: Some(ComponentDatatype::Float),
                buffer: Some(fid1_buf),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let frame_state = GeometryFrameState::default();
    GeometryPipelineStage::process(&mut rr, &primitive, &frame_state).unwrap();

    assert_eq!(rr.feature_id_vertex_attribute_set_index, 2);

    let vs_source = rr.shader_builder.create_vertex_shader_source();
    let vs = vs_source.sources.join("\n");

    assert!(vs_source.defines.contains(&"HAS_FEATURE_ID_0".to_string()));
    assert!(vs_source.defines.contains(&"HAS_FEATURE_ID_1".to_string()));
    assert!(vs.contains("v_featureId_0 = attributes.featureId_0;"));
    assert!(vs.contains("v_featureId_1 = attributes.featureId_1;"));
}

#[test]
fn test_sets_primitive_type_points_for_point_primitive_types() {
    let mut sb = ShaderBuilder::default();
    let mut rr = create_mock_resources(&mut sb);

    let pos_buf = create_test_buffer(&[0u8; 24]);
    let primitive = Primitive {
        attributes: vec![Attribute {
            semantic: Some("POSITION".to_string()),
            count: 2,
            attribute_type: Some(AttributeType::Vec3),
            component_datatype: Some(ComponentDatatype::Float),
            buffer: Some(pos_buf),
            ..Default::default()
        }],
        primitive_type: Some(PrimitiveType::Points),
        ..Default::default()
    };

    let frame_state = GeometryFrameState::default();
    GeometryPipelineStage::process(&mut rr, &primitive, &frame_state).unwrap();

    let vs_source = rr.shader_builder.create_vertex_shader_source();
    assert!(vs_source.defines.contains(&"PRIMITIVE_TYPE_POINTS".to_string()));
}

#[test]
fn test_uses_flat_varying_qualifier_for_point_primitive_vertex_attributes() {
    let mut sb = ShaderBuilder::default();
    let mut rr = create_mock_resources(&mut sb);

    let pos_buf = create_test_buffer(&[0u8; 24]);
    let norm_buf = create_test_buffer(&[0u8; 24]);
    let primitive = Primitive {
        attributes: vec![
            Attribute {
                semantic: Some("POSITION".to_string()),
                count: 2,
                attribute_type: Some(AttributeType::Vec3),
                component_datatype: Some(ComponentDatatype::Float),
                buffer: Some(pos_buf),
                ..Default::default()
            },
            Attribute {
                semantic: Some("NORMAL".to_string()),
                count: 2,
                attribute_type: Some(AttributeType::Vec3),
                component_datatype: Some(ComponentDatatype::Float),
                buffer: Some(norm_buf),
                ..Default::default()
            },
        ],
        primitive_type: Some(PrimitiveType::Points),
        ..Default::default()
    };

    let frame_state = GeometryFrameState {
        mode: SceneMode::Scene3D,
        scene_3d_only: false,
        webgl2: true,
    };
    GeometryPipelineStage::process(&mut rr, &primitive, &frame_state).unwrap();

    let vs = rr.shader_builder.create_vertex_shader_source().sources.join("\n");
    let fs = rr.shader_builder.create_fragment_shader_source().sources.join("\n");

    assert!(vs.contains("flat out vec3 v_normalEC;"));
    assert!(fs.contains("flat in vec3 v_normalEC;"));
}

#[test]
fn test_prepares_draco_model_for_dequantization_stage() {
    let mut sb = ShaderBuilder::default();
    let mut rr = create_mock_resources(&mut sb);

    let pos_buf = create_test_buffer(&[0u8; 12]);
    let primitive = Primitive {
        attributes: vec![Attribute {
            semantic: Some("POSITION".to_string()),
            count: 2,
            attribute_type: Some(AttributeType::Vec3),
            component_datatype: Some(ComponentDatatype::Float),
            quantization: Some(Quantization {
                attribute_type: Some(AttributeType::Vec3),
                component_datatype: Some(ComponentDatatype::UnsignedShort),
                ..Default::default()
            }),
            buffer: Some(pos_buf),
            ..Default::default()
        }],
        ..Default::default()
    };

    let frame_state = GeometryFrameState::default();
    GeometryPipelineStage::process(&mut rr, &primitive, &frame_state).unwrap();

    let vs = rr.shader_builder.create_vertex_shader_source().sources.join("\n");

    assert!(vs.contains("in vec3 a_quantized_positionMC;"));
    // Initializer should skip adding attributes.positionMC = ...
    assert!(!vs.contains("attributes.positionMC = a_positionMC;"));
    assert!(!vs.contains("attributes.positionMC = a_quantized_positionMC;"));
}

#[test]
fn test_prepares_draco_model_with_tangents_for_dequantization_stage() {
    let mut sb = ShaderBuilder::default();
    let mut rr = create_mock_resources(&mut sb);

    let tan_buf = create_test_buffer(&[0u8; 16]);
    let pos_buf = create_test_buffer(&[0u8; 12]);

    let primitive = Primitive {
        attributes: vec![
            Attribute {
                semantic: Some("POSITION".to_string()),
                count: 2,
                attribute_type: Some(AttributeType::Vec3),
                component_datatype: Some(ComponentDatatype::Float),
                quantization: Some(Quantization {
                    attribute_type: Some(AttributeType::Vec3),
                    component_datatype: Some(ComponentDatatype::UnsignedShort),
                    ..Default::default()
                }),
                buffer: Some(pos_buf),
                ..Default::default()
            },
            Attribute {
                semantic: Some("TANGENT".to_string()),
                count: 2,
                attribute_type: Some(AttributeType::Vec4),
                component_datatype: Some(ComponentDatatype::Float),
                quantization: Some(Quantization {
                    attribute_type: Some(AttributeType::Vec4),
                    component_datatype: Some(ComponentDatatype::Short),
                    ..Default::default()
                }),
                buffer: Some(tan_buf),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let frame_state = GeometryFrameState::default();
    GeometryPipelineStage::process(&mut rr, &primitive, &frame_state).unwrap();

    let vs = rr.shader_builder.create_vertex_shader_source().sources.join("\n");
    assert!(vs.contains("in vec4 a_quantized_tangentMC;"));
    assert!(!vs.contains("attributes.tangentMC = a_tangentMC.xyz;"));
}

#[test]
fn test_processes_draco_model_for_2d() {
    let mut sb = ShaderBuilder::default();
    let mut rr = create_mock_resources(&mut sb);
    rr.model.project_to_2d = true;
    let pos2d_buf = create_test_buffer(&[0u8; 24]);
    rr.runtime_primitive.position_buffer_2d = Some(pos2d_buf);

    let pos_buf = create_test_buffer(&[0u8; 12]);
    let primitive = Primitive {
        attributes: vec![Attribute {
            semantic: Some("POSITION".to_string()),
            count: 2,
            attribute_type: Some(AttributeType::Vec3),
            component_datatype: Some(ComponentDatatype::Float),
            quantization: Some(Quantization {
                attribute_type: Some(AttributeType::Vec3),
                component_datatype: Some(ComponentDatatype::UnsignedShort),
                ..Default::default()
            }),
            buffer: Some(pos_buf),
            ..Default::default()
        }],
        ..Default::default()
    };

    let frame_state = GeometryFrameState {
        mode: SceneMode::Scene2D,
        scene_3d_only: false,
        webgl2: true,
    };
    GeometryPipelineStage::process(&mut rr, &primitive, &frame_state).unwrap();

    let vs = rr.shader_builder.create_vertex_shader_source().sources.join("\n");
    assert!(vs.contains("in vec3 a_quantized_positionMC;"));
    assert!(vs.contains("in vec3 a_position2D;"));
    // 2D position is still initialized
    assert!(vs.contains("attributes.position2D = a_position2D;"));
}

#[test]
fn test_processes_model_with_matrix_attributes() {
    let mut sb = ShaderBuilder::default();
    let mut rr = create_mock_resources(&mut sb);

    let mat_buf = create_test_buffer(&[0u8; 128]);
    let primitive = Primitive {
        attributes: vec![Attribute {
            name: Some("TRANSFORM".to_string()),
            count: 2,
            attribute_type: Some(AttributeType::Mat4), // 4 locations!
            component_datatype: Some(ComponentDatatype::Float),
            byte_offset: 16,
            byte_stride: Some(64),
            buffer: Some(mat_buf),
            ..Default::default()
        }],
        ..Default::default()
    };

    let frame_state = GeometryFrameState::default();
    GeometryPipelineStage::process(&mut rr, &primitive, &frame_state).unwrap();

    // Mat4 has 4 columns -> 4 vertex attributes
    assert_eq!(rr.attributes.len(), 4);
    assert_eq!(rr.attributes[0].index, 1);
    assert_eq!(rr.attributes[0].offset_in_bytes, 16);
    assert_eq!(rr.attributes[0].components_per_attribute, 4);

    assert_eq!(rr.attributes[1].index, 2);
    assert_eq!(rr.attributes[1].offset_in_bytes, 32);

    assert_eq!(rr.attributes[2].index, 3);
    assert_eq!(rr.attributes[2].offset_in_bytes, 48);

    assert_eq!(rr.attributes[3].index, 4);
    assert_eq!(rr.attributes[3].offset_in_bytes, 64);

    assert_eq!(rr.attribute_index, 5);
}

#[test]
fn test_processes_position_attribute_for_instanced_model_for_2d() {
    let mut sb = ShaderBuilder::default();
    let mut rr = create_mock_resources(&mut sb);
    rr.model.project_to_2d = true;
    rr.runtime_node.is_instanced = true;

    let pos_buf = create_test_buffer(&[0u8; 24]);
    let primitive = Primitive {
        attributes: vec![Attribute {
            semantic: Some("POSITION".to_string()),
            count: 2,
            attribute_type: Some(AttributeType::Vec3),
            component_datatype: Some(ComponentDatatype::Float),
            buffer: Some(pos_buf),
            ..Default::default()
        }],
        ..Default::default()
    };

    let frame_state = GeometryFrameState {
        mode: SceneMode::Scene2D,
        scene_3d_only: false,
        webgl2: true,
    };
    GeometryPipelineStage::process(&mut rr, &primitive, &frame_state).unwrap();

    assert_eq!(rr.attributes.len(), 1); // No 2D position attribute added
    let vs = rr.shader_builder.create_vertex_shader_source().sources.join("\n");
    assert!(!vs.contains("in vec3 a_position2D;"));
    // But struct still gets position2D
    assert!(vs.contains("vec3 position2D;"));
}

#[test]
fn test_throws_when_attribute_lacks_buffer_and_constant() {
    let mut sb = ShaderBuilder::default();
    let mut rr = create_mock_resources(&mut sb);

    let primitive = Primitive {
        attributes: vec![Attribute {
            semantic: Some("POSITION".to_string()),
            count: 2,
            attribute_type: Some(AttributeType::Vec3),
            component_datatype: Some(ComponentDatatype::Float),
            buffer: None,
            constant: None,
            ..Default::default()
        }],
        ..Default::default()
    };

    let frame_state = GeometryFrameState::default();
    let res = GeometryPipelineStage::process(&mut rr, &primitive, &frame_state);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("Attributes must be provided as a Buffer or constant value"));
}
