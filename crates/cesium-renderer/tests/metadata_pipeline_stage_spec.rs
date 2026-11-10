//! Tests for MetadataPipelineStage matching CesiumJS `Scene/Model/MetadataPipelineStageSpec.js` (Apache-2.0).

use std::collections::{BTreeMap, HashMap};
use serde_json::Value;

use cesium_math::Matrix3;
use cesium_renderer::metadata_pipeline_stage::{
    MetadataFrameContext, MetadataFrameState, MetadataModel, MetadataPipelineStage,
    MetadataRenderResources, MetadataStatistics, PropertyAttribute, PropertyAttributeProperty,
    PropertyStatistics, PropertyTexture, PropertyTextureProperty, StructuralMetadata,
    FUNCTION_SIGNATURE_INITIALIZE_METADATA, FUNCTION_SIGNATURE_SET_METADATA_VARYINGS,
    METADATA_CLASS_FIELDS, STRUCT_NAME_METADATA, STRUCT_NAME_METADATA_CLASS,
    STRUCT_NAME_METADATA_STATISTICS,
};
use cesium_renderer::model_components::{Attribute, ModelTextureReader, Primitive};
use cesium_renderer::shader_builder::ShaderBuilder;

struct MockSetup {
    shader_builder: ShaderBuilder,
    model: MetadataModel,
    uniform_map: HashMap<String, Box<dyn Fn() -> Value + Send + Sync>>,
}

impl MockSetup {
    fn new() -> Self {
        Self {
            shader_builder: ShaderBuilder::default(),
            model: MetadataModel::default(),
            uniform_map: HashMap::new(),
        }
    }

    fn as_render_resources(&mut self) -> MetadataRenderResources<'_> {
        MetadataRenderResources {
            shader_builder: &mut self.shader_builder,
            model: &mut self.model,
            uniform_map: &mut self.uniform_map,
        }
    }
}

fn check_metadata_class_structs(sb: &ShaderBuilder, metadata_types: &[&str]) {
    let vs = sb.create_vertex_shader_source().sources.join("\n");
    let fs = sb.create_fragment_shader_source().sources.join("\n");

    for m_type in metadata_types {
        let struct_name = format!("{m_type}MetadataClass");
        assert!(vs.contains(&format!("struct {struct_name}")), "VS missing struct {struct_name}");
        assert!(fs.contains(&format!("struct {struct_name}")), "FS missing struct {struct_name}");

        for field in &METADATA_CLASS_FIELDS {
            let field_decl = format!("{m_type} {};", field.shader_name);
            assert!(vs.contains(&field_decl), "VS missing field {field_decl} in {struct_name}");
            assert!(fs.contains(&field_decl), "FS missing field {field_decl} in {struct_name}");
        }
    }
}

#[test]
fn test_handles_primitives_without_metadata_gracefully() {
    let mut setup = MockSetup::new();
    let primitive = Primitive::default();
    let frame_state = MetadataFrameState {
        context: MetadataFrameContext { webgl2: true },
    };

    {
        let mut rr = setup.as_render_resources();
        MetadataPipelineStage::process(&mut rr, &primitive, &frame_state);
    }

    let vs = setup.shader_builder.create_vertex_shader_source().sources.join("\n");
    let fs = setup.shader_builder.create_fragment_shader_source().sources.join("\n");

    // Both vertex and fragment should declare Metadata structs
    assert!(vs.contains(&format!("struct {STRUCT_NAME_METADATA}")));
    assert!(fs.contains(&format!("struct {STRUCT_NAME_METADATA}")));
    assert!(vs.contains(&format!("struct {STRUCT_NAME_METADATA_CLASS}")));
    assert!(fs.contains(&format!("struct {STRUCT_NAME_METADATA_CLASS}")));
    assert!(vs.contains(&format!("struct {STRUCT_NAME_METADATA_STATISTICS}")));
    assert!(fs.contains(&format!("struct {STRUCT_NAME_METADATA_STATISTICS}")));

    // Functions declared
    assert!(vs.contains(FUNCTION_SIGNATURE_INITIALIZE_METADATA));
    assert!(fs.contains(FUNCTION_SIGNATURE_INITIALIZE_METADATA));
    assert!(vs.contains(FUNCTION_SIGNATURE_SET_METADATA_VARYINGS));

    // No uniforms added
    assert!(setup.shader_builder.vertex_uniform_lines().is_empty());
    assert!(setup.shader_builder.fragment_uniform_lines().is_empty());
    assert!(setup.uniform_map.is_empty());
}

#[test]
fn test_adds_property_attributes_to_the_shader() {
    let mut setup = MockSetup::new();

    let mut properties = BTreeMap::new();
    properties.insert(
        "circleT".to_string(),
        PropertyAttributeProperty {
            attribute: "circle_t".to_string(),
            glsl_type: "float".to_string(),
            has_value_transform: false,
            ..Default::default()
        },
    );
    properties.insert(
        "iteration".to_string(),
        PropertyAttributeProperty {
            attribute: "featureId_0".to_string(),
            glsl_type: "float".to_string(),
            has_value_transform: false,
            ..Default::default()
        },
    );
    properties.insert(
        "pointId".to_string(),
        PropertyAttributeProperty {
            attribute: "featureId_1".to_string(),
            glsl_type: "float".to_string(),
            has_value_transform: false,
            ..Default::default()
        },
    );
    properties.insert(
        "toroidalNormalized".to_string(),
        PropertyAttributeProperty {
            attribute: "featureId_0".to_string(),
            glsl_type: "float".to_string(),
            has_value_transform: true,
            offset: Some(Value::from(0.0)),
            scale: Some(Value::from(0.034482758620689655)),
            ..Default::default()
        },
    );
    properties.insert(
        "poloidalNormalized".to_string(),
        PropertyAttributeProperty {
            attribute: "featureId_1".to_string(),
            glsl_type: "float".to_string(),
            has_value_transform: true,
            offset: Some(Value::from(0.0)),
            scale: Some(Value::from(0.05263157894736842)),
            ..Default::default()
        },
    );
    properties.insert(
        "toroidalAngle".to_string(),
        PropertyAttributeProperty {
            attribute: "featureId_0".to_string(),
            glsl_type: "float".to_string(),
            has_value_transform: true,
            offset: Some(Value::from(0.0)),
            scale: Some(Value::from(0.21666156231653746)),
            ..Default::default()
        },
    );
    properties.insert(
        "poloidalAngle".to_string(),
        PropertyAttributeProperty {
            attribute: "featureId_1".to_string(),
            glsl_type: "float".to_string(),
            has_value_transform: true,
            offset: Some(Value::from(-std::f64::consts::PI)),
            scale: Some(Value::from(0.3306939635357677)),
            ..Default::default()
        },
    );

    let property_attribute = PropertyAttribute {
        class_id: "pointCloudClass".to_string(),
        properties,
    };

    setup.model.structural_metadata = Some(StructuralMetadata {
        property_attributes: vec![property_attribute],
        property_textures: vec![],
    });

    let primitive = Primitive {
        attributes: vec![
            Attribute { name: Some("circle_t".to_string()), ..Default::default() },
            Attribute { name: Some("featureId_0".to_string()), ..Default::default() },
            Attribute { name: Some("featureId_1".to_string()), ..Default::default() },
        ],
        ..Default::default()
    };

    let frame_state = MetadataFrameState {
        context: MetadataFrameContext { webgl2: true },
    };

    {
        let mut rr = setup.as_render_resources();
        MetadataPipelineStage::process(&mut rr, &primitive, &frame_state);
    }

    check_metadata_class_structs(&setup.shader_builder, &["float"]);

    let vs = setup.shader_builder.create_vertex_shader_source().sources.join("\n");
    let fs = setup.shader_builder.create_fragment_shader_source().sources.join("\n");

    let expected_fields = [
        "float circleT;",
        "float iteration;",
        "float pointId;",
        "float toroidalNormalized;",
        "float poloidalNormalized;",
        "float toroidalAngle;",
        "float poloidalAngle;",
    ];

    for field in &expected_fields {
        assert!(vs.contains(field), "VS missing field {field}");
        assert!(fs.contains(field), "FS missing field {field}");
    }

    let expected_assignments = [
        "metadata.circleT = attributes.circle_t;",
        "metadata.iteration = attributes.featureId_0;",
        "metadata.pointId = attributes.featureId_1;",
        "metadata.toroidalNormalized = czm_valueTransform(u_toroidalNormalized_offset, u_toroidalNormalized_scale, attributes.featureId_0);",
        "metadata.poloidalNormalized = czm_valueTransform(u_poloidalNormalized_offset, u_poloidalNormalized_scale, attributes.featureId_1);",
        "metadata.toroidalAngle = czm_valueTransform(u_toroidalAngle_offset, u_toroidalAngle_scale, attributes.featureId_0);",
        "metadata.poloidalAngle = czm_valueTransform(u_poloidalAngle_offset, u_poloidalAngle_scale, attributes.featureId_1);",
    ];

    for assign in &expected_assignments {
        assert!(vs.contains(assign), "VS missing assignment {assign}");
        assert!(fs.contains(assign), "FS missing assignment {assign}");
    }

    // Check uniforms
    let expected_uniforms = [
        "uniform float u_toroidalNormalized_offset;",
        "uniform float u_toroidalNormalized_scale;",
        "uniform float u_poloidalNormalized_offset;",
        "uniform float u_poloidalNormalized_scale;",
        "uniform float u_toroidalAngle_offset;",
        "uniform float u_toroidalAngle_scale;",
        "uniform float u_poloidalAngle_offset;",
        "uniform float u_poloidalAngle_scale;",
    ];

    let vs_uniforms = setup.shader_builder.vertex_uniform_lines().join("\n");
    let fs_uniforms = setup.shader_builder.fragment_uniform_lines().join("\n");

    for u in &expected_uniforms {
        assert!(vs_uniforms.contains(u), "VS missing uniform {u}");
        assert!(fs_uniforms.contains(u), "FS missing uniform {u}");
    }

    assert_eq!((setup.uniform_map["u_toroidalNormalized_offset"])().as_f64().unwrap(), 0.0);
    assert!(((setup.uniform_map["u_toroidalNormalized_scale"])().as_f64().unwrap() - 0.034482758620689655).abs() < 1e-12);
    assert_eq!((setup.uniform_map["u_poloidalNormalized_offset"])().as_f64().unwrap(), 0.0);
    assert!(((setup.uniform_map["u_poloidalNormalized_scale"])().as_f64().unwrap() - 0.05263157894736842).abs() < 1e-12);
    assert_eq!((setup.uniform_map["u_toroidalAngle_offset"])().as_f64().unwrap(), 0.0);
    assert!(((setup.uniform_map["u_toroidalAngle_scale"])().as_f64().unwrap() - 0.21666156231653746).abs() < 1e-12);
    assert!(((setup.uniform_map["u_poloidalAngle_offset"])().as_f64().unwrap() - (-std::f64::consts::PI)).abs() < 1e-12);
    assert!(((setup.uniform_map["u_poloidalAngle_scale"])().as_f64().unwrap() - 0.3306939635357677).abs() < 1e-12);
}

#[test]
fn test_adds_property_textures_to_the_shader() {
    let mut setup = MockSetup::new();

    let mut properties = BTreeMap::new();
    properties.insert(
        "insideTemperature".to_string(),
        PropertyTextureProperty {
            texture_reader: ModelTextureReader {
                index: 1,
                tex_coord: 0,
                ..Default::default()
            },
            channels: "r".to_string(),
            glsl_type: "uint".to_string(),
            component_count: 1,
            normalized: false,
            ..Default::default()
        },
    );
    properties.insert(
        "outsideTemperature".to_string(),
        PropertyTextureProperty {
            texture_reader: ModelTextureReader {
                index: 1,
                tex_coord: 0,
                ..Default::default()
            },
            channels: "g".to_string(),
            glsl_type: "uint".to_string(),
            component_count: 1,
            normalized: false,
            ..Default::default()
        },
    );
    properties.insert(
        "insulation".to_string(),
        PropertyTextureProperty {
            texture_reader: ModelTextureReader {
                index: 1,
                tex_coord: 0,
                ..Default::default()
            },
            channels: "b".to_string(),
            glsl_type: "float".to_string(),
            component_count: 1,
            normalized: true,
            default_value: Some(Value::from(1)),
            ..Default::default()
        },
    );

    let property_texture = PropertyTexture {
        id: 0,
        class_id: "buildingClass".to_string(),
        properties,
    };

    setup.model.structural_metadata = Some(StructuralMetadata {
        property_attributes: vec![],
        property_textures: vec![property_texture],
    });

    let primitive = Primitive::default();
    let frame_state = MetadataFrameState {
        context: MetadataFrameContext { webgl2: true },
    };

    {
        let mut rr = setup.as_render_resources();
        MetadataPipelineStage::process(&mut rr, &primitive, &frame_state);
    }

    check_metadata_class_structs(&setup.shader_builder, &["uint", "float"]);

    let fs = setup.shader_builder.create_fragment_shader_source().sources.join("\n");

    let expected_fs_fields = [
        "uint insideTemperature;",
        "uint outsideTemperature;",
        "float insulation;",
    ];
    for field in &expected_fs_fields {
        assert!(fs.contains(field), "FS missing field {field}");
    }

    let expected_unpacking = [
        "uint insideTemperature_unpackedValue;",
        "uint insideTemperature_rawBits;",
        "float insideTemperature_rawChannels = texture(u_propertyTexture_1, attributes.texCoord_0).r;",
        "insideTemperature_rawBits = czm_unpackTexture(insideTemperature_rawChannels);",
        "insideTemperature_unpackedValue = ((insideTemperature_rawBits));",
        "metadata.insideTemperature = insideTemperature_unpackedValue;",
        "uint outsideTemperature_unpackedValue;",
        "uint outsideTemperature_rawBits;",
        "float outsideTemperature_rawChannels = texture(u_propertyTexture_1, attributes.texCoord_0).g;",
        "outsideTemperature_rawBits = czm_unpackTexture(outsideTemperature_rawChannels);",
        "outsideTemperature_unpackedValue = ((outsideTemperature_rawBits));",
        "metadata.outsideTemperature = outsideTemperature_unpackedValue;",
        "float insulation_unpackedValue;",
        "uint insulation_rawBits;",
        "float insulation_rawChannels = texture(u_propertyTexture_1, attributes.texCoord_0).b;",
        "insulation_rawBits = czm_unpackTexture(insulation_rawChannels);",
        "insulation_unpackedValue = float((insulation_rawBits)) * 0.00392156862745098;",
        "metadata.insulation = insulation_unpackedValue;",
        "metadataClass.insulation.defaultValue = float(1);",
    ];
    for line in &expected_unpacking {
        assert!(fs.contains(line), "FS missing line: {line}");
    }

    let fs_uniforms = setup.shader_builder.fragment_uniform_lines().join("\n");
    assert!(fs_uniforms.contains("uniform sampler2D u_propertyTexture_1;"));
    assert_eq!((setup.uniform_map["u_propertyTexture_1"])().as_u64().unwrap(), 1);
}

#[test]
fn test_adds_property_texture_transform_to_the_shader() {
    let mut setup = MockSetup::new();

    let transform = Matrix3::new(
        2.0, 0.0, 0.5,
        0.0, 2.0, 0.5,
        0.0, 0.0, 1.0,
    );

    let mut properties = BTreeMap::new();
    properties.insert(
        "exampleProperty".to_string(),
        PropertyTextureProperty {
            texture_reader: ModelTextureReader {
                index: 0,
                tex_coord: 0,
                transform: Some(transform),
                ..Default::default()
            },
            channels: "r".to_string(),
            glsl_type: "float".to_string(),
            component_count: 1,
            normalized: true,
            ..Default::default()
        },
    );

    let property_texture = PropertyTexture {
        id: 0,
        class_id: "exampleClass".to_string(),
        properties,
    };

    setup.model.structural_metadata = Some(StructuralMetadata {
        property_attributes: vec![],
        property_textures: vec![property_texture],
    });

    let primitive = Primitive::default();
    let frame_state = MetadataFrameState {
        context: MetadataFrameContext { webgl2: true },
    };

    {
        let mut rr = setup.as_render_resources();
        MetadataPipelineStage::process(&mut rr, &primitive, &frame_state);
    }

    check_metadata_class_structs(&setup.shader_builder, &["float"]);

    let fs = setup.shader_builder.create_fragment_shader_source().sources.join("\n");
    assert!(fs.contains("float exampleProperty;"));

    let fs_uniforms = setup.shader_builder.fragment_uniform_lines().join("\n");
    assert!(fs_uniforms.contains("uniform sampler2D u_propertyTexture_0;"));
    assert!(fs_uniforms.contains("uniform mat3 u_propertyTexture_0Transform;"));
    assert!(fs.contains("vec2(u_propertyTexture_0Transform * vec3(attributes.texCoord_0, 1.0))"));
}

#[test]
fn test_handles_property_textures_with_vector_values() {
    let mut setup = MockSetup::new();

    let mut properties = BTreeMap::new();
    properties.insert(
        "vec2Property".to_string(),
        PropertyTextureProperty {
            texture_reader: ModelTextureReader {
                index: 1,
                tex_coord: 0,
                ..Default::default()
            },
            channels: "gb".to_string(),
            glsl_type: "vec2".to_string(),
            component_count: 2,
            normalized: true,
            ..Default::default()
        },
    );
    properties.insert(
        "uint8Property".to_string(),
        PropertyTextureProperty {
            texture_reader: ModelTextureReader {
                index: 1,
                tex_coord: 0,
                ..Default::default()
            },
            channels: "r".to_string(),
            glsl_type: "uint".to_string(),
            component_count: 1,
            normalized: false,
            ..Default::default()
        },
    );
    properties.insert(
        "uint8vec3Property".to_string(),
        PropertyTextureProperty {
            texture_reader: ModelTextureReader {
                index: 1,
                tex_coord: 0,
                ..Default::default()
            },
            channels: "rgb".to_string(),
            glsl_type: "uvec3".to_string(),
            component_count: 3,
            normalized: false,
            no_data: Some(serde_json::json!([19, 13, 50])),
            default_value: Some(serde_json::json!([255, 0, 0])),
            min_value: Some(serde_json::json!([10, 10, 10])),
            max_value: Some(serde_json::json!([30, 17, 50])),
            ..Default::default()
        },
    );
    properties.insert(
        "arrayProperty".to_string(),
        PropertyTextureProperty {
            texture_reader: ModelTextureReader {
                index: 1,
                tex_coord: 0,
                ..Default::default()
            },
            channels: "rgb".to_string(),
            glsl_type: "vec3".to_string(),
            component_count: 3,
            normalized: true,
            ..Default::default()
        },
    );
    properties.insert(
        "valueTransformProperty".to_string(),
        PropertyTextureProperty {
            texture_reader: ModelTextureReader {
                index: 1,
                tex_coord: 0,
                ..Default::default()
            },
            channels: "rg".to_string(),
            glsl_type: "vec2".to_string(),
            component_count: 2,
            normalized: true,
            has_value_transform: true,
            offset: Some(serde_json::json!([1.0, 1.0])),
            scale: Some(serde_json::json!([2.0, 2.0])),
            ..Default::default()
        },
    );

    let property_texture = PropertyTexture {
        id: 0,
        class_id: "vectorClass".to_string(),
        properties,
    };

    setup.model.structural_metadata = Some(StructuralMetadata {
        property_attributes: vec![],
        property_textures: vec![property_texture],
    });

    let primitive = Primitive::default();
    let frame_state = MetadataFrameState {
        context: MetadataFrameContext { webgl2: true },
    };

    {
        let mut rr = setup.as_render_resources();
        MetadataPipelineStage::process(&mut rr, &primitive, &frame_state);
    }

    check_metadata_class_structs(&setup.shader_builder, &["vec2", "uint", "uvec3", "vec3"]);

    let fs = setup.shader_builder.create_fragment_shader_source().sources.join("\n");

    let expected_fs_fields = [
        "vec2 vec2Property;",
        "uint uint8Property;",
        "uvec3 uint8vec3Property;",
        "vec3 arrayProperty;",
        "vec2 valueTransformProperty;",
    ];
    for field in &expected_fs_fields {
        assert!(fs.contains(field), "FS missing field {field}");
    }

    let expected_class_fields = [
        "vec2MetadataClass vec2Property;",
        "uintMetadataClass uint8Property;",
        "uvec3MetadataClass uint8vec3Property;",
        "vec3MetadataClass arrayProperty;",
        "vec2MetadataClass valueTransformProperty;",
    ];
    for field in &expected_class_fields {
        assert!(fs.contains(field), "FS missing class field {field}");
    }

    let expected_lines = [
        "vec2Property_rawBits = czm_unpackTexture(vec2Property_rawChannels.r);",
        "vec2Property_unpackedValue[0] = float((vec2Property_rawBits)) * 0.00392156862745098;",
        "vec2Property_rawBits = czm_unpackTexture(vec2Property_rawChannels.g);",
        "vec2Property_unpackedValue[1] = float((vec2Property_rawBits)) * 0.00392156862745098;",
        "metadata.vec2Property = vec2Property_unpackedValue;",
        "metadataClass.uint8vec3Property.noData = uvec3(19,13,50);",
        "metadataClass.uint8vec3Property.defaultValue = uvec3(255,0,0);",
        "metadataClass.uint8vec3Property.minValue = uvec3(10,10,10);",
        "metadataClass.uint8vec3Property.maxValue = uvec3(30,17,50);",
        "metadata.valueTransformProperty = czm_valueTransform(u_valueTransformProperty_offset, u_valueTransformProperty_scale, valueTransformProperty_unpackedValue);",
    ];
    for line in &expected_lines {
        assert!(fs.contains(line), "FS missing expected line: {line}");
    }
}

#[test]
fn test_handles_property_textures_that_use_multiple_channels_to_represent_higher_precision_types() {
    let mut setup = MockSetup::new();

    let mut properties = BTreeMap::new();
    properties.insert(
        "insideTemperature".to_string(),
        PropertyTextureProperty {
            texture_reader: ModelTextureReader {
                index: 0,
                tex_coord: 0,
                ..Default::default()
            },
            channels: "rgba".to_string(),
            glsl_type: "float".to_string(),
            component_count: 1,
            normalized: false,
            is_float_scalar: true,
            ..Default::default()
        },
    );

    let property_texture = PropertyTexture {
        id: 0,
        class_id: "float32Class".to_string(),
        properties,
    };

    setup.model.structural_metadata = Some(StructuralMetadata {
        property_attributes: vec![],
        property_textures: vec![property_texture],
    });

    let primitive = Primitive::default();
    let frame_state = MetadataFrameState {
        context: MetadataFrameContext { webgl2: true },
    };

    {
        let mut rr = setup.as_render_resources();
        MetadataPipelineStage::process(&mut rr, &primitive, &frame_state);
    }

    check_metadata_class_structs(&setup.shader_builder, &["float"]);

    let fs = setup.shader_builder.create_fragment_shader_source().sources.join("\n");
    assert!(fs.contains("float insideTemperature;"));
    assert!(fs.contains("floatMetadataClass insideTemperature;"));

    let expected_lines = [
        "float insideTemperature_unpackedValue;",
        "uint insideTemperature_rawBits;",
        "vec4 insideTemperature_rawChannels = texture(u_propertyTexture_0, attributes.texCoord_0).rgba;",
        "insideTemperature_rawBits = czm_unpackTexture(insideTemperature_rawChannels.rgba);",
        "insideTemperature_unpackedValue = (uintBitsToFloat(insideTemperature_rawBits));",
        "metadata.insideTemperature = insideTemperature_unpackedValue;",
    ];
    for line in &expected_lines {
        assert!(fs.contains(line), "FS missing line: {line}");
    }
}

#[test]
fn test_handles_a_tileset_with_metadata_statistics() {
    let mut setup = MockSetup::new();

    let mut properties = BTreeMap::new();
    properties.insert(
        "classification".to_string(),
        PropertyAttributeProperty {
            attribute: "classification".to_string(),
            glsl_type: "float".to_string(),
            has_value_transform: false,
            ..Default::default()
        },
    );
    properties.insert(
        "intensity".to_string(),
        PropertyAttributeProperty {
            attribute: "intensity".to_string(),
            glsl_type: "float".to_string(),
            has_value_transform: false,
            ..Default::default()
        },
    );

    let property_attribute = PropertyAttribute {
        class_id: "pointCloudStatsClass".to_string(),
        properties,
    };

    setup.model.structural_metadata = Some(StructuralMetadata {
        property_attributes: vec![property_attribute],
        property_textures: vec![],
    });

    let mut stats_map = HashMap::new();
    stats_map.insert(
        "intensity".to_string(),
        PropertyStatistics {
            min: Some(serde_json::json!(0)),
            max: Some(Value::from(0.6333333849906921)),
            mean: Some(Value::from(0.28973701532415364)),
            median: Some(Value::from(0.25416669249534607)),
            standard_deviation: Some(Value::from(0.18222664489583626)),
            variance: Some(Value::from(0.03320655011)),
            sum: Some(Value::from(8500.30455558002)),
        },
    );

    let mut classes_stats = HashMap::new();
    classes_stats.insert("pointCloudStatsClass".to_string(), stats_map);
    setup.model.statistics = Some(MetadataStatistics {
        classes: classes_stats,
    });

    let primitive = Primitive {
        attributes: vec![
            Attribute { name: Some("classification".to_string()), ..Default::default() },
            Attribute { name: Some("intensity".to_string()), ..Default::default() },
        ],
        ..Default::default()
    };

    let frame_state = MetadataFrameState {
        context: MetadataFrameContext { webgl2: true },
    };

    {
        let mut rr = setup.as_render_resources();
        MetadataPipelineStage::process(&mut rr, &primitive, &frame_state);
    }

    check_metadata_class_structs(&setup.shader_builder, &["float"]);

    let vs = setup.shader_builder.create_vertex_shader_source().sources.join("\n");
    let fs = setup.shader_builder.create_fragment_shader_source().sources.join("\n");

    // floatMetadataStatistics sub-struct declared in both VS and FS
    let stats_fields = [
        "float minValue;",
        "float maxValue;",
        "float mean;",
        "float median;",
        "float standardDeviation;",
        "float variance;",
        "float sum;",
    ];
    for f in &stats_fields {
        assert!(vs.contains(f), "VS missing stats field {f}");
        assert!(fs.contains(f), "FS missing stats field {f}");
    }

    // Main metadata structs
    assert!(vs.contains("float classification;"));
    assert!(vs.contains("float intensity;"));
    assert!(fs.contains("float classification;"));
    assert!(fs.contains("float intensity;"));

    // MetadataClass fields
    assert!(vs.contains("floatMetadataClass classification;"));
    assert!(vs.contains("floatMetadataClass intensity;"));
    assert!(fs.contains("floatMetadataClass classification;"));
    assert!(fs.contains("floatMetadataClass intensity;"));

    // MetadataStatistics fields
    assert!(vs.contains("floatMetadataStatistics intensity;"));
    assert!(fs.contains("floatMetadataStatistics intensity;"));

    // Function assignments in VS and FS
    let expected_assignments = [
        "metadata.classification = attributes.classification;",
        "metadata.intensity = attributes.intensity;",
        "metadataStatistics.intensity.mean = float(0.28973701532415364);",
        "metadataStatistics.intensity.median = float(0.25416669249534607);",
        "metadataStatistics.intensity.standardDeviation = float(0.18222664489583626);",
        "metadataStatistics.intensity.variance = float(0.03320655011);",
        "metadataStatistics.intensity.sum = float(8500.30455558002);",
        "metadataStatistics.intensity.minValue = float(0);",
        "metadataStatistics.intensity.maxValue = float(0.6333333849906921);",
    ];

    for assign in &expected_assignments {
        assert!(vs.contains(assign), "VS missing assignment {assign}");
        assert!(fs.contains(assign), "FS missing assignment {assign}");
    }
}
