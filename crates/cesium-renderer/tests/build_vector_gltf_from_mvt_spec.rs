use cesium_renderer::{
    build_vector_gltf_from_mvt, DecodedVectorTile, TileCoordinates, VectorTileFeature,
    VectorTileLayer,
};
use serde_json::{json, Value};

fn parse_glb_json(glb: &[u8]) -> Value {
    // Header is 12 bytes
    // JSON chunk length at offset 12..16 (u32 little endian)
    let json_len = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
    // JSON chunk starts at byte 20
    let json_bytes = &glb[20..20 + json_len];
    serde_json::from_slice(json_bytes).expect("valid json in glb")
}

#[test]
fn test_returns_undefined_for_empty_decoded_tile() {
    let tile_coordinates = TileCoordinates {
        tile_x: 0,
        tile_y: 0,
        tile_z: 0,
    };

    let decoded = DecodedVectorTile {
        layers: vec![VectorTileLayer {
            name: Some("empty".to_string()),
            extent: 4096.0,
            features: vec![],
        }],
    };

    let gltf = build_vector_gltf_from_mvt(&decoded, tile_coordinates);
    assert!(gltf.is_none());
}

#[test]
fn test_builds_vector_primitives_for_point_line_and_polygon_features() {
    let tile_coordinates = TileCoordinates {
        tile_x: 0,
        tile_y: 0,
        tile_z: 0,
    };

    let decoded = DecodedVectorTile {
        layers: vec![VectorTileLayer {
            name: Some("testlayer".to_string()),
            extent: 4096.0,
            features: vec![
                VectorTileFeature {
                    feature_type: "Point".to_string(),
                    geometry: json!([{ "x": 2000, "y": 2000 }]),
                    properties: Some(serde_json::Map::new()),
                },
                VectorTileFeature {
                    feature_type: "LineString".to_string(),
                    geometry: json!([
                        [
                            { "x": 1600, "y": 1600 },
                            { "x": 2200, "y": 1800 },
                            { "x": 2600, "y": 2100 }
                        ]
                    ]),
                    properties: Some(serde_json::Map::new()),
                },
                VectorTileFeature {
                    feature_type: "Polygon".to_string(),
                    geometry: json!([
                        [
                            { "x": 1200, "y": 1200 },
                            { "x": 2200, "y": 1200 },
                            { "x": 2200, "y": 2200 },
                            { "x": 1200, "y": 2200 },
                            { "x": 1200, "y": 1200 }
                        ]
                    ]),
                    properties: Some(serde_json::Map::new()),
                },
            ],
        }],
    };

    let glb = build_vector_gltf_from_mvt(&decoded, tile_coordinates).expect("built glb");
    let gltf = parse_glb_json(&glb);

    let extensions_used = gltf["extensionsUsed"].as_array().unwrap();
    let ext_names: Vec<&str> = extensions_used.iter().filter_map(|v| v.as_str()).collect();
    assert!(ext_names.contains(&"CESIUM_mesh_vector"));
    assert!(ext_names.contains(&"EXT_mesh_features"));
    assert!(ext_names.contains(&"EXT_structural_metadata"));
    assert!(gltf.get("extensionsRequired").is_none());

    let primitives = gltf["meshes"][0]["primitives"].as_array().unwrap();
    assert_eq!(primitives.len(), 3);

    let point_prim = primitives.iter().find(|p| p["mode"] == 0).unwrap();
    let line_prim = primitives.iter().find(|p| p["mode"] == 3).unwrap();
    let polygon_prim = primitives.iter().find(|p| p["mode"] == 4).unwrap();

    assert_eq!(
        point_prim["extensions"]["CESIUM_mesh_vector"]["vector"],
        true
    );
    assert_eq!(
        line_prim["extensions"]["CESIUM_mesh_vector"]["vector"],
        true
    );
    assert_eq!(
        polygon_prim["extensions"]["CESIUM_mesh_vector"]["vector"],
        true
    );

    assert!(polygon_prim["extensions"]["CESIUM_mesh_vector"]["polygonAttributeOffsets"].is_array());
    assert!(polygon_prim["extensions"]["CESIUM_mesh_vector"]["polygonIndicesOffsets"].is_array());

    assert!(point_prim["extensions"].get("EXT_mesh_features").is_some());
    assert!(line_prim["extensions"].get("EXT_mesh_features").is_some());
    assert!(polygon_prim["extensions"].get("EXT_mesh_features").is_some());
}

#[test]
fn test_encodes_feature_properties_into_ext_structural_metadata() {
    let tile_coordinates = TileCoordinates {
        tile_x: 0,
        tile_y: 0,
        tile_z: 0,
    };

    let mut props1 = serde_json::Map::new();
    props1.insert("name".to_string(), json!("Main St"));
    props1.insert("lanes".to_string(), json!(4.0));
    props1.insert("oneway".to_string(), json!(true));

    let mut props2 = serde_json::Map::new();
    props2.insert("name".to_string(), json!("Oak Ave"));
    props2.insert("lanes".to_string(), json!(2.0));
    props2.insert("oneway".to_string(), json!(false));

    let mut props3 = serde_json::Map::new();
    props3.insert("name".to_string(), json!("Tower"));
    props3.insert("height".to_string(), json!(120.5));

    let decoded = DecodedVectorTile {
        layers: vec![
            VectorTileLayer {
                name: Some("roads".to_string()),
                extent: 4096.0,
                features: vec![
                    VectorTileFeature {
                        feature_type: "LineString".to_string(),
                        geometry: json!([[{ "x": 100, "y": 100 }, { "x": 200, "y": 200 }]]),
                        properties: Some(props1),
                    },
                    VectorTileFeature {
                        feature_type: "LineString".to_string(),
                        geometry: json!([[{ "x": 300, "y": 300 }, { "x": 400, "y": 400 }]]),
                        properties: Some(props2),
                    },
                ],
            },
            VectorTileLayer {
                name: Some("buildings".to_string()),
                extent: 4096.0,
                features: vec![VectorTileFeature {
                    feature_type: "Polygon".to_string(),
                    geometry: json!([[{ "x": 500, "y": 500 }]]),
                    properties: Some(props3),
                }],
            },
        ],
    };

    let glb = build_vector_gltf_from_mvt(&decoded, tile_coordinates).expect("built glb");
    let gltf = parse_glb_json(&glb);

    let metadata = &gltf["extensions"]["EXT_structural_metadata"];
    let schema = &metadata["schema"];
    let class_props = &schema["classes"]["mvt_feature"]["properties"];

    assert_eq!(class_props["_layer"]["type"], "STRING");
    assert_eq!(class_props["name"]["type"], "STRING");
    assert_eq!(class_props["lanes"]["type"], "SCALAR");
    assert_eq!(class_props["lanes"]["componentType"], "FLOAT64");
    assert_eq!(class_props["oneway"]["type"], "BOOLEAN");
    assert_eq!(class_props["height"]["type"], "SCALAR");
    assert_eq!(class_props["height"]["componentType"], "FLOAT64");

    let table = &metadata["propertyTables"][0];
    assert_eq!(table["class"], "mvt_feature");
    assert_eq!(table["count"], 3);

    assert!(table["properties"]["_layer"]["values"].is_number());
    assert!(table["properties"]["_layer"]["stringOffsets"].is_number());
    assert!(table["properties"]["name"]["values"].is_number());
    assert!(table["properties"]["name"]["stringOffsets"].is_number());
    assert!(table["properties"]["lanes"]["values"].is_number());
    assert!(table["properties"]["oneway"]["values"].is_number());
    assert!(table["properties"]["height"]["values"].is_number());

    let primitives = gltf["meshes"][0]["primitives"].as_array().unwrap();
    for prim in primitives {
        assert_eq!(
            prim["extensions"]["EXT_mesh_features"]["featureIds"][0]["propertyTable"],
            0
        );
    }
}

#[test]
fn test_encodes_string_properties_with_binary_values_and_offsets() {
    let tile_coordinates = TileCoordinates {
        tile_x: 0,
        tile_y: 0,
        tile_z: 0,
    };

    let mut props1 = serde_json::Map::new();
    props1.insert("name".to_string(), json!("Hello"));

    let mut props2 = serde_json::Map::new();
    props2.insert("name".to_string(), json!("World"));

    let decoded = DecodedVectorTile {
        layers: vec![VectorTileLayer {
            name: Some("places".to_string()),
            extent: 4096.0,
            features: vec![
                VectorTileFeature {
                    feature_type: "Point".to_string(),
                    geometry: json!([{ "x": 1000, "y": 1000 }]),
                    properties: Some(props1),
                },
                VectorTileFeature {
                    feature_type: "Point".to_string(),
                    geometry: json!([{ "x": 2000, "y": 2000 }]),
                    properties: Some(props2),
                },
            ],
        }],
    };

    let glb = build_vector_gltf_from_mvt(&decoded, tile_coordinates).expect("built glb");
    let gltf = parse_glb_json(&glb);
    let metadata = &gltf["extensions"]["EXT_structural_metadata"];
    let table = &metadata["propertyTables"][0];

    assert!(table["properties"]["name"]["values"].is_number());
    assert!(table["properties"]["name"]["stringOffsets"].is_number());
    assert_eq!(table["properties"]["name"]["stringOffsetType"], "UINT32");

    assert!(table["properties"]["_layer"]["values"].is_number());
    assert!(table["properties"]["_layer"]["stringOffsets"].is_number());
}

#[test]
fn test_handles_features_with_no_properties_gracefully() {
    let tile_coordinates = TileCoordinates {
        tile_x: 0,
        tile_y: 0,
        tile_z: 0,
    };

    let decoded = DecodedVectorTile {
        layers: vec![VectorTileLayer {
            name: Some("minimal".to_string()),
            extent: 4096.0,
            features: vec![VectorTileFeature {
                feature_type: "Point".to_string(),
                geometry: json!([{ "x": 500, "y": 500 }]),
                properties: None,
            }],
        }],
    };

    let glb = build_vector_gltf_from_mvt(&decoded, tile_coordinates).expect("built glb");
    let gltf = parse_glb_json(&glb);
    let metadata = &gltf["extensions"]["EXT_structural_metadata"];
    assert!(metadata.is_object());
    assert!(metadata["schema"]["classes"]["mvt_feature"]["properties"]["_layer"].is_object());
}
