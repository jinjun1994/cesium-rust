use cesium_formats::gltf_pipeline::read_accessor_packed;
use serde_json::{json, Value};

fn create_gltf(elements: &[f32], byte_stride: usize) -> Value {
    let mut source_bytes = Vec::new();
    for &val in elements {
        source_bytes.extend_from_slice(&val.to_le_bytes());
    }
    let byte_length = source_bytes.len();
    let source_json: Vec<Value> = source_bytes.iter().map(|&b| json!(b)).collect();

    json!({
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
fn test_reads_contiguous_accessor() {
    let contiguous_data: Vec<f32> = vec![
        -1.0, 1.0, -1.0, 0.0, 0.0, 0.0, 3.0, 2.0, 1.0, -1.0, -2.0, -3.0,
    ];
    let gltf = create_gltf(&contiguous_data, 12);
    let values = read_accessor_packed(&gltf, &gltf["accessors"][0]).unwrap();
    let expected: Vec<f64> = contiguous_data.iter().map(|&v| v as f64).collect();
    assert_eq!(values, expected);
}

#[test]
fn test_reads_non_contiguous_accessor() {
    let nan = f32::NAN;
    let non_contiguous_data: Vec<f32> = vec![
        -1.0, 1.0, -1.0, nan, nan, nan,
        0.0, 0.0, 0.0, nan, nan, nan,
        3.0, 2.0, 1.0, nan, nan, nan,
        -1.0, -2.0, -3.0, nan, nan, nan,
    ];
    let gltf = create_gltf(&non_contiguous_data, 24);
    let values = read_accessor_packed(&gltf, &gltf["accessors"][0]).unwrap();
    let expected: Vec<f64> = vec![
        -1.0, 1.0, -1.0, 0.0, 0.0, 0.0, 3.0, 2.0, 1.0, -1.0, -2.0, -3.0,
    ];
    assert_eq!(values, expected);
}

#[test]
fn test_reads_accessor_that_does_not_have_a_buffer_view() {
    let gltf = json!({
        "accessors": [
            {
                "componentType": 5126,
                "count": 4,
                "type": "VEC3"
            }
        ]
    });
    let values = read_accessor_packed(&gltf, &gltf["accessors"][0]).unwrap();
    let expected = vec![0.0; 12];
    assert_eq!(values, expected);
}
