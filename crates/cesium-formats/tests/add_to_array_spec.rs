use cesium_formats::gltf_pipeline::add_to_array;
use serde_json::json;

#[test]
fn test_adds_item_to_array_and_returns_its_index() {
    let mut gltf = json!({
        "buffers": []
    });
    let buffer0 = json!({
        "byteLength": 100
    });
    let buffer1 = json!({
        "byteLength": 200
    });

    assert_eq!(add_to_array(&mut gltf["buffers"], buffer0.clone(), false), 0);
    assert_eq!(add_to_array(&mut gltf["buffers"], buffer1.clone(), false), 1);
    assert_eq!(gltf["buffers"], json!([buffer0, buffer1]));
}

#[test]
fn test_returns_index_of_duplicate_element_when_check_duplicates_is_true() {
    let mut gltf = json!({
        "buffers": []
    });
    let buffer0 = json!({
        "byteLength": 100
    });
    let buffer1 = json!({
        "byteLength": 200
    });

    assert_eq!(add_to_array(&mut gltf["buffers"], buffer0.clone(), true), 0);
    assert_eq!(add_to_array(&mut gltf["buffers"], buffer1.clone(), true), 1);
    assert_eq!(add_to_array(&mut gltf["buffers"], buffer0.clone(), true), 0);
    assert_eq!(gltf["buffers"].as_array().unwrap().len(), 2);
    assert_eq!(add_to_array(&mut gltf["buffers"], buffer0.clone(), false), 2);
    assert_eq!(gltf["buffers"].as_array().unwrap().len(), 3);
}
