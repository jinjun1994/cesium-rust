use cesium_formats::gltf_pipeline::remove_extensions_used;
use serde_json::json;

#[test]
fn test_removes_extension_from_extensions_used() {
    let mut gltf = json!({
        "extensionsRequired": ["extension1", "extension2"],
        "extensionsUsed": ["extension1", "extension2"]
    });

    remove_extensions_used(&mut gltf, "extension1");
    assert_eq!(gltf["extensionsRequired"], json!(["extension2"]));
    assert_eq!(gltf["extensionsUsed"], json!(["extension2"]));

    remove_extensions_used(&mut gltf, "extension2");
    assert!(gltf.get("extensionsRequired").is_none());
    assert!(gltf.get("extensionsUsed").is_none());

    let mut empty_gltf = json!({});
    remove_extensions_used(&mut empty_gltf, "extension1");
    assert_eq!(empty_gltf, json!({}));
}
