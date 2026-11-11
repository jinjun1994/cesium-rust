use cesium_renderer::vertex_attribute_semantic::VertexAttributeSemantic;

#[test]
fn test_has_set_index() {
    let semantics = [
        VertexAttributeSemantic::POSITION,
        VertexAttributeSemantic::NORMAL,
        VertexAttributeSemantic::TANGENT,
        VertexAttributeSemantic::TEXCOORD,
        VertexAttributeSemantic::COLOR,
        VertexAttributeSemantic::JOINTS,
        VertexAttributeSemantic::WEIGHTS,
        VertexAttributeSemantic::FEATURE_ID,
        VertexAttributeSemantic::SCALE,
        VertexAttributeSemantic::ROTATION,
    ];

    let has_set_index = [
        false, false, false, true, true, true, true, true, true, true,
    ];

    for (sem, expected) in semantics.iter().zip(has_set_index.iter()) {
        assert_eq!(
            VertexAttributeSemantic::has_set_index(sem).unwrap(),
            *expected
        );
    }

    assert!(VertexAttributeSemantic::has_set_index("OTHER").is_err());
}

#[test]
fn test_from_gltf_semantic() {
    let gltf_semantics = [
        "POSITION",
        "NORMAL",
        "TANGENT",
        "TEXCOORD_0",
        "TEXCOORD_1",
        "COLOR_0",
        "COLOR_1",
        "JOINTS_0",
        "JOINTS_1",
        "WEIGHTS_0",
        "WEIGHTS_1",
        "_FEATURE_ID_0",
        "_FEATURE_ID_1",
        "_OTHER",
        "_SCALE",
        "KHR_gaussian_splatting:SCALE",
        "_ROTATION",
        "KHR_gaussian_splatting:ROTATION",
    ];

    let expected_semantics = [
        Some(VertexAttributeSemantic::POSITION),
        Some(VertexAttributeSemantic::NORMAL),
        Some(VertexAttributeSemantic::TANGENT),
        Some(VertexAttributeSemantic::TEXCOORD),
        Some(VertexAttributeSemantic::TEXCOORD),
        Some(VertexAttributeSemantic::COLOR),
        Some(VertexAttributeSemantic::COLOR),
        Some(VertexAttributeSemantic::JOINTS),
        Some(VertexAttributeSemantic::JOINTS),
        Some(VertexAttributeSemantic::WEIGHTS),
        Some(VertexAttributeSemantic::WEIGHTS),
        Some(VertexAttributeSemantic::FEATURE_ID),
        Some(VertexAttributeSemantic::FEATURE_ID),
        None,
        Some(VertexAttributeSemantic::SCALE),
        Some(VertexAttributeSemantic::SCALE),
        Some(VertexAttributeSemantic::ROTATION),
        Some(VertexAttributeSemantic::ROTATION),
    ];

    for (input, expected) in gltf_semantics.iter().zip(expected_semantics.iter()) {
        let result = VertexAttributeSemantic::from_gltf_semantic(input).unwrap();
        assert_eq!(result, *expected, "Mismatch for semantic: {}", input);
    }
}

#[test]
fn test_from_pnts_semantic() {
    let pnts_semantics = [
        "POSITION",
        "POSITION_QUANTIZED",
        "RGBA",
        "RGB",
        "RGB565",
        "NORMAL",
        "NORMAL_OCT16P",
        "BATCH_ID",
    ];

    let expected_semantics = [
        VertexAttributeSemantic::POSITION,
        VertexAttributeSemantic::POSITION,
        VertexAttributeSemantic::COLOR,
        VertexAttributeSemantic::COLOR,
        VertexAttributeSemantic::COLOR,
        VertexAttributeSemantic::NORMAL,
        VertexAttributeSemantic::NORMAL,
        VertexAttributeSemantic::FEATURE_ID,
    ];

    for (input, expected) in pnts_semantics.iter().zip(expected_semantics.iter()) {
        let result = VertexAttributeSemantic::from_pnts_semantic(input).unwrap();
        assert_eq!(result, *expected, "Mismatch for pnts semantic: {}", input);
    }

    assert!(VertexAttributeSemantic::from_pnts_semantic("OTHER").is_err());
}

#[test]
fn test_get_glsl_type() {
    let semantics = [
        VertexAttributeSemantic::POSITION,
        VertexAttributeSemantic::NORMAL,
        VertexAttributeSemantic::TANGENT,
        VertexAttributeSemantic::TEXCOORD,
        VertexAttributeSemantic::COLOR,
        VertexAttributeSemantic::JOINTS,
        VertexAttributeSemantic::WEIGHTS,
        VertexAttributeSemantic::FEATURE_ID,
        VertexAttributeSemantic::SCALE,
        VertexAttributeSemantic::ROTATION,
    ];

    let expected_types = [
        "vec3", "vec3", "vec3", "vec2", "vec4", "ivec4", "vec4", "int", "vec3", "vec4",
    ];

    for (sem, expected) in semantics.iter().zip(expected_types.iter()) {
        assert_eq!(
            VertexAttributeSemantic::get_glsl_type(sem).unwrap(),
            *expected
        );
    }

    assert!(VertexAttributeSemantic::get_glsl_type("OTHER").is_err());
}

#[test]
fn test_get_variable_name() {
    let semantics = [
        VertexAttributeSemantic::POSITION,
        VertexAttributeSemantic::NORMAL,
        VertexAttributeSemantic::TANGENT,
        VertexAttributeSemantic::TEXCOORD,
        VertexAttributeSemantic::COLOR,
        VertexAttributeSemantic::JOINTS,
        VertexAttributeSemantic::WEIGHTS,
        VertexAttributeSemantic::FEATURE_ID,
        VertexAttributeSemantic::SCALE,
        VertexAttributeSemantic::ROTATION,
    ];

    let expected_names = [
        "positionMC",
        "normalMC",
        "tangentMC",
        "texCoord",
        "color",
        "joints",
        "weights",
        "featureId",
        "scale",
        "rotation",
    ];

    for (sem, expected) in semantics.iter().zip(expected_names.iter()) {
        assert_eq!(
            VertexAttributeSemantic::get_variable_name(sem, None).unwrap(),
            *expected
        );
    }

    assert_eq!(
        VertexAttributeSemantic::get_variable_name(VertexAttributeSemantic::FEATURE_ID, Some(0))
            .unwrap(),
        "featureId_0"
    );

    assert!(VertexAttributeSemantic::get_variable_name("OTHER", None).is_err());
}
