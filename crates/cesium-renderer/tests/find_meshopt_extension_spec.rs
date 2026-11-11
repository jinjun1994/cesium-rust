//! Spec matching CesiumJS `Specs/Scene/findMeshoptExtensionSpec.js` (Apache-2.0).

use cesium_renderer::find_meshopt_extension;
use serde_json::json;

#[test]
fn test_returns_undefined_when_neither_extension_is_present() {
    assert!(find_meshopt_extension(&json!({})).is_none());
    assert!(find_meshopt_extension(&json!({ "extensions": {} })).is_none());
}

#[test]
fn test_returns_ext_object_when_only_ext_is_present() {
    let ext = json!({
        "buffer": 1,
        "byteOffset": 0,
        "byteLength": 100,
        "mode": "ATTRIBUTES",
    });
    let gltf_object = json!({
        "extensions": {
            "EXT_meshopt_compression": ext.clone(),
        }
    });

    let result = find_meshopt_extension(&gltf_object);
    assert_eq!(result, Some(&ext));
}

#[test]
fn test_returns_khr_object_when_only_khr_is_present() {
    let khr = json!({
        "buffer": 2,
        "byteOffset": 4,
        "byteLength": 200,
        "mode": "ATTRIBUTES",
        "filter": "COLOR",
    });
    let gltf_object = json!({
        "extensions": {
            "KHR_meshopt_compression": khr.clone(),
        }
    });

    let result = find_meshopt_extension(&gltf_object);
    assert_eq!(result, Some(&khr));
}

#[test]
fn test_prefers_khr_when_both_extensions_are_present() {
    let khr = json!({
        "buffer": 2,
        "mode": "ATTRIBUTES",
    });
    let ext = json!({
        "buffer": 1,
        "mode": "ATTRIBUTES",
    });
    let gltf_object = json!({
        "extensions": {
            "KHR_meshopt_compression": khr.clone(),
            "EXT_meshopt_compression": ext,
        }
    });

    let result = find_meshopt_extension(&gltf_object);
    assert_eq!(result, Some(&khr));
}
