use cesium_math::{
    BoundingSphere, BufferPolyline, BufferPolylineCollection, BufferPolylineCollectionOptions,
    BufferPolylineMaterial, Cartesian3, Color, Ellipsoid, HeightReference, WidthUnits,
};
use glam::DVec3;

#[test]
fn test_polyline_collection_add_update_and_units() {
    let col = BufferPolylineCollection::new(BufferPolylineCollectionOptions {
        width_units: Some(WidthUnits::Meters),
        allow_picking: Some(true),
        ..Default::default()
    });
    assert_eq!(col.height_reference(), HeightReference::ClampToTerrain);
    assert_eq!(col.width_units(), WidthUnits::Meters);
    assert!(col.allow_picking());

    let p0 = BufferPolyline::new(
        vec![
            Cartesian3::from_degrees(0.0, 0.0, None, None).unwrap(),
            Cartesian3::from_degrees(1.0, 0.0, None, None).unwrap(),
        ],
        BufferPolylineMaterial {
            color: Color::LIME,
            width: 5.0,
        },
    );
    col.add(p0);
    assert_eq!(col.primitive_count(), 1);
    assert_eq!(col.get(0).unwrap().material.width, 5.0);

    col.set_polyline_positions(
        0,
        vec![
            Cartesian3::from_degrees(2.0, 2.0, None, None).unwrap(),
            Cartesian3::from_degrees(3.0, 3.0, None, None).unwrap(),
        ],
    );
    col.set_polyline_material(
        0,
        BufferPolylineMaterial {
            color: Color::BLUE,
            ..Default::default()
        },
    );
    assert_eq!(col.get(0).unwrap().material.color, Color::BLUE);
    assert_eq!(col.get(0).unwrap().positions.len(), 2);
}

#[test]
fn test_polyline_collection_bounding_volume_and_clean() {
    let col = BufferPolylineCollection::new(BufferPolylineCollectionOptions::default());
    col.add(BufferPolyline::new(
        vec![
            Cartesian3::from_degrees(0.0, 0.0, None, None).unwrap(),
            Cartesian3::from_degrees(1.0, 1.0, None, None).unwrap(),
        ],
        BufferPolylineMaterial::default(),
    ));
    let bv = col.bounding_volume(&Ellipsoid::WGS84);
    assert!(bv.radius > 0.0);

    col.make_clean();
    assert_eq!(col.dirty_count(), 0);

    let static_bv = BoundingSphere::new(DVec3::ZERO, 99.0);
    let col_static = BufferPolylineCollection::new(BufferPolylineCollectionOptions {
        bounding_volume: Some(static_bv),
        ..Default::default()
    });
    assert_eq!(col_static.bounding_volume(&Ellipsoid::WGS84), static_bv);
}
