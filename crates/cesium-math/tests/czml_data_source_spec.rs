use cesium_math::{
    Cartesian3, Cartographic, Color, CzmlDataSource, CzmlLoadOptions, Ellipsoid,
    HeightReference, HorizontalOrigin, JulianDate, LabelStyle, NearFarScalar,
    PolylineDashMaterialProperty, PolylineGlowMaterialProperty, PolylineOutlineMaterialProperty,
    ReferenceFrame, VerticalOrigin,
};
use serde_json::json;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_default_constructor_and_show() {
    let mut ds = CzmlDataSource::new(None);
    assert_eq!(ds.name(), "");
    assert!(ds.show());
    assert!(!ds.is_loading());
    assert!(ds.clock().is_none());
    assert_eq!(ds.entities().values().len(), 0);

    ds.set_name("My CZML");
    assert_eq!(ds.name(), "My CZML");

    ds.set_show(false);
    assert!(!ds.show());
    assert!(!ds.entities().show());
}

#[test]
fn test_document_packet_name_and_clock() {
    let czml = json!([
        {
            "id": "document",
            "name": "Simple Document",
            "version": "1.0",
            "clock": {
                "interval": "2012-03-15T10:00:00Z/2012-03-16T10:00:00Z",
                "currentTime": "2012-03-15T10:00:00Z",
                "multiplier": 60,
                "range": "LOOP_STOP",
                "step": "SYSTEM_CLOCK_MULTIPLIER"
            }
        }
    ]);

    let mut ds = CzmlDataSource::new(None);
    ds.load_value(&czml, None).unwrap();

    assert_eq!(ds.name(), "Simple Document");
    let clock = ds.clock().expect("clock should be set");
    assert_eq!(
        clock.start_time,
        Some(JulianDate::from_iso8601("2012-03-15T10:00:00Z").unwrap())
    );
    assert_eq!(
        clock.stop_time,
        Some(JulianDate::from_iso8601("2012-03-16T10:00:00Z").unwrap())
    );
    assert_eq!(
        clock.current_time,
        Some(JulianDate::from_iso8601("2012-03-15T10:00:00Z").unwrap())
    );
    assert_eq!(clock.multiplier, Some(60.0));
}

#[test]
fn test_fallback_availability_clock() {
    let czml = json!([
        {
            "id": "document",
            "version": "1.0"
        },
        {
            "id": "entity1",
            "availability": "2012-08-04T16:00:00Z/2012-08-04T16:05:00Z"
        }
    ]);

    let mut ds = CzmlDataSource::new(None);
    ds.load_value(&czml, None).unwrap();

    let clock = ds.clock().expect("fallback clock should be computed from availability");
    assert_eq!(
        clock.start_time,
        Some(JulianDate::from_iso8601("2012-08-04T16:00:00Z").unwrap())
    );
    assert_eq!(
        clock.stop_time,
        Some(JulianDate::from_iso8601("2012-08-04T16:05:00Z").unwrap())
    );
}

#[test]
fn test_load_vs_process_and_delete_packets() {
    let czml1 = json!([
        { "id": "document", "version": "1.0", "name": "Doc1" },
        { "id": "e1", "name": "Entity 1" },
        { "id": "e2", "name": "Entity 2" }
    ]);
    let czml2 = json!([
        { "id": "e2", "delete": true },
        { "id": "e3", "name": "Entity 3" }
    ]);

    let mut ds = CzmlDataSource::new(None);
    ds.load_value(&czml1, None).unwrap();
    assert_eq!(ds.entities().values().len(), 2);

    // process appends/updates/deletes without clearing existing entities
    ds.process_value(&czml2, None).unwrap();
    assert_eq!(ds.entities().values().len(), 2);
    assert!(ds.entities().get_by_id("e1").is_some());
    assert!(ds.entities().get_by_id("e2").is_none());
    assert!(ds.entities().get_by_id("e3").is_some());

    // load clears previous entities
    ds.load_value(&czml1, None).unwrap();
    assert_eq!(ds.entities().values().len(), 2);
    assert!(ds.entities().get_by_id("e3").is_none());
}

#[test]
fn test_events_changed_loading_and_error() {
    let mut ds = CzmlDataSource::new(None);
    let changed_count = Rc::new(RefCell::new(0));
    let loading_states = Rc::new(RefCell::new(Vec::new()));
    let error_count = Rc::new(RefCell::new(0));

    {
        let c = changed_count.clone();
        ds.changed_event.add_listener(move |_| {
            *c.borrow_mut() += 1;
        });
    }
    {
        let l = loading_states.clone();
        ds.loading_event.add_listener(move |&state| {
            l.borrow_mut().push(state);
        });
    }
    {
        let e = error_count.clone();
        ds.error_event.add_listener(move |_| {
            *e.borrow_mut() += 1;
        });
    }

    ds.load_str(r#"[{"id":"document","name":"EvtTest"}]"#, None).unwrap();
    assert_eq!(*changed_count.borrow(), 1);
    assert_eq!(*loading_states.borrow(), vec![true, false]);
    assert_eq!(*error_count.borrow(), 0);

    let err = ds.load_str("{not valid json", None);
    assert!(err.is_err());
    assert_eq!(*error_count.borrow(), 1);
    assert_eq!(*loading_states.borrow(), vec![true, false, true, false]);
}

#[test]
fn test_position_constant_and_sampled_and_custom_ellipsoid() {
    let czml = json!([
        { "id": "document", "version": "1.0" },
        {
            "id": "cart_const",
            "position": { "cartesian": [100.0, 200.0, 300.0] }
        },
        {
            "id": "deg_const",
            "position": { "cartographicDegrees": [0.0, 0.0, 10.0] }
        },
        {
            "id": "rad_const",
            "position": { "cartographicRadians": [0.0, 0.0, 20.0] }
        },
        {
            "id": "sampled_pos",
            "position": {
                "epoch": "2012-08-04T16:00:00Z",
                "referenceFrame": "INERTIAL",
                "cartesian": [
                    0, 0.0, 0.0, 0.0,
                    10, 100.0, 200.0, 300.0
                ]
            }
        }
    ]);

    let custom_ellipsoid = Ellipsoid::UNIT_SPHERE;
    let opts = CzmlLoadOptions {
        ellipsoid: Some(custom_ellipsoid),
        ..Default::default()
    };

    let mut ds = CzmlDataSource::new(None);
    ds.load_value(&czml, Some(opts)).unwrap();

    let t0 = JulianDate::from_iso8601("2012-08-04T16:00:00Z").unwrap();
    let t5 = t0.add_seconds(5.0);

    let e_cart = ds.entities().get_by_id("cart_const").unwrap();
    let pos_cart = e_cart.position.as_ref().unwrap().get_value(Some(&t0)).unwrap();
    assert!((pos_cart.x - 100.0).abs() < 1e-9);
    assert!((pos_cart.y - 200.0).abs() < 1e-9);
    assert!((pos_cart.z - 300.0).abs() < 1e-9);

    let e_deg = ds.entities().get_by_id("deg_const").unwrap();
    let expected_deg = custom_ellipsoid
        .cartographic_to_cartesian(Cartographic::new(0.0, 0.0, 10.0))
        .unwrap();
    let pos_deg = e_deg.position.as_ref().unwrap().get_value(Some(&t0)).unwrap();
    assert!((pos_deg.x - expected_deg.x).abs() < 1e-9);

    let e_rad = ds.entities().get_by_id("rad_const").unwrap();
    let expected_rad = custom_ellipsoid
        .cartographic_to_cartesian(Cartographic::new(0.0, 0.0, 20.0))
        .unwrap();
    let pos_rad = e_rad.position.as_ref().unwrap().get_value(Some(&t0)).unwrap();
    assert!((pos_rad.x - expected_rad.x).abs() < 1e-9);

    let e_sampled = ds.entities().get_by_id("sampled_pos").unwrap();
    let pos_prop = e_sampled.position.as_ref().unwrap();
    assert_eq!(pos_prop.reference_frame(), ReferenceFrame::Inertial);
    let pos_mid = pos_prop
        .get_value_in_reference_frame(Some(&t5), ReferenceFrame::Inertial)
        .unwrap();
    assert!((pos_mid.x - 50.0).abs() < 1e-6);
    assert!((pos_mid.y - 100.0).abs() < 1e-6);
    assert!((pos_mid.z - 150.0).abs() < 1e-6);
}

#[test]
fn test_billboard_label_point_graphics() {
    let czml = json!([
        { "id": "document", "version": "1.0" },
        {
            "id": "marker",
            "billboard": {
                "image": "https://example.com/icon.png",
                "scale": 1.5,
                "color": [
                    {
                        "interval": "2012-08-04T16:00:00Z/2012-08-04T16:01:00Z",
                        "rgba": [255, 0, 0, 255]
                    },
                    {
                        "interval": "2012-08-04T16:01:00Z/2012-08-04T16:02:00Z",
                        "rgba": [0, 255, 0, 255]
                    }
                ],
                "scaleByDistance": { "nearFarScalar": [100.0, 2.0, 1000.0, 0.5] },
                "horizontalOrigin": "LEFT",
                "verticalOrigin": "BOTTOM",
                "heightReference": "CLAMP_TO_GROUND",
                "show": true
            },
            "label": {
                "text": "Hello CZML",
                "font": "20px sans-serif",
                "style": "FILL_AND_OUTLINE",
                "fillColor": { "rgba": [255, 255, 0, 255] },
                "outlineColor": { "rgba": [0, 0, 0, 255] },
                "outlineWidth": 2.0
            },
            "point": {
                "color": { "rgbaf": [0.0, 0.5, 1.0, 1.0] },
                "pixelSize": 12.0,
                "outlineWidth": 3.0
            }
        }
    ]);

    let mut ds = CzmlDataSource::new(None);
    ds.load_value(&czml, None).unwrap();

    let t1 = JulianDate::from_iso8601("2012-08-04T16:00:30Z").unwrap();
    let t2 = JulianDate::from_iso8601("2012-08-04T16:01:30Z").unwrap();

    let entity = ds.entities().get_by_id("marker").unwrap();
    let bb = entity.billboard.as_ref().unwrap();
    assert_eq!(
        bb.image.as_ref().unwrap().get_value(Some(&t1)).as_deref(),
        Some("https://example.com/icon.png")
    );
    assert_eq!(bb.scale.as_ref().unwrap().get_value(Some(&t1)), Some(1.5));
    assert_eq!(bb.color.as_ref().unwrap().get_value(Some(&t1)), Some(Color::RED));
    assert_eq!(bb.color.as_ref().unwrap().get_value(Some(&t2)), Some(Color::LIME));
    assert_eq!(
        bb.scale_by_distance.as_ref().unwrap().get_value(Some(&t1)),
        Some(NearFarScalar::new(100.0, 2.0, 1000.0, 0.5))
    );
    assert_eq!(
        bb.horizontal_origin.as_ref().unwrap().get_value(Some(&t1)),
        Some(HorizontalOrigin::Left)
    );
    assert_eq!(
        bb.vertical_origin.as_ref().unwrap().get_value(Some(&t1)),
        Some(VerticalOrigin::Bottom)
    );
    assert_eq!(
        bb.height_reference.as_ref().unwrap().get_value(Some(&t1)),
        Some(HeightReference::ClampToGround)
    );

    let lbl = entity.label.as_ref().unwrap();
    assert_eq!(
        lbl.text.as_ref().unwrap().get_value(Some(&t1)).as_deref(),
        Some("Hello CZML")
    );
    assert_eq!(
        lbl.style.as_ref().unwrap().get_value(Some(&t1)),
        Some(LabelStyle::FillAndOutline)
    );
    assert_eq!(lbl.outline_width.as_ref().unwrap().get_value(Some(&t1)), Some(2.0));

    let pt = entity.point.as_ref().unwrap();
    assert_eq!(pt.pixel_size.as_ref().unwrap().get_value(Some(&t1)), Some(12.0));
    assert_eq!(pt.outline_width.as_ref().unwrap().get_value(Some(&t1)), Some(3.0));
    let pt_color = pt.color.as_ref().unwrap().get_value(Some(&t1)).unwrap();
    assert!((pt_color.green - 0.5).abs() < 1e-6);
}

#[test]
fn test_polyline_materials_and_polygon_ellipse_box_path_properties() {
    let czml = json!([
        { "id": "document", "version": "1.0" },
        {
            "id": "poly_outline",
            "polyline": {
                "positions": { "cartesian": [0, 0, 0, 100, 0, 0] },
                "width": 5,
                "material": {
                    "polylineOutline": {
                        "color": { "rgba": [255, 0, 0, 255] },
                        "outlineColor": { "rgba": [0, 0, 255, 255] },
                        "outlineWidth": 2
                    }
                }
            }
        },
        {
            "id": "poly_glow",
            "polyline": {
                "positions": { "cartesian": [0, 0, 0, 100, 0, 0] },
                "material": {
                    "polylineGlow": {
                        "color": { "rgba": [0, 255, 0, 255] },
                        "glowPower": 0.25,
                        "taperPower": 0.5
                    }
                }
            }
        },
        {
            "id": "poly_dash",
            "polyline": {
                "positions": { "cartesian": [0, 0, 0, 100, 0, 0] },
                "material": {
                    "polylineDash": {
                        "color": { "rgba": [255, 255, 255, 255] },
                        "dashLength": 16,
                        "dashPattern": 255
                    }
                }
            }
        },
        {
            "id": "pgon",
            "polygon": {
                "positions": { "cartographicDegrees": [0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0] },
                "holes": {
                    "cartographicDegrees": [
                        [0.2, 0.2, 0, 0.8, 0.2, 0, 0.8, 0.8, 0, 0.2, 0.8, 0]
                    ]
                },
                "height": 100.0,
                "extrudedHeight": 200.0,
                "material": { "solidColor": { "color": { "rgba": [0, 0, 255, 128] } } }
            }
        },
        {
            "id": "ell",
            "ellipse": {
                "semiMajorAxis": 500.0,
                "semiMinorAxis": 300.0,
                "rotation": 0.5,
                "material": { "solidColor": { "color": { "rgba": [255, 0, 0, 255] } } }
            }
        },
        {
            "id": "bx",
            "box": {
                "dimensions": { "cartesian": [10.0, 20.0, 30.0] },
                "fill": true,
                "outline": true
            }
        },
        {
            "id": "pth",
            "path": {
                "width": 3.0,
                "leadTime": 60.0,
                "trailTime": 120.0,
                "resolution": 5.0
            },
            "properties": {
                "customNum": 42,
                "customStr": "hello"
            }
        }
    ]);

    let mut ds = CzmlDataSource::new(None);
    ds.load_value(&czml, None).unwrap();
    let t = JulianDate::now();

    let po = ds.entities().get_by_id("poly_outline").unwrap().polyline.as_ref().unwrap();
    assert_eq!(po.width.as_ref().unwrap().get_value(Some(&t)), Some(5.0));
    let po_mat = po.material.as_ref().unwrap();
    assert!(po_mat.as_any().downcast_ref::<PolylineOutlineMaterialProperty>().is_some());

    let pg = ds.entities().get_by_id("poly_glow").unwrap().polyline.as_ref().unwrap();
    assert!(pg.material.as_ref().unwrap().as_any().downcast_ref::<PolylineGlowMaterialProperty>().is_some());

    let pd = ds.entities().get_by_id("poly_dash").unwrap().polyline.as_ref().unwrap();
    assert!(pd.material.as_ref().unwrap().as_any().downcast_ref::<PolylineDashMaterialProperty>().is_some());

    let pgon = ds.entities().get_by_id("pgon").unwrap().polygon.as_ref().unwrap();
    assert_eq!(pgon.height.as_ref().unwrap().get_value(Some(&t)), Some(100.0));
    assert_eq!(pgon.extruded_height.as_ref().unwrap().get_value(Some(&t)), Some(200.0));
    let hier = pgon.hierarchy.as_ref().unwrap().get_value(Some(&t)).unwrap();
    assert_eq!(hier.positions.len(), 4);
    assert_eq!(hier.holes.len(), 1);
    assert_eq!(hier.holes[0].positions.len(), 4);

    let ell = ds.entities().get_by_id("ell").unwrap().ellipse.as_ref().unwrap();
    assert_eq!(ell.semi_major_axis.as_ref().unwrap().get_value(Some(&t)), Some(500.0));
    assert_eq!(ell.semi_minor_axis.as_ref().unwrap().get_value(Some(&t)), Some(300.0));
    assert_eq!(ell.rotation.as_ref().unwrap().get_value(Some(&t)), Some(0.5));

    let bx = ds.entities().get_by_id("bx").unwrap().box_graphics.as_ref().unwrap();
    assert_eq!(
        bx.dimensions.as_ref().unwrap().get_value(Some(&t)),
        Some(Cartesian3::new(10.0, 20.0, 30.0))
    );
    assert_eq!(bx.outline.as_ref().unwrap().get_value(Some(&t)), Some(true));

    let pth_ent = ds.entities().get_by_id("pth").unwrap();
    let pth = pth_ent.path.as_ref().unwrap();
    assert_eq!(pth.width.as_ref().unwrap().get_value(Some(&t)), Some(3.0));
    assert_eq!(pth.lead_time.as_ref().unwrap().get_value(Some(&t)), Some(60.0));
    assert_eq!(pth.trail_time.as_ref().unwrap().get_value(Some(&t)), Some(120.0));
    assert_eq!(pth.resolution.as_ref().unwrap().get_value(Some(&t)), Some(5.0));

    let props = pth_ent.properties.get_value(Some(&t));
    assert_eq!(props.get("customNum").and_then(|v| v.as_ref()), Some(&json!(42)));
    assert_eq!(props.get("customStr").and_then(|v| v.as_ref()), Some(&json!("hello")));
}
