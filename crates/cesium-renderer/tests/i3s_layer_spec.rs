use cesium_math::Color;
use cesium_renderer::i3s_layer::{I3SLayer, I3STilesetOptions};
use serde_json::json;

fn make_mock_layer_data() -> serde_json::Value {
    json!({
        "href": "mockLayerUrl",
        "layerType": "3DObject",
        "nodePages": {
            "lodSelectionMetricType": "maxScreenThresholdSQ",
            "nodesPerPage": 2,
            "rootIndex": 0
        },
        "attributeStorageInfo": [],
        "store": {
            "defaultGeometrySchema": {},
            "version": "1.7"
        },
        "geometryDefinitions": [
            {
                "geometryBuffers": [
                    {
                        "color": { "type": "UInt8", "component": 4 },
                        "normal": { "type": "Float32", "component": 3 },
                        "position": { "type": "Float32", "component": 3 },
                        "uv0": { "type": "Float32", "component": 2 }
                    },
                    {
                        "compressedAttributes": {
                            "attributes": ["position", "normal", "uv0", "color", "feature-index"],
                            "encoding": "draco"
                        }
                    }
                ]
            }
        ],
        "fullExtent": { "xmin": 0.0, "ymin": 1.0, "xmax": 2.0, "ymax": 3.0 },
        "spatialReference": { "wkid": 4326 }
    })
}

fn make_mock_layer_data_symbology() -> serde_json::Value {
    json!({
        "id": 0,
        "layerType": "3DObject",
        "nodePages": {
            "lodSelectionMetricType": "maxScreenThresholdSQ",
            "nodesPerPage": 2,
            "rootIndex": 0
        },
        "store": {
            "defaultGeometrySchema": {},
            "extent": [0.0, 1.0, 2.0, 3.0],
            "version": "1.7"
        },
        "spatialReference": { "wkid": 4326 },
        "drawingInfo": {
            "renderer": {
                "type": "simple",
                "symbol": {
                    "symbolLayers": [
                        {
                            "type": "Fill",
                            "edges": {
                                "color": [255, 255, 255]
                            }
                        }
                    ]
                }
            }
        }
    })
}

#[test]
fn constructs_i3s_layer_from_url() {
    let layer_data = make_mock_layer_data();
    let layer = I3SLayer::new("mockProviderUrl?testQuery=test", &layer_data);

    assert!(layer.url.contains("mockProviderUrl/mockLayerUrl/"));
    assert_eq!(layer.query_params, Some("testQuery=test".to_string()));
    assert_eq!(layer.version, "1.7");
    assert_eq!(layer.major_version, 1);
    assert_eq!(layer.minor_version, 7);
    assert!(!layer.legacy_version_16);

    let extent = layer.extent.unwrap();
    assert!((extent.west - 0.0f64.to_radians()).abs() < 1e-6);
    assert!((extent.south - 1.0f64.to_radians()).abs() < 1e-6);
    assert!((extent.east - 2.0f64.to_radians()).abs() < 1e-6);
    assert!((extent.north - 3.0f64.to_radians()).abs() < 1e-6);
}

#[test]
fn constructs_i3s_layer_from_single_layer_url() {
    let layer_data = make_mock_layer_data_symbology();
    let layer = I3SLayer::new("mockProviderUrl", &layer_data);

    assert!(layer.url.contains("mockProviderUrl/layers/0/"));
    let extent = layer.extent.unwrap();
    assert!((extent.west - 0.0f64.to_radians()).abs() < 1e-6);
    assert!((extent.south - 1.0f64.to_radians()).abs() < 1e-6);
    assert!((extent.east - 2.0f64.to_radians()).abs() < 1e-6);
    assert!((extent.north - 3.0f64.to_radians()).abs() < 1e-6);
}

#[test]
fn loads_root_node() {
    let layer_data = make_mock_layer_data();
    let mut layer = I3SLayer::new("mockProviderUrl", &layer_data);

    layer.set_root_node(
        0,
        json!({
            "index": 0,
            "children": [1, 2],
            "obb": { "center": [-90, 45, 0] }
        }),
    );

    let root = layer.root_node.as_ref().unwrap();
    assert_eq!(root.index, 0);
    assert_eq!(root.data["children"], json!([1, 2]));
}

#[test]
fn creates_3d_tileset() {
    let layer_data = make_mock_layer_data();
    let mut layer = I3SLayer::new("mockProviderUrl", &layer_data);

    layer.load(None).unwrap();
    let tileset = layer.tileset.as_ref().unwrap();
    assert!(tileset.show);
}

#[test]
fn creates_3d_tileset_with_options() {
    let layer_data = make_mock_layer_data();
    let mut layer = I3SLayer::new("mockProviderUrl", &layer_data);

    layer
        .load(Some(I3STilesetOptions {
            debug_show_bounding_volume: Some(true),
            maximum_screen_space_error: Some(8.0),
            outline_color: None,
        }))
        .unwrap();

    let tileset = layer.tileset.as_ref().unwrap();
    assert!(tileset.debug_show_bounding_volume);
    assert_eq!(tileset.maximum_screen_space_error, 8.0);
}

#[test]
fn load_rejects_unsupported_spatial_reference() {
    let mut layer_data = make_mock_layer_data();
    layer_data["spatialReference"]["wkid"] = json!(3857);

    let mut layer = I3SLayer::new("mockProviderUrl", &layer_data);
    let err = layer.load(None).unwrap_err();
    assert!(err.message.contains("Unsupported spatial reference: 3857"));
}

#[test]
fn creates_3d_tileset_with_outline_color_from_symbology() {
    let layer_data = make_mock_layer_data_symbology();
    let mut layer = I3SLayer::new("mockProviderUrl", &layer_data);

    layer.load(None).unwrap();
    let tileset = layer.tileset.as_ref().unwrap();
    assert_eq!(tileset.outline_color, Color::new(1.0, 1.0, 1.0, 1.0));
}

#[test]
fn creates_3d_tileset_with_outline_color_from_options() {
    let layer_data = make_mock_layer_data_symbology();
    let mut layer = I3SLayer::new("mockProviderUrl", &layer_data);

    let custom_color = Color::new(0.5, 0.5, 0.5, 0.5);
    layer
        .load(Some(I3STilesetOptions {
            outline_color: Some(custom_color),
            ..Default::default()
        }))
        .unwrap();

    let tileset = layer.tileset.as_ref().unwrap();
    assert_eq!(tileset.outline_color, custom_color);
}

#[test]
fn ranks_geometry_buffers_favoring_compression() {
    let layer_data = make_mock_layer_data();
    let layer = I3SLayer::new("mockProviderUrl", &layer_data);

    assert_eq!(layer.geometry_definitions.len(), 1);
    let buffers = &layer.geometry_definitions[0];
    assert_eq!(buffers.len(), 2);
    // Compressed buffer should be ranked first (index 0)
    assert!(buffers[0].compressed);
    assert!(!buffers[1].compressed);
}

#[test]
fn finds_best_geometry_buffers() {
    let layer_data = make_mock_layer_data();
    let layer = I3SLayer::new("mockProviderUrl", &layer_data);

    // Both compressed and uncompressed contain position, normal, uv0, color
    // Compressed buffer has original index 1 in geometryBuffers array
    let best_idx = layer.find_best_geometry_buffers(0, &["position", "normal"]);
    assert_eq!(best_idx, Some(1));
}

#[test]
fn updates_visibility_and_filters() {
    let layer_data = make_mock_layer_data();
    let mut layer = I3SLayer::new("mockProviderUrl", &layer_data);
    layer.load(None).unwrap();

    assert!(layer.tileset.as_ref().unwrap().show);
    layer.update_visibility(false);
    assert!(!layer.tileset.as_ref().unwrap().show);

    assert!(layer.filter_by_attributes().is_ok());
}
