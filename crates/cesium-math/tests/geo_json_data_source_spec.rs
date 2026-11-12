//! Direct port and verification against CesiumJS `GeoJsonDataSourceSpec.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use serde_json::json;

use cesium_math::{
    Cartesian3, Color, GeoJsonDataSource, GeoJsonLoadOptions, HeightReference, JulianDate,
    MaterialValue,
};

#[test]
fn default_constructor_has_expected_values() {
    let ds = GeoJsonDataSource::new(Some("myData"));
    assert_eq!(ds.name(), "myData");
    assert_eq!(ds.clock(), None);
    assert_eq!(ds.entities().values().len(), 0);
    assert!(!ds.is_loading());
    assert!(ds.show());
    assert_eq!(ds.credit(), None);
}

#[test]
fn credit_gets_set_from_options() {
    let mut ds = GeoJsonDataSource::default();
    let point = json!({ "type": "Point", "coordinates": [102.0, 0.5] });
    let opts = GeoJsonLoadOptions {
        credit: Some("Some Credit".to_string()),
        ..Default::default()
    };
    ds.load_value(&point, Some(opts)).unwrap();
    assert_eq!(ds.credit(), Some("Some Credit"));
}

#[test]
fn setting_name_raises_changed_event() {
    let mut ds = GeoJsonDataSource::default();
    let count = Rc::new(RefCell::new(0));
    let c = Rc::clone(&count);
    let _id = ds.changed_event.add_listener(move |_| {
        *c.borrow_mut() += 1;
    });

    ds.set_name("New Name");
    assert_eq!(ds.name(), "New Name");
    assert_eq!(*count.borrow(), 1);

    // Same name doesn't raise
    ds.set_name("New Name");
    assert_eq!(*count.borrow(), 1);
}

#[test]
fn show_sets_underlying_entity_collection_show() {
    let mut ds = GeoJsonDataSource::default();
    assert!(ds.show());
    assert!(ds.entities().show());

    ds.set_show(false);
    assert!(!ds.show());
    assert!(!ds.entities().show());
}

#[test]
fn works_with_null_geometry() {
    let mut ds = GeoJsonDataSource::default();
    let feature = json!({
        "type": "Feature",
        "geometry": null,
        "properties": { "name": "NullGeom" }
    });
    ds.load_value(&feature, None).unwrap();
    assert_eq!(ds.entities().values().len(), 1);
    let e = &ds.entities().values()[0];
    assert_eq!(e.name.as_deref(), Some("NullGeom"));
    assert!(e.position.is_none());
    assert!(e.billboard.is_none());
}

#[test]
fn adds_feature_without_removing_existing_entities() {
    let mut ds = GeoJsonDataSource::default();
    let f1 = json!({
        "type": "Feature",
        "id": "f1",
        "geometry": { "type": "Point", "coordinates": [102.0, 0.5] }
    });
    let f2 = json!({
        "type": "Feature",
        "id": "f2",
        "geometry": { "type": "Point", "coordinates": [103.0, 1.5] }
    });

    ds.load_value(&f1, None).unwrap();
    assert_eq!(ds.entities().values().len(), 1);

    ds.process_value(&f2, None).unwrap();
    assert_eq!(ds.entities().values().len(), 2);
}

#[test]
fn creates_default_and_custom_description_from_properties() {
    let mut ds = GeoJsonDataSource::default();
    let feature = json!({
        "type": "Feature",
        "geometry": { "type": "Point", "coordinates": [102.0, 0.5] },
        "properties": {
            "name": "TestPoint",
            "population": 1000
        }
    });

    ds.load_value(&feature, None).unwrap();
    let e = &ds.entities().values()[0];
    let desc = e.description.as_ref().unwrap().get_value(None).unwrap();
    assert!(desc.contains("<table"));
    assert!(desc.contains("TestPoint"));
    assert!(desc.contains("1000"));

    // Custom describe
    let opts = GeoJsonLoadOptions {
        describe: Some(Rc::new(|props, _e| {
            let pop = props.get("population").and_then(|v| v.as_i64()).unwrap_or(0);
            Some(format!("Pop: {}", pop))
        })),
        ..Default::default()
    };
    ds.load_value(&feature, Some(opts)).unwrap();
    let e2 = &ds.entities().values()[0];
    assert_eq!(
        e2.description.as_ref().unwrap().get_value(None).as_deref(),
        Some("Pop: 1000")
    );
}

#[test]
fn uses_description_if_present_and_handles_null_description() {
    let mut ds = GeoJsonDataSource::default();
    let f1 = json!({
        "type": "Feature",
        "geometry": { "type": "Point", "coordinates": [102.0, 0.5] },
        "properties": {
            "description": "Explicit Description",
            "other": "val"
        }
    });
    ds.load_value(&f1, None).unwrap();
    let e1 = &ds.entities().values()[0];
    assert_eq!(
        e1.description.as_ref().unwrap().get_value(None).as_deref(),
        Some("Explicit Description")
    );

    let f2 = json!({
        "type": "Feature",
        "geometry": { "type": "Point", "coordinates": [102.0, 0.5] },
        "properties": {
            "description": null,
            "name": null
        }
    });
    ds.load_value(&f2, None).unwrap();
    let e2 = &ds.entities().values()[0];
    assert!(e2.description.is_none());
    assert!(e2.name.is_none());
}

#[test]
fn works_with_feature_id_null_id_and_null_properties() {
    let mut ds = GeoJsonDataSource::default();
    let fc = json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "id": "custom-id-1",
                "geometry": { "type": "Point", "coordinates": [10.0, 20.0] },
                "properties": null
            },
            {
                "type": "Feature",
                "id": null,
                "geometry": { "type": "Point", "coordinates": [11.0, 21.0] },
                "properties": { "title": "TitleName" }
            }
        ]
    });

    ds.load_value(&fc, None).unwrap();
    assert_eq!(ds.entities().values().len(), 2);
    assert!(ds.entities().get_by_id("custom-id-1").is_some());
    assert_eq!(
        ds.entities().values()[1].name.as_deref(),
        Some("TitleName")
    );
}

#[test]
fn works_with_point_clamped_and_simplestyle() {
    let time = JulianDate::now();
    let mut ds = GeoJsonDataSource::default();

    let f = json!({
        "type": "Feature",
        "geometry": { "type": "Point", "coordinates": [102.0, 0.5, 100.0] },
        "properties": {
            "marker-size": "large",
            "marker-symbol": "star",
            "marker-color": "#ff0000"
        }
    });

    let opts = GeoJsonLoadOptions {
        clamp_to_ground: Some(true),
        ..Default::default()
    };

    ds.load_value(&f, Some(opts)).unwrap();
    let e = &ds.entities().values()[0];

    let expected_pos = Cartesian3::from_degrees(102.0, 0.5, Some(100.0), None).unwrap();
    assert_eq!(
        e.position.as_ref().unwrap().get_value(Some(&time)),
        Some(expected_pos)
    );

    let bb = e.billboard.as_ref().unwrap();
    assert_eq!(
        bb.width.as_ref().unwrap().get_value(Some(&time)),
        Some(64.0)
    );
    assert_eq!(
        bb.color.as_ref().unwrap().get_value(Some(&time)),
        Some(Color::RED)
    );
    assert_eq!(
        bb.height_reference.as_ref().unwrap().get_value(Some(&time)),
        Some(HeightReference::ClampToGround)
    );
}

#[test]
fn works_with_multipoint_linestring_and_multilinestring() {
    let time = JulianDate::now();
    let mut ds = GeoJsonDataSource::default();

    let mp = json!({
        "type": "MultiPoint",
        "coordinates": [[100.0, 0.0], [101.0, 1.0]]
    });
    ds.load_value(&mp, None).unwrap();
    assert_eq!(ds.entities().values().len(), 2);

    let ls = json!({
        "type": "Feature",
        "geometry": {
            "type": "LineString",
            "coordinates": [[100.0, 0.0], [101.0, 1.0]]
        },
        "properties": {
            "stroke": "#00ff00",
            "stroke-width": 5,
            "stroke-opacity": 0.5
        }
    });
    let opts = GeoJsonLoadOptions {
        clamp_to_ground: Some(true),
        ..Default::default()
    };
    ds.load_value(&ls, Some(opts)).unwrap();
    assert_eq!(ds.entities().values().len(), 1);
    let poly = ds.entities().values()[0].polyline.as_ref().unwrap();
    assert_eq!(
        poly.width.as_ref().unwrap().get_value(Some(&time)),
        Some(5.0)
    );
    assert_eq!(
        poly.clamp_to_ground.as_ref().unwrap().get_value(Some(&time)),
        Some(true)
    );
    let mat_val = poly
        .material
        .as_ref()
        .unwrap()
        .get_material_value(Some(&time));
    assert_eq!(
        mat_val,
        Some(MaterialValue::Color(Color::new(0.0, 1.0, 0.0, 0.5)))
    );

    let mls = json!({
        "type": "MultiLineString",
        "coordinates": [
            [[100.0, 0.0], [101.0, 1.0]],
            [[102.0, 2.0], [103.0, 3.0]]
        ]
    });
    ds.load_value(&mls, None).unwrap();
    assert_eq!(ds.entities().values().len(), 2);
}

#[test]
fn works_with_polygon_holes_heights_and_multipolygon() {
    let time = JulianDate::now();
    let mut ds = GeoJsonDataSource::default();

    let poly_holes = json!({
        "type": "Feature",
        "geometry": {
            "type": "Polygon",
            "coordinates": [
                [[100.0, 0.0], [101.0, 0.0], [101.0, 1.0], [100.0, 1.0], [100.0, 0.0]],
                [[100.2, 0.2], [100.8, 0.2], [100.8, 0.8], [100.2, 0.8], [100.2, 0.2]]
            ]
        },
        "properties": {
            "fill": null,
            "fill-opacity": 0.25,
            "stroke": null,
            "stroke-opacity": 0.75
        }
    });

    ds.load_value(&poly_holes, None).unwrap();
    assert_eq!(ds.entities().values().len(), 1);
    let pg = ds.entities().values()[0].polygon.as_ref().unwrap();
    let hierarchy = pg.hierarchy.as_ref().unwrap().get_value(Some(&time)).unwrap();
    assert_eq!(hierarchy.positions.len(), 5);
    assert_eq!(hierarchy.holes.len(), 1);
    assert_eq!(hierarchy.holes[0].positions.len(), 5);

    // Check fill-opacity applied to default fill
    let fill_mat = pg.material.as_ref().unwrap().get_material_value(Some(&time));
    match fill_mat {
        Some(MaterialValue::Color(c)) => assert!((c.alpha - 0.25).abs() < 1e-6),
        _ => panic!("Expected Color material"),
    }

    // Check height polygon sets per_position_height
    let poly_heights = json!({
        "type": "Polygon",
        "coordinates": [
            [[100.0, 0.0, 10.0], [101.0, 0.0, 20.0], [101.0, 1.0, 30.0], [100.0, 0.0, 10.0]]
        ]
    });
    ds.load_value(&poly_heights, None).unwrap();
    let pg2 = ds.entities().values()[0].polygon.as_ref().unwrap();
    assert_eq!(
        pg2.per_position_height
            .as_ref()
            .unwrap()
            .get_value(Some(&time)),
        Some(true)
    );

    // MultiPolygon
    let mp = json!({
        "type": "MultiPolygon",
        "coordinates": [
            [[[100.0, 0.0], [101.0, 0.0], [101.0, 1.0], [100.0, 0.0]]],
            [[[102.0, 2.0], [103.0, 2.0], [103.0, 3.0], [102.0, 2.0]]]
        ]
    });
    ds.load_value(&mp, None).unwrap();
    assert_eq!(ds.entities().values().len(), 2);
}

#[test]
fn works_with_topojson_and_geometry_collection() {
    let mut ds = GeoJsonDataSource::default();

    let topo = json!({
        "type": "Topology",
        "transform": {
            "scale": [0.001, 0.001],
            "translate": [100.0, 0.0]
        },
        "objects": {
            "collection": {
                "type": "GeometryCollection",
                "geometries": [
                    {
                        "type": "Polygon",
                        "arcs": [[0]],
                        "properties": { "name": "TopoPoly" }
                    },
                    {
                        "type": "LineString",
                        "arcs": [0],
                        "properties": { "name": "TopoLine" }
                    }
                ]
            }
        },
        "arcs": [
            [[0, 0], [1000, 0], [0, 1000], [-1000, 0], [0, -1000]]
        ]
    });

    ds.load_value(&topo, None).unwrap();
    assert_eq!(ds.entities().values().len(), 2);
    assert_eq!(ds.entities().values()[0].name.as_deref(), Some("TopoPoly"));
    assert!(ds.entities().values()[0].polygon.is_some());
    assert_eq!(ds.entities().values()[1].name.as_deref(), Some("TopoLine"));
    assert!(ds.entities().values()[1].polyline.is_some());

    // GeometryCollection
    let gc = json!({
        "type": "GeometryCollection",
        "geometries": [
            { "type": "Point", "coordinates": [100.0, 0.0] },
            { "type": "LineString", "coordinates": [[100.0, 0.0], [101.0, 1.0]] }
        ]
    });
    ds.load_value(&gc, None).unwrap();
    assert_eq!(ds.entities().values().len(), 2);
}

#[test]
fn works_with_crs_variants_and_rejects_invalid_crs() {
    let mut ds = GeoJsonDataSource::default();

    for crs in [
        json!({ "type": "name", "properties": { "name": "EPSG:4326" } }),
        json!({ "type": "name", "properties": { "name": "urn:ogc:def:crs:OGC:1.3:CRS84" } }),
        json!({ "type": "name", "properties": { "name": "urn:ogc:def:crs:EPSG::4326" } }),
        json!({ "type": "EPSG", "properties": { "code": 4326 } }),
    ] {
        let pt = json!({ "type": "Point", "coordinates": [102.0, 0.5], "crs": crs });
        assert!(ds.load_value(&pt, None).is_ok());
    }

    // Link CRS with handler
    let link_pt = json!({
        "type": "Point",
        "coordinates": [102.0, 0.5],
        "crs": { "type": "link", "properties": { "href": "http://crs.example" } }
    });
    let opts = GeoJsonLoadOptions {
        crs_link_handler: Some(Rc::new(|_href| {
            Ok(Rc::new(|coords: &[f64]| {
                Cartesian3::from_degrees(coords[0], coords[1], None, None)
                    .map_err(|_| cesium_math::DeveloperError::new("err"))
            }))
        })),
        ..Default::default()
    };
    assert!(ds.load_value(&link_pt, Some(opts)).is_ok());

    // Invalid CRS cases
    let invalid_cases = [
        json!({ "type": "Point", "coordinates": [102.0, 0.5], "crs": null }),
        json!({ "type": "Point", "coordinates": [102.0, 0.5], "crs": { "type": "unknown", "properties": {} } }),
        json!({ "type": "Point", "coordinates": [102.0, 0.5], "crs": { "type": "name" } }),
        json!({ "type": "Point", "coordinates": [102.0, 0.5], "crs": { "type": "name", "properties": { "name": "EPSG:9999" } } }),
        json!({ "type": "Point", "coordinates": [102.0, 0.5], "crs": { "type": "link", "properties": { "href": "http://unknown" } } }),
    ];

    for bad in invalid_cases {
        assert!(ds.load_value(&bad, None).is_err());
    }
}

#[test]
fn rejects_unknown_geometry_and_raises_loading_and_error_events() {
    let mut ds = GeoJsonDataSource::default();
    let loading_events = Rc::new(RefCell::new(Vec::new()));
    let error_events = Rc::new(RefCell::new(Vec::new()));

    let le = Rc::clone(&loading_events);
    let _l_id = ds.loading_event.add_listener(move |v| {
        le.borrow_mut().push(*v);
    });
    let ee = Rc::clone(&error_events);
    let _e_id = ds.error_event.add_listener(move |msg| {
        ee.borrow_mut().push(msg.clone());
    });

    let unknown = json!({ "type": "UnknownType", "coordinates": [0.0, 0.0] });
    assert!(ds.load_value(&unknown, None).is_err());
    assert_eq!(*loading_events.borrow(), vec![true, false]);
    assert_eq!(error_events.borrow().len(), 1);

    assert!(ds.load_str("not valid json", None).is_err());
    assert_eq!(error_events.borrow().len(), 2);
}
