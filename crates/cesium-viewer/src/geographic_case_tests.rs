use crate::{entities, geodesic, geographic, outline};
use cesium_math::Cartographic;
use glam::DVec3;
#[test]
fn ellipse_override_keeps_original_position_and_extrusion() {
    let (e, _) = entities::czml(include_bytes!(
        "../../../tests/sandcastle/full/czml-circles-and-ellipses.czml"
    ))
    .unwrap();
    assert_eq!(e[0].position_degrees.unwrap()[2], 150000.);
    assert_eq!(e[2].position_degrees.unwrap()[2], 100000.);
    let p = e[2].primitive().unwrap();
    let model = e[2].model(0.).unwrap();
    let h: Vec<_> = p
        .vertices
        .iter()
        .map(|v| {
            Cartographic::from_ecef(
                model.transform_point3(DVec3::from_array(v.position.map(f64::from))),
            )
            .unwrap()
            .height
        })
        .collect();
    assert!(h.iter().any(|h| (*h - 10.).abs() < 1.));
    assert!(h.iter().any(|h| (*h - 200010.).abs() < 1.));
    assert!(outline::primitive(&e[2]).unwrap().unwrap().indices.len() > 500);
}
#[test]
fn ellipse_large_axes_rotation_and_height_bounds() {
    let c = [-103., 40., 150000.];
    let ring = geographic::ellipse_ring(c, 400000., 250000., 0., Some(200000.)).unwrap();
    assert!(ring.len() >= 192 && ring.len().is_multiple_of(4));
    assert!(ring.iter().all(|p| p[2] == 200000.));
    let center = Cartographic::degrees(c[0], c[1], 0.).unwrap();
    let frame = center.enu_to_ecef().inverse();
    let local = |p: [f64; 3]| {
        frame.transform_point3(Cartographic::degrees(p[0], p[1], 0.).unwrap().to_ecef())
    };
    let east = ring.iter().map(|p| local(*p).x.abs()).fold(0., f64::max);
    let north = ring.iter().map(|p| local(*p).y.abs()).fold(0., f64::max);
    assert!(east > 390000. && north < 260000.);
    let r = geographic::ellipse_ring(c, 400000., 150000., std::f64::consts::FRAC_PI_4, Some(0.))
        .unwrap();
    let v = local(r[r.len() / 4]);
    assert!(v.x > 200000. && v.y > 200000.);
    assert!(geographic::ellipse_ring(c, 1000001., 10., 0., None).is_err());
    assert!(geographic::ellipse_ring(c, 200000., 300000., 0., None).is_err());
}
#[test]
fn wall_geodesic_path_and_sawtooth_heights() {
    let specs =
        entities::decode(include_bytes!("../../../tests/sandcastle/full/wall.json")).unwrap();
    let e = &specs[0];
    let p = e.primitive().unwrap();
    let m = e.model(0.).unwrap();
    let geo: Vec<_> = p
        .vertices
        .iter()
        .map(|v| {
            Cartographic::from_ecef(
                m.transform_point3(DVec3::from_array(v.position.map(f64::from))),
            )
            .unwrap()
        })
        .collect();
    assert!(geo.iter().any(|g| g.latitude.to_degrees() > 44.65));
    assert!(geo.iter().all(|g| g.height > 99999. && g.height < 200011.));
    let arc = geodesic::Line::new([-115., 44.], [-90., 44.]).unwrap();
    assert!(arc.distance > 1900000. && arc.distance < 2100000.);
    let saw = &specs[2];
    let p = saw.primitive().unwrap();
    assert!(p
        .vertices
        .iter()
        .flat_map(|v| v.position)
        .all(f32::is_finite));
    assert!(outline::primitive(saw).unwrap().is_some());
}
#[test]
fn original_czml_point_pixel_graphic() {
    let (e, _) = entities::czml(include_bytes!(
        "../../../tests/sandcastle/full/czml-point.czml"
    ))
    .unwrap();
    assert_eq!(e[0].position_degrees, Some([-111., 40., 0.]));
    let entities::Shape::Point {
        pixel_size,
        outline_width,
        outline_color,
    } = e[0].shape
    else {
        panic!("point required")
    };
    assert_eq!(pixel_size, 20.);
    assert_eq!(outline_width, 4.);
    assert_eq!(outline_color, [1., 0., 0., 1.]);
    assert!(!e[0].outline);
    let p = e[0].primitive().unwrap();
    let pattern = p.material.pattern.unwrap();
    assert_eq!(pattern.kind, 5);
    assert!((pattern.repeat[0] - 20. / 28.).abs() < 1e-6);
}
