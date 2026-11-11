use cesium_formats::gltf_pipeline::{add_defaults, WEBGL_ARRAY_BUFFER, WEBGL_ELEMENT_ARRAY_BUFFER, WEBGL_REPEAT, WEBGL_TRIANGLES};
use serde_json::json;

#[test]
fn test_adds_mesh_accessor_and_bufferview_defaults() {
    let gltf = json!({
        "meshes": [
            {
                "primitives": [
                    {
                        "attributes": {
                            "POSITION": 0
                        },
                        "indices": 2,
                        "targets": [
                            {
                                "POSITION": 1
                            }
                        ]
                    }
                ]
            }
        ],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5126, // FLOAT
                "count": 24,
                "type": "VEC3",
                "min": [-1.0, -1.0, -1.0],
                "max": [1.0, 1.0, 1.0]
            },
            {
                "bufferView": 1,
                "componentType": 5126, // FLOAT
                "count": 24,
                "type": "VEC3",
                "min": [-1.0, -1.0, -1.0],
                "max": [1.0, 1.0, 1.0]
            },
            {
                "bufferView": 2,
                "componentType": 5123, // UNSIGNED_SHORT
                "count": 36,
                "type": "SCALAR",
                "min": [0],
                "max": [24]
            }
        ],
        "bufferViews": [
            {
                "buffer": 0,
                "byteLength": 288
            },
            {
                "buffer": 0,
                "byteLength": 288,
                "byteOffset": 288
            },
            {
                "buffer": 0,
                "byteLength": 72,
                "byteOffset": 576
            },
            {
                "buffer": 0,
                "byteLength": 10,
                "byteOffset": 648
            }
        ]
    });

    let gltf_with_defaults = add_defaults(gltf);
    let primitive = &gltf_with_defaults["meshes"][0]["primitives"][0];
    let material = &gltf_with_defaults["materials"][0];
    let position_accessor = &gltf_with_defaults["accessors"][0];
    let position_target_accessor = &gltf_with_defaults["accessors"][1];
    let indices_accessor = &gltf_with_defaults["accessors"][2];
    let position_buffer_view = &gltf_with_defaults["bufferViews"][0];
    let position_target_buffer_view = &gltf_with_defaults["bufferViews"][1];
    let indices_buffer_view = &gltf_with_defaults["bufferViews"][2];
    let other_buffer_view = &gltf_with_defaults["bufferViews"][3];

    assert_eq!(primitive["mode"], WEBGL_TRIANGLES);
    assert_eq!(primitive["material"], 0);

    assert_eq!(material["emissiveFactor"], json!([0.0, 0.0, 0.0]));
    assert_eq!(material["alphaMode"], "OPAQUE");
    assert_eq!(material["doubleSided"], false);
    assert_eq!(material["name"], "default");

    assert_eq!(position_accessor["byteOffset"], 0);
    assert_eq!(position_accessor["normalized"], false);
    assert_eq!(position_buffer_view["byteOffset"], 0);
    assert_eq!(position_buffer_view["byteStride"], 12);
    assert_eq!(position_buffer_view["target"], WEBGL_ARRAY_BUFFER);

    assert_eq!(position_target_accessor["byteOffset"], 0);
    assert_eq!(position_target_accessor["normalized"], false);
    assert_eq!(position_target_buffer_view["byteStride"], 12);

    assert_eq!(indices_accessor["byteOffset"], 0);
    assert!(indices_accessor.get("normalized").is_none());
    assert!(indices_buffer_view.get("byteStride").is_none());
    assert_eq!(indices_buffer_view["target"], WEBGL_ELEMENT_ARRAY_BUFFER);

    assert!(other_buffer_view.get("target").is_none());
}

#[test]
fn test_adds_material_defaults() {
    let gltf = json!({
        "materials": [
            {
                "emissiveTexture": {
                    "index": 0
                },
                "normalTexture": {
                    "index": 1
                },
                "occlusionTexture": {
                    "index": 2
                }
            },
            {
                "alphaMode": "MASK"
            },
            {
                "extensions": {
                    "KHR_techniques_webgl": {
                        "values": {
                            "u_custom": {
                                "index": 3
                            }
                        }
                    }
                }
            }
        ]
    });

    let gltf_with_defaults = add_defaults(gltf);
    let material_opaque = &gltf_with_defaults["materials"][0];
    let material_alpha_mask = &gltf_with_defaults["materials"][1];
    let material_technique =
        &gltf_with_defaults["materials"][2]["extensions"]["KHR_techniques_webgl"];

    assert_eq!(material_opaque["emissiveFactor"], json!([0.0, 0.0, 0.0]));
    assert_eq!(material_opaque["alphaMode"], "OPAQUE");
    assert_eq!(material_opaque["doubleSided"], false);

    assert_eq!(material_opaque["emissiveTexture"]["texCoord"], 0);
    assert_eq!(material_opaque["normalTexture"]["texCoord"], 0);
    assert_eq!(material_opaque["occlusionTexture"]["texCoord"], 0);

    assert_eq!(material_alpha_mask["alphaCutoff"], 0.5);

    assert_eq!(material_technique["values"]["u_custom"]["texCoord"], 0);
}

#[test]
fn test_adds_metallic_roughness_defaults() {
    let gltf = json!({
        "materials": [
            {
                "pbrMetallicRoughness": {
                    "baseColorTexture": {
                        "index": 0
                    },
                    "metallicRoughnessTexture": {
                        "index": 1
                    }
                }
            }
        ]
    });

    let gltf_with_defaults = add_defaults(gltf);
    let pbr_metallic_roughness =
        &gltf_with_defaults["materials"][0]["pbrMetallicRoughness"];

    assert_eq!(
        pbr_metallic_roughness["baseColorFactor"],
        json!([1.0, 1.0, 1.0, 1.0])
    );
    assert_eq!(pbr_metallic_roughness["metallicFactor"], 1.0);
    assert_eq!(pbr_metallic_roughness["roughnessFactor"], 1.0);
    assert_eq!(
        pbr_metallic_roughness["baseColorTexture"]["texCoord"],
        0
    );
    assert_eq!(
        pbr_metallic_roughness["metallicRoughnessTexture"]["texCoord"],
        0
    );
}

#[test]
fn test_adds_specular_glossiness_defaults() {
    let gltf = json!({
        "materials": [
            {
                "extensions": {
                    "KHR_materials_pbrSpecularGlossiness": {
                        "specularGlossinessTexture": {
                            "index": 0
                        }
                    }
                }
            }
        ]
    });

    let gltf_with_defaults = add_defaults(gltf);
    let pbr_specular_glossiness =
        &gltf_with_defaults["materials"][0]["extensions"]["KHR_materials_pbrSpecularGlossiness"];

    assert_eq!(
        pbr_specular_glossiness["diffuseFactor"],
        json!([1.0, 1.0, 1.0, 1.0])
    );
    assert_eq!(
        pbr_specular_glossiness["specularFactor"],
        json!([1.0, 1.0, 1.0])
    );
    assert_eq!(pbr_specular_glossiness["glossinessFactor"], 1.0);
    assert_eq!(
        pbr_specular_glossiness["specularGlossinessTexture"]["texCoord"],
        0
    );
}

#[test]
fn test_adds_materials_common_defaults() {
    let gltf = json!({
        "materials": [
            {
                "extensions": {
                    "KHR_materials_common": {
                        "technique": "BLINN"
                    }
                }
            },
            {
                "extensions": {
                    "KHR_materials_common": {
                        "technique": "CONSTANT"
                    }
                }
            },
            {
                "extensions": {
                    "KHR_materials_common": {
                        "technique": "LAMBERT"
                    }
                }
            }
        ]
    });

    let gltf_with_defaults = add_defaults(gltf);
    let mc_blinn =
        &gltf_with_defaults["materials"][0]["extensions"]["KHR_materials_common"];
    let mc_constant =
        &gltf_with_defaults["materials"][1]["extensions"]["KHR_materials_common"];
    let mc_lambert =
        &gltf_with_defaults["materials"][2]["extensions"]["KHR_materials_common"];

    assert_eq!(mc_blinn["values"]["ambient"], json!([0.0, 0.0, 0.0, 1.0]));
    assert_eq!(mc_blinn["values"]["diffuse"], json!([0.0, 0.0, 0.0, 1.0]));
    assert_eq!(mc_blinn["values"]["emission"], json!([0.0, 0.0, 0.0, 1.0]));
    assert_eq!(mc_blinn["values"]["specular"], json!([0.0, 0.0, 0.0, 1.0]));
    assert_eq!(mc_blinn["values"]["shininess"], 0.0);
    assert_eq!(mc_blinn["values"]["transparency"], 1.0);
    assert_eq!(mc_blinn["transparent"], false);
    assert_eq!(mc_blinn["doubleSided"], false);

    assert!(mc_constant["values"].get("diffuse").is_none());
    assert!(mc_constant["values"].get("specular").is_none());
    assert!(mc_constant["values"].get("shininess").is_none());

    assert!(mc_lambert["values"].get("specular").is_none());
    assert!(mc_lambert["values"].get("shininess").is_none());
}

#[test]
fn test_adds_sampler_defaults() {
    let gltf = json!({
        "samplers": [
            {}
        ]
    });

    let gltf_with_defaults = add_defaults(gltf);
    let sampler = &gltf_with_defaults["samplers"][0];
    assert_eq!(sampler["wrapS"], WEBGL_REPEAT);
    assert_eq!(sampler["wrapT"], WEBGL_REPEAT);
}

#[test]
fn test_adds_node_defaults() {
    let gltf = json!({
        "animations": [
            {
                "channels": [
                    {
                        "sampler": 0,
                        "target": {
                            "node": 0,
                            "path": "rotation"
                        }
                    }
                ],
                "samplers": [
                    {
                        "input": 0,
                        "output": 1
                    }
                ]
            }
        ],
        "nodes": [
            {
                "mesh": 0
            },
            {
                "mesh": 1
            },
            {
                "mesh": 2,
                "translation": [1.0, 0.0, 0.0]
            }
        ]
    });

    let gltf_with_defaults = add_defaults(gltf);
    let animated_node = &gltf_with_defaults["nodes"][0];
    let static_node1 = &gltf_with_defaults["nodes"][1];
    let static_node2 = &gltf_with_defaults["nodes"][2];

    assert_eq!(
        gltf_with_defaults["animations"][0]["samplers"][0]["interpolation"],
        "LINEAR"
    );

    assert!(animated_node.get("matrix").is_none());
    assert_eq!(animated_node["translation"], json!([0.0, 0.0, 0.0]));
    assert_eq!(animated_node["rotation"], json!([0.0, 0.0, 0.0, 1.0]));
    assert_eq!(animated_node["scale"], json!([1.0, 1.0, 1.0]));

    assert_eq!(
        static_node1["matrix"],
        json!([1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0])
    );
    assert!(static_node1.get("translation").is_none());
    assert!(static_node1.get("rotation").is_none());
    assert!(static_node1.get("scale").is_none());

    assert!(static_node2.get("matrix").is_none());
    assert_eq!(static_node2["translation"], json!([1.0, 0.0, 0.0]));
    assert_eq!(static_node2["rotation"], json!([0.0, 0.0, 0.0, 1.0]));
    assert_eq!(static_node2["scale"], json!([1.0, 1.0, 1.0]));
}

#[test]
fn test_adds_scene_defaults() {
    let gltf = json!({
        "scenes": [
            {
                "nodes": [0]
            }
        ]
    });

    let gltf_with_defaults = add_defaults(gltf);
    assert_eq!(gltf_with_defaults["scene"], 0);
}
