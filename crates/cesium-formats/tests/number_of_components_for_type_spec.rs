use cesium_formats::gltf_pipeline::number_of_components_for_type;

#[test]
fn test_number_of_components_for_type() {
    assert_eq!(number_of_components_for_type("SCALAR"), 1);
    assert_eq!(number_of_components_for_type("VEC2"), 2);
    assert_eq!(number_of_components_for_type("VEC3"), 3);
    assert_eq!(number_of_components_for_type("VEC4"), 4);
    assert_eq!(number_of_components_for_type("MAT2"), 4);
    assert_eq!(number_of_components_for_type("MAT3"), 9);
    assert_eq!(number_of_components_for_type("MAT4"), 16);
}
