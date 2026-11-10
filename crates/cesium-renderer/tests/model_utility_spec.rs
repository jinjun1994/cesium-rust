use cesium_math::{Cartesian3, Matrix4, PrimitiveType, Quaternion};
use cesium_renderer::axis::Axis;
use cesium_renderer::model_utility::{
    AttributeQuantization, AttributeType, FeatureIdSetInfo, ModelAttributeDescription, ModelUtility,
};
use cesium_renderer::render_state::CullFace;

#[test]
fn test_get_node_transform_works_when_node_has_matrix() {
    let matrix = Matrix4::IDENTITY;
    let computed = ModelUtility::get_node_transform(Some(matrix), None, None, None);
    assert_eq!(computed, Matrix4::IDENTITY);
}

#[test]
fn test_get_node_transform_works_when_node_has_trs() {
    let t = Cartesian3::new(0.0, 0.0, 0.0);
    let r = Quaternion::new(0.0, 0.0, 0.0, 1.0);
    let s = Cartesian3::new(1.0, 1.0, 1.0);
    let computed = ModelUtility::get_node_transform(None, Some(t), Some(r), Some(s));
    assert_eq!(computed, Matrix4::IDENTITY);
}

#[test]
fn test_has_quantized_attributes() {
    assert!(!ModelUtility::has_quantized_attributes(None));

    let mut attrs = vec![
        ModelAttributeDescription {
            name: None,
            semantic: Some("POSITION".to_string()),
            set_index: None,
            attribute_type: AttributeType::Vec3,
            min: Some(Cartesian3::new(-0.5, -0.5, -0.5)),
            max: Some(Cartesian3::new(0.5, 0.5, 0.5)),
            quantization: None,
        },
        ModelAttributeDescription {
            name: None,
            semantic: Some("NORMAL".to_string()),
            set_index: None,
            attribute_type: AttributeType::Vec3,
            min: None,
            max: None,
            quantization: None,
        },
    ];

    assert!(!ModelUtility::has_quantized_attributes(Some(&attrs)));

    attrs[1].quantization = Some(AttributeQuantization {
        attribute_type: AttributeType::Vec2,
    });
    assert!(ModelUtility::has_quantized_attributes(Some(&attrs)));
}

#[test]
fn test_get_attribute_info() {
    // 1. Built-in POSITION
    let pos_attr = ModelAttributeDescription {
        name: None,
        semantic: Some("POSITION".to_string()),
        set_index: None,
        attribute_type: AttributeType::Vec3,
        min: Some(Cartesian3::new(-0.5, -0.5, -0.5)),
        max: Some(Cartesian3::new(0.5, 0.5, 0.5)),
        quantization: None,
    };
    let info = ModelUtility::get_attribute_info(&pos_attr);
    assert!(!info.is_quantized);
    assert_eq!(info.variable_name, "positionMC");
    assert!(info.has_semantic);
    assert_eq!(info.glsl_type, "vec3");
    assert_eq!(info.quantized_glsl_type, None);

    // 2. TEXCOORD with set index
    let tex_attr = ModelAttributeDescription {
        name: None,
        semantic: Some("TEXCOORD".to_string()),
        set_index: Some(0),
        attribute_type: AttributeType::Vec2,
        min: None,
        max: None,
        quantization: None,
    };
    let info = ModelUtility::get_attribute_info(&tex_attr);
    assert!(!info.is_quantized);
    assert_eq!(info.variable_name, "texCoord_0");
    assert!(info.has_semantic);
    assert_eq!(info.glsl_type, "vec2");

    // 3. COLOR promoted to vec4
    let color_attr = ModelAttributeDescription {
        name: None,
        semantic: Some("COLOR".to_string()),
        set_index: Some(0),
        attribute_type: AttributeType::Vec3,
        min: None,
        max: None,
        quantization: None,
    };
    let info = ModelUtility::get_attribute_info(&color_attr);
    assert_eq!(info.variable_name, "color_0");
    assert_eq!(info.glsl_type, "vec4");

    // 4. Custom attribute
    let custom_attr = ModelAttributeDescription {
        name: Some("_TEMPERATURE".to_string()),
        semantic: None,
        set_index: None,
        attribute_type: AttributeType::Scalar,
        min: None,
        max: None,
        quantization: None,
    };
    let info = ModelUtility::get_attribute_info(&custom_attr);
    assert!(!info.is_quantized);
    assert_eq!(info.variable_name, "temperature");
    assert!(!info.has_semantic);
    assert_eq!(info.glsl_type, "float");

    // 5. Quantized POSITION and NORMAL
    let mut q_pos = pos_attr.clone();
    q_pos.quantization = Some(AttributeQuantization {
        attribute_type: AttributeType::Vec3,
    });
    let info = ModelUtility::get_attribute_info(&q_pos);
    assert!(info.is_quantized);
    assert_eq!(info.glsl_type, "vec3");
    assert_eq!(info.quantized_glsl_type.as_deref(), Some("vec3"));

    let q_norm = ModelAttributeDescription {
        name: None,
        semantic: Some("NORMAL".to_string()),
        set_index: None,
        attribute_type: AttributeType::Vec3,
        min: None,
        max: None,
        quantization: Some(AttributeQuantization {
            attribute_type: AttributeType::Vec2,
        }),
    };
    let info = ModelUtility::get_attribute_info(&q_norm);
    assert!(info.is_quantized);
    assert_eq!(info.variable_name, "normalMC");
    assert_eq!(info.glsl_type, "vec3");
    assert_eq!(info.quantized_glsl_type.as_deref(), Some("vec2"));

    // 6. Quantized COLOR promoted to vec4
    let mut q_col = color_attr.clone();
    q_col.quantization = Some(AttributeQuantization {
        attribute_type: AttributeType::Vec3,
    });
    let info = ModelUtility::get_attribute_info(&q_col);
    assert!(info.is_quantized);
    assert_eq!(info.variable_name, "color_0");
    assert_eq!(info.glsl_type, "vec4");
    assert_eq!(info.quantized_glsl_type.as_deref(), Some("vec4"));
}

#[test]
fn test_get_position_min_max() {
    let min = Cartesian3::new(-0.5, -0.5, -0.5);
    let max = Cartesian3::new(0.5, 0.5, 0.5);

    let (out_min, out_max) = ModelUtility::get_position_min_max(min, max, None, None);
    assert_eq!(out_min, min);
    assert_eq!(out_max, max);

    let inst_min = Cartesian3::new(-5.0, -5.0, -5.0);
    let inst_max = Cartesian3::new(5.0, 5.0, 5.0);
    let (out_min, out_max) = ModelUtility::get_position_min_max(min, max, Some(inst_min), Some(inst_max));
    assert_eq!(out_min, Cartesian3::new(-5.5, -5.5, -5.5));
    assert_eq!(out_max, Cartesian3::new(5.5, 5.5, 5.5));
}

#[test]
fn test_get_axis_correction_matrix() {
    let y_to_z = Axis::Y_UP_TO_Z_UP;
    let x_to_z = Axis::X_UP_TO_Z_UP;
    let combined = y_to_z.multiply(&Axis::Z_UP_TO_X_UP);

    // ECEF (Z-up, X-forward) returns identity
    let m = ModelUtility::get_axis_correction_matrix(Axis::Z, Axis::X);
    assert_eq!(m, Matrix4::IDENTITY);

    // glTF standard: Y-up, Z-forward
    let m = ModelUtility::get_axis_correction_matrix(Axis::Y, Axis::Z);
    assert_eq!(m, combined);

    // Y-up, X-forward
    let m = ModelUtility::get_axis_correction_matrix(Axis::Y, Axis::X);
    assert_eq!(m, y_to_z);

    // X-up, Y-forward
    let m = ModelUtility::get_axis_correction_matrix(Axis::X, Axis::Y);
    assert_eq!(m, x_to_z);
}

#[test]
fn test_get_attribute_by_semantic() {
    let attrs = [
        ModelAttributeDescription {
            name: None,
            semantic: Some("POSITION".to_string()),
            set_index: None,
            attribute_type: AttributeType::Vec3,
            min: None,
            max: None,
            quantization: None,
        },
        ModelAttributeDescription {
            name: None,
            semantic: Some("NORMAL".to_string()),
            set_index: None,
            attribute_type: AttributeType::Vec3,
            min: None,
            max: None,
            quantization: None,
        },
        ModelAttributeDescription {
            name: None,
            semantic: Some("TEXCOORD".to_string()),
            set_index: Some(0),
            attribute_type: AttributeType::Vec2,
            min: None,
            max: None,
            quantization: None,
        },
        ModelAttributeDescription {
            name: None,
            semantic: Some("TEXCOORD".to_string()),
            set_index: Some(1),
            attribute_type: AttributeType::Vec2,
            min: None,
            max: None,
            quantization: None,
        },
    ];

    assert!(ModelUtility::get_attribute_by_semantic(&attrs, "POSITION", None).is_some());
    assert!(ModelUtility::get_attribute_by_semantic(&attrs, "NORMAL", None).is_some());
    assert!(ModelUtility::get_attribute_by_semantic(&attrs, "TEXCOORD", Some(0)).is_some());
    assert!(ModelUtility::get_attribute_by_semantic(&attrs, "TEXCOORD", Some(1)).is_some());
    assert!(ModelUtility::get_attribute_by_semantic(&attrs, "UNKNOWN", None).is_none());
}

#[test]
fn test_get_feature_ids_by_label() {
    let sets = [
        FeatureIdSetInfo {
            label: Some("perVertex".to_string()),
            positional_label: Some("featureId_0".to_string()),
        },
        FeatureIdSetInfo {
            label: Some("perFace".to_string()),
            positional_label: Some("featureId_1".to_string()),
        },
    ];

    assert_eq!(
        ModelUtility::get_feature_ids_by_label(&sets, "perVertex"),
        Some(&sets[0])
    );
    assert_eq!(
        ModelUtility::get_feature_ids_by_label(&sets, "perFace"),
        Some(&sets[1])
    );
    assert_eq!(
        ModelUtility::get_feature_ids_by_label(&sets, "featureId_0"),
        Some(&sets[0])
    );
    assert_eq!(
        ModelUtility::get_feature_ids_by_label(&sets, "featureId_1"),
        Some(&sets[1])
    );
    assert_eq!(
        ModelUtility::get_feature_ids_by_label(&sets, "unknown"),
        None
    );
}

#[test]
fn test_get_cull_face() {
    let neg = Matrix4::from_uniform_scale(-1.0);
    assert_eq!(ModelUtility::get_cull_face(&neg, PrimitiveType::Points), CullFace::Back);
    assert_eq!(ModelUtility::get_cull_face(&neg, PrimitiveType::Lines), CullFace::Back);
    assert_eq!(ModelUtility::get_cull_face(&neg, PrimitiveType::LineLoop), CullFace::Back);
    assert_eq!(ModelUtility::get_cull_face(&neg, PrimitiveType::LineStrip), CullFace::Back);

    let identity = Matrix4::IDENTITY;
    assert_eq!(ModelUtility::get_cull_face(&identity, PrimitiveType::Triangles), CullFace::Back);
    assert_eq!(ModelUtility::get_cull_face(&identity, PrimitiveType::TriangleStrip), CullFace::Back);
    assert_eq!(ModelUtility::get_cull_face(&identity, PrimitiveType::TriangleFan), CullFace::Back);

    assert_eq!(ModelUtility::get_cull_face(&neg, PrimitiveType::Triangles), CullFace::Front);
    assert_eq!(ModelUtility::get_cull_face(&neg, PrimitiveType::TriangleStrip), CullFace::Front);
    assert_eq!(ModelUtility::get_cull_face(&neg, PrimitiveType::TriangleFan), CullFace::Front);
}

#[test]
fn test_sanitize_glsl_identifier() {
    assert_eq!(ModelUtility::sanitize_glsl_identifier("temperature ℃"), "temperature_");
    assert_eq!(ModelUtility::sanitize_glsl_identifier("custom__property"), "custom_property");
    assert_eq!(ModelUtility::sanitize_glsl_identifier("gl_customProperty"), "customProperty");
    assert_eq!(ModelUtility::sanitize_glsl_identifier("1234"), "_1234");
    assert_eq!(ModelUtility::sanitize_glsl_identifier("gl_1st__test℃_variable"), "_1st_test_variable");
}

#[test]
fn test_check_supported_extensions() {
    let supported = ["AGI_articulations", "CESIUM_mesh_vector", "CESIUM_primitive_outline"];
    assert!(ModelUtility::check_supported_extensions(&supported).is_ok());

    let unsupported = ["AGI_articulations", "NOT_supported_extension", "CESIUM_primitive_outline"];
    assert!(ModelUtility::check_supported_extensions(&unsupported).is_err());
}
