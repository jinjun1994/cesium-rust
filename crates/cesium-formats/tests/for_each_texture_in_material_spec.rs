use cesium_formats::gltf_for_each::for_each_texture_in_material;
use serde_json::{json, Value};

#[test]
#[should_panic(expected = "material must be defined")]
fn test_throws_without_material() {
    for_each_texture_in_material(None, |_, _| None::<()>);
}

#[test]
fn test_does_nothing_for_material_with_no_textures() {
    let material = json!({});
    let mut indices = Vec::new();
    for_each_texture_in_material(Some(&material), |index, _| {
        indices.push(index);
        None::<()>
    });
    assert!(indices.is_empty());
}

#[test]
fn test_iterates_over_pbr_metallic_roughness_textures() {
    let base_color_texture = json!({ "index": 0 });
    let metallic_roughness_texture = json!({ "index": 1 });
    let material = json!({
        "pbrMetallicRoughness": {
            "baseColorTexture": base_color_texture,
            "metallicRoughnessTexture": metallic_roughness_texture
        }
    });

    let mut visited: Vec<(usize, Value)> = Vec::new();
    for_each_texture_in_material(Some(&material), |index, texture_info| {
        visited.push((index, texture_info.clone()));
        None::<()>
    });

    assert_eq!(
        visited,
        vec![
            (0, json!({ "index": 0 })),
            (1, json!({ "index": 1 }))
        ]
    );
}

#[test]
fn test_iterates_over_khr_materials_pbr_specular_glossiness_textures() {
    let material = json!({
        "extensions": {
            "KHR_materials_pbrSpecularGlossiness": {
                "diffuseTexture": { "index": 0 },
                "specularGlossinessTexture": { "index": 1 }
            }
        }
    });

    let mut indices = Vec::new();
    for_each_texture_in_material(Some(&material), |index, _| {
        indices.push(index);
        None::<()>
    });
    assert_eq!(indices, vec![0, 1]);
}

#[test]
fn test_iterates_over_khr_materials_specular_textures() {
    let material = json!({
        "extensions": {
            "KHR_materials_specular": {
                "specularTexture": { "index": 0 },
                "specularColorTexture": { "index": 1 }
            }
        }
    });

    let mut indices = Vec::new();
    for_each_texture_in_material(Some(&material), |index, _| {
        indices.push(index);
        None::<()>
    });
    assert_eq!(indices, vec![0, 1]);
}

#[test]
fn test_iterates_over_khr_materials_transmission_texture() {
    let material = json!({
        "extensions": {
            "KHR_materials_transmission": {
                "transmissionTexture": { "index": 0 }
            }
        }
    });

    let mut indices = Vec::new();
    for_each_texture_in_material(Some(&material), |index, _| {
        indices.push(index);
        None::<()>
    });
    assert_eq!(indices, vec![0]);
}

#[test]
fn test_iterates_over_khr_materials_common_texture_values() {
    let material = json!({
        "extensions": {
            "KHR_materials_common": {
                "values": {
                    "diffuse": { "index": 0 },
                    "ambient": { "index": 1 },
                    "emission": { "index": 2 },
                    "specular": { "index": 3 }
                }
            }
        }
    });

    let mut indices = Vec::new();
    for_each_texture_in_material(Some(&material), |index, _| {
        indices.push(index);
        None::<()>
    });
    assert_eq!(indices, vec![0, 1, 2, 3]);
}

#[test]
fn test_ignores_non_texture_khr_materials_common_values() {
    let material = json!({
        "extensions": {
            "KHR_materials_common": {
                "values": {
                    "diffuse": [1.0, 0.0, 0.0, 1.0]
                }
            }
        }
    });

    let mut indices = Vec::new();
    for_each_texture_in_material(Some(&material), |index, _| {
        indices.push(index);
        None::<()>
    });
    assert!(indices.is_empty());
}

#[test]
fn test_iterates_over_khr_techniques_webgl_material_values_that_are_textures() {
    let material = json!({
        "extensions": {
            "KHR_techniques_webgl": {
                "values": {
                    "u_diffuse": { "index": 2 },
                    "u_shininess": 256
                }
            }
        }
    });

    let mut indices = Vec::new();
    for_each_texture_in_material(Some(&material), |index, _| {
        indices.push(index);
        None::<()>
    });
    assert_eq!(indices, vec![2]);
}

#[test]
fn test_iterates_over_top_level_textures() {
    let material = json!({
        "emissiveTexture": { "index": 0 },
        "normalTexture": { "index": 1 },
        "occlusionTexture": { "index": 2 }
    });

    let mut indices = Vec::new();
    for_each_texture_in_material(Some(&material), |index, _| {
        indices.push(index);
        None::<()>
    });
    assert_eq!(indices, vec![0, 1, 2]);
}

#[test]
fn test_stops_iterating_and_returns_value_when_handler_returns_one() {
    let metallic_roughness_texture = json!({ "index": 1 });
    let material = json!({
        "pbrMetallicRoughness": {
            "baseColorTexture": { "index": 0 },
            "metallicRoughnessTexture": metallic_roughness_texture
        },
        "emissiveTexture": { "index": 2 }
    });

    let mut visited = Vec::new();
    let return_val = for_each_texture_in_material(Some(&material), |index, texture_info| {
        visited.push(index);
        if index == 1 {
            Some(texture_info.clone())
        } else {
            None
        }
    });

    assert_eq!(visited, vec![0, 1]);
    assert_eq!(return_val, Some(json!({ "index": 1 })));
}
