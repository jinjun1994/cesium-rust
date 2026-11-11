use cesium_formats::gltf_pipeline::find_accessor_min_max;
use serde_json::{json, Value};

fn create_gltf(elements: &[f32], byte_stride: usize) -> Value {
    let mut source_bytes = Vec::new();
    for &val in elements {
        source_bytes.extend_from_slice(&val.to_le_bytes());
    }
    let byte_length = source_bytes.len();
    let source_json: Vec<Value> = source_bytes.iter().map(|&b| json!(b)).collect();

    json!({
        "asset": {
            "version": "2.0"
        },
        "accessors": [
            {
                "bufferView": 0,
                "byteOffset": 0,
                "componentType": 5126, // FLOAT
                "count": 4,
                "type": "VEC3"
            }
        ],
        "bufferViews": [
            {
                "buffer": 0,
                "byteOffset": 0,
                "byteLength": byte_length,
                "byteStride": byte_stride
            }
        ],
        "buffers": [
            {
                "byteLength": byte_length,
                "extras": {
                    "_pipeline": {
                        "source": source_json
                    }
                }
            }
        ]
    })
}

#[test]
fn test_finds_min_and_max_of_contiguous_accessor() {
    let contiguous_data: Vec<f32> = vec![
        -1.0, -2.0, -3.0, 3.0, 2.0, 1.0, 0.0, 0.0, 0.0, 0.5, -0.5, 0.5,
    ];
    let gltf = create_gltf(&contiguous_data, 12);
    let min_max = find_accessor_min_max(&gltf, &gltf["accessors"][0]).unwrap();
    assert_eq!(min_max.min, vec![-1.0, -2.0, -3.0]);
    assert_eq!(min_max.max, vec![3.0, 2.0, 1.0]);
}

#[test]
fn test_finds_min_and_max_of_non_contiguous_accessor() {
    let nan = f32::NAN;
    let non_contiguous_data: Vec<f32> = vec![
        -1.0, 1.0, -1.0, nan, nan, nan,
        0.0, 0.0, 0.0, nan, nan, nan,
        3.0, 2.0, 1.0, nan, nan, nan,
        -1.0, -2.0, -3.0, nan, nan, nan,
    ];
    let gltf = create_gltf(&non_contiguous_data, 24);
    let min_max = find_accessor_min_max(&gltf, &gltf["accessors"][0]).unwrap();
    assert_eq!(min_max.min, vec![-1.0, -2.0, -3.0]);
    assert_eq!(min_max.max, vec![3.0, 2.0, 1.0]);
}
