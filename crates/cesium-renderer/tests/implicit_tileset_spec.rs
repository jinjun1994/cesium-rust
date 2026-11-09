use std::f64::consts::PI;
use cesium_renderer::implicit_subdivision_scheme::ImplicitSubdivisionScheme;
use cesium_renderer::implicit_tileset::ImplicitTileset;
use serde_json::json;

#[test]
fn gathers_information_from_both_tile_json_and_extension() {
    let content_uri_pattern = "https://example.com/{level}/{x}/{y}.b3dm";
    let subtree_uri_pattern = "https://example.com/{level}/{x}/{y}.subtree";

    let tile_json = json!({
        "geometricError": 500.0,
        "refine": "ADD",
        "boundingVolume": {
            "region": [0.0, 0.0, PI / 24.0, PI / 24.0, 0.0, 1000.0]
        },
        "content": {
            "uri": content_uri_pattern,
            "extras": {
                "author": "Cesium"
            }
        },
        "implicitTiling": {
            "subdivisionScheme": "QUADTREE",
            "subtreeLevels": 3,
            "availableLevels": 5,
            "subtrees": {
                "uri": subtree_uri_pattern
            }
        },
        "extras": {
            "creationDate": "2021-02-22"
        }
    });

    let implicit_tileset = ImplicitTileset::new(
        Some("https://example.com/tileset.json".to_string()),
        &tile_json,
        None,
    ).unwrap();

    assert!(implicit_tileset.metadata_schema.is_none());
    assert_eq!(implicit_tileset.subtree_levels, 3);
    assert_eq!(implicit_tileset.available_levels, 5);
    assert_eq!(implicit_tileset.subdivision_scheme, ImplicitSubdivisionScheme::Quadtree);
    assert_eq!(implicit_tileset.bounding_volume, tile_json["boundingVolume"]);
    assert_eq!(implicit_tileset.refine, "ADD");
    assert_eq!(implicit_tileset.geometric_error, 500.0);
    assert_eq!(implicit_tileset.content_uri_templates, vec![content_uri_pattern]);
    assert_eq!(implicit_tileset.subtree_uri_template, subtree_uri_pattern);
}

#[test]
fn stores_a_template_of_the_tile_json_structure() {
    let tile_json = json!({
        "geometricError": 500.0,
        "refine": "ADD",
        "boundingVolume": {
            "region": [0.0, 0.0, PI / 24.0, PI / 24.0, 0.0, 1000.0]
        },
        "content": {
            "uri": "https://example.com/{level}/{x}/{y}.b3dm"
        },
        "implicitTiling": {
            "subdivisionScheme": "QUADTREE",
            "subtreeLevels": 3,
            "availableLevels": 5,
            "subtrees": {
                "uri": "https://example.com/{level}/{x}/{y}.subtree"
            }
        },
        "extras": {
            "creationDate": "2021-02-22"
        }
    });

    let implicit_tileset = ImplicitTileset::new(None, &tile_json, None).unwrap();
    let expected = json!({
        "geometricError": 500.0,
        "refine": "ADD",
        "boundingVolume": {
            "region": [0.0, 0.0, PI / 24.0, PI / 24.0, 0.0, 1000.0]
        },
        "extras": {
            "creationDate": "2021-02-22"
        }
    });
    assert_eq!(implicit_tileset.tile_header, expected);
}

#[test]
fn tile_header_stores_additional_extensions() {
    let tile_json = json!({
        "geometricError": 500.0,
        "boundingVolume": {
            "region": [0.0, 0.0, PI / 24.0, PI / 24.0, 0.0, 1000.0]
        },
        "implicitTiling": {
            "subdivisionScheme": "QUADTREE",
            "subtreeLevels": 3,
            "subtrees": {
                "uri": "https://example.com/{level}/{x}/{y}.subtree"
            }
        },
        "extensions": {
            "3DTILES_extension": {}
        }
    });

    let implicit_tileset = ImplicitTileset::new(None, &tile_json, None).unwrap();
    assert_eq!(
        implicit_tileset.tile_header["extensions"]["3DTILES_extension"],
        json!({})
    );
}

#[test]
fn stores_content_headers_and_allows_undefined_content() {
    let content_json = json!({
        "uri": "https://example.com/{level}/{x}/{y}.b3dm",
        "extras": { "author": "Cesium" }
    });
    let tile_json = json!({
        "geometricError": 500.0,
        "boundingVolume": {
            "region": [0.0, 0.0, PI / 24.0, PI / 24.0, 0.0, 1000.0]
        },
        "content": content_json,
        "implicitTiling": {
            "subdivisionScheme": "QUADTREE",
            "subtreeLevels": 3,
            "subtrees": {
                "uri": "https://example.com/{level}/{x}/{y}.subtree"
            }
        }
    });

    let implicit_tileset = ImplicitTileset::new(None, &tile_json, None).unwrap();
    assert_eq!(implicit_tileset.content_headers[0], content_json);
    assert_eq!(implicit_tileset.content_count, 1);

    // Undefined content
    let mut no_content = tile_json.clone();
    no_content.as_object_mut().unwrap().remove("content");
    let implicit_tileset_no_content = ImplicitTileset::new(None, &no_content, None).unwrap();
    assert!(implicit_tileset_no_content.content_uri_templates.is_empty());
    assert_eq!(implicit_tileset_no_content.content_count, 0);
}

#[test]
fn accepts_bounding_volume_s2_and_rejects_spheres() {
    let s2_tile = json!({
        "geometricError": 500.0,
        "boundingVolume": {
            "extensions": {
                "3DTILES_bounding_volume_S2": {
                    "token": "1",
                    "minimumHeight": 0,
                    "maximumHeight": 100
                }
            }
        },
        "implicitTiling": {
            "subdivisionScheme": "QUADTREE",
            "subtreeLevels": 3,
            "subtrees": {
                "uri": "https://example.com/{level}/{x}/{y}.subtree"
            }
        }
    });

    let implicit_tileset = ImplicitTileset::new(None, &s2_tile, None).unwrap();
    let s2 = &implicit_tileset.bounding_volume["extensions"]["3DTILES_bounding_volume_S2"];
    assert_eq!(s2["token"], "1");
    assert_eq!(s2["minimumHeight"], 0);
    assert_eq!(s2["maximumHeight"], 100);

    // Reject sphere
    let sphere_tile = json!({
        "geometricError": 500.0,
        "boundingVolume": {
            "sphere": [0, 0, 0, 100]
        },
        "implicitTiling": {
            "subdivisionScheme": "QUADTREE",
            "subtreeLevels": 3,
            "subtrees": {
                "uri": "https://example.com/{level}/{x}/{y}.subtree"
            }
        }
    });
    assert!(ImplicitTileset::new(None, &sphere_tile, None).is_err());
}

#[test]
fn supports_legacy_3dtiles_implicit_tiling_and_maximum_level() {
    let legacy_json = json!({
        "geometricError": 500.0,
        "refine": "ADD",
        "boundingVolume": {
            "region": [0.0, 0.0, PI / 24.0, PI / 24.0, 0.0, 1000.0]
        },
        "extensions": {
            "3DTILES_implicit_tiling": {
                "subdivisionScheme": "QUADTREE",
                "subtreeLevels": 3,
                "maximumLevel": 4,
                "subtrees": {
                    "uri": "https://example.com/{level}/{x}/{y}.subtree"
                }
            }
        }
    });

    let implicit_tileset = ImplicitTileset::new(None, &legacy_json, None).unwrap();
    assert_eq!(implicit_tileset.available_levels, 5); // maximumLevel 4 + 1
    assert_eq!(implicit_tileset.subdivision_scheme, ImplicitSubdivisionScheme::Quadtree);
    assert!(implicit_tileset.tile_header.get("extensions").is_none());
}

#[test]
fn supports_multiple_contents_standard_and_legacy() {
    let multi_content = json!({
        "geometricError": 500.0,
        "boundingVolume": {
            "region": [0.0, 0.0, PI / 24.0, PI / 24.0, 0.0, 1000.0]
        },
        "implicitTiling": {
            "subdivisionScheme": "QUADTREE",
            "subtreeLevels": 3,
            "subtrees": {
                "uri": "https://example.com/{level}/{x}/{y}.subtree"
            }
        },
        "contents": [
            { "uri": "https://example.com/{level}/{x}/{y}.b3dm" },
            { "uri": "https://example.com/{level}/{x}/{y}.pnts" },
            { "uri": "https://example.com/{level}/{x}/{y}.gltf" }
        ]
    });

    let ts = ImplicitTileset::new(None, &multi_content, None).unwrap();
    assert_eq!(ts.content_uri_templates.len(), 3);
    assert_eq!(ts.content_headers.len(), 3);
    assert_eq!(ts.content_count, 3);

    let legacy_multi_content = json!({
        "geometricError": 500.0,
        "boundingVolume": {
            "region": [0.0, 0.0, PI / 24.0, PI / 24.0, 0.0, 1000.0]
        },
        "extensions": {
            "3DTILES_implicit_tiling": {
                "subdivisionScheme": "QUADTREE",
                "subtreeLevels": 3,
                "subtrees": {
                    "uri": "https://example.com/{level}/{x}/{y}.subtree"
                }
            },
            "3DTILES_multiple_contents": {
                "contents": [
                    { "uri": "https://example.com/{level}/{x}/{y}.b3dm" },
                    { "uri": "https://example.com/{level}/{x}/{y}.pnts" }
                ]
            }
        }
    });

    let legacy_ts = ImplicitTileset::new(None, &legacy_multi_content, None).unwrap();
    assert_eq!(legacy_ts.content_uri_templates.len(), 2);
    assert_eq!(legacy_ts.content_headers.len(), 2);
    assert!(legacy_ts.tile_header.get("extensions").is_none());
}

#[test]
fn stores_metadata_schema() {
    let schema_json = json!({
        "classes": {
            "tile": {
                "properties": {
                    "buildingCount": {
                        "type": "SCALAR",
                        "componentType": "UINT16"
                    }
                }
            }
        }
    });

    let tile_json = json!({
        "geometricError": 500.0,
        "boundingVolume": {
            "region": [0.0, 0.0, PI / 24.0, PI / 24.0, 0.0, 1000.0]
        },
        "implicitTiling": {
            "subdivisionScheme": "QUADTREE",
            "subtreeLevels": 3,
            "subtrees": {
                "uri": "https://example.com/{level}/{x}/{y}.subtree"
            }
        }
    });

    let ts = ImplicitTileset::new(None, &tile_json, Some(schema_json.clone())).unwrap();
    assert_eq!(ts.metadata_schema, Some(schema_json));
}
