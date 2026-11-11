use cesium_formats::gltf_pipeline::{parse_glb, remove_pipeline_extras};
use serde_json::json;

fn write_string(glb: &mut [u8], s: &str, offset: usize) {
    let bytes = s.as_bytes();
    glb[offset..offset + bytes.len()].copy_from_slice(bytes);
}

fn write_u32(glb: &mut [u8], value: u32, offset: usize) {
    glb[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

#[test]
fn test_throws_an_error_with_invalid_magic() {
    let mut glb = vec![0u8; 20];
    write_string(&mut glb, "NOPE", 0);

    let err = parse_glb(&glb).unwrap_err();
    assert_eq!(err, "File is not valid binary glTF");
}

#[test]
fn test_throws_an_error_if_version_is_not_1_or_2() {
    let mut glb = vec![0u8; 20];
    write_string(&mut glb, "glTF", 0);
    write_u32(&mut glb, 3, 4);

    let err = parse_glb(&glb).unwrap_err();
    assert_eq!(err, "Binary glTF version is not 1 or 2");
}

#[test]
fn test_glb_1_throws_an_error_if_content_format_is_not_json() {
    let mut glb = vec![0u8; 20];
    write_string(&mut glb, "glTF", 0);
    write_u32(&mut glb, 1, 4);
    write_u32(&mut glb, 20, 8);
    write_u32(&mut glb, 0, 12);
    write_u32(&mut glb, 1, 16);

    let err = parse_glb(&glb).unwrap_err();
    assert_eq!(err, "Binary glTF scene format is not JSON");
}

#[test]
fn test_glb_1_loads_binary_gltf() {
    let binary_data = vec![0u8, 1, 2, 3, 4, 5];
    let gltf = json!({
        "bufferViews": {
            "imageBufferView": {
                "byteLength": 0
            },
            "shaderBufferView": {
                "byteLength": 0
            }
        },
        "buffers": {
            "binary_glTF": {
                "byteLength": binary_data.len(),
                "uri": "data:,"
            }
        },
        "images": {
            "image": {
                "extensions": {
                    "KHR_binary_glTF": {
                        "bufferView": "imageBufferView",
                        "mimeType": "image/jpg"
                    }
                }
            }
        },
        "shaders": {
            "shader": {
                "extensions": {
                    "KHR_binary_glTF": {
                        "bufferView": "shaderBufferView"
                    }
                }
            }
        },
        "extensionsUsed": ["KHR_binary_glTF"]
    });

    let mut gltf_string = gltf.to_string();
    while !gltf_string.len().is_multiple_of(4) {
        gltf_string.push(' ');
    }

    let total_len = 20 + gltf_string.len() + binary_data.len();
    let mut glb = vec![0u8; total_len];
    write_string(&mut glb, "glTF", 0);
    write_u32(&mut glb, 1, 4);
    write_u32(&mut glb, total_len as u32, 8);
    write_u32(&mut glb, gltf_string.len() as u32, 12);
    write_u32(&mut glb, 0, 16);
    write_string(&mut glb, &gltf_string, 20);
    glb[20 + gltf_string.len()..].copy_from_slice(&binary_data);

    let parsed_gltf = parse_glb(&glb).unwrap();
    assert!(parsed_gltf.get("extensionsUsed").is_none());

    let buffer = &parsed_gltf["buffers"]["binary_glTF"];
    let source = buffer["extras"]["_pipeline"]["source"].as_array().unwrap();
    for (i, item) in source.iter().enumerate() {
        assert_eq!(item.as_u64().unwrap() as u8, binary_data[i]);
    }
    assert!(buffer.get("uri").is_none());

    let image = &parsed_gltf["images"]["image"];
    assert_eq!(
        image["extensions"]["KHR_binary_glTF"]["bufferView"],
        "imageBufferView"
    );
    assert_eq!(
        image["extensions"]["KHR_binary_glTF"]["mimeType"],
        "image/jpg"
    );

    let shader = &parsed_gltf["shaders"]["shader"];
    assert_eq!(
        shader["extensions"]["KHR_binary_glTF"]["bufferView"],
        "shaderBufferView"
    );
}

#[test]
fn test_glb_2_loads_binary_gltf() {
    let binary_data = vec![0u8, 1, 2, 3, 4, 5];
    let gltf = json!({
        "asset": {
            "version": "2.0"
        },
        "buffers": [
            {
                "byteLength": binary_data.len()
            }
        ],
        "images": [
            {
                "bufferView": 0,
                "mimeType": "image/jpg"
            }
        ]
    });

    let mut gltf_string = gltf.to_string();
    while !gltf_string.len().is_multiple_of(4) {
        gltf_string.push(' ');
    }

    let total_len = 28 + gltf_string.len() + binary_data.len();
    let mut glb = vec![0u8; total_len];
    write_string(&mut glb, "glTF", 0);
    write_u32(&mut glb, 2, 4);
    write_u32(&mut glb, (12 + 8 + gltf_string.len() + 8 + binary_data.len()) as u32, 8);
    write_u32(&mut glb, gltf_string.len() as u32, 12);
    write_u32(&mut glb, 0x4e4f534a, 16);
    write_string(&mut glb, &gltf_string, 20);
    write_u32(&mut glb, binary_data.len() as u32, 20 + gltf_string.len());
    write_u32(&mut glb, 0x004e4942, 24 + gltf_string.len());
    glb[28 + gltf_string.len()..].copy_from_slice(&binary_data);

    let mut parsed_gltf = parse_glb(&glb).unwrap();
    let buffer = &parsed_gltf["buffers"][0];
    let source = buffer["extras"]["_pipeline"]["source"].as_array().unwrap();
    for (i, item) in source.iter().enumerate() {
        assert_eq!(item.as_u64().unwrap() as u8, binary_data[i]);
    }

    remove_pipeline_extras(&mut parsed_gltf);
    assert_eq!(parsed_gltf, gltf);
}
