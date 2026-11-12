use cesium_math::{
    Cartesian3, Color, ColorMaterialProperty, HeightReference, Iso8601, JulianDate, KmlDataSource,
    KmlLoadOptions, Rectangle,
};
use serde_json::json;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_default_constructor_and_events() {
    let mut ds = KmlDataSource::new(None);
    assert!(ds.name().is_none());
    assert!(ds.clock().is_none());
    assert!(!ds.is_loading());
    assert!(ds.show());
    assert!(ds.update(&Iso8601::minimum_value()));

    let changed = Rc::new(RefCell::new(0));
    let loading = Rc::new(RefCell::new(Vec::new()));
    let errors = Rc::new(RefCell::new(0));
    let unsupported = Rc::new(RefCell::new(Vec::new()));

    {
        let c = changed.clone();
        ds.changed_event.add_listener(move |_| *c.borrow_mut() += 1);
    }
    {
        let l = loading.clone();
        ds.loading_event.add_listener(move |&s| l.borrow_mut().push(s));
    }
    {
        let e = errors.clone();
        ds.error_event.add_listener(move |_| *e.borrow_mut() += 1);
    }
    {
        let u = unsupported.clone();
        ds.unsupported_node_event
            .add_listener(move |n| u.borrow_mut().push(n.clone()));
    }

    ds.set_name("KML Test");
    assert_eq!(*changed.borrow(), 1);

    ds.set_show(false);
    assert!(!ds.show());
    assert!(!ds.entities().show());

    assert!(ds.load_str("", None).is_err());
    assert_eq!(*errors.borrow(), 1);
    assert!(ds.load_str("<gpx/>", None).is_err());
    assert_eq!(*errors.borrow(), 2);

    let kml = r#"<kml><Document><UnknownTag/></Document></kml>"#;
    ds.load_str(kml, None).unwrap();
    assert_eq!(*unsupported.borrow(), vec!["UnknownTag".to_string()]);
    assert_eq!(*loading.borrow(), vec![true, false, true, false]);
}

#[test]
fn test_document_name_source_uri_and_features() {
    let kml = r#"<kml xmlns="http://www.opengis.net/kml/2.2">
        <Document>
            <name>Doc Name</name>
            <Placemark id="p1">
                <name>Place 1</name>
                <Snippet>Short snippet</Snippet>
                <address>123 Main St</address>
                <phoneNumber>555-1234</phoneNumber>
                <visibility>0</visibility>
                <ExtendedData>
                    <Data name="role"><value>admin</value></Data>
                </ExtendedData>
                <Point><coordinates>10,20,30</coordinates></Point>
            </Placemark>
            <Placemark id="p1">
                <name>Duplicate ID</name>
                <visibility>true</visibility>
                <Point><coordinates>11,21</coordinates></Point>
            </Placemark>
        </Document>
    </kml>"#;

    let mut ds = KmlDataSource::new(None);
    ds.load_str(kml, None).unwrap();
    assert_eq!(ds.name(), Some("Doc Name"));
    assert_eq!(ds.entities().values().len(), 2);

    let p1 = ds.entities().get_by_id("p1").unwrap();
    assert!(!p1.show);
    assert_eq!(p1.name.as_deref(), Some("Place 1"));
    let desc = p1.description.as_ref().unwrap().get_value(None).unwrap();
    assert!(desc.contains("Short snippet"));
    assert!(desc.contains("123 Main St"));
    assert!(desc.contains("555-1234"));
    let props = p1.properties.get_value(None);
    assert_eq!(props.get("role").and_then(|v| v.as_ref()), Some(&json!("admin")));

    let p2 = ds.entities().values()[1];
    assert_ne!(p2.id, "p1");
    assert!(p2.show);

    // Fallback to sourceUri when no Document name
    let kml_no_name = r#"<kml><Placemark><Point><coordinates>0,0</coordinates></Point></Placemark></kml>"#;
    ds.load_str(
        kml_no_name,
        Some(KmlLoadOptions {
            source_uri: Some("path/to/my_file.kml".to_string()),
            ..Default::default()
        }),
    )
    .unwrap();
    assert_eq!(ds.name(), Some("my_file.kml"));
}

#[test]
fn test_timespan_timestamp_and_clock() {
    let kml = r#"<kml>
        <Document>
            <Placemark id="flipped">
                <TimeSpan>
                    <begin>2020-01-02T00:00:00Z</begin>
                    <end>2020-01-01T00:00:00Z</end>
                </TimeSpan>
            </Placemark>
            <Placemark id="begin_only">
                <TimeSpan><begin>2020-01-03T00:00:00Z</begin></TimeSpan>
            </Placemark>
            <Placemark id="stamp">
                <TimeStamp><when>2020-01-04T00:00:00Z</when></TimeStamp>
            </Placemark>
        </Document>
    </kml>"#;

    let mut ds = KmlDataSource::new(None);
    ds.load_str(kml, None).unwrap();

    let t1 = JulianDate::from_iso8601("2020-01-01T00:00:00Z").unwrap();
    let t2 = JulianDate::from_iso8601("2020-01-02T00:00:00Z").unwrap();
    let t4 = JulianDate::from_iso8601("2020-01-04T00:00:00Z").unwrap();

    let flipped = ds.entities().get_by_id("flipped").unwrap();
    let avail = flipped.availability.as_ref().unwrap();
    assert_eq!(avail.start(), Some(t1));
    assert_eq!(avail.stop(), Some(t2));

    let begin_only = ds.entities().get_by_id("begin_only").unwrap();
    assert_eq!(
        begin_only.availability.as_ref().unwrap().stop(),
        Some(Iso8601::maximum_value())
    );

    let stamp = ds.entities().get_by_id("stamp").unwrap();
    assert_eq!(stamp.availability.as_ref().unwrap().start(), Some(t4));
    assert_eq!(stamp.availability.as_ref().unwrap().stop(), Some(t4));

    let clk = ds.clock().expect("clock should be computed from availability");
    assert_eq!(clk.start_time, Some(t1));
    assert_eq!(clk.stop_time, Some(t4));
}

#[test]
fn test_styles_point_linestring_polygon_and_gx_track() {
    let kml = r#"<kml xmlns:gx="http://www.google.com/kml/ext/2.2">
        <Document>
            <Style id="s1">
                <IconStyle>
                    <color>ff0000ff</color>
                    <scale>2.0</scale>
                    <Icon><href>custom.png</href></Icon>
                </IconStyle>
                <LineStyle>
                    <color>ffff0000</color>
                    <width>3.5</width>
                </LineStyle>
                <PolyStyle>
                    <color>8000ff00</color>
                    <fill>1</fill>
                    <outline>0</outline>
                </PolyStyle>
            </Style>
            <StyleMap id="sm1">
                <Pair><key>normal</key><styleUrl>#s1</styleUrl></Pair>
            </StyleMap>
            <Placemark id="pt_empty_icon">
                <Style><IconStyle><Icon/></IconStyle></Style>
                <Point><coordinates>1,2,3</coordinates></Point>
            </Placemark>
            <Placemark id="pt_styled">
                <name>Styled Pt</name>
                <styleUrl>#sm1</styleUrl>
                <Point><altitudeMode>relativeToGround</altitudeMode><coordinates>1,2,3</coordinates></Point>
            </Placemark>
            <Placemark id="line">
                <styleUrl>s1</styleUrl>
                <LineString><coordinates>0,0,0 1,1,0</coordinates></LineString>
            </Placemark>
            <Placemark id="poly">
                <styleUrl>#s1</styleUrl>
                <Polygon>
                    <extrude>1</extrude>
                    <altitudeMode>absolute</altitudeMode>
                    <outerBoundaryIs><LinearRing><coordinates>0,0,10 1,0,10 1,1,10 0,1,10</coordinates></LinearRing></outerBoundaryIs>
                    <innerBoundaryIs><LinearRing><coordinates>0.2,0.2,10 0.8,0.2,10 0.8,0.8,10</coordinates></LinearRing></innerBoundaryIs>
                </Polygon>
            </Placemark>
            <Placemark id="trk">
                <gx:Track>
                    <when>2020-01-01T00:00:00Z</when>
                    <when>2020-01-01T00:00:10Z</when>
                    <gx:coord>0 0 0</gx:coord>
                    <gx:coord>1 1 100</gx:coord>
                </gx:Track>
            </Placemark>
        </Document>
    </kml>"#;

    let mut ds = KmlDataSource::new(None);
    ds.load_str(kml, None).unwrap();

    let pt_empty = ds.entities().get_by_id("pt_empty_icon").unwrap();
    assert!(pt_empty.billboard.as_ref().unwrap().image.is_none());

    let pt_styled = ds.entities().get_by_id("pt_styled").unwrap();
    let bb = pt_styled.billboard.as_ref().unwrap();
    assert_eq!(bb.image.as_ref().unwrap().get_value(None).as_deref(), Some("custom.png"));
    assert_eq!(bb.color.as_ref().unwrap().get_value(None), Some(Color::RED));
    assert_eq!(bb.scale.as_ref().unwrap().get_value(None), Some(2.0));
    assert_eq!(
        bb.height_reference.as_ref().unwrap().get_value(None),
        Some(HeightReference::RelativeToGround)
    );
    assert_eq!(
        pt_styled.label.as_ref().unwrap().text.as_ref().unwrap().get_value(None).as_deref(),
        Some("Styled Pt")
    );

    let line = ds.entities().get_by_id("line").unwrap().polyline.as_ref().unwrap();
    assert_eq!(line.width.as_ref().unwrap().get_value(None), Some(3.5));
    assert_eq!(line.clamp_to_ground.as_ref().unwrap().get_value(None), Some(true));
    let line_mat = line
        .material
        .as_ref()
        .unwrap()
        .as_any()
        .downcast_ref::<ColorMaterialProperty>()
        .unwrap();
    assert_eq!(line_mat.color.as_ref().unwrap().get_value(None), Some(Color::BLUE));

    let poly = ds.entities().get_by_id("poly").unwrap().polygon.as_ref().unwrap();
    assert_eq!(poly.fill.as_ref().unwrap().get_value(None), Some(true));
    assert_eq!(poly.outline.as_ref().unwrap().get_value(None), Some(false));
    assert_eq!(poly.per_position_height.as_ref().unwrap().get_value(None), Some(true));
    assert_eq!(poly.extruded_height.as_ref().unwrap().get_value(None), Some(0.0));
    let hier = poly.hierarchy.as_ref().unwrap().get_value(None).unwrap();
    assert_eq!(hier.positions.len(), 4);
    assert_eq!(hier.holes.len(), 1);
    assert_eq!(hier.holes[0].positions.len(), 3);

    let trk = ds.entities().get_by_id("trk").unwrap();
    let t_mid = JulianDate::from_iso8601("2020-01-01T00:00:05Z").unwrap();
    assert!(trk.position.as_ref().unwrap().get_value(Some(&t_mid)).is_some());
    assert!(trk.path.is_some());
}

#[test]
fn test_ground_overlay_and_network_link() {
    let kml = r#"<kml xmlns:gx="http://www.google.com/kml/ext/2.2">
        <Document>
            <GroundOverlay id="go_box">
                <color>ffffffff</color>
                <altitude>50</altitude>
                <altitudeMode>absolute</altitudeMode>
                <LatLonBox>
                    <north>10</north><south>0</south><east>20</east><west>10</west>
                    <rotation>45</rotation>
                </LatLonBox>
            </GroundOverlay>
            <GroundOverlay id="go_quad">
                <gx:LatLonQuad><coordinates>0,0 1,0 1,1 0,1</coordinates></gx:LatLonQuad>
            </GroundOverlay>
            <NetworkLink><Link><href>https://example.com/sub.kml</href></Link></NetworkLink>
        </Document>
    </kml>"#;

    let mut ds = KmlDataSource::new(None);
    let refreshed = Rc::new(RefCell::new(Vec::new()));
    {
        let r = refreshed.clone();
        ds.refresh_event.add_listener(move |u| r.borrow_mut().push(u.clone()));
    }

    ds.load_str(kml, None).unwrap();
    assert_eq!(*refreshed.borrow(), vec!["https://example.com/sub.kml".to_string()]);

    let go_box = ds.entities().get_by_id("go_box").unwrap().rectangle.as_ref().unwrap();
    assert_eq!(
        go_box.coordinates.as_ref().unwrap().get_value(None),
        Some(Rectangle::from_degrees(10.0, 0.0, 20.0, 10.0))
    );
    assert_eq!(go_box.height.as_ref().unwrap().get_value(None), Some(50.0));
    assert!(
        (go_box.rotation.as_ref().unwrap().get_value(None).unwrap() - 45.0_f64.to_radians()).abs()
            < 1e-12
    );

    let go_quad = ds.entities().get_by_id("go_quad").unwrap().polygon.as_ref().unwrap();
    assert_eq!(
        go_quad.hierarchy.as_ref().unwrap().get_value(None).unwrap().positions.len(),
        4
    );
    let _ = Cartesian3::ZERO;
}
