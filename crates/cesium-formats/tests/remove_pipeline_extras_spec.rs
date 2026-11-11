use cesium_formats::gltf_pipeline::{add_pipeline_extras, remove_pipeline_extras};
use serde_json::json;

#[test]
fn test_removes_pipeline_extras() {
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
        }
    });

    let mut gltf_with_extras = add_pipeline_extras(gltf);
    assert!(gltf_with_extras["buffers"][0].get("extras").is_some());
    assert!(gltf_with_extras["extensions"]["KHR_techniques_webgl"]["shaders"][0].get("extras").is_some());

    remove_pipeline_extras(&mut gltf_with_extras);
    assert!(gltf_with_extras["buffers"][0].get("extras").is_none());
    assert!(gltf_with_extras["extensions"]["KHR_techniques_webgl"]["shaders"][0].get("extras").is_none());
}
