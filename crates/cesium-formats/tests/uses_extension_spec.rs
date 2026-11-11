use cesium_formats::gltf_for_each::uses_extension;
use serde_json::json;

#[test]
fn test_uses_extension() {
    let gltf = json!({
        "extensionsUsed": ["extension1", "extension2"]
    });

    assert!(uses_extension(&gltf, "extension1"));
    assert!(uses_extension(&gltf, "extension2"));
    assert!(!uses_extension(&gltf, "extension3"));

    let empty_gltf = json!({});
    assert!(!uses_extension(&empty_gltf, "extension1"));
}
