use cesium_formats::gltf_pipeline::update_accessor_component_types;
use serde_json::json;

fn create_test_buffer() -> serde_json::Value {
    let byte_buffer = vec![0u8; 96];
    let float_buffer = vec![0u8; 96 * 4];
    let unsigned_short_buffer = vec![0u8; 96 * 2];

    let mut source = Vec::new();
    source.extend_from_slice(&byte_buffer);
    source.extend_from_slice(&float_buffer);
    source.extend_from_slice(&unsigned_short_buffer);

    let source_json: Vec<serde_json::Value> = source.iter().map(|&b| json!(b)).collect();

    json!({
        "byteLength": source.len(),
        "extras": {
            "_pipeline": {
                "source": source_json
            }
        }
    })
}

#[test]
fn test_converts_joints_accessor_types() {
    let mut gltf = json!({
        "meshes": [
            {
                "primitives": [
                    {
                        "attributes": {
                            "JOINTS_0": 0
                        }
                    },
                    {
                        "attributes": {
                            "JOINTS_0": 1
                        }
                    },
                    {
                        "attributes": {
                            "JOINTS_0": 2
                        }
                    }
                ]
            }
        ],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5120, // BYTE
                "count": 24,
                "type": "VEC4"
            },
            {
                "bufferView": 1,
                "componentType": 5126, // FLOAT
                "byteOffset": 12,
                "count": 24,
                "type": "VEC4"
            },
            {
                "bufferView": 2,
                "componentType": 5123, // UNSIGNED_SHORT
                "count": 24,
                "type": "VEC4"
            }
        ],
        "bufferViews": [
            {
                "buffer": 0,
                "byteOffset": 0,
                "byteLength": 96
            },
            {
                "buffer": 0,
                "byteOffset": 96,
                "byteLength": 384
            },
            {
                "buffer": 0,
                "byteOffset": 480,
                "byteLength": 192
            }
        ],
        "buffers": [create_test_buffer()]
    });

    update_accessor_component_types(&mut gltf);

    assert_eq!(gltf["accessors"].as_array().unwrap().len(), 3);
    assert_eq!(gltf["bufferViews"].as_array().unwrap().len(), 5);
    assert_eq!(gltf["buffers"].as_array().unwrap().len(), 3);

    assert_eq!(gltf["accessors"][0]["componentType"], 5121); // UNSIGNED_BYTE
    assert_eq!(gltf["accessors"][0]["bufferView"], 3);
    assert_eq!(gltf["bufferViews"][3]["buffer"], 1);
    assert_eq!(gltf["bufferViews"][3]["byteLength"], 96);

    assert_eq!(gltf["accessors"][1]["componentType"], 5123); // UNSIGNED_SHORT
    assert_eq!(gltf["accessors"][1]["bufferView"], 4);
    assert_eq!(gltf["accessors"][1]["byteOffset"], 0);
    assert_eq!(gltf["bufferViews"][4]["buffer"], 2);
    assert_eq!(gltf["bufferViews"][4]["byteLength"], 192);

    assert_eq!(gltf["accessors"][2]["componentType"], 5123); // UNSIGNED_SHORT
    assert_eq!(gltf["accessors"][2]["bufferView"], 2);
}

#[test]
fn test_converts_weights_accessor_types() {
    let mut gltf = json!({
        "meshes": [
            {
                "primitives": [
                    {
                        "attributes": {
                            "WEIGHTS_0": 0
                        }
                    },
                    {
                        "attributes": {
                            "WEIGHTS_0": 1
                        }
                    },
                    {
                        "attributes": {
                            "WEIGHTS_0": 2
                        }
                    }
                ]
            }
        ],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5126, // FLOAT
                "count": 24,
                "type": "VEC4"
            },
            {
                "bufferView": 1,
                "componentType": 5120, // BYTE
                "byteOffset": 12,
                "count": 24,
                "type": "VEC4"
            },
            {
                "bufferView": 2,
                "componentType": 5122, // SHORT
                "count": 24,
                "type": "VEC4"
            }
        ],
        "bufferViews": [
            {
                "buffer": 0,
                "byteLength": 12
            },
            {
                "buffer": 0,
                "byteOffset": 12,
                "byteLength": 12
            },
            {
                "buffer": 0,
                "byteLength": 12
            }
        ],
        "buffers": [create_test_buffer()]
    });

    update_accessor_component_types(&mut gltf);

    assert_eq!(gltf["accessors"].as_array().unwrap().len(), 3);
    assert_eq!(gltf["bufferViews"].as_array().unwrap().len(), 5);
    assert_eq!(gltf["buffers"].as_array().unwrap().len(), 3);

    assert_eq!(gltf["accessors"][0]["componentType"], 5126); // FLOAT
    assert_eq!(gltf["accessors"][0]["bufferView"], 0);

    assert_eq!(gltf["accessors"][1]["componentType"], 5121); // UNSIGNED_BYTE
    assert_eq!(gltf["accessors"][1]["bufferView"], 3);
    assert_eq!(gltf["accessors"][1]["byteOffset"], 0);
    assert_eq!(gltf["bufferViews"][3]["buffer"], 1);
    assert_eq!(gltf["bufferViews"][3]["byteLength"], 96);

    assert_eq!(gltf["accessors"][2]["componentType"], 5123); // UNSIGNED_SHORT
    assert_eq!(gltf["accessors"][2]["bufferView"], 4);
    assert_eq!(gltf["bufferViews"][4]["buffer"], 2);
    assert_eq!(gltf["bufferViews"][4]["byteLength"], 192);
}
