use cesium_formats::gltf_pipeline::move_techniques_to_extension;
use serde_json::json;

#[test]
fn test_moves_techniques_shaders_and_programs_to_extension() {
    let gltf = json!({
        "programs": {
            "program_0": {
                "attributes": ["a_normal", "a_position", "a_texcoord0"],
                "fragmentShader": "BoxTextured0FS",
                "vertexShader": "BoxTextured0VS"
            },
            "program_1": {
                "attributes": ["a_normal", "a_position", "a_texcoord0"],
                "fragmentShader": "BoxTextured1FS",
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
            },
            "BoxTextured1FS": {
                "type": 35632, // FRAGMENT_SHADER
                "uri": "BoxTextured1FS.glsl"
            }
        },
        "techniques": {
            "technique0": {
                "attributes": {
                    "a_normal": "normal",
                    "a_position": "position",
                    "a_texcoord0": "texcoord0"
                },
                "parameters": {
                    "diffuse": {
                        "type": 35678 // SAMPLER_2D
                    },
                    "modelViewMatrix": {
                        "semantic": "MODELVIEW",
                        "type": 35676 // FLOAT_MAT4
                    },
                    "normal": {
                        "semantic": "NORMAL",
                        "type": 35665 // FLOAT_VEC3
                    },
                    "normalMatrix": {
                        "semantic": "MODELVIEWINVERSETRANSPOSE",
                        "type": 35675 // FLOAT_MAT3
                    },
                    "position": {
                        "semantic": "POSITION",
                        "type": 35665 // FLOAT_VEC3
                    },
                    "projectionMatrix": {
                        "semantic": "PROJECTION",
                        "type": 35676 // FLOAT_MAT4
                    },
                    "shininess": {
                        "type": 5126 // FLOAT
                    },
                    "specular": {
                        "type": 35666 // FLOAT_VEC4
                    },
                    "texcoord0": {
                        "semantic": "TEXCOORD_0",
                        "type": 35664 // FLOAT_VEC2
                    }
                },
                "program": "program_0",
                "states": {
                    "enable": [2929, 2884] // DEPTH_TEST, CULL_FACE
                },
                "uniforms": {
                    "u_diffuse": "diffuse",
                    "u_modelViewMatrix": "modelViewMatrix",
                    "u_normalMatrix": "normalMatrix",
                    "u_projectionMatrix": "projectionMatrix",
                    "u_shininess": "shininess",
                    "u_specular": "specular"
                }
            },
            "technique1": {
                "program": "program_1"
            },
            "technique2": {
                "parameters": {
                    "diffuse": {
                        "type": 35666 // FLOAT_VEC4
                    }
                },
                "program": "program_0",
                "uniforms": {
                    "u_diffuse": "diffuse"
                }
            }
        },
        "materials": [
            {
                "name": "Texture",
                "technique": "technique0",
                "values": {
                    "diffuse": "texture_Image0001",
                    "shininess": 256,
                    "specular": [0.2, 0.2, 0.2, 1.0]
                }
            },
            {
                "name": "Color",
                "technique": "technique2",
                "values": {
                    "diffuse": [0.2, 0.2, 0.2, 1.0]
                }
            }
        ]
    });

    let gltf_with_techniques = move_techniques_to_extension(gltf);
    assert!(gltf_with_techniques.get("extensions").is_some());
    let techniques = &gltf_with_techniques["extensions"]["KHR_techniques_webgl"];
    assert!(techniques.is_object());
    assert_eq!(techniques["techniques"].as_array().unwrap().len(), 3);

    let technique0 = &techniques["techniques"][0];
    let attributes = &technique0["attributes"];
    assert_eq!(attributes["a_position"]["semantic"], "POSITION");

    let uniforms = &technique0["uniforms"];
    assert_eq!(uniforms["u_modelViewMatrix"]["semantic"], "MODELVIEW");
    assert_eq!(uniforms["u_modelViewMatrix"]["type"], 35676);

    assert_eq!(technique0["program"], 0);
    assert!(technique0.get("parameters").is_none());
    assert!(technique0.get("states").is_none());

    assert_eq!(techniques["programs"].as_array().unwrap().len(), 2);
    let program0 = &techniques["programs"][0];
    let fs_idx = program0["fragmentShader"].as_u64().unwrap() as usize;
    let vs_idx = program0["vertexShader"].as_u64().unwrap() as usize;

    assert_eq!(techniques["shaders"].as_array().unwrap().len(), 3);
    assert_eq!(techniques["shaders"][fs_idx]["type"], 35632);
    assert_eq!(techniques["shaders"][vs_idx]["type"], 35633);

    assert!(gltf_with_techniques.get("techniques").is_none());
    assert!(gltf_with_techniques.get("programs").is_none());
    assert!(gltf_with_techniques.get("shaders").is_none());

    let material0 = &gltf_with_techniques["materials"][0];
    assert!(material0.get("extensions").is_some());
    let mat_tech = &material0["extensions"]["KHR_techniques_webgl"];
    assert_eq!(mat_tech["technique"], 0);
    assert_eq!(mat_tech["values"]["u_shininess"], 256);
    assert_eq!(mat_tech["values"]["u_diffuse"], "texture_Image0001");

    assert!(material0.get("technique").is_none());
    assert!(material0.get("values").is_none());

    let technique1 = &techniques["techniques"][1];
    let prog1_idx = technique1["program"].as_u64().unwrap() as usize;
    let program1 = &techniques["programs"][prog1_idx];
    assert_eq!(program1["vertexShader"], program0["vertexShader"]);
    assert_ne!(program1["fragmentShader"], program0["fragmentShader"]);

    let technique2 = &techniques["techniques"][2];
    assert_eq!(technique2["program"], 0);
    assert_eq!(technique2["uniforms"]["u_diffuse"]["type"], 35666);
    assert_eq!(
        gltf_with_techniques["materials"][1]["extensions"]["KHR_techniques_webgl"]["values"]["u_diffuse"],
        json!([0.2, 0.2, 0.2, 1.0])
    );
}
