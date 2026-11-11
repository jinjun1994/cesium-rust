use cesium_formats::gltf_pipeline::remove_extension;
use serde_json::json;

#[test]
fn test_removes_extension() {
    let mut gltf = json!({
        "extensionsRequired": ["extension1", "extension2", "extension3"],
        "extensionsUsed": ["extension1", "extension2", "extension3"],
        "extensions": {
            "extension1": {
                "value": 9
            },
            "extension2": [0, 1, 2]
        },
        "materials": [
            {
                "baseColorFactor": [1.0, 0.0, 0.0, 1.0],
                "extensions": {
                    "extension1": {
                        "value": 10
                    }
                }
            },
            {
                "baseColorFactor": [0.0, 0.0, 1.0, 1.0],
                "extensions": {
                    "extension1": {
                        "value": 11
                    }
                }
            }
        ],
        "cameras": [
            {
                "extensions": {
                    "extension1": {
                        "value": 9
                    },
                    "extension2": [3, 4, 5]
                }
            }
        ]
    });

    let extension1 = remove_extension(&mut gltf, "extension1");
    assert_eq!(
        gltf["extensionsRequired"],
        json!(["extension2", "extension3"])
    );
    assert_eq!(
        gltf["extensionsUsed"],
        json!(["extension2", "extension3"])
    );
    assert_eq!(
        gltf["extensions"],
        json!({
            "extension2": [0, 1, 2]
        })
    );
    assert!(gltf["materials"][0].get("extensions").is_none());
    assert!(gltf["materials"][1].get("extensions").is_none());
    assert_eq!(
        gltf["cameras"][0]["extensions"],
        json!({
            "extension2": [3, 4, 5]
        })
    );
    assert_eq!(extension1, Some(json!({ "value": 9 })));

    let extension2 = remove_extension(&mut gltf, "extension2");
    assert_eq!(gltf["extensionsRequired"], json!(["extension3"]));
    assert_eq!(gltf["extensionsUsed"], json!(["extension3"]));
    assert!(gltf.get("extensions").is_none());
    assert!(gltf["materials"][0].get("extensions").is_none());
    assert!(gltf["materials"][1].get("extensions").is_none());
    assert!(gltf["cameras"][0].get("extensions").is_none());
    assert_eq!(extension2, Some(json!([0, 1, 2])));

    let extension3 = remove_extension(&mut gltf, "extension3");
    assert!(gltf.get("extensionsRequired").is_none());
    assert!(gltf.get("extensionsUsed").is_none());
    assert!(gltf.get("extensions").is_none());
    assert!(gltf["materials"][0].get("extensions").is_none());
    assert!(gltf["materials"][1].get("extensions").is_none());
    assert!(gltf["cameras"][0].get("extensions").is_none());
    assert!(extension3.is_none());

    let mut empty_gltf = json!({});
    remove_extension(&mut empty_gltf, "extension1");
    assert_eq!(empty_gltf, json!({}));
}

#[test]
fn test_removes_cesium_rtc_extension() {
    let mut gltf = json!({
        "extensionsRequired": ["CESIUM_RTC", "KHR_techniques_webgl"],
        "extensionsUsed": ["CESIUM_RTC", "KHR_techniques_webgl"],
        "extensions": {
            "CESIUM_RTC": {
                "center": [1.0, 2.0, 3.0]
            },
            "KHR_techniques_webgl": {
                "techniques": [
                    {
                        "uniforms": {
                            "u_modelViewMatrix": {
                                "type": 35676,
                                "semantic": "CESIUM_RTC_MODELVIEW"
                            }
                        }
                    }
                ]
            }
        }
    });

    let extension = remove_extension(&mut gltf, "CESIUM_RTC");
    assert_eq!(extension, Some(json!({ "center": [1.0, 2.0, 3.0] })));
    assert_eq!(gltf["extensionsRequired"], json!(["KHR_techniques_webgl"]));
    assert_eq!(gltf["extensionsUsed"], json!(["KHR_techniques_webgl"]));
    assert_eq!(
        gltf["extensions"]["KHR_techniques_webgl"]["techniques"][0]["uniforms"]["u_modelViewMatrix"]["semantic"],
        "MODELVIEW"
    );
}
