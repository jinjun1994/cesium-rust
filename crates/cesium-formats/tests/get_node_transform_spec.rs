use cesium_formats::gltf_pipeline::get_node_transform;
use serde_json::json;

#[test]
fn test_returns_none_when_node_has_no_transform() {
    assert!(get_node_transform(&json!({})).is_none());
}

#[test]
fn test_returns_matrix_when_node_has_one() {
    let matrix = vec![1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 5.0, 6.0, 7.0, 1.0];
    let result = get_node_transform(&json!({ "matrix": matrix })).unwrap();
    assert_eq!(result, matrix);
}

#[test]
fn test_composes_translation_rotation_and_scale() {
    let transform = get_node_transform(&json!({
        "translation": [1.0, 2.0, 3.0],
        "rotation": [0.0, 0.0, 0.0, 1.0],
        "scale": [2.0, 3.0, 4.0]
    }))
    .unwrap();
    assert_eq!(
        transform,
        vec![2.0, 0.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 0.0, 4.0, 0.0, 1.0, 2.0, 3.0, 1.0]
    );
}

#[test]
fn test_uses_identity_values_for_missing_translation_rotation_and_scale() {
    let transform1 = get_node_transform(&json!({
        "translation": [1.0, 2.0, 3.0]
    }))
    .unwrap();
    assert_eq!(
        transform1,
        vec![1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 2.0, 3.0, 1.0]
    );

    let transform2 = get_node_transform(&json!({
        "scale": [2.0, 3.0, 4.0]
    }))
    .unwrap();
    assert_eq!(
        transform2,
        vec![2.0, 0.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 0.0, 4.0, 0.0, 0.0, 0.0, 0.0, 1.0]
    );
}
