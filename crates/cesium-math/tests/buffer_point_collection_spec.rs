use cesium_math::{
    BoundingSphere, BufferPoint, BufferPointCollection, BufferPointCollectionOptions,
    BufferPointMaterial, Cartesian3, Color, Ellipsoid, HeightReference,
};
use glam::DVec3;

#[test]
fn test_point_collection_add_get_update_and_defaults() {
    let col = BufferPointCollection::new(BufferPointCollectionOptions::default());
    assert_eq!(col.height_reference(), HeightReference::None);
    assert!(!col.allow_picking());
    assert_eq!(col.primitive_count(), 0);

    let p0 = BufferPoint::new(
        Cartesian3::new(100.0, 200.0, 300.0),
        BufferPointMaterial {
            color: Color::RED,
            outline_color: Color::BLACK,
            outline_width: 2.0,
            size: 10.0,
        },
    );
    let idx0 = col.add(p0);
    assert_eq!(idx0, 0);
    assert_eq!(col.primitive_count(), 1);

    let got = col.get(0).unwrap();
    assert_eq!(got.feature_id, 0);
    assert!(got.show);
    assert_eq!(got.position, Cartesian3::new(100.0, 200.0, 300.0));
    assert_eq!(got.material.color, Color::RED);
    assert_eq!(got.material.size, 10.0);

    col.set_point_position(0, Cartesian3::new(1.0, 2.0, 3.0));
    col.set_point_material(
        0,
        BufferPointMaterial {
            color: Color::BLUE,
            ..Default::default()
        },
    );
    col.set_point_show(0, false);

    let updated = col.get(0).unwrap();
    assert_eq!(updated.position, Cartesian3::new(1.0, 2.0, 3.0));
    assert_eq!(updated.material.color, Color::BLUE);
    assert!(!updated.show);
}

#[test]
fn test_point_collection_sort_clone_byte_length_and_bounding_volume() {
    let col = BufferPointCollection::new(BufferPointCollectionOptions {
        allow_picking: Some(true),
        ..Default::default()
    });
    assert!(col.allow_picking());

    col.add(BufferPoint::new(
        Cartesian3::new(10.0, 0.0, 0.0),
        BufferPointMaterial::default(),
    ));
    col.add(BufferPoint::new(
        Cartesian3::new(2.0, 0.0, 0.0),
        BufferPointMaterial::default(),
    ));
    assert!(col.byte_length() > 0);

    col.sort_by(|a, b| a.position.x.partial_cmp(&b.position.x).unwrap());
    assert_eq!(col.get(0).unwrap().position.x, 2.0);
    assert_eq!(col.get(0).unwrap().feature_id, 0);
    assert_eq!(col.get(1).unwrap().position.x, 10.0);
    assert_eq!(col.get(1).unwrap().feature_id, 1);

    let cloned = col.clone_collection();
    assert_eq!(cloned.primitive_count(), 2);
    assert_eq!(cloned.get(0).unwrap().position, col.get(0).unwrap().position);

    let bv = col.bounding_volume(&Ellipsoid::WGS84);
    assert!(bv.radius > 0.0);

    let static_bv = BoundingSphere::new(DVec3::new(1.0, 2.0, 3.0), 42.0);
    let col_static = BufferPointCollection::new(BufferPointCollectionOptions {
        bounding_volume: Some(static_bv),
        ..Default::default()
    });
    assert_eq!(col_static.bounding_volume(&Ellipsoid::WGS84), static_bv);
}
