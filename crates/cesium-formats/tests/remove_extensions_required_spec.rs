use cesium_formats::gltf_pipeline::remove_extensions_required;
use serde_json::json;

#[test]
fn test_removes_extension_from_extensions_required() {
    let mut gltf = json!({
        "extensionsRequired": ["extension1", "extension2"],
        "extensionsUsed": ["extension1", "extension2"]
    });

    remove_extensions_required(&mut gltf, "extension1");
    assert_eq!(gltf["extensionsRequired"], json!(["extension2"]));

    remove_extensions_required(&mut gltf, "extension2");
    assert!(gltf.get("extensionsRequired").is_none());
    assert_eq!(gltf["extensionsUsed"], json!(["extension1", "extension2"]));

    let mut empty_gltf = json!({});
    remove_extensions_required(&mut empty_gltf, "extension1");
    assert_eq!(empty_gltf, json!({}));
}
