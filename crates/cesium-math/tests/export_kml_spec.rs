use cesium_math::{
    export_kml, BillboardGraphics, Cartesian3, Color, ColorMaterialProperty,
    ConstantPositionProperty, ConstantProperty, Entity, EntityCollection, ExportKmlOptions,
    HeightReference, HorizontalOrigin, JulianDate, KmlDataSource, LabelGraphics, PointGraphics,
    PolygonGraphics, PolygonHierarchy, PolylineGraphics, Rectangle, RectangleGraphics,
    SampledPositionProperty, TimeInterval, TimeIntervalCollection, VerticalOrigin,
};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_export_hierarchy_folders_and_placemark() {
    let mut e1 = Entity::with_id("folder1");
    e1.name = Some("Root Folder".to_string());
    e1.show = false;
    let e1_rc = Rc::new(RefCell::new(e1.clone()));

    let mut e2 = Entity::with_id("folder2");
    e2.name = Some("Sub Folder".to_string());
    e2.parent = Some(e1_rc);
    let e2_rc = Rc::new(RefCell::new(e2.clone()));

    let mut e3 = Entity::with_id("pt1");
    e3.name = Some("Child Point".to_string());
    e3.parent = Some(e2_rc);
    e3.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::from_degrees(10.0, 20.0, Some(100.0), None).unwrap()),
        None,
    ));
    e3.point = Some(PointGraphics::new());

    let mut entities = EntityCollection::new();
    entities.add(e1).unwrap();
    entities.add(e2).unwrap();
    entities.add(e3).unwrap();

    let res = export_kml(&ExportKmlOptions::new(&entities)).unwrap();
    assert!(res.kml.contains(r#"<Folder id="folder1">"#));
    assert!(res.kml.contains("<visibility>0</visibility>"));
    assert!(res.kml.contains(r#"<Folder id="folder2">"#));
    assert!(res.kml.contains(r#"<Placemark id="pt1">"#));
    assert!(res.kml.contains("<altitudeMode>absolute</altitudeMode>"));
    assert!(res.kml.contains("<Point>"));
}

#[test]
fn test_export_point_billboard_and_label() {
    let mut pt_ent = Entity::with_id("pt_styled");
    pt_ent.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::from_degrees(1.0, 2.0, Some(0.0), None).unwrap()),
        None,
    ));
    let mut pt = PointGraphics::new();
    pt.color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));
    pt.pixel_size = Some(Rc::new(ConstantProperty::from_value(16.0)));
    pt.height_reference = Some(Rc::new(ConstantProperty::from_value(
        HeightReference::ClampToGround,
    )));
    pt_ent.point = Some(pt);

    let mut lbl = LabelGraphics::new();
    lbl.fill_color = Some(Rc::new(ConstantProperty::from_value(Color::YELLOW)));
    lbl.scale = Some(Rc::new(ConstantProperty::from_value(1.5)));
    pt_ent.label = Some(lbl);

    let mut bb_ent = Entity::with_id("bb_styled");
    bb_ent.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::from_degrees(3.0, 4.0, Some(0.0), None).unwrap()),
        None,
    ));
    let mut bb = BillboardGraphics::new();
    bb.image = Some(Rc::new(ConstantProperty::from_value("pin.png".to_string())));
    bb.rotation = Some(Rc::new(ConstantProperty::from_value(0.0)));
    bb.horizontal_origin = Some(Rc::new(ConstantProperty::from_value(
        HorizontalOrigin::Center,
    )));
    bb.vertical_origin = Some(Rc::new(ConstantProperty::from_value(
        VerticalOrigin::Bottom,
    )));
    bb_ent.billboard = Some(bb);

    let mut entities = EntityCollection::new();
    entities.add(pt_ent).unwrap();
    entities.add(bb_ent).unwrap();

    let res = export_kml(&ExportKmlOptions::new(&entities)).unwrap();
    assert!(res.kml.contains("<color>ff0000ff</color>"));
    assert!(res.kml.contains("<scale>0.5</scale>"));
    assert!(res.kml.contains("<altitudeMode>clampToGround</altitudeMode>"));
    assert!(res.kml.contains("<LabelStyle>"));
    assert!(res.kml.contains("<heading>360</heading>"));
    assert!(res.kml.contains("<href>pin.png</href>"));
    assert!(res.kml.contains(r#"<hotSpot x="0.5" y="0" xunits="fraction" yunits="fraction"/>"#));
}

#[test]
fn test_export_sampled_position_gx_track() {
    let t1 = JulianDate::from_iso8601("2020-01-01T00:00:00Z").unwrap();
    let t2 = JulianDate::from_iso8601("2020-01-01T00:01:00Z").unwrap();
    let mut sampled = SampledPositionProperty::new(None);
    sampled.add_sample(t1, Cartesian3::from_degrees(0.0, 0.0, Some(10.0), None).unwrap());
    sampled.add_sample(t2, Cartesian3::from_degrees(1.0, 1.0, Some(20.0), None).unwrap());

    let mut ent = Entity::with_id("track1");
    ent.position = Some(ConstantPositionProperty::from_dynamic(Rc::new(sampled)));
    let mut avail = TimeIntervalCollection::new();
    avail.add_interval(TimeInterval::new(t1, t2, true, true));
    ent.availability = Some(avail);

    let mut entities = EntityCollection::new();
    entities.add(ent).unwrap();

    let res = export_kml(&ExportKmlOptions::new(&entities)).unwrap();
    assert!(res.kml.contains("<gx:Track>"));
    assert!(res.kml.contains("<when>"));
    assert!(res.kml.contains("<gx:coord>"));
}

#[test]
fn test_export_polyline_polygon_rectangle_and_roundtrip() {
    let mut pl_ent = Entity::with_id("line1");
    let mut pl = PolylineGraphics::new();
    pl.positions = Some(Rc::new(ConstantProperty::from_value(vec![
        Cartesian3::from_degrees(0.0, 0.0, Some(0.0), None).unwrap(),
        Cartesian3::from_degrees(1.0, 1.0, Some(0.0), None).unwrap(),
    ])));
    pl.clamp_to_ground = Some(Rc::new(ConstantProperty::from_value(true)));
    pl.width = Some(Rc::new(ConstantProperty::from_value(4.0)));
    pl.material = Some(Rc::new(ColorMaterialProperty::from_color(Color::BLUE)));
    pl_ent.polyline = Some(pl);

    let mut pg_ent = Entity::with_id("poly1");
    let mut pg = PolygonGraphics::new();
    let mut hier = PolygonHierarchy::new(vec![
        Cartesian3::from_degrees(0.0, 0.0, Some(100.0), None).unwrap(),
        Cartesian3::from_degrees(1.0, 0.0, Some(100.0), None).unwrap(),
        Cartesian3::from_degrees(1.0, 1.0, Some(100.0), None).unwrap(),
        Cartesian3::from_degrees(0.0, 1.0, Some(100.0), None).unwrap(),
    ]);
    hier.holes.push(PolygonHierarchy::new(vec![
        Cartesian3::from_degrees(0.2, 0.2, Some(100.0), None).unwrap(),
        Cartesian3::from_degrees(0.8, 0.2, Some(100.0), None).unwrap(),
        Cartesian3::from_degrees(0.8, 0.8, Some(100.0), None).unwrap(),
    ]));
    pg.hierarchy = Some(Rc::new(ConstantProperty::from_value(hier)));
    pg.outline = Some(Rc::new(ConstantProperty::from_value(true)));
    pg.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::BLACK)));
    pg.extruded_height = Some(Rc::new(ConstantProperty::from_value(0.0)));
    pg.per_position_height = Some(Rc::new(ConstantProperty::from_value(true)));
    pg_ent.polygon = Some(pg);

    let mut rect_ent = Entity::with_id("rect1");
    let mut rg = RectangleGraphics::new();
    rg.coordinates = Some(Rc::new(ConstantProperty::from_value(
        Rectangle::from_degrees(10.0, 10.0, 12.0, 12.0),
    )));
    rg.height = Some(Rc::new(ConstantProperty::from_value(50.0)));
    rect_ent.rectangle = Some(rg);

    let mut entities = EntityCollection::new();
    entities.add(pl_ent).unwrap();
    entities.add(pg_ent).unwrap();
    entities.add(rect_ent).unwrap();

    let res = export_kml(&ExportKmlOptions::new(&entities)).unwrap();
    assert!(res.kml.contains("<LineString>"));
    assert!(res.kml.contains("<tessellate>1</tessellate>"));
    assert!(res.kml.contains("<innerBoundaryIs>"));
    assert!(res.kml.contains("<extrude>1</extrude>"));

    // Roundtrip with KmlDataSource
    let mut ds = KmlDataSource::new(None);
    ds.load_str(&res.kml, None).unwrap();
    assert_eq!(ds.entities().values().len(), 3);
}
