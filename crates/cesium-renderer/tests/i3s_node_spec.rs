//! Tests adapted from CesiumJS Scene/I3SNodeSpec.js (Apache-2.0).

use std::collections::HashMap;
use serde_json::{json, Value};

use cesium_math::{Cartesian3, Cartographic, Ellipsoid, GeoRectangle, Resource};
use cesium_renderer::i3s_decoder::{I3SDecodeResult, I3SMeshData};
use cesium_renderer::i3s_field::{
    I3SAttributeStorageInfo, I3SField, I3SHeaderItem, I3SValueDesc,
};
use cesium_renderer::i3s_layer::{I3SAttributeFilter, I3SLayer};
use cesium_renderer::i3s_node::{
    GeoidData, I3SCustomAttributes, I3SFeature, I3SGeometry, I3SNode, I3SNodeRef,
    I3STileBatchTable, I3STileContent,
};

fn base64_to_bytes(s: &str) -> Vec<u8> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .expect("valid base64")
}

const INT8_ATTR_BUFFER_BASE64: &str = "AgAAAAEC";
const INT16_ATTR_BUFFER_BASE64: &str = "AgAAAAEAAgA=";
const INT32_ATTR_BUFFER_BASE64: &str = "AgAAAAEAAAD/////";
const INT64_ATTR_BUFFER_BASE64: &str = "AgAAAAAAAAABAAAAAAAAAP//////////";
const FLOAT32_ATTR_BUFFER_BASE64: &str = "AgAAAAAAgD8AAABA";
const FLOAT64_ATTR_BUFFER_BASE64: &str = "AgAAAAAAAAAAAAAAAADwPwAAAAAAAABA";
const STRING_ATTR_BUFFER_BASE64: &str = "AgAAABAAAAAEAAAABAAAAGFiYwBkZWYA";

fn make_attr_storage_info() -> Vec<I3SAttributeStorageInfo> {
    vec![
        I3SAttributeStorageInfo {
            key: "f_0".to_string(),
            name: "testUInt8".to_string(),
            header: vec![I3SHeaderItem {
                property: "count".to_string(),
                value_type: "UInt32".to_string(),
            }],
            ordering: vec!["attributeValues".to_string()],
            attribute_values: Some(I3SValueDesc {
                value_type: "UInt8".to_string(),
                values_per_element: Some(1),
                encoding: None,
            }),
            object_ids: None,
            attribute_byte_counts: None,
            custom_props: HashMap::new(),
        },
        I3SAttributeStorageInfo {
            key: "f_1".to_string(),
            name: "testInt8".to_string(),
            header: vec![I3SHeaderItem {
                property: "count".to_string(),
                value_type: "UInt32".to_string(),
            }],
            ordering: vec!["attributeValues".to_string()],
            attribute_values: Some(I3SValueDesc {
                value_type: "Int8".to_string(),
                values_per_element: Some(1),
                encoding: None,
            }),
            object_ids: None,
            attribute_byte_counts: None,
            custom_props: HashMap::new(),
        },
        I3SAttributeStorageInfo {
            key: "f_2".to_string(),
            name: "testUInt16".to_string(),
            header: vec![I3SHeaderItem {
                property: "count".to_string(),
                value_type: "UInt32".to_string(),
            }],
            ordering: vec!["attributeValues".to_string()],
            attribute_values: Some(I3SValueDesc {
                value_type: "UInt16".to_string(),
                values_per_element: Some(1),
                encoding: None,
            }),
            object_ids: None,
            attribute_byte_counts: None,
            custom_props: HashMap::new(),
        },
        I3SAttributeStorageInfo {
            key: "f_3".to_string(),
            name: "testInt16".to_string(),
            header: vec![I3SHeaderItem {
                property: "count".to_string(),
                value_type: "UInt32".to_string(),
            }],
            ordering: vec!["attributeValues".to_string()],
            attribute_values: Some(I3SValueDesc {
                value_type: "Int16".to_string(),
                values_per_element: Some(1),
                encoding: None,
            }),
            object_ids: None,
            attribute_byte_counts: None,
            custom_props: HashMap::new(),
        },
        I3SAttributeStorageInfo {
            key: "f_4".to_string(),
            name: "testOid32".to_string(),
            header: vec![I3SHeaderItem {
                property: "count".to_string(),
                value_type: "UInt32".to_string(),
            }],
            ordering: vec!["attributeValues".to_string()],
            attribute_values: Some(I3SValueDesc {
                value_type: "Oid32".to_string(),
                values_per_element: Some(1),
                encoding: None,
            }),
            object_ids: None,
            attribute_byte_counts: None,
            custom_props: HashMap::new(),
        },
        I3SAttributeStorageInfo {
            key: "f_5".to_string(),
            name: "testUInt32".to_string(),
            header: vec![I3SHeaderItem {
                property: "count".to_string(),
                value_type: "UInt32".to_string(),
            }],
            ordering: vec!["attributeValues".to_string()],
            attribute_values: Some(I3SValueDesc {
                value_type: "UInt32".to_string(),
                values_per_element: Some(1),
                encoding: None,
            }),
            object_ids: None,
            attribute_byte_counts: None,
            custom_props: HashMap::new(),
        },
        I3SAttributeStorageInfo {
            key: "f_6".to_string(),
            name: "testInt32".to_string(),
            header: vec![I3SHeaderItem {
                property: "count".to_string(),
                value_type: "UInt32".to_string(),
            }],
            ordering: vec!["attributeValues".to_string()],
            attribute_values: Some(I3SValueDesc {
                value_type: "Int32".to_string(),
                values_per_element: Some(1),
                encoding: None,
            }),
            object_ids: None,
            attribute_byte_counts: None,
            custom_props: HashMap::new(),
        },
        I3SAttributeStorageInfo {
            key: "f_7".to_string(),
            name: "testUInt64".to_string(),
            header: vec![I3SHeaderItem {
                property: "count".to_string(),
                value_type: "UInt32".to_string(),
            }],
            ordering: vec!["attributeValues".to_string()],
            attribute_values: Some(I3SValueDesc {
                value_type: "UInt64".to_string(),
                values_per_element: Some(1),
                encoding: None,
            }),
            object_ids: None,
            attribute_byte_counts: None,
            custom_props: HashMap::new(),
        },
        I3SAttributeStorageInfo {
            key: "f_8".to_string(),
            name: "testInt64".to_string(),
            header: vec![I3SHeaderItem {
                property: "count".to_string(),
                value_type: "UInt32".to_string(),
            }],
            ordering: vec!["attributeValues".to_string()],
            attribute_values: Some(I3SValueDesc {
                value_type: "Int64".to_string(),
                values_per_element: Some(1),
                encoding: None,
            }),
            object_ids: None,
            attribute_byte_counts: None,
            custom_props: HashMap::new(),
        },
        I3SAttributeStorageInfo {
            key: "f_9".to_string(),
            name: "testFloat32".to_string(),
            header: vec![I3SHeaderItem {
                property: "count".to_string(),
                value_type: "UInt32".to_string(),
            }],
            ordering: vec!["attributeValues".to_string()],
            attribute_values: Some(I3SValueDesc {
                value_type: "Float32".to_string(),
                values_per_element: Some(1),
                encoding: None,
            }),
            object_ids: None,
            attribute_byte_counts: None,
            custom_props: HashMap::new(),
        },
        I3SAttributeStorageInfo {
            key: "f_10".to_string(),
            name: "testFloat64".to_string(),
            header: vec![I3SHeaderItem {
                property: "count".to_string(),
                value_type: "UInt32".to_string(),
            }],
            ordering: vec!["attributeValues".to_string()],
            attribute_values: Some(I3SValueDesc {
                value_type: "Float64".to_string(),
                values_per_element: Some(1),
                encoding: None,
            }),
            object_ids: None,
            attribute_byte_counts: None,
            custom_props: HashMap::new(),
        },
        I3SAttributeStorageInfo {
            key: "f_11".to_string(),
            name: "testString".to_string(),
            header: vec![
                I3SHeaderItem {
                    property: "count".to_string(),
                    value_type: "UInt32".to_string(),
                },
                I3SHeaderItem {
                    property: "attributeValuesByteCount".to_string(),
                    value_type: "UInt32".to_string(),
                },
            ],
            ordering: vec![
                "attributeByteCounts".to_string(),
                "attributeValues".to_string(),
            ],
            attribute_values: Some(I3SValueDesc {
                value_type: "String".to_string(),
                values_per_element: Some(1),
                encoding: Some("UTF-8".to_string()),
            }),
            object_ids: None,
            attribute_byte_counts: Some(I3SValueDesc {
                value_type: "UInt32".to_string(),
                values_per_element: Some(1),
                encoding: None,
            }),
            custom_props: HashMap::new(),
        },
    ]
}

fn sample_geoid_data_list() -> Vec<GeoidData> {
    vec![GeoidData {
        buffer: vec![0.0, 10.0, 10.0, 20.0],
        height: 2,
        width: 2,
        native_extent: GeoRectangle {
            west: -20037508.342787e7,
            south: -20037508.342787e7,
            east: 20037508.342787e7,
            north: 20037508.342787e7,
        },
        offset: 0.0,
        scale: 1.0,
        projection_type: "WebMercator".to_string(),
    }]
}

fn make_root_node_with_children() -> Value {
    json!({
        "id": "root",
        "level": 0,
        "mbs": [-90.0, 45.0, 0.0, 28288.6903196],
        "obb": {
            "center": [-90.0, 45.0, 0.0],
            "halfSize": [20000.0, 20000.0, 500.0],
            "quaternion": [1.0, 0.0, 0.0, 0.0]
        },
        "lodSelection": [
            { "metricType": "maxScreenThresholdSQ", "maxError": 4.0 },
            { "metricType": "maxScreenThreshold", "maxError": 2.0 }
        ],
        "children": [
            { "id": 1, "href": "../1" },
            { "id": 2, "href": "../2" }
        ]
    })
}

fn make_node_with_content() -> Value {
    json!({
        "id": 1,
        "level": 1,
        "mbs": [-90.0, 45.0, 0.0, 14144.3451598],
        "lodSelection": [
            { "metricType": "maxScreenThreshold", "maxError": 2.0 },
            { "metricType": "maxScreenThresholdSQ", "maxError": 4.0 }
        ],
        "children": [],
        "featureData": [{ "href": "mockFeatureDataUrl" }],
        "geometryData": [{ "href": "mockGeometryDataUrl" }]
    })
}

fn make_node_with_textured_content() -> Value {
    json!({
        "id": 1,
        "level": 1,
        "mbs": [-90.0, 45.0, 0.0, 14144.3451598],
        "lodSelection": [
            { "metricType": "maxScreenThreshold", "maxError": 2.0 },
            { "metricType": "maxScreenThresholdSQ", "maxError": 4.0 }
        ],
        "children": [],
        "featureData": [{ "href": "mockFeatureDataUrl" }],
        "geometryData": [{ "href": "mockGeometryDataUrl" }],
        "textureData": [{ "href": "mockTextureDataUrl" }]
    })
}

fn make_child_node_page_entry() -> Value {
    json!({
        "index": 1,
        "children": [],
        "obb": {
            "center": [-90.0, 45.0, 0.0],
            "halfSize": [10000.0, 10000.0, 250.0],
            "quaternion": [1.0, 0.0, 0.0, 0.0]
        },
        "lodThreshold": 500.0,
        "parentIndex": 0,
        "mesh": {
            "attribute": { "resource": 1 },
            "geometry": {
                "definition": 0,
                "featureCount": 2,
                "resource": 1,
                "vertexCount": 6
            },
            "material": { "definition": 0, "resource": 1 }
        }
    })
}

fn make_textured_child_node_page_entry() -> Value {
    json!({
        "index": 2,
        "children": [],
        "obb": {
            "center": [-90.0, 45.0, 0.0],
            "halfSize": [10000.0, 10000.0, 250.0],
            "quaternion": [1.0, 0.0, 0.0, 0.0]
        },
        "lodThreshold": 500.0,
        "parentIndex": 0,
        "mesh": {
            "attribute": { "resource": 2 },
            "geometry": {
                "definition": 0,
                "featureCount": 2,
                "resource": 2,
                "vertexCount": 6
            },
            "material": { "definition": 1, "resource": 2 }
        }
    })
}

fn make_custom_attributes() -> I3SCustomAttributes {
    let carto = Cartographic::degrees(-90.0, 45.0, 0.0).unwrap();
    let center = Cartesian3::from(Ellipsoid::WGS84.cartographic_to_cartesian(carto).unwrap());
    I3SCustomAttributes {
        cartesian_center: center,
        feature_index: Some(vec![0, 0, 1, 1]),
        positions: vec![
            -20.0, -20.0, 0.0, 20.0, 0.0, 0.0, -20.0, 0.0, 0.0, -20.0, -20.0, 0.0, 20.0, -20.0,
            0.0, 20.0, 0.0, 0.0, -20.0, 0.0, 0.0, 20.0, 20.0, 0.0, -20.0, 20.0, 0.0, -20.0, 0.0,
            0.0, 20.0, 0.0, 0.0, 20.0, 20.0, 0.0,
        ],
        indices: None,
        parent_rotation: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
    }
}

fn make_custom_attributes_indexed() -> I3SCustomAttributes {
    let carto = Cartographic::degrees(-90.0, 45.0, 0.0).unwrap();
    let center = Cartesian3::from(Ellipsoid::WGS84.cartographic_to_cartesian(carto).unwrap());
    I3SCustomAttributes {
        cartesian_center: center,
        feature_index: Some(vec![0, 0, 1, 1]),
        indices: Some(vec![0, 1, 2, 0, 3, 1, 2, 4, 5, 2, 1, 4]),
        positions: vec![
            -20.0, -20.0, 0.0, 20.0, 0.0, 0.0, -20.0, 0.0, 0.0, 20.0, -20.0, 0.0, 20.0, 20.0,
            0.0, -20.0, 20.0, 0.0,
        ],
        parent_rotation: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
    }
}

fn make_mock_decode_result() -> I3SDecodeResult {
    I3SDecodeResult {
        mesh_data: I3SMeshData {
            nodes_in_scene: vec![json!(0)],
            nodes: vec![json!({ "mesh": 0 })],
            meshes: vec![json!({
                "primitives": [{
                    "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 },
                    "indices": 4,
                    "material": 0
                }]
            })],
            buffers: vec![json!({
                "byteLength": 336,
                "uri": "i3sMockBufferUrl"
            })],
            buffer_views: vec![
                json!({ "buffer": 0, "byteLength": 108, "byteOffset": 48, "target": 34962 }),
                json!({ "buffer": 0, "byteLength": 108, "byteOffset": 156, "target": 34962 }),
                json!({ "buffer": 0, "byteLength": 72, "byteOffset": 264, "target": 34962 }),
                json!({ "buffer": 0, "byteLength": 48, "byteOffset": 0, "target": 34963 }),
            ],
            accessors: vec![
                json!({ "bufferView": 0, "byteOffset": 0, "componentType": 5126, "count": 9, "max": [20.0, 20.0, 0.0], "min": [-20.0, -20.0, 0.0], "type": "VEC3" }),
                json!({ "bufferView": 1, "byteOffset": 0, "componentType": 5126, "count": 9, "max": [0.0, 0.0, 1.0], "min": [0.0, 0.0, 1.0], "type": "VEC3" }),
                json!({ "bufferView": 2, "byteOffset": 0, "componentType": 5126, "count": 9, "max": [0.0, 1.0], "min": [0.0, 1.0], "type": "VEC2" }),
                json!({ "bufferView": 3, "byteOffset": 0, "componentType": 5125, "count": 12, "max": [8], "min": [0], "type": "SCALAR" }),
            ],
        },
    }
}

#[test]
fn constructs_nodes() {
    let layer_data = json!({
        "href": "mockLayerUrl",
        "layerType": "3DObject",
        "store": { "defaultGeometrySchema": {}, "version": "1.6" }
    });
    let layer = I3SLayer::new("mockProviderUrl", &layer_data);

    let root_node = I3SNode::new_root_from_url(layer.resource.as_ref(), "mockRootUrl");
    assert!(root_node.resource.as_ref().unwrap().url().contains("mockProviderUrl/mockLayerUrl/mockRootUrl/"));
    assert_eq!(root_node.level, 0);
    assert!(root_node.is_root);

    let child_node = I3SNode::new_child(&root_node, I3SNodeRef::Index(3));
    assert!(child_node.resource.is_none());
    assert_eq!(child_node.node_index, Some(3));
    assert_eq!(child_node.level, 1);
    assert!(!child_node.is_root);
}

#[test]
fn loads_root_node_from_uri() {
    let layer_data = json!({
        "href": "mockLayerUrl",
        "layerType": "3DObject",
        "store": { "defaultGeometrySchema": {}, "version": "1.6" }
    });
    let layer = I3SLayer::new("mockProviderUrl", &layer_data);
    let mut root_node = I3SNode::new_root_from_url(layer.resource.as_ref(), "mockUrl");

    root_node.load_with_data(make_root_node_with_children(), None, None, None, layer.resource.as_ref());
    let children = root_node.data.as_ref().unwrap().get("children").unwrap().as_array().unwrap();
    assert_eq!(children.len(), 2);
}

#[test]
fn loads_child_node_from_uri() {
    let layer_data = json!({
        "href": "mockLayerUrl",
        "layerType": "3DObject",
        "store": { "defaultGeometrySchema": {}, "version": "1.6" }
    });
    let layer = I3SLayer::new("mockProviderUrl", &layer_data);
    let mut root_node = I3SNode::new_root_from_url(layer.resource.as_ref(), "mockUrl");
    let mut child_node = I3SNode::new_child(&root_node, I3SNodeRef::Uri("mockUrlChild".to_string()));

    root_node.load_with_data(make_root_node_with_children(), None, None, None, layer.resource.as_ref());
    child_node.load_with_data(make_node_with_content(), root_node.global_transform, None, None, layer.resource.as_ref());

    let children = child_node.data.as_ref().unwrap().get("children").unwrap().as_array().unwrap();
    assert_eq!(children.len(), 0);
    assert!(child_node.tile.is_some());
}

#[test]
fn loads_root_from_node_pages() {
    let layer_data = json!({
        "href": "mockLayerUrl",
        "layerType": "3DObject",
        "nodePages": { "lodSelectionMetricType": "maxScreenThresholdSQ", "nodesPerPage": 64 }
    });
    let layer = I3SLayer::new("mockProviderUrl?testQuery=test", &layer_data);
    let mut root_node = I3SNode::new_root_from_index(0);

    root_node.load_with_data(make_root_node_with_children(), None, None, Some("maxScreenThresholdSQ"), layer.resource.as_ref());
    let children = root_node.data.as_ref().unwrap().get("children").unwrap().as_array().unwrap();
    assert_eq!(children.len(), 2);
    assert!(root_node.data.as_ref().unwrap().get("mesh").is_none());
}

#[test]
fn loads_node_from_node_pages() {
    let layer_data = json!({
        "href": "mockLayerUrl",
        "layerType": "3DObject",
        "nodePages": { "lodSelectionMetricType": "maxScreenThresholdSQ", "nodesPerPage": 64 }
    });
    let layer = I3SLayer::new("mockProviderUrl?testQuery=test", &layer_data);
    let mut root_node = I3SNode::new_root_from_index(0);
    let mut child_node = I3SNode::new_child(&root_node, I3SNodeRef::Index(1));

    root_node.load_with_data(make_root_node_with_children(), None, None, Some("maxScreenThresholdSQ"), layer.resource.as_ref());
    child_node.load_with_data(make_child_node_page_entry(), root_node.global_transform, None, Some("maxScreenThresholdSQ"), layer.resource.as_ref());

    let children = child_node.data.as_ref().unwrap().get("children").unwrap().as_array().unwrap();
    assert_eq!(children.len(), 0);
    assert!(child_node.data.as_ref().unwrap().get("mesh").is_some());
}

#[test]
fn loads_node_with_geoid_conversion() {
    let geoid_list = sample_geoid_data_list();
    let layer_data = json!({
        "href": "mockLayerUrl",
        "layerType": "3DObject",
        "nodePages": { "lodSelectionMetricType": "maxScreenThresholdSQ", "nodesPerPage": 64 }
    });
    let layer = I3SLayer::new("mockProviderUrl?testQuery=test", &layer_data);
    let mut root_node = I3SNode::new_root_from_index(0);
    let mut child_node = I3SNode::new_child(&root_node, I3SNodeRef::Index(1));

    root_node.load_with_data(make_root_node_with_children(), None, Some(&geoid_list), Some("maxScreenThresholdSQ"), layer.resource.as_ref());
    child_node.load_with_data(make_child_node_page_entry(), root_node.global_transform, Some(&geoid_list), Some("maxScreenThresholdSQ"), layer.resource.as_ref());

    let gt = child_node.global_transform.unwrap();
    let node_origin = Cartesian3::new(gt[3], gt[7], gt[11]);
    let carto_origin = Ellipsoid::WGS84.cartesian_to_cartographic(node_origin.into()).unwrap();

    let expected_height = 10.0;
    let expected_lon = -90.0_f64.to_radians();
    let expected_lat = 45.0_f64.to_radians();

    assert!((carto_origin.longitude - expected_lon).abs() < 1e-3);
    assert!((carto_origin.latitude - expected_lat).abs() < 1e-3);
    assert!((carto_origin.height - expected_height).abs() < 1e-3);
}

#[test]
fn loads_children_and_leaf() {
    let mut root_node = I3SNode::new_root_from_index(0);
    root_node.load_children(vec![
        (I3SNodeRef::Index(1), make_child_node_page_entry()),
        (I3SNodeRef::Index(2), make_textured_child_node_page_entry()),
    ]);
    assert_eq!(root_node.children.len(), 2);
    assert_eq!(root_node.children[0].node_index, Some(1));
    assert_eq!(root_node.children[1].node_index, Some(2));

    let mut leaf_node = I3SNode::new_root_from_index(1);
    leaf_node.load_children(vec![]);
    assert_eq!(leaf_node.children.len(), 0);
}

#[test]
fn loads_fields_and_retrieves_feature_values() {
    let storage_info = make_attr_storage_info();
    let layer_data = json!({
        "href": "mockLayerUrl",
        "layerType": "3DObject",
        "attributeStorageInfo": storage_info
    });
    let layer = I3SLayer::new("mockProviderUrl?testQuery=test", &layer_data);
    let mut node = I3SNode::new_root_from_index(1);

    let mut field_buffers = HashMap::new();
    field_buffers.insert("testUInt8".to_string(), base64_to_bytes(INT8_ATTR_BUFFER_BASE64));
    field_buffers.insert("testInt8".to_string(), base64_to_bytes(INT8_ATTR_BUFFER_BASE64));
    field_buffers.insert("testUInt16".to_string(), base64_to_bytes(INT16_ATTR_BUFFER_BASE64));
    field_buffers.insert("testInt16".to_string(), base64_to_bytes(INT16_ATTR_BUFFER_BASE64));
    field_buffers.insert("testOid32".to_string(), base64_to_bytes(INT32_ATTR_BUFFER_BASE64));
    field_buffers.insert("testUInt32".to_string(), base64_to_bytes(INT32_ATTR_BUFFER_BASE64));
    field_buffers.insert("testInt32".to_string(), base64_to_bytes(INT32_ATTR_BUFFER_BASE64));
    field_buffers.insert("testUInt64".to_string(), base64_to_bytes(INT64_ATTR_BUFFER_BASE64));
    field_buffers.insert("testInt64".to_string(), base64_to_bytes(INT64_ATTR_BUFFER_BASE64));
    field_buffers.insert("testFloat32".to_string(), base64_to_bytes(FLOAT32_ATTR_BUFFER_BASE64));
    field_buffers.insert("testFloat64".to_string(), base64_to_bytes(FLOAT64_ATTR_BUFFER_BASE64));
    field_buffers.insert("testString".to_string(), base64_to_bytes(STRING_ATTR_BUFFER_BASE64));

    node.load_fields_from_storage(&storage_info, &field_buffers, layer.resource.as_ref());

    let int8_field = node.fields.get("testInt8").unwrap();
    assert_eq!(int8_field.name(), "testInt8");
    assert!(int8_field.resource.as_ref().unwrap().url().contains("mockProviderUrl/mockLayerUrl/nodes/1/attributes/f_1/0"));

    let feat0 = node.get_fields_for_feature(0);
    let feat1 = node.get_fields_for_feature(1);

    assert_eq!(feat0.get("testInt8").unwrap(), &json!(1));
    assert_eq!(feat1.get("testInt8").unwrap(), &json!(2));
    assert_eq!(feat0.get("testUInt8").unwrap(), &json!(1));
    assert_eq!(feat1.get("testUInt8").unwrap(), &json!(2));
    assert_eq!(feat0.get("testInt16").unwrap(), &json!(1));
    assert_eq!(feat1.get("testInt16").unwrap(), &json!(2));
    assert_eq!(feat0.get("testUInt16").unwrap(), &json!(1));
    assert_eq!(feat1.get("testUInt16").unwrap(), &json!(2));
    assert_eq!(feat0.get("testInt32").unwrap(), &json!(1));
    assert_eq!(feat1.get("testInt32").unwrap(), &json!(-1));
    assert_eq!(feat0.get("testUInt32").unwrap(), &json!(1));
    assert_eq!(feat0.get("testInt64").unwrap(), &json!(1));
    assert_eq!(feat1.get("testInt64").unwrap(), &json!(-1));
    assert_eq!(feat0.get("testUInt64").unwrap(), &json!(1));
    assert_eq!(feat0.get("testFloat32").unwrap(), &json!(1.0));
    assert_eq!(feat1.get("testFloat32").unwrap(), &json!(2.0));
    assert_eq!(feat0.get("testFloat64").unwrap(), &json!(1.0));
    assert_eq!(feat1.get("testFloat64").unwrap(), &json!(2.0));
    assert_eq!(feat0.get("testString").unwrap(), &json!("abc"));
    assert_eq!(feat1.get("testString").unwrap(), &json!("def"));
}

#[test]
fn loads_existing_fields_and_single_field() {
    let storage_info = vec![I3SAttributeStorageInfo {
        key: "f_0".to_string(),
        name: "testField".to_string(),
        header: vec![I3SHeaderItem {
            property: "count".to_string(),
            value_type: "UInt32".to_string(),
        }],
        ordering: vec!["attributeValues".to_string()],
        attribute_values: Some(I3SValueDesc {
            value_type: "UInt8".to_string(),
            values_per_element: Some(1),
            encoding: None,
        }),
        object_ids: None,
        attribute_byte_counts: None,
        custom_props: HashMap::new(),
    }];

    let layer = I3SLayer::new("mockProviderUrl", &json!({}));
    let mut node = I3SNode::new_root_from_index(1);

    node.load_field("testField", &storage_info, Some(base64_to_bytes(INT8_ATTR_BUFFER_BASE64)), layer.resource.as_ref());
    assert!(node.fields.contains_key("testField"));

    // Loading already existing field does not reload or overwrite
    node.load_field("testField", &storage_info, None, layer.resource.as_ref());
    assert_eq!(node.fields.get("testField").unwrap().values.len(), 2);

    // Non existing field does not panic
    node.load_field("nonExistent", &storage_info, None, layer.resource.as_ref());
    assert!(!node.fields.contains_key("nonExistent"));
}

#[test]
fn loads_geometry_from_node_pages_and_url() {
    let layer_data = json!({
        "href": "mockLayerUrl",
        "layerType": "3DObject"
    });
    let layer = I3SLayer::new("mockProviderUrl?testQuery=test", &layer_data);

    // From node pages
    let mut node_pages = I3SNode::new_root_from_index(1);
    node_pages.load_with_data(make_child_node_page_entry(), None, None, None, layer.resource.as_ref());
    node_pages.load_geometry_data(layer.resource.as_ref(), Some(vec![1, 2, 3]));
    assert_eq!(node_pages.geometry_data.len(), 1);
    let geom = &node_pages.geometry_data[0];
    assert!(geom.resource.as_ref().unwrap().url().contains("mockProviderUrl/mockLayerUrl/nodes/1/geometries/1/"));
    assert_eq!(geom.data.as_ref().unwrap(), &vec![1, 2, 3]);

    // From url
    let mut node_url = I3SNode::new_root_from_url(layer.resource.as_ref(), "mockNodeUrl");
    node_url.load_with_data(make_node_with_content(), None, None, None, layer.resource.as_ref());
    node_url.load_geometry_data(layer.resource.as_ref(), Some(vec![4, 5, 6]));
    assert_eq!(node_url.geometry_data.len(), 1);
    let geom_url = &node_url.geometry_data[0];
    assert!(geom_url.resource.as_ref().unwrap().url().contains("mockProviderUrl/mockLayerUrl/mockNodeUrl/mockGeometryDataUrl"));
    assert_eq!(geom_url.data.as_ref().unwrap(), &vec![4, 5, 6]);

    // Clear geometry data
    node_url.clear_geometry_data();
    assert_eq!(node_url.geometry_data.len(), 0);
}

#[test]
fn generate_geometry_and_textured_geometry_gltf() {
    let layer_data = json!({
        "href": "mockLayerUrl",
        "materialDefinitions": [
            {
                "doubleSided": true,
                "pbrMetallicRoughness": {
                    "metallicFactor": 0.0,
                    "baseColorFactor": [1.0, 1.0, 1.0, 1.0]
                },
                "emissiveFactor": [0.0, 0.0, 0.0]
            },
            {
                "doubleSided": true,
                "pbrMetallicRoughness": {
                    "baseColorTexture": { "textureSetDefinitionId": 0 },
                    "metallicFactor": 0.0
                }
            }
        ],
        "textureSetDefinitions": [
            {
                "formats": [
                    { "name": "0", "format": "dds" },
                    { "name": "1", "format": "jpg" }
                ]
            }
        ]
    });
    let layer = I3SLayer::new("mockProviderUrl", &layer_data);
    let mut node = I3SNode::new_root_from_index(1);
    node.load_with_data(make_child_node_page_entry(), None, None, None, layer.resource.as_ref());
    node.load_geometry_data(layer.resource.as_ref(), Some(vec![]));

    let decode_result = make_mock_decode_result();
    let raw_gltf = node.geometry_data[0].generate_gltf(
        &decode_result.mesh_data.nodes_in_scene.iter().map(|n| json!(n)).collect::<Vec<_>>(),
        &decode_result.mesh_data.nodes,
        &decode_result.mesh_data.meshes,
        &decode_result.mesh_data.buffers,
        &decode_result.mesh_data.buffer_views,
        &decode_result.mesh_data.accessors,
        Some(&layer_data),
        node.data.as_ref(),
        Some(&layer.url),
    );

    assert_eq!(raw_gltf.get("scene").unwrap(), &json!(0));
    assert_eq!(raw_gltf.get("scenes").unwrap().as_array().unwrap().len(), 1);
    assert_eq!(raw_gltf.get("nodes").unwrap().as_array().unwrap().len(), 1);
    assert_eq!(raw_gltf.get("meshes").unwrap().as_array().unwrap().len(), 1);
    assert_eq!(raw_gltf.get("textures").unwrap().as_array().unwrap().len(), 0);

    // Textured node
    let mut textured_node = I3SNode::new_root_from_index(2);
    textured_node.load_with_data(make_textured_child_node_page_entry(), None, None, None, layer.resource.as_ref());
    textured_node.load_geometry_data(layer.resource.as_ref(), Some(vec![]));

    let raw_gltf_textured = textured_node.geometry_data[0].generate_gltf(
        &decode_result.mesh_data.nodes_in_scene.iter().map(|n| json!(n)).collect::<Vec<_>>(),
        &decode_result.mesh_data.nodes,
        &decode_result.mesh_data.meshes,
        &decode_result.mesh_data.buffers,
        &decode_result.mesh_data.buffer_views,
        &decode_result.mesh_data.accessors,
        Some(&layer_data),
        textured_node.data.as_ref(),
        Some(&layer.url),
    );

    assert_eq!(raw_gltf_textured.get("textures").unwrap().as_array().unwrap().len(), 1);
    assert_eq!(raw_gltf_textured.get("images").unwrap().as_array().unwrap().len(), 1);
    assert_eq!(raw_gltf_textured.get("samplers").unwrap().as_array().unwrap().len(), 1);

    // Textured node from url
    let mut url_textured_node = I3SNode::new_root_from_url(layer.resource.as_ref(), "mockNodeUrl");
    url_textured_node.load_with_data(make_node_with_textured_content(), None, None, None, layer.resource.as_ref());
    url_textured_node.load_geometry_data(layer.resource.as_ref(), Some(vec![]));

    let raw_gltf_url_textured = url_textured_node.geometry_data[0].generate_gltf(
        &decode_result.mesh_data.nodes_in_scene.iter().map(|n| json!(n)).collect::<Vec<_>>(),
        &decode_result.mesh_data.nodes,
        &decode_result.mesh_data.meshes,
        &decode_result.mesh_data.buffers,
        &decode_result.mesh_data.buffer_views,
        &decode_result.mesh_data.accessors,
        None,
        url_textured_node.data.as_ref(),
        Some(&layer.url),
    );
    assert_eq!(raw_gltf_url_textured.get("textures").unwrap().as_array().unwrap().len(), 1);
    assert_eq!(raw_gltf_url_textured.get("images").unwrap().as_array().unwrap().len(), 1);
}

#[test]
fn picks_closest_point_in_non_indexed_and_indexed_geometry() {
    let custom_attrs = make_custom_attributes();
    let custom_attrs_indexed = make_custom_attributes_indexed();

    let mut geom = I3SGeometry::new(None);
    geom.custom_attributes = Some(custom_attrs.clone());

    let center = custom_attrs.cartesian_center;
    let intersect_point = Cartesian3::new(-19.0 + center.x, -19.0 + center.y, 0.0 + center.z);
    let border_point = Cartesian3::new(20.0 + center.x, 15.0 + center.y, 0.0 + center.z);
    let floating_intersect_point = Cartesian3::new(-19.0 + center.x, 19.0 + center.y, 2.0 + center.z);
    let no_intersect_point = Cartesian3::new(-21.0 + center.x, -21.0 + center.y, 0.0 + center.z);

    assert_eq!(geom.get_closest_point_index_on_triangle(intersect_point.x, intersect_point.y, intersect_point.z).index, 0);
    assert_eq!(geom.get_closest_point_index_on_triangle(border_point.x, border_point.y, border_point.z).index, 11);
    assert_eq!(geom.get_closest_point_index_on_triangle(floating_intersect_point.x, floating_intersect_point.y, floating_intersect_point.z).index, 8);
    assert_eq!(geom.get_closest_point_index_on_triangle(no_intersect_point.x, no_intersect_point.y, no_intersect_point.z).index, -1);

    // Indexed geometry
    let mut geom_indexed = I3SGeometry::new(None);
    geom_indexed.custom_attributes = Some(custom_attrs_indexed);

    assert_eq!(geom_indexed.get_closest_point_index_on_triangle(intersect_point.x, intersect_point.y, intersect_point.z).index, 0);
    assert_eq!(geom_indexed.get_closest_point_index_on_triangle(border_point.x, border_point.y, border_point.z).index, 4);
    assert_eq!(geom_indexed.get_closest_point_index_on_triangle(floating_intersect_point.x, floating_intersect_point.y, floating_intersect_point.z).index, 5);
    assert_eq!(geom_indexed.get_closest_point_index_on_triangle(no_intersect_point.x, no_intersect_point.y, no_intersect_point.z).index, -1);
}

#[test]
fn loads_feature_data_and_derives_urls() {
    let layer_data = json!({ "href": "mockLayerUrl" });
    let layer = I3SLayer::new("mockProviderUrl", &layer_data);
    let mut node = I3SNode::new_root_from_url(layer.resource.as_ref(), "mockUrl");
    node.load_with_data(make_node_with_content(), None, None, None, layer.resource.as_ref());

    node.load_feature_data(Some(json!({ "featureId": 42 })));
    assert_eq!(node.feature_data.len(), 1);
    assert_eq!(node.feature_data[0].data.as_ref().unwrap().get("featureId").unwrap(), &json!(42));
    assert!(node.feature_data[0].resource.as_ref().unwrap().url().contains("mockProviderUrl/mockLayerUrl/mockUrl/mockFeatureDataUrl"));

    // Feature url for indexed nodes
    let layer_res = Resource::from_url("testLayerUrl");
    let feat = I3SFeature::new(None, Some(&layer_res), Some("testAttributeUrl"), "testNode");
    assert!(feat.resource.as_ref().unwrap().url().ends_with("nodes/testAttributeUrl/testNode"));
}

#[test]
fn filters_features_by_attribute_values() {
    let mut node = I3SNode::new_root_from_index(1);
    let mut field = I3SField::new(I3SAttributeStorageInfo {
        name: "BldgLevel".to_string(),
        key: "f_0".to_string(),
        ..Default::default()
    });
    field.values = vec![json!(1), json!(1), json!(2)];
    node.fields.insert("BldgLevel".to_string(), field);

    let tile_content = I3STileContent {
        batch_table: Some(I3STileBatchTable::new(3)),
        model_ready: true,
        model_show: false,
    };
    node.tile = Some(cesium_renderer::i3s_node::I3STile {
        content: Some(tile_content),
        ..Default::default()
    });

    // Filter for BldgLevel == 2
    let filters = vec![I3SAttributeFilter {
        name: "BldgLevel".to_string(),
        values: vec![json!(2)],
    }];
    node.filter_features(&filters);

    let batch = node.tile.as_ref().unwrap().content.as_ref().unwrap().batch_table.as_ref().unwrap();
    assert_eq!(batch.features, vec![false, false, true]);

    // Filter with non existing attribute hides all
    let non_existing_filters = vec![I3SAttributeFilter {
        name: "NotExisting".to_string(),
        values: vec![json!(2)],
    }];
    node.filter_features(&non_existing_filters);
    let batch2 = node.tile.as_ref().unwrap().content.as_ref().unwrap().batch_table.as_ref().unwrap();
    assert_eq!(batch2.features, vec![false, false, false]);

    // Empty filter preserves all
    node.filter_features(&[]);
    let batch3 = node.tile.as_ref().unwrap().content.as_ref().unwrap().batch_table.as_ref().unwrap();
    assert_eq!(batch3.features, vec![true, true, true]);
}

#[test]
fn creates_3d_tile_content_glb() {
    let layer_data = json!({
        "href": "mockLayerUrl",
        "materialDefinitions": [
            {
                "doubleSided": true,
                "pbrMetallicRoughness": {
                    "metallicFactor": 0.0,
                    "baseColorFactor": [1.0, 1.0, 1.0, 1.0]
                }
            }
        ]
    });
    let layer = I3SLayer::new("mockProviderUrl", &layer_data);
    let mut node = I3SNode::new_root_from_index(1);
    node.load_with_data(make_child_node_page_entry(), None, None, None, layer.resource.as_ref());
    node.load_geometry_data(layer.resource.as_ref(), Some(vec![0; 336]));

    let decode_result = make_mock_decode_result();
    let glb_bytes = node.create_content_glb(&decode_result, Some(&layer_data), Some(&layer.url));
    assert!(glb_bytes.len() > 12);
    // Magic check: glTF
    assert_eq!(&glb_bytes[0..4], b"glTF");
    assert_eq!(u32::from_le_bytes(glb_bytes[4..8].try_into().unwrap()), 2);
}
