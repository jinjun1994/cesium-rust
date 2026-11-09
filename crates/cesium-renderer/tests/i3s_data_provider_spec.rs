use cesium_renderer::i3s_data_provider::{
    I3SDataProvider, I3SDataProviderOptions, I3SStatistics,
};
use serde_json::json;

#[test]
fn constructs_with_default_options() {
    let provider = I3SDataProvider::new(I3SDataProviderOptions::default());

    assert!(provider.name().is_none());
    assert!(provider.show());
    assert!(!provider.show_features());
    assert!(!provider.adjust_material_alpha_mode());
    assert!(!provider.apply_symbology());
    assert!(!provider.calculate_normals());
    assert!(provider.layers().is_empty());
    assert!(provider.sublayers().is_empty());
    assert!(!provider.is_destroyed());
}

#[test]
fn constructs_with_options() {
    let provider = I3SDataProvider::new(I3SDataProviderOptions {
        name: Some("test-dataset".to_string()),
        show: Some(false),
        show_features: Some(true),
        adjust_material_alpha_mode: Some(true),
        apply_symbology: Some(true),
        calculate_normals: Some(true),
    });

    assert_eq!(provider.name(), Some("test-dataset"));
    assert!(!provider.show());
    assert!(provider.show_features());
    assert!(provider.adjust_material_alpha_mode());
    assert!(provider.apply_symbology());
    assert!(provider.calculate_normals());
}

#[test]
fn from_url_loads_3dobject_layer_and_calculates_normals_without_textures() {
    pollster::block_on(async {
        let mock_layer_data = json!({
            "href": "layers/0/",
            "layerType": "3DObject",
            "attributeStorageInfo": [],
            "store": { "rootNode": "mockRootNodeUrl", "version": "1.6" },
            "fullExtent": { "xmin": 0.0, "ymin": 1.0, "xmax": 2.0, "ymax": 3.0 },
            "spatialReference": { "wkid": 4326 },
            "id": 0
        });

        let fetch_fn = move |_url: String| {
            let data = mock_layer_data.clone();
            async move { Ok(data) }
        };

        let provider = I3SDataProvider::from_url(
            "https://example.com/i3s/layers/0",
            I3SDataProviderOptions::default(),
            fetch_fn,
        )
        .await
        .unwrap();

        assert_eq!(provider.layers().len(), 1);
        assert_eq!(provider.layers()[0].layer_type, "3DObject");
        assert!(provider.calculate_normals()); // default true because no texture
        assert!(provider.extent().is_some());
    });
}

#[test]
fn from_url_loads_textured_layer_without_forcing_normals() {
    pollster::block_on(async {
        let mock_textured_layer = json!({
            "href": "layers/0/",
            "layerType": "IntegratedMesh",
            "fullExtent": { "xmin": 0.0, "ymin": 1.0, "xmax": 2.0, "ymax": 3.0 },
            "id": 0,
            "textureSetDefinitions": [
                {
                    "formats": [{ "name": "0", "format": "jpg" }]
                }
            ]
        });

        let fetch_fn = move |_url: String| {
            let data = mock_textured_layer.clone();
            async move { Ok(data) }
        };

        let provider = I3SDataProvider::from_url(
            "https://example.com/i3s",
            I3SDataProviderOptions::default(),
            fetch_fn,
        )
        .await
        .unwrap();

        assert_eq!(provider.layers().len(), 1);
        assert!(!provider.calculate_normals());
    });
}

#[test]
fn from_url_skips_unsupported_point_layers() {
    pollster::block_on(async {
        let mock_scene_data = json!({
            "layers": [
                {
                    "layerType": "Point",
                    "id": 1,
                    "name": "PointsLayer"
                },
                {
                    "layerType": "3DObject",
                    "id": 2,
                    "name": "ObjectsLayer"
                }
            ]
        });

        let fetch_fn = move |_url: String| {
            let data = mock_scene_data.clone();
            async move { Ok(data) }
        };

        let provider = I3SDataProvider::from_url(
            "https://example.com/i3s",
            I3SDataProviderOptions::default(),
            fetch_fn,
        )
        .await
        .unwrap();

        assert_eq!(provider.layers().len(), 1);
        assert_eq!(provider.layers()[0].name, "ObjectsLayer");
    });
}

#[test]
fn from_url_loads_building_scene_layer_with_sublayers_and_statistics() {
    pollster::block_on(async {
        let mock_building_data = json!({
            "layerType": "Building",
            "name": "Test-Building",
            "version": "1.6",
            "id": 0,
            "statisticsHRef": "statistics/summary",
            "fullExtent": { "xmin": 10.0, "ymin": 20.0, "xmax": 30.0, "ymax": 40.0 },
            "sublayers": [
                {
                    "id": 200,
                    "layerType": "group",
                    "visibility": false,
                    "name": "Full Model",
                    "modelName": "FullModel",
                    "sublayers": [
                        {
                            "id": 210,
                            "layerType": "group",
                            "name": "Cat1",
                            "visibility": false,
                            "sublayers": [
                                {
                                    "id": 1,
                                    "layerType": "3DObject",
                                    "visibility": false,
                                    "name": "SubCat1"
                                }
                            ]
                        }
                    ]
                },
                {
                    "id": 300,
                    "layerType": "3DObject",
                    "visibility": true,
                    "name": "Overview",
                    "modelName": "Overview",
                    "sublayers": []
                }
            ]
        });

        let mock_stats_data = json!({
            "stats": [
                {
                    "name": "BldgLevel",
                    "values": [1, 3]
                },
                {
                    "name": "NoValues",
                    "values": []
                }
            ]
        });

        let fetch_fn = move |url: String| {
            let bldg = mock_building_data.clone();
            let stats = mock_stats_data.clone();
            async move {
                if url.contains("statistics") {
                    Ok(stats)
                } else {
                    Ok(bldg)
                }
            }
        };

        let mut provider = I3SDataProvider::from_url(
            "https://example.com/i3s",
            I3SDataProviderOptions::default(),
            fetch_fn,
        )
        .await
        .unwrap();

        // Building Scene Layer enables features, transparency, symbology and flat normals
        assert!(provider.show_features());
        assert!(provider.adjust_material_alpha_mode());
        assert!(provider.apply_symbology());
        assert!(provider.calculate_normals());

        // Check sublayers hierarchy
        assert_eq!(provider.sublayers().len(), 2);
        assert_eq!(provider.sublayers()[0].name, "Full Model");
        assert_eq!(provider.sublayers()[0].model_name, "FullModel");
        assert!(!provider.sublayers()[0].visibility);

        assert_eq!(provider.sublayers()[0].sublayers.len(), 1);
        assert_eq!(provider.sublayers()[0].sublayers[0].name, "Cat1");
        assert_eq!(provider.sublayers()[0].sublayers[0].sublayers.len(), 1);
        assert_eq!(provider.sublayers()[0].sublayers[0].sublayers[0].name, "SubCat1");

        assert_eq!(provider.sublayers()[1].name, "Overview");
        assert!(provider.sublayers()[1].visibility);

        // Mutate sublayer visibility
        provider.sublayers_mut()[0].set_visibility(true);
        assert!(provider.sublayers()[0].visibility);

        // Attribute statistics queries
        let attrs = provider.get_attribute_names();
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0], "BldgLevel");
        assert_eq!(attrs[1], "NoValues");

        let values = provider.get_attribute_values("BldgLevel");
        assert_eq!(values, vec![json!(1), json!(3)]);

        let no_values = provider.get_attribute_values("NoValues");
        assert!(no_values.is_empty());

        let non_existent = provider.get_attribute_values("NonExistent");
        assert!(non_existent.is_empty());
    });
}

#[test]
fn empty_statistics_loads_gracefully() {
    let mut stats = I3SStatistics::new("https://example.com/stats".to_string());
    stats.load_from_json(json!({}));

    assert!(stats.names.is_empty());
    assert!(stats.get_values("any").is_none());
}

#[test]
fn from_url_throws_on_error_response() {
    pollster::block_on(async {
        let fetch_fn = |_url: String| async {
            Ok(json!({
                "error": {
                    "code": 404,
                    "message": "Scene layer not found"
                }
            }))
        };

        let err = I3SDataProvider::from_url(
            "https://example.com/i3s",
            I3SDataProviderOptions::default(),
            fetch_fn,
        )
        .await
        .unwrap_err();

        assert!(err.message.contains("Scene layer not found"));
    });
}

#[test]
fn binarize_gltf_creates_valid_glb() {
    let raw_gltf = json!({
        "asset": { "version": "2.0" },
        "scenes": [{ "nodes": [0] }]
    });

    let glb = I3SDataProvider::binarize_gltf(&raw_gltf);

    assert!(glb.len() >= 20);
    assert_eq!(&glb[0..4], b"glTF");
    let version = u32::from_le_bytes(glb[4..8].try_into().unwrap());
    assert_eq!(version, 2);

    let total_len = u32::from_le_bytes(glb[8..12].try_into().unwrap());
    assert_eq!(total_len as usize, glb.len());

    let chunk_type = u32::from_le_bytes(glb[16..20].try_into().unwrap());
    assert_eq!(chunk_type, 0x4E4F534A);
}

#[test]
fn show_setter_updates_layers_visibility() {
    let mut provider = I3SDataProvider::new(I3SDataProviderOptions {
        show: Some(true),
        ..Default::default()
    });

    assert!(provider.show());
    provider.set_show(false);
    assert!(!provider.show());
}

#[test]
fn is_destroyed_and_destroy() {
    let mut provider = I3SDataProvider::new(I3SDataProviderOptions::default());

    assert!(!provider.is_destroyed());
    provider.destroy();
    assert!(provider.is_destroyed());
}
