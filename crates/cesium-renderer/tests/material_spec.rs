use cesium_math::DeveloperError;
use cesium_renderer::{Fabric, Material, MaterialOptions};
use std::collections::HashMap;

#[test]
fn test_material_type() {
    let material = Material::from_options(MaterialOptions {
        strict: Some(true),
        fabric: Some(Fabric {
            r#type: Some("Color".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(material.material_type, "Color");
}

#[test]
fn test_opaque_and_translucent() {
    let m1 = Material::from_options(MaterialOptions {
        translucent: Some(true),
        strict: Some(true),
        fabric: Some(Fabric {
            r#type: Some("Color".to_string()),
            ..Default::default()
        }),
    })
    .unwrap();
    assert!(m1.is_translucent());

    let m2 = Material::from_options(MaterialOptions {
        translucent: Some(false),
        strict: Some(true),
        fabric: Some(Fabric {
            r#type: Some("Color".to_string()),
            ..Default::default()
        }),
    })
    .unwrap();
    assert!(!m2.is_translucent());
}

#[test]
fn test_creates_new_material_type_and_builds_off_it() {
    let m1 = Material::from_options(MaterialOptions {
        strict: Some(true),
        fabric: Some(Fabric {
            r#type: Some("New".to_string()),
            components: Some(HashMap::from([(
                "diffuse".to_string(),
                "vec3(0.0, 0.0, 0.0)".to_string(),
            )])),
            ..Default::default()
        }),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(m1.material_type, "New");

    let m2 = Material::from_options(MaterialOptions {
        strict: Some(true),
        fabric: Some(Fabric {
            materials: Some(HashMap::from([(
                "first".to_string(),
                Fabric {
                    r#type: Some("New".to_string()),
                    ..Default::default()
                },
            )])),
            components: Some(HashMap::from([(
                "diffuse".to_string(),
                "first.diffuse".to_string(),
            )])),
            ..Default::default()
        }),
        ..Default::default()
    })
    .unwrap();
    assert!(m2.materials.contains_key("first"));
    assert!(m2.shader_source.contains("first.diffuse"));
}

#[test]
fn test_creates_material_inside_material_inside_material() {
    let m = Material::from_options(MaterialOptions {
        strict: Some(true),
        fabric: Some(Fabric {
            materials: Some(HashMap::from([(
                "first".to_string(),
                Fabric {
                    materials: Some(HashMap::from([(
                        "second".to_string(),
                        Fabric {
                            components: Some(HashMap::from([(
                                "diffuse".to_string(),
                                "vec3(0.0, 0.0, 0.0)".to_string(),
                            )])),
                            ..Default::default()
                        },
                    )])),
                    components: Some(HashMap::from([(
                        "diffuse".to_string(),
                        "second.diffuse".to_string(),
                    )])),
                    ..Default::default()
                },
            )])),
            components: Some(HashMap::from([(
                "diffuse".to_string(),
                "first.diffuse".to_string(),
            )])),
            ..Default::default()
        }),
        ..Default::default()
    })
    .unwrap();

    assert!(m.materials.contains_key("first"));
    let first = &m.materials["first"];
    assert!(first.materials.contains_key("second"));
}

#[test]
fn test_uniform_types_bool_and_matrix() {
    let m_bool = Material::from_options(MaterialOptions {
        strict: Some(true),
        fabric: Some(Fabric {
            uniforms: Some(HashMap::from([("value".to_string(), serde_json::json!(true))])),
            components: Some(HashMap::from([(
                "diffuse".to_string(),
                "float(value) * vec3(1.0)".to_string(),
            )])),
            ..Default::default()
        }),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(m_bool.uniforms.get("value"), Some(&serde_json::json!(true)));

    let m_mat2 = Material::from_options(MaterialOptions {
        strict: Some(true),
        fabric: Some(Fabric {
            uniforms: Some(HashMap::from([(
                "value".to_string(),
                serde_json::json!([0.5, 0.5, 0.5, 0.5]),
            )])),
            components: Some(HashMap::from([(
                "diffuse".to_string(),
                "vec3(value[0][0], value[0][1], value[1][0])".to_string(),
            )])),
            ..Default::default()
        }),
        ..Default::default()
    })
    .unwrap();
    assert!(m_mat2.shader_source.contains("value"));

    let m_mat4 = Material::from_options(MaterialOptions {
        strict: Some(true),
        fabric: Some(Fabric {
            uniforms: Some(HashMap::from([(
                "value".to_string(),
                serde_json::json!([
                    0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5
                ]),
            )])),
            components: Some(HashMap::from([(
                "diffuse".to_string(),
                "vec3(value[0][0], value[0][1], value[1][0])".to_string(),
            )])),
            ..Default::default()
        }),
        ..Default::default()
    })
    .unwrap();
    assert!(m_mat4.shader_source.contains("value"));
}

#[test]
fn test_from_type_and_overrides() {
    let m = Material::from_type(Material::COLOR_TYPE).unwrap();
    assert_eq!(m.material_type, "Color");

    let m_override = Material::from_type_with_uniforms(
        Material::COLOR_TYPE,
        HashMap::from([("color".to_string(), serde_json::json!([0.0, 1.0, 0.0, 1.0]))]),
    )
    .unwrap();
    assert_eq!(
        m_override.uniforms.get("color"),
        Some(&serde_json::json!([0.0, 1.0, 0.0, 1.0]))
    );
    assert!(!m_override.is_translucent()); // alpha is 1.0 -> opaque
}

#[test]
fn test_sub_materials_of_same_type() {
    let m = Material::from_options(MaterialOptions {
        strict: Some(false),
        fabric: Some(Fabric {
            materials: Some(HashMap::from([
                (
                    "color1".to_string(),
                    Fabric {
                        r#type: Some("Color".to_string()),
                        uniforms: Some(HashMap::from([(
                            "color".to_string(),
                            serde_json::json!([0.0, 1.0, 0.0, 1.0]),
                        )])),
                        ..Default::default()
                    },
                ),
                (
                    "color2".to_string(),
                    Fabric {
                        r#type: Some("Color".to_string()),
                        uniforms: Some(HashMap::from([(
                            "color".to_string(),
                            serde_json::json!([0.0, 0.0, 1.0, 1.0]),
                        )])),
                        ..Default::default()
                    },
                ),
            ])),
            components: Some(HashMap::from([(
                "diffuse".to_string(),
                "color1.diffuse + color2.diffuse".to_string(),
            )])),
            ..Default::default()
        }),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(m.materials.len(), 2);
    assert!(m.materials.contains_key("color1"));
    assert!(m.materials.contains_key("color2"));
}

#[test]
fn test_from_type_async() {
    pollster::block_on(async {
        let m = Material::from_type_async(Material::COLOR_TYPE)
            .await
            .unwrap();
        assert_eq!(m.material_type, "Color");

        let err = Material::from_type_async_with_uniforms(
            Material::DIFFUSE_MAP_TYPE,
            HashMap::from([("image".to_string(), serde_json::json!("i_dont_exist.png"))]),
        )
        .await;
        assert!(err.is_err());
        assert_eq!(err.unwrap_err().message, "Image loading failed");
    });
}

#[test]
fn test_throws_with_source_and_components_in_same_template() {
    let res = Material::from_options(MaterialOptions {
        strict: Some(true),
        fabric: Some(Fabric {
            components: Some(HashMap::from([(
                "diffuse".to_string(),
                "vec3(0.0, 0.0, 0.0)".to_string(),
            )])),
            source: Some("czm_material czm_getMaterial(czm_materialInput materialInput) { return czm_getDefaultMaterial(materialInput); }".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    });
    assert!(res.is_err());

    let res2 = Material::from_options(MaterialOptions {
        strict: Some(true),
        fabric: Some(Fabric {
            r#type: Some("DiffuseMap".to_string()),
            components: Some(HashMap::from([(
                "diffuse".to_string(),
                "vec3(0.0, 0.0, 0.0)".to_string(),
            )])),
            ..Default::default()
        }),
        ..Default::default()
    });
    assert!(res2.is_err());
}

#[test]
fn test_throws_with_duplicate_names_in_materials_and_uniforms() {
    let res = Material::from_options(MaterialOptions {
        strict: Some(false),
        fabric: Some(Fabric {
            uniforms: Some(HashMap::from([
                ("first".to_string(), serde_json::json!(0.0)),
                ("second".to_string(), serde_json::json!(0.0)),
            ])),
            materials: Some(HashMap::from([(
                "second".to_string(),
                Fabric::default(),
            )])),
            ..Default::default()
        }),
        ..Default::default()
    });
    assert!(res.is_err());
}

#[test]
fn test_throws_with_invalid_template_property() {
    let json_str = r#"{
        "type": "New",
        "invalid": 3.0
    }"#;
    let fabric: Fabric = serde_json::from_str(json_str).unwrap();
    let res = Material::from_options(MaterialOptions {
        strict: Some(true),
        fabric: Some(fabric),
        ..Default::default()
    });
    assert!(res.is_err());
}

#[test]
fn test_throws_with_invalid_component_type() {
    let res = Material::from_options(MaterialOptions {
        strict: Some(true),
        fabric: Some(Fabric {
            components: Some(HashMap::from([(
                "difuse".to_string(), // typo
                "vec3(0.0, 0.0, 0.0)".to_string(),
            )])),
            ..Default::default()
        }),
        ..Default::default()
    });
    assert!(res.is_err());
}

#[test]
fn test_throws_with_invalid_uniform_type() {
    // 5-component array
    let res1 = Material::from_options(MaterialOptions {
        strict: Some(true),
        fabric: Some(Fabric {
            uniforms: Some(HashMap::from([(
                "value".to_string(),
                serde_json::json!([0.0, 0.0, 0.0, 0.0, 0.0]),
            )])),
            ..Default::default()
        }),
        ..Default::default()
    });
    assert!(res1.is_err());

    // 5-property object (not vec2, vec3, vec4 or cube map)
    let res2 = Material::from_options(MaterialOptions {
        strict: Some(true),
        fabric: Some(Fabric {
            uniforms: Some(HashMap::from([(
                "value".to_string(),
                serde_json::json!({"x": 0.0, "y": 0.0, "z": 0.0, "w": 0.0, "t": 0.0}),
            )])),
            ..Default::default()
        }),
        ..Default::default()
    });
    assert!(res2.is_err());
}

#[test]
fn test_throws_with_unused_uniform_in_strict_mode() {
    // Throws when strict is true
    let res = Material::from_options(MaterialOptions {
        strict: Some(true),
        fabric: Some(Fabric {
            uniforms: Some(HashMap::from([(
                "first".to_string(),
                serde_json::json!({"x": 0.0, "y": 0.0, "z": 0.0}),
            )])),
            components: Some(HashMap::from([(
                "diffuse".to_string(),
                "vec3(1.0)".to_string(), // doesn't use 'first'
            )])),
            ..Default::default()
        }),
        ..Default::default()
    });
    assert!(res.is_err());

    // When strict is false, unused uniforms are ignored
    let ok = Material::from_options(MaterialOptions {
        strict: Some(false),
        fabric: Some(Fabric {
            uniforms: Some(HashMap::from([(
                "first".to_string(),
                serde_json::json!({"x": 0.0, "y": 0.0, "z": 0.0}),
            )])),
            components: Some(HashMap::from([(
                "diffuse".to_string(),
                "vec3(1.0)".to_string(),
            )])),
            ..Default::default()
        }),
        ..Default::default()
    });
    assert!(ok.is_ok());
}

#[test]
fn test_throws_with_unused_material_in_strict_mode() {
    let res = Material::from_options(MaterialOptions {
        strict: Some(true),
        fabric: Some(Fabric {
            materials: Some(HashMap::from([(
                "first".to_string(),
                Fabric {
                    r#type: Some("DiffuseMap".to_string()),
                    ..Default::default()
                },
            )])),
            components: Some(HashMap::from([(
                "diffuse".to_string(),
                "vec3(1.0)".to_string(), // doesn't use 'first'
            )])),
            ..Default::default()
        }),
        ..Default::default()
    });
    assert!(res.is_err());

    // If strict is false, unused sub-materials are ignored
    let ok = Material::from_options(MaterialOptions {
        strict: Some(false),
        fabric: Some(Fabric {
            materials: Some(HashMap::from([(
                "first".to_string(),
                Fabric {
                    r#type: Some("DiffuseMap".to_string()),
                    ..Default::default()
                },
            )])),
            components: Some(HashMap::from([(
                "diffuse".to_string(),
                "vec3(1.0)".to_string(),
            )])),
            ..Default::default()
        }),
        ..Default::default()
    });
    assert!(ok.is_ok());
}

#[test]
fn test_throws_with_invalid_type_in_from_type() {
    let res = Material::from_type("NonExistentType");
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        DeveloperError::new("material with type 'NonExistentType' does not exist.")
    );
}

#[test]
fn test_destroys_material_and_sub_materials() {
    let mut material = Material::from_options(MaterialOptions {
        strict: Some(true),
        fabric: Some(Fabric {
            materials: Some(HashMap::from([(
                "diffuseMap".to_string(),
                Fabric {
                    r#type: Some("DiffuseMap".to_string()),
                    ..Default::default()
                },
            )])),
            uniforms: Some(HashMap::from([(
                "value".to_string(),
                serde_json::json!({"x": 0.0, "y": 0.0, "z": 0.0}),
            )])),
            components: Some(HashMap::from([(
                "diffuse".to_string(),
                "value + diffuseMap.diffuse".to_string(),
            )])),
            ..Default::default()
        }),
        ..Default::default()
    })
    .unwrap();

    assert!(!material.is_destroyed());
    assert!(!material.materials["diffuseMap"].is_destroyed());

    material.destroy();

    assert!(material.is_destroyed());
    assert!(material.materials["diffuseMap"].is_destroyed());
}
