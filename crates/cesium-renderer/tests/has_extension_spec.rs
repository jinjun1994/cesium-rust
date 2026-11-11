//! Spec matching CesiumJS `Specs/Scene/hasExtensionSpec.js` (Apache-2.0).

use cesium_renderer::has_extension;
use serde_json::json;

#[test]
fn test_detects_the_presence_of_an_extension() {
    let tile = json!({
        "extensions": {
            "3DTILES_extension": {}
        }
    });
    assert!(has_extension(&tile, "3DTILES_extension"));
}

#[test]
fn test_detects_the_absence_of_an_extension() {
    let tile1 = json!({});
    assert!(!has_extension(&tile1, "3DTILES_extension"));

    let tile2 = json!({
        "extensions": {}
    });
    assert!(!has_extension(&tile2, "3DTILES_extension"));

    let tile3 = json!({
        "extensions": {
            "3DTILES_other_extension": {}
        }
    });
    assert!(!has_extension(&tile3, "3DTILES_extension"));
}
