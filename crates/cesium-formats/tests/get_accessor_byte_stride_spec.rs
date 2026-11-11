use cesium_formats::gltf_pipeline::get_accessor_byte_stride;
use serde_json::json;

#[test]
fn test_gets_accessor_byte_stride() {
    let gltf = json!({
        "accessors": [
            {
                "componentType": 5126, // WebGLConstants.FLOAT
                "count": 24,
                "type": "VEC3",
                "min": [-1.0, -1.0, -1.0],
                "max": [1.0, 1.0, 1.0]
            },
            {
                "bufferView": 0,
                "componentType": 5126, // WebGLConstants.FLOAT
                "count": 24,
                "type": "VEC3",
                "min": [-1.0, -1.0, -1.0],
                "max": [1.0, 1.0, 1.0]
            },
            {
                "bufferView": 1,
                "componentType": 5126, // WebGLConstants.FLOAT
                "count": 24,
                "type": "VEC3",
                "min": [-1.0, -1.0, -1.0],
                "max": [1.0, 1.0, 1.0]
            },
            {
                "componentType": 5126, // WebGLConstants.FLOAT
                "count": 24,
                "type": "VEC2",
                "min": [0.0, 0.0],
                "max": [1.0, 1.0]
            },
            {
                "componentType": 5124, // WebGLConstants.INT
                "count": 36,
                "type": "SCALAR",
                "min": [0],
                "max": [24]
            }
        ],
        "bufferViews": [
            {
                "buffer": 0,
                "byteLength": 288,
                "byteOffset": 0
            },
            {
                "buffer": 0,
                "byteLength": 288,
                "byteOffset": 288,
                "byteStride": 32
            }
        ]
    });

    assert_eq!(get_accessor_byte_stride(&gltf, &gltf["accessors"][0]), 12);
    assert_eq!(get_accessor_byte_stride(&gltf, &gltf["accessors"][1]), 12);
    assert_eq!(get_accessor_byte_stride(&gltf, &gltf["accessors"][2]), 32);
    assert_eq!(get_accessor_byte_stride(&gltf, &gltf["accessors"][3]), 8);
    assert_eq!(get_accessor_byte_stride(&gltf, &gltf["accessors"][4]), 4);
}
