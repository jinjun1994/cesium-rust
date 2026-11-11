use cesium_formats::add_buffer;
use serde_json::json;

#[test]
fn test_adds_buffer_to_gltf_and_returns_its_bufferview_id() {
    let mut gltf = json!({
        "buffers": [],
        "bufferViews": []
    });

    let buffer0 = vec![0u8; 100];
    let buffer1 = vec![1u8; 200];

    let id0 = add_buffer(&mut gltf, &buffer0);
    assert_eq!(id0, 0);

    let id1 = add_buffer(&mut gltf, &buffer1);
    assert_eq!(id1, 1);

    assert_eq!(gltf["buffers"].as_array().unwrap().len(), 2);
    assert_eq!(gltf["bufferViews"].as_array().unwrap().len(), 2);

    let source0: Vec<u8> = gltf["buffers"][0]["extras"]["_pipeline"]["source"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u8)
        .collect();
    assert_eq!(source0, buffer0);

    let source1: Vec<u8> = gltf["buffers"][1]["extras"]["_pipeline"]["source"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u8)
        .collect();
    assert_eq!(source1, buffer1);

    assert_eq!(gltf["bufferViews"][0]["buffer"], 0);
    assert_eq!(gltf["bufferViews"][0]["byteLength"], 100);
    assert_eq!(gltf["bufferViews"][0]["byteOffset"], 0);

    assert_eq!(gltf["bufferViews"][1]["buffer"], 1);
    assert_eq!(gltf["bufferViews"][1]["byteLength"], 200);
    assert_eq!(gltf["bufferViews"][1]["byteOffset"], 0);
}
