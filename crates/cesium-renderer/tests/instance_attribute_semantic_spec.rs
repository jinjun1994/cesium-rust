use cesium_renderer::instance_attribute_semantic::InstanceAttributeSemantic;

#[test]
fn from_gltf_semantic_matches_expected() {
    let gltf_semantics = [
        "TRANSLATION",
        "ROTATION",
        "SCALE",
        "_FEATURE_ID_0",
        "_FEATURE_ID_1",
        "_OTHER",
    ];

    let expected_semantics = [
        Some(InstanceAttributeSemantic::Translation),
        Some(InstanceAttributeSemantic::Rotation),
        Some(InstanceAttributeSemantic::Scale),
        Some(InstanceAttributeSemantic::FeatureId),
        Some(InstanceAttributeSemantic::FeatureId),
        None,
    ];

    for (semantic, expected) in gltf_semantics.iter().zip(expected_semantics.iter()) {
        assert_eq!(
            InstanceAttributeSemantic::from_gltf_semantic(semantic),
            *expected
        );
    }
}

#[test]
fn string_representation_matches_gltf_standard() {
    assert_eq!(InstanceAttributeSemantic::Translation.as_str(), "TRANSLATION");
    assert_eq!(InstanceAttributeSemantic::Rotation.as_str(), "ROTATION");
    assert_eq!(InstanceAttributeSemantic::Scale.as_str(), "SCALE");
    assert_eq!(InstanceAttributeSemantic::FeatureId.as_str(), "_FEATURE_ID");
}
