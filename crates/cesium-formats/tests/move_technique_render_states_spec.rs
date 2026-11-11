use cesium_formats::gltf_pipeline::move_technique_render_states;
use serde_json::json;

fn create_base_gltf() -> serde_json::Value {
    json!({
        "programs": {
            "program_0": {
                "attributes": ["a_normal", "a_position", "a_texcoord0"],
                "fragmentShader": "BoxTextured0FS",
                "vertexShader": "BoxTextured0VS"
            }
        },
        "shaders": {
            "BoxTextured0FS": {
                "type": 35632, // FRAGMENT_SHADER
                "uri": "BoxTextured0FS.glsl"
            },
            "BoxTextured0VS": {
                "type": 35633, // VERTEX_SHADER
                "uri": "BoxTextured0VS.glsl"
            }
        },
        "techniques": {
            "technique0": {
                "states": {
                    "enable": [
                        2929, // DEPTH_TEST
                        3042, // BLEND
                        2884  // CULL_FACE
                    ]
                }
            }
        },
        "materials": [
            {
                "technique": "technique0"
            }
        ]
    })
}

#[test]
fn test_sets_material_double_sided_if_cull_face_not_enabled() {
    let base_gltf = create_base_gltf();
    let updated = move_technique_render_states(base_gltf);
    assert!(updated["materials"][0].get("doubleSided").is_none());

    let mut gltf_double_sided = create_base_gltf();
    gltf_double_sided["techniques"]["technique0"]["states"] = json!({
        "enable": [2929, 3042] // DEPTH_TEST, BLEND (no CULL_FACE)
    });
    let updated_ds = move_technique_render_states(gltf_double_sided);
    assert_eq!(updated_ds["materials"][0]["doubleSided"], true);
}

#[test]
fn test_sets_alpha_mode_and_moves_technique_render_state_blending_functions_to_khr_blend() {
    let mut gltf = create_base_gltf();
    gltf["techniques"]["technique0"]["states"] = json!({
        "enable": [2929, 3042, 2884],
        "functions": {
            "blendEquationSeparate": [32774, 32774], // FUNC_ADD, FUNC_ADD
            "blendFuncSeparate": [1, 771, 1, 771]   // ONE, ONE_MINUS_SRC_ALPHA, ONE, ONE_MINUS_SRC_ALPHA
        }
    });

    let updated = move_technique_render_states(gltf);
    let extensions_used = updated["extensionsUsed"].as_array().unwrap();
    assert!(extensions_used.iter().any(|v| v == "KHR_blend"));
    assert!(updated.get("extensionsRequired").is_none());

    let material = &updated["materials"][0];
    assert_eq!(material["alphaMode"], "BLEND");
    let blending = &material["extensions"]["KHR_blend"];
    assert_eq!(blending["blendEquation"], json!([32774, 32774]));
    assert_eq!(blending["blendFactors"], json!([1, 771, 1, 771]));
}

#[test]
fn test_provides_defaults_for_extension_properties_if_not_provided() {
    let mut gltf1 = create_base_gltf();
    gltf1["techniques"]["technique0"]["states"] = json!({
        "enable": [2929, 3042, 2884],
        "functions": {
            "blendFuncSeparate": [1, 771, 1, 771]
        }
    });
    let updated1 = move_technique_render_states(gltf1);
    let blending1 = &updated1["materials"][0]["extensions"]["KHR_blend"];
    assert_eq!(blending1["blendEquation"], json!([32774, 32774]));
    assert_eq!(blending1["blendFactors"], json!([1, 771, 1, 771]));

    let mut gltf2 = create_base_gltf();
    gltf2["techniques"]["technique0"]["states"] = json!({
        "enable": [2929, 3042, 2884],
        "functions": {
            "blendEquationSeparate": [32774, 32774]
        }
    });
    let updated2 = move_technique_render_states(gltf2);
    let blending2 = &updated2["materials"][0]["extensions"]["KHR_blend"];
    assert_eq!(blending2["blendEquation"], json!([32774, 32774]));
    assert_eq!(blending2["blendFactors"], json!([1, 0, 1, 0]));
}

#[test]
fn test_falls_back_to_default_blending_factors_if_unsupported_factor_is_found() {
    let mut gltf = create_base_gltf();
    gltf["techniques"]["technique0"]["states"] = json!({
        "enable": [2929, 3042, 2884],
        "functions": {
            "blendFuncSeparate": [776, 771, 776, 771] // 776: SRC_ALPHA_SATURATE is unsupported in WebGL for dst
        }
    });

    let updated = move_technique_render_states(gltf);
    let blending = &updated["materials"][0]["extensions"]["KHR_blend"];
    assert_eq!(blending["blendFactors"], json!([1, 0, 1, 0]));
}

#[test]
fn test_does_not_set_alpha_mode_or_add_khr_blend_if_no_blending_is_found() {
    let mut gltf = create_base_gltf();
    gltf["techniques"]["technique0"]["states"]["enable"] = json!([2929, 2884]);

    let updated = move_technique_render_states(gltf);
    assert!(updated.get("extensionsUsed").is_none());
    assert!(updated.get("extensionsRequired").is_none());

    let material = &updated["materials"][0];
    assert!(material.get("alphaMode").is_none());
    assert!(material.get("extensions").is_none());
}
