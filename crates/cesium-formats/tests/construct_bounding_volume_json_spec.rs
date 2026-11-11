use cesium_formats::gltf_pipeline::construct_bounding_volume_json;
use serde_json::json;

#[test]
fn test_converts_box_size_to_half_axes_centered_at_origin() {
    let shapes = vec![
        json!({ "type": "box", "box": { "size": [2.0, 4.0, 6.0] } }),
        json!({ "type": "sphere", "sphere": { "radius": 1.0 } }),
        json!({ "type": "sphere", "sphere": {} }),
        json!({ "type": "capsule", "capsule": {} }),
    ];

    let result = construct_bounding_volume_json(&json!({ "shape": 0 }), &shapes).unwrap();
    assert_eq!(
        result,
        json!({
            "box": [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 3.0]
        })
    );
}

#[test]
fn test_applies_translation_to_box_center() {
    let shapes = vec![
        json!({ "type": "box", "box": { "size": [2.0, 4.0, 6.0] } }),
    ];

    let result = construct_bounding_volume_json(
        &json!({ "shape": 0, "translation": [10.0, 20.0, 30.0] }),
        &shapes,
    )
    .unwrap();
    assert_eq!(
        result["box"],
        json!([10.0, 20.0, 30.0, 1.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 3.0])
    );
}

#[test]
fn test_applies_scale_to_half_axes() {
    let shapes = vec![
        json!({ "type": "box", "box": { "size": [2.0, 4.0, 6.0] } }),
    ];

    let result = construct_bounding_volume_json(
        &json!({ "shape": 0, "scale": [2.0, 3.0, 4.0] }),
        &shapes,
    )
    .unwrap();
    assert_eq!(
        result["box"],
        json!([0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 6.0, 0.0, 0.0, 0.0, 12.0])
    );
}

#[test]
fn test_applies_rotation_to_half_axes() {
    let half_sqrt2 = std::f64::consts::FRAC_1_SQRT_2;
    let shapes = vec![
        json!({ "type": "box", "box": { "size": [2.0, 4.0, 6.0] } }),
    ];

    let result = construct_bounding_volume_json(
        &json!({ "shape": 0, "rotation": [0.0, 0.0, half_sqrt2, half_sqrt2] }),
        &shapes,
    )
    .unwrap();

    let expected = vec![0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -2.0, 0.0, 0.0, 0.0, 0.0, 3.0];
    let actual = result["box"].as_array().unwrap();
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected.iter()) {
        let diff = (a.as_f64().unwrap() - *e).abs();
        assert!(diff < 1e-6, "actual {} != expected {}", a, e);
    }
}

#[test]
fn test_does_not_modify_translation_of_input() {
    let shapes = vec![
        json!({ "type": "box", "box": { "size": [2.0, 4.0, 6.0] } }),
    ];
    let input = json!({ "shape": 0, "translation": [1.0, 2.0, 3.0] });
    let _ = construct_bounding_volume_json(&input, &shapes).unwrap();
    assert_eq!(input["translation"], json!([1.0, 2.0, 3.0]));
}

#[test]
fn test_converts_sphere_radius_centered_at_origin() {
    let shapes = vec![
        json!({ "type": "box", "box": { "size": [2.0, 4.0, 6.0] } }),
        json!({ "type": "sphere", "sphere": { "radius": 1.0 } }),
        json!({ "type": "sphere", "sphere": {} }),
        json!({ "type": "capsule", "capsule": {} }),
    ];

    let result = construct_bounding_volume_json(&json!({ "shape": 1 }), &shapes).unwrap();
    assert_eq!(result, json!({ "sphere": [0.0, 0.0, 0.0, 1.0] }));
}

#[test]
fn test_applies_translation_to_sphere_center() {
    let shapes = vec![
        json!({ "type": "box", "box": { "size": [2.0, 4.0, 6.0] } }),
        json!({ "type": "sphere", "sphere": { "radius": 1.0 } }),
    ];

    let result = construct_bounding_volume_json(
        &json!({ "shape": 1, "translation": [10.0, 20.0, 30.0] }),
        &shapes,
    )
    .unwrap();
    assert_eq!(result["sphere"], json!([10.0, 20.0, 30.0, 1.0]));
}

#[test]
fn test_uses_default_sphere_radius_of_half() {
    let shapes = vec![
        json!({ "type": "sphere", "sphere": {} }),
    ];

    let result = construct_bounding_volume_json(&json!({ "shape": 0 }), &shapes).unwrap();
    assert_eq!(result["sphere"], json!([0.0, 0.0, 0.0, 0.5]));
}

#[test]
fn test_applies_uniform_scale_to_sphere_radius() {
    let shapes = vec![
        json!({ "type": "sphere", "sphere": { "radius": 1.0 } }),
    ];

    let result = construct_bounding_volume_json(
        &json!({ "shape": 0, "scale": [3.0, 3.0, 3.0] }),
        &shapes,
    )
    .unwrap();
    assert_eq!(result["sphere"], json!([0.0, 0.0, 0.0, 3.0]));
}

#[test]
fn test_uses_largest_scale_component_for_sphere_radius() {
    let shapes = vec![
        json!({ "type": "sphere", "sphere": { "radius": 1.0 } }),
    ];

    let result = construct_bounding_volume_json(
        &json!({ "shape": 0, "scale": [2.0, -5.0, 3.0] }),
        &shapes,
    )
    .unwrap();
    assert_eq!(result["sphere"], json!([0.0, 0.0, 0.0, 5.0]));
}

#[test]
fn test_ignores_rotation_for_spheres() {
    let half_sqrt2 = std::f64::consts::FRAC_1_SQRT_2;
    let shapes = vec![
        json!({ "type": "sphere", "sphere": { "radius": 1.0 } }),
    ];

    let result = construct_bounding_volume_json(
        &json!({ "shape": 0, "rotation": [0.0, 0.0, half_sqrt2, half_sqrt2] }),
        &shapes,
    )
    .unwrap();
    assert_eq!(result["sphere"], json!([0.0, 0.0, 0.0, 1.0]));
}

#[test]
fn test_throws_for_shapes_other_than_box_and_sphere() {
    let shapes = vec![
        json!({ "type": "capsule", "capsule": {} }),
    ];

    let result = construct_bounding_volume_json(&json!({ "shape": 0 }), &shapes);
    assert!(result.is_err());
}
