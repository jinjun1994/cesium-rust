use cesium_math::{IndexDatatype, Matrix4, PrimitiveType};
use cesium_renderer::geo_json_loader::{GeoJsonLoader, GeoJsonLoaderOptions};
use cesium_renderer::model_components::FeatureId;
use serde_json::json;

fn make_point_geojson() -> serde_json::Value {
    json!({
        "type": "Feature",
        "geometry": {
            "type": "Point",
            "coordinates": [100.0, 0.0]
        },
        "properties": {
            "name": "UL",
            "code": 12
        }
    })
}

fn make_multipoint_geojson() -> serde_json::Value {
    let mut coords = Vec::new();
    for i in 0..10 {
        coords.push(vec![100.0 + (i as f64) * 0.1, (i as f64) * 0.1]);
    }
    json!({
        "type": "Feature",
        "geometry": {
            "type": "MultiPoint",
            "coordinates": coords
        },
        "properties": {
            "name": "UL",
            "code": 12
        }
    })
}

fn make_polygon_geojson(has_properties: bool, with_height: bool) -> serde_json::Value {
    let ring = if with_height {
        vec![
            vec![100.0, 0.0, 10.0],
            vec![101.0, 0.0, 10.0],
            vec![101.0, 1.0, 10.0],
            vec![100.0, 1.0, 10.0],
            vec![100.0, 0.0, 10.0],
        ]
    } else {
        vec![
            vec![100.0, 0.0],
            vec![101.0, 0.0],
            vec![101.0, 1.0],
            vec![100.0, 1.0],
            vec![100.0, 0.0],
        ]
    };

    let props = if has_properties {
        json!({
            "name": "UL",
            "code": 12
        })
    } else {
        json!({})
    };

    json!({
        "type": "Feature",
        "geometry": {
            "type": "Polygon",
            "coordinates": [ring]
        },
        "properties": props
    })
}

fn make_polygon_hole_geojson() -> serde_json::Value {
    let outer_ring = vec![
        vec![100.0, 0.0],
        vec![101.0, 0.0],
        vec![101.0, 1.0],
        vec![100.0, 1.0],
        vec![100.0, 0.0],
    ];
    let inner_ring = vec![
        vec![100.2, 0.2],
        vec![100.8, 0.2],
        vec![100.8, 0.8],
        vec![100.2, 0.8],
        vec![100.2, 0.2],
    ];

    json!({
        "type": "Feature",
        "geometry": {
            "type": "Polygon",
            "coordinates": [outer_ring, inner_ring]
        },
        "properties": {
            "name": "UL",
            "code": 12
        }
    })
}

fn make_multipolygon_geojson() -> serde_json::Value {
    let poly1 = vec![vec![
        vec![100.0, 0.0],
        vec![101.0, 0.0],
        vec![101.0, 1.0],
        vec![100.0, 1.0],
        vec![100.0, 0.0],
    ]];
    let poly2 = vec![vec![
        vec![102.0, 0.0],
        vec![103.0, 0.0],
        vec![103.0, 1.0],
        vec![102.0, 1.0],
        vec![102.0, 0.0],
    ]];

    json!({
        "type": "Feature",
        "geometry": {
            "type": "MultiPolygon",
            "coordinates": [poly1, poly2]
        },
        "properties": {
            "name": "UL",
            "code": 12
        }
    })
}

fn make_linestring_geojson() -> serde_json::Value {
    let line = vec![
        vec![100.0, 0.0],
        vec![101.0, 0.0],
        vec![101.0, 1.0],
        vec![100.0, 1.0],
        vec![100.0, 0.0],
    ];

    json!({
        "type": "Feature",
        "geometry": {
            "type": "LineString",
            "coordinates": line
        },
        "properties": {
            "name": "UL",
            "code": 12
        }
    })
}

fn make_multilinestring_geojson() -> serde_json::Value {
    let line1 = vec![
        vec![100.0, 0.0],
        vec![101.0, 0.0],
        vec![101.0, 1.0],
        vec![100.0, 1.0],
        vec![100.0, 0.0],
    ];
    let line2 = vec![
        vec![102.0, 0.0],
        vec![103.0, 0.0],
        vec![103.0, 1.0],
        vec![102.0, 1.0],
        vec![102.0, 0.0],
    ];

    json!({
        "type": "Feature",
        "geometry": {
            "type": "MultiLineString",
            "coordinates": [line1, line2]
        },
        "properties": {
            "name": "UL",
            "code": 12
        }
    })
}

fn make_multiple_features_geojson() -> serde_json::Value {
    let f1 = json!({
        "type": "Feature",
        "geometry": {
            "type": "LineString",
            "coordinates": [
                [100.0, 0.0],
                [101.0, 0.0],
                [101.0, 1.0],
                [100.0, 1.0],
                [100.0, 0.0]
            ]
        },
        "properties": {
            "name": "AL",
            "code": 11
        }
    });
    let f2 = json!({
        "type": "Feature",
        "geometry": {
            "type": "LineString",
            "coordinates": [
                [102.0, 0.0],
                [103.0, 0.0],
                [103.0, 1.0],
                [102.0, 1.0],
                [102.0, 0.0]
            ]
        },
        "properties": {
            "name": "UL",
            "code": 12
        }
    });

    json!({
        "type": "FeatureCollection",
        "features": [f1, f2]
    })
}

struct Expected {
    vertex_count: usize,
    index_count: usize,
    feature_count: usize,
    has_properties: bool,
}

fn test_loader(geo_json: serde_json::Value, expected: Expected) {
    let mut loader = GeoJsonLoader::new(GeoJsonLoaderOptions { geo_json })
        .expect("Loader creation should succeed");

    loader.load().expect("load should succeed");
    loader.process().expect("process should succeed");

    let components = loader.components().expect("components should exist");
    let scene = components.scene.as_ref().expect("scene should exist");
    let node_idx = scene.nodes[0];
    let root_node = &components.nodes[node_idx];
    let primitive = &root_node.primitives[0];
    let attributes = &primitive.attributes;

    let pos_attr = attributes
        .iter()
        .find(|a| a.semantic.as_deref() == Some("POSITION"))
        .expect("Position attribute should exist");

    let fid_attr = attributes
        .iter()
        .find(|a| a.semantic.as_deref() == Some("_FEATURE_ID_0"))
        .expect("FeatureId attribute should exist");

    assert!(pos_attr.buffer.is_some());
    assert_eq!(pos_attr.buffer.as_ref().unwrap().size_in_bytes, expected.vertex_count * 12);
    assert_eq!(pos_attr.count, expected.vertex_count);
    assert!(pos_attr.min.is_some());
    assert!(pos_attr.max.is_some());

    assert!(fid_attr.buffer.is_some());
    assert_eq!(fid_attr.buffer.as_ref().unwrap().size_in_bytes, expected.vertex_count * 4);
    assert_eq!(fid_attr.count, expected.vertex_count);

    if primitive.primitive_type == Some(PrimitiveType::Lines) {
        let indices = primitive.indices.as_ref().expect("Indices must exist");
        assert!(indices.buffer.is_some());
        assert_eq!(indices.buffer.as_ref().unwrap().size_in_bytes, expected.index_count * 2);
        assert_eq!(indices.count, expected.index_count);
        assert_eq!(indices.index_datatype, Some(IndexDatatype::UnsignedShort));
    }

    assert!(primitive.material.as_ref().unwrap().unlit);
    assert_eq!(primitive.feature_ids.len(), 1);

    if let FeatureId::Attribute(ref fid_acc) = primitive.feature_ids[0] {
        assert_eq!(fid_acc.feature_count, expected.feature_count);
        assert!(fid_acc.null_feature_id.is_none());
        assert_eq!(fid_acc.property_table_id, Some(0));
        assert_eq!(fid_acc.set_index, Some(0));
        assert_eq!(fid_acc.positional_label.as_deref(), Some("featureId_0"));
    } else {
        panic!("feature_ids[0] should be Attribute");
    }

    assert_ne!(components.transform, Matrix4::IDENTITY);

    let structural_metadata = loader
        .structural_metadata
        .as_ref()
        .expect("Structural metadata should exist");
    let prop_table = structural_metadata
        .get_property_table(0)
        .expect("PropertyTable 0 should exist");
    assert_eq!(prop_table.id, 0);
    assert_eq!(prop_table.count, expected.feature_count);

    if expected.has_properties && expected.feature_count == 1 {
        assert_eq!(prop_table.get_property(0, "name"), Some(json!("UL")));
        assert_eq!(prop_table.get_property(0, "code"), Some(json!(12)));
    } else if expected.has_properties && expected.feature_count == 2 {
        assert_eq!(prop_table.get_property(0, "name"), Some(json!("AL")));
        assert_eq!(prop_table.get_property(0, "code"), Some(json!(11)));
        assert_eq!(prop_table.get_property(1, "name"), Some(json!("UL")));
        assert_eq!(prop_table.get_property(1, "code"), Some(json!(12)));
    }
}

#[test]
fn test_loads_geojson_multipoint() {
    test_loader(
        make_multipoint_geojson(),
        Expected {
            vertex_count: 10,
            index_count: 0,
            feature_count: 1,
            has_properties: true,
        },
    );
}

#[test]
fn test_loads_geojson_point() {
    test_loader(
        make_point_geojson(),
        Expected {
            vertex_count: 1,
            index_count: 0,
            feature_count: 1,
            has_properties: true,
        },
    );
}

#[test]
fn test_loads_geojson_multipolygon() {
    test_loader(
        make_multipolygon_geojson(),
        Expected {
            vertex_count: 10,
            index_count: 16,
            feature_count: 1,
            has_properties: true,
        },
    );
}

#[test]
fn test_loads_geojson_polygon() {
    test_loader(
        make_polygon_geojson(true, false),
        Expected {
            vertex_count: 5,
            index_count: 8,
            feature_count: 1,
            has_properties: true,
        },
    );
}

#[test]
fn test_loads_geojson_polygon_with_heights() {
    test_loader(
        make_polygon_geojson(true, true),
        Expected {
            vertex_count: 5,
            index_count: 8,
            feature_count: 1,
            has_properties: true,
        },
    );
}

#[test]
fn test_loads_geojson_polygon_with_hole() {
    test_loader(
        make_polygon_hole_geojson(),
        Expected {
            vertex_count: 10,
            index_count: 16,
            feature_count: 1,
            has_properties: true,
        },
    );
}

#[test]
fn test_loads_geojson_polygon_with_no_properties() {
    test_loader(
        make_polygon_geojson(false, false),
        Expected {
            vertex_count: 5,
            index_count: 8,
            feature_count: 1,
            has_properties: false,
        },
    );
}

#[test]
fn test_loads_geojson_linestring() {
    test_loader(
        make_linestring_geojson(),
        Expected {
            vertex_count: 5,
            index_count: 8,
            feature_count: 1,
            has_properties: true,
        },
    );
}

#[test]
fn test_loads_geojson_multilinestring() {
    test_loader(
        make_multilinestring_geojson(),
        Expected {
            vertex_count: 10,
            index_count: 16,
            feature_count: 1,
            has_properties: true,
        },
    );
}

#[test]
fn test_loads_geojson_with_multiple_features() {
    test_loader(
        make_multiple_features_geojson(),
        Expected {
            vertex_count: 10,
            index_count: 16,
            feature_count: 2,
            has_properties: true,
        },
    );
}

#[test]
fn test_destroys_geojson_loader() {
    let mut loader = GeoJsonLoader::new(GeoJsonLoaderOptions {
        geo_json: make_polygon_geojson(true, false),
    })
    .expect("Loader creation should succeed");

    loader.process().expect("process should succeed");
    assert!(loader.components().is_some());
    assert!(!loader.is_destroyed());

    loader.destroy();
    assert!(loader.components().is_none());
    assert!(loader.is_destroyed());
}
