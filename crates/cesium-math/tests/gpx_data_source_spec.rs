use cesium_math::{
    Cartesian3, Color, GpxDataSource, GpxLoadOptions, HeightReference, Iso8601, JulianDate,
    PolylineOutlineMaterialProperty, VerticalOrigin,
};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_default_constructor_and_update() {
    let mut ds = GpxDataSource::new(None);
    assert!(ds.name().is_none());
    assert!(ds.clock().is_none());
    assert!(!ds.is_loading());
    assert!(ds.show());
    assert_eq!(ds.entities().values().len(), 0);
    assert!(ds.update(&Iso8601::minimum_value()));

    ds.set_show(false);
    assert!(!ds.show());
}

#[test]
fn test_load_rejects_empty_and_non_gpx() {
    let mut ds = GpxDataSource::new(None);
    assert!(ds.load_str("", None).is_err());
    assert!(ds.load_str("<kml><Document/></kml>", None).is_err());
}

#[test]
fn test_creator_version_and_metadata() {
    let gpx = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
        <gpx xmlns="http://www.topografix.com/GPX/1/1" version="1.1" creator="TestCreator">
            <metadata>
                <name>The name</name>
                <desc>The desc</desc>
                <time>2020-01-01T00:00:00Z</time>
                <keywords>The keyword</keywords>
                <author>
                    <name>Author Name</name>
                    <email><id>user</id><domain>email.com</domain></email>
                    <link href="www.a.com"><text>A website</text><type>text/html</type></link>
                </author>
                <copyright author="The author">
                    <year>2015</year>
                    <license>The license</license>
                </copyright>
                <bounds><minlat>1</minlat><maxlat>2</maxlat><minlon>3</minlon><maxlon>4</maxlon></bounds>
            </metadata>
        </gpx>"#;

    let mut ds = GpxDataSource::new(None);
    ds.load_str(gpx, None).unwrap();

    assert_eq!(ds.version(), Some("1.1"));
    assert_eq!(ds.creator(), Some("TestCreator"));
    assert_eq!(ds.name(), Some("The name"));

    let meta = ds.metadata().expect("metadata should be parsed");
    assert_eq!(meta.name.as_deref(), Some("The name"));
    assert_eq!(meta.desc.as_deref(), Some("The desc"));
    assert_eq!(meta.time.as_deref(), Some("2020-01-01T00:00:00Z"));
    assert_eq!(meta.keywords.as_deref(), Some("The keyword"));

    let author = meta.author.as_ref().unwrap();
    assert_eq!(author.name.as_deref(), Some("Author Name"));
    assert_eq!(author.email.as_deref(), Some("user@email.com"));
    let link = author.link.as_ref().unwrap();
    assert_eq!(link.href.as_deref(), Some("www.a.com"));
    assert_eq!(link.text.as_deref(), Some("A website"));
    assert_eq!(link.mime_type.as_deref(), Some("text/html"));

    let copy = meta.copyright.as_ref().unwrap();
    assert_eq!(copy.author.as_deref(), Some("The author"));
    assert_eq!(copy.year.as_deref(), Some("2015"));
    assert_eq!(copy.license.as_deref(), Some("The license"));

    let bounds = meta.bounds.as_ref().unwrap();
    assert_eq!(bounds.min_lat, 1.0);
    assert_eq!(bounds.max_lat, 2.0);
    assert_eq!(bounds.min_lon, 3.0);
    assert_eq!(bounds.max_lon, 4.0);
}

#[test]
fn test_events_changed_and_loading() {
    let mut ds = GpxDataSource::new(None);
    let changed = Rc::new(RefCell::new(0));
    let loading = Rc::new(RefCell::new(Vec::new()));

    {
        let c = changed.clone();
        ds.changed_event.add_listener(move |_| {
            *c.borrow_mut() += 1;
        });
    }
    {
        let l = loading.clone();
        ds.loading_event.add_listener(move |&state| {
            l.borrow_mut().push(state);
        });
    }

    ds.set_name("Custom Name");
    assert_eq!(*changed.borrow(), 1);

    ds.load_str(r#"<gpx version="1.1"></gpx>"#, None).unwrap();
    assert_eq!(*changed.borrow(), 2);
    assert_eq!(*loading.borrow(), vec![true, false]);
}

#[test]
fn test_waypoints_and_billboard_options_and_descriptions() {
    let invalid_gpx = r#"<gpx version="1.1"><wpt lon="abc" lat="2"/></gpx>"#;
    let missing_gpx = r#"<gpx version="1.1"><wpt/></gpx>"#;
    let mut ds = GpxDataSource::new(None);
    assert!(ds.load_str(invalid_gpx, None).is_err());
    assert!(ds.load_str(missing_gpx, None).is_err());

    let gpx = r#"<?xml version="1.0" encoding="UTF-8"?>
        <gpx version="1.1" creator="andre">
            <wpt lon="38.737125" lat="-9.139242">
                <ele>3</ele>
                <name>Position 1</name>
                <desc>A waypoint</desc>
                <cmt>Nice spot</cmt>
                <src>GPS</src>
                <number>7</number>
                <type>POI</type>
            </wpt>
        </gpx>"#;

    ds.load_str(gpx, None).unwrap();
    assert_eq!(ds.entities().values().len(), 1);
    let e = ds.entities().values()[0];
    assert_eq!(e.name.as_deref(), Some("Position 1"));
    let expected = Cartesian3::from_degrees(38.737125, -9.139242, Some(3.0), None).unwrap();
    assert_eq!(
        e.position.as_ref().unwrap().get_value(Some(&Iso8601::minimum_value())),
        Some(expected)
    );
    let bb = e.billboard.as_ref().unwrap();
    assert_eq!(bb.width.as_ref().unwrap().get_value(None), Some(32.0));
    assert_eq!(bb.height.as_ref().unwrap().get_value(None), Some(32.0));
    assert_eq!(
        bb.vertical_origin.as_ref().unwrap().get_value(None),
        Some(VerticalOrigin::Bottom)
    );
    assert!(bb.height_reference.is_none());

    let desc = e.description.as_ref().unwrap().get_value(None).unwrap();
    assert!(desc.contains("A waypoint"));
    assert!(desc.contains("Nice spot"));
    assert!(desc.contains("GPS"));
    assert!(desc.contains("7"));
    assert!(desc.contains("POI"));

    let opts = GpxLoadOptions {
        clamp_to_ground: true,
        waypoint_image: Some("wpt.png".to_string()),
        ..Default::default()
    };
    ds.load_str(gpx, Some(opts)).unwrap();
    let e2 = ds.entities().values()[0];
    let bb2 = e2.billboard.as_ref().unwrap();
    assert_eq!(
        bb2.height_reference.as_ref().unwrap().get_value(None),
        Some(HeightReference::ClampToGround)
    );
    assert_eq!(
        bb2.image.as_ref().unwrap().get_value(None).as_deref(),
        Some("wpt.png")
    );
}

#[test]
fn test_routes_and_tracks_static_and_dynamic() {
    let route_gpx = r#"<gpx version="1.1">
        <rte>
            <name>Test Route</name>
            <rtept lon="1" lat="2"><ele>1</ele><name>P1</name></rtept>
            <rtept lon="3" lat="4"><ele>1</ele><name>P2</name></rtept>
            <rtept lon="5" lat="6"><ele>1</ele><name>P3</name></rtept>
            <rtept lon="7" lat="8"><ele>1</ele><name>P4</name></rtept>
        </rte>
    </gpx>"#;

    let mut ds = GpxDataSource::new(None);
    ds.load_str(
        route_gpx,
        Some(GpxLoadOptions {
            route_color: Some(Color::CYAN),
            clamp_to_ground: true,
            ..Default::default()
        }),
    )
    .unwrap();
    assert_eq!(ds.entities().values().len(), 5);
    let route_pl = ds.entities().values()[0].polyline.as_ref().unwrap();
    assert_eq!(route_pl.clamp_to_ground.as_ref().unwrap().get_value(None), Some(true));
    let mat = route_pl
        .material
        .as_ref()
        .unwrap()
        .as_any()
        .downcast_ref::<PolylineOutlineMaterialProperty>()
        .unwrap();
    assert_eq!(mat.color.as_ref().unwrap().get_value(None), Some(Color::CYAN));

    let dyn_track_gpx = r#"<gpx version="1.1">
        <trk>
            <name>Test Track</name>
            <trkseg>
                <trkpt lon="1" lat="2"><ele>1.0</ele><time>2000-01-01T00:00:00Z</time></trkpt>
                <trkpt lon="3" lat="4"><ele>1.0</ele><time>2000-01-01T00:00:01Z</time></trkpt>
                <trkpt lon="5" lat="6"><ele>1.0</ele><time>2000-01-01T00:00:02Z</time></trkpt>
            </trkseg>
        </trk>
    </gpx>"#;

    ds.load_str(dyn_track_gpx, None).unwrap();
    assert_eq!(ds.entities().values().len(), 1);
    let trk = ds.entities().values()[0];
    let t1 = JulianDate::from_iso8601("2000-01-01T00:00:00Z").unwrap();
    let t2 = JulianDate::from_iso8601("2000-01-01T00:00:01Z").unwrap();
    let t3 = JulianDate::from_iso8601("2000-01-01T00:00:02Z").unwrap();

    assert_eq!(
        trk.position.as_ref().unwrap().get_value(Some(&t1)),
        Some(Cartesian3::from_degrees(1.0, 2.0, Some(1.0), None).unwrap())
    );
    assert_eq!(
        trk.position.as_ref().unwrap().get_value(Some(&t2)),
        Some(Cartesian3::from_degrees(3.0, 4.0, Some(1.0), None).unwrap())
    );
    assert_eq!(
        trk.position.as_ref().unwrap().get_value(Some(&t3)),
        Some(Cartesian3::from_degrees(5.0, 6.0, Some(1.0), None).unwrap())
    );
    let avail = trk.availability.as_ref().unwrap();
    assert_eq!(avail.start(), Some(t1));
    assert_eq!(avail.stop(), Some(t3));

    let trk_pl = trk.polyline.as_ref().unwrap();
    assert_eq!(trk_pl.width.as_ref().unwrap().get_value(None), Some(4.0));
    let trk_mat = trk_pl
        .material
        .as_ref()
        .unwrap()
        .as_any()
        .downcast_ref::<PolylineOutlineMaterialProperty>()
        .unwrap();
    assert_eq!(trk_mat.color.as_ref().unwrap().get_value(None), Some(Color::RED));
    assert_eq!(trk_mat.outline_width.as_ref().unwrap().get_value(None), Some(2.0));
    assert_eq!(trk_mat.outline_color.as_ref().unwrap().get_value(None), Some(Color::BLACK));
}
