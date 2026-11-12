use cesium_renderer::create_vector_tile_buffers::{BufferPoint, BufferPolygon, BufferPolyline};
use cesium_renderer::geo_json_primitive::GeoJsonPrimitive;
use serde_json::json;

#[test]
fn builds_buffer_primitive_collections_from_mixed_feature_geometries() {
    let geo_json = json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "id": "feature-0",
                "properties": { "name": "mixed" },
                "geometry": {
                    "type": "GeometryCollection",
                    "geometries": [
                        {
                            "type": "MultiPoint",
                            "coordinates": [[0.0, 0.0], [1.0, 1.0]]
                        },
                        {
                            "type": "MultiLineString",
                            "coordinates": [
                                [[0.0, 0.0], [0.0, 1.0]],
                                [[1.0, 1.0], [2.0, 2.0], [3.0, 3.0]]
                            ]
                        },
                        {
                            "type": "Polygon",
                            "coordinates": [
                                [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0], [0.0, 0.0]],
                                [[0.5, 0.5], [1.5, 0.5], [1.5, 1.5], [0.5, 1.5], [0.5, 0.5]]
                            ]
                        }
                    ]
                }
            },
            {
                "type": "Feature",
                "id": 42,
                "properties": { "name": "multiPolygon" },
                "geometry": {
                    "type": "MultiPolygon",
                    "coordinates": [
                        [[[10.0, 0.0], [12.0, 0.0], [12.0, 2.0], [10.0, 2.0], [10.0, 0.0]]],
                        [[[20.0, 0.0], [21.0, 0.0], [21.0, 1.0], [20.0, 1.0], [20.0, 0.0]]]
                    ]
                }
            }
        ]
    });

    let loader = GeoJsonPrimitive::from_geo_json(&geo_json).unwrap();
    assert_eq!(loader.feature_count(), 2);
    assert_eq!(loader.get_id(0), Some(&json!("feature-0")));
    assert_eq!(loader.get_id(1), Some(&json!(42)));
    assert_eq!(loader.get_properties(0).unwrap()["name"], json!("mixed"));
    assert_eq!(
        loader.get_properties(1).unwrap()["name"],
        json!("multiPolygon")
    );

    let points = loader.points.as_ref().unwrap();
    let polylines = loader.polylines.as_ref().unwrap();
    let polygons = loader.polygons.as_ref().unwrap();

    assert_eq!(points.primitive_count(), 2);
    assert_eq!(polylines.primitive_count(), 2);
    assert_eq!(polygons.primitive_count(), 3);
    assert_eq!(polygons.hole_count(), 1);
    assert!(polygons.triangle_count() > 0);

    let mut point = BufferPoint::default();
    points.get(0, &mut point);
    assert_eq!(point.feature_id, 0);
    points.get(1, &mut point);
    assert_eq!(point.feature_id, 0);

    let mut polyline = BufferPolyline::default();
    polylines.get(0, &mut polyline);
    assert_eq!(polyline.feature_id, 0);
    polylines.get(1, &mut polyline);
    assert_eq!(polyline.feature_id, 0);

    let mut polygon = BufferPolygon::default();
    polygons.get(0, &mut polygon);
    assert_eq!(polygon.feature_id, 0);
    polygons.get(1, &mut polygon);
    assert_eq!(polygon.feature_id, 1);
    polygons.get(2, &mut polygon);
    assert_eq!(polygon.feature_id, 1);
}

#[test]
fn accepts_top_level_geometry_objects() {
    let geo_json = json!({
        "type": "MultiPoint",
        "coordinates": [[100.0, 0.0], [101.0, 1.0]]
    });
    let loader = GeoJsonPrimitive::from_geo_json(&geo_json).unwrap();
    assert_eq!(loader.feature_count(), 1);
    assert_eq!(loader.ids(), &[None]);
    assert_eq!(loader.points.as_ref().unwrap().primitive_count(), 2);
    assert!(loader.polylines.is_none());
    assert!(loader.polygons.is_none());
}

#[test]
fn throws_for_unsupported_top_level_types() {
    let invalid = json!({
        "type": "Topology",
        "objects": {}
    });
    assert!(GeoJsonPrimitive::from_geo_json(&invalid).is_err());
}

#[test]
fn from_url_rejects_without_a_url() {
    assert!(GeoJsonPrimitive::from_url(None).is_err());
    assert!(GeoJsonPrimitive::from_url(Some("")).is_err());
}
