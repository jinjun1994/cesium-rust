use cesium_formats::gltf_pipeline::{update_version, UpdateVersionOptions};
use serde_json::json;

#[test]
fn test_defaults_to_1_0_if_gltf_has_no_version() {
    let mut gltf = json!({});
    update_version(
        &mut gltf,
        Some(UpdateVersionOptions {
            target_version: Some("1.0".to_string()),
            ..Default::default()
        }),
    );
    assert_eq!(gltf["asset"]["version"], "1.0");
}

#[test]
fn test_updates_empty_gltf_with_version_from_0_8_to_2_0() {
    let mut gltf = json!({
        "version": "0.8"
    });
    update_version(&mut gltf, None);
    assert!(gltf.get("version").is_none());
    assert_eq!(gltf["asset"]["version"], "2.0");
}

#[test]
fn test_updates_empty_gltf_with_version_1_0_to_2_0() {
    let mut gltf = json!({
        "asset": {
            "version": "1.0"
        }
    });
    update_version(&mut gltf, None);
    assert_eq!(gltf["asset"]["version"], "2.0");
}

#[test]
fn test_updates_gltf_with_non_standard_version_to_2_0() {
    let mut gltf = json!({
        "asset": {
            "version": "1.0.1"
        }
    });
    update_version(&mut gltf, None);
    assert_eq!(gltf["asset"]["version"], "2.0");
}

#[test]
fn test_updates_gltf_from_0_8_to_1_0() {
    let mut gltf = json!({
        "version": "0.8",
        "asset": {
            "profile": "WebGL 1.0"
        },
        "allExtensions": ["extension"],
        "nodes": {
            "node0": {
                "rotation": [0.0, 0.0, 1.0, 0.0] // axis (0,0,1), angle 0.0
            }
        }
    });

    update_version(
        &mut gltf,
        Some(UpdateVersionOptions {
            target_version: Some("1.0".to_string()),
            ..Default::default()
        }),
    );

    assert_eq!(gltf["asset"]["version"], "1.0");
    assert_eq!(gltf["extensionsUsed"], json!(["extension"]));
    // Quaternion for axis (0,0,1) angle 0 is [0, 0, 0, 1]
    assert_eq!(gltf["nodes"]["node0"]["rotation"], json!([0.0, 0.0, 0.0, 1.0]));
}

#[test]
fn test_updates_gltf_1_0_with_khr_materials_common_constant_technique() {
    let mut gltf = json!({
        "asset": {
            "version": "1.0"
        },
        "nodes": {
            "node0": {
                "mesh": "mesh0"
            }
        },
        "meshes": {
            "mesh0": {
                "primitives": [
                    {
                        "material": "mat0"
                    }
                ]
            }
        },
        "materials": {
            "mat0": {
                "extensions": {
                    "KHR_materials_common": {
                        "technique": "CONSTANT",
                        "values": {
                            "emission": [1.0, 0.0, 0.0, 1.0]
                        }
                    }
                }
            }
        }
    });

    update_version(&mut gltf, None);

    assert_eq!(gltf["asset"]["version"], "2.0");
    let mat = &gltf["materials"][0];
    assert!(mat["extensions"]["KHR_materials_unlit"].is_object());
    let exts_used = gltf["extensionsUsed"].as_array().unwrap();
    assert!(exts_used.iter().any(|e| e == "KHR_materials_unlit"));
    assert!(mat["extensions"].get("KHR_materials_common").is_none());
}

#[test]
fn test_creates_pbr_material_from_khr_techniques_webgl_with_custom_diffuse_name() {
    let mut gltf = json!({
        "asset": {
            "version": "2.0"
        },
        "materials": [
            {
                "extensions": {
                    "KHR_techniques_webgl": {
                        "values": {
                            "u_diffuse": {
                                "index": 0
                            }
                        }
                    }
                }
            }
        ]
    });

    update_version(
        &mut gltf,
        Some(UpdateVersionOptions {
            base_color_texture_names: Some(vec!["u_diffuse".to_string()]),
            ..Default::default()
        }),
    );

    let mat = &gltf["materials"][0];
    assert_eq!(mat["pbrMetallicRoughness"]["roughnessFactor"], 1.0);
    assert_eq!(mat["pbrMetallicRoughness"]["metallicFactor"], 0.0);
    assert_eq!(mat["pbrMetallicRoughness"]["baseColorTexture"]["index"], 0);
    assert!(mat.get("extensions").is_none());
}

#[test]
fn test_does_not_create_pbr_material_when_diffuse_texture_name_is_unknown() {
    let mut gltf = json!({
        "asset": {
            "version": "2.0"
        },
        "materials": [
            {
                "extensions": {
                    "KHR_techniques_webgl": {
                        "values": {
                            "u_diffuse": {
                                "index": 0
                            }
                        }
                    }
                }
            }
        ]
    });

    update_version(
        &mut gltf,
        Some(UpdateVersionOptions {
            base_color_texture_names: Some(vec!["NOT_u_diffuse".to_string()]),
            ..Default::default()
        }),
    );

    let mat = &gltf["materials"][0];
    assert!(mat.get("pbrMetallicRoughness").is_none());
    assert!(mat.get("extensions").is_none());
}
