use cesium_formats::gltf_pipeline::add_pipeline_extras;
use serde_json::json;

#[test]
fn test_adds_pipeline_extras_to_gltf_1_assets() {
    let gltf = json!({
        "buffers": {
            "sampleBuffer0": {
                "byteLength": 100
            }
        },
        "shaders": {
            "sample0VS": {
                "type": 35633, // VERTEX_SHADER
                "uri": "data:,"
            }
        }
    });

    let gltf_with_extras = add_pipeline_extras(gltf);
    assert!(gltf_with_extras["buffers"]["sampleBuffer0"]["extras"]["_pipeline"].is_object());
    assert!(gltf_with_extras["shaders"]["sample0VS"]["extras"]["_pipeline"].is_object());
}

#[test]
fn test_adds_pipeline_extras_to_gltf_2_assets() {
    let gltf = json!({
        "buffers": [
            {
                "byteLength": 100
            }
        ],
        "extensions": {
            "KHR_techniques_webgl": {
                "shaders": [
                    {
                        "type": 35633, // VERTEX_SHADER
                        "uri": "data:,"
                    }
                ]
            }
        },
        "extensionsRequired": ["KHR_techniques_webgl"],
        "extensionsUsed": ["KHR_techniques_webgl"]
    });

    let gltf_with_extras = add_pipeline_extras(gltf);
    assert!(gltf_with_extras["buffers"][0]["extras"]["_pipeline"].is_object());
    assert!(gltf_with_extras["extensions"]["KHR_techniques_webgl"]["shaders"][0]["extras"]["_pipeline"].is_object());
}
