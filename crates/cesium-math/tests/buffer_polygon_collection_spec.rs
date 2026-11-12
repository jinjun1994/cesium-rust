use cesium_math::{
    BoundingSphere, BufferPolygon, BufferPolygonCollection, BufferPolygonCollectionOptions,
    BufferPolygonMaterial, Cartesian3, Color, Ellipsoid, HeightReference,
};
use glam::DVec3;

#[test]
fn test_polygon_collection_add_holes_material_and_defaults() {
    let col = BufferPolygonCollection::new(BufferPolygonCollectionOptions::default());
    assert_eq!(col.height_reference(), HeightReference::ClampToTerrain);
    assert!(!col.allow_picking());
    assert_eq!(col.primitive_count(), 0);

    let positions = vec![
        Cartesian3::from_degrees(0.0, 0.0, None, None).unwrap(),
        Cartesian3::from_degrees(1.0, 0.0, None, None).unwrap(),
        Cartesian3::from_degrees(1.0, 1.0, None, None).unwrap(),
        Cartesian3::from_degrees(0.0, 1.0, None, None).unwrap(),
        Cartesian3::from_degrees(0.2, 0.2, None, None).unwrap(),
        Cartesian3::from_degrees(0.8, 0.2, None, None).unwrap(),
        Cartesian3::from_degrees(0.8, 0.8, None, None).unwrap(),
    ];
    let poly = BufferPolygon::new(
        positions.clone(),
        vec![4],
        BufferPolygonMaterial { color: Color::RED },
    );
    col.add(poly);
    assert_eq!(col.primitive_count(), 1);

    let got = col.get(0).unwrap();
    assert_eq!(got.positions.len(), 7);
    assert_eq!(got.holes, vec![4]);
    assert_eq!(got.material.color, Color::RED);
    assert!(got.show);
}

#[test]
fn test_polygon_collection_bounding_volume_and_dirty_state() {
    let col = BufferPolygonCollection::new(BufferPolygonCollectionOptions {
        height_reference: Some(HeightReference::None),
        allow_picking: Some(true),
        ..Default::default()
    });
    assert_eq!(col.height_reference(), HeightReference::None);
    assert!(col.allow_picking());

    col.add(BufferPolygon::new(
        vec![
            Cartesian3::from_degrees(0.0, 0.0, None, None).unwrap(),
            Cartesian3::from_degrees(1.0, 0.0, None, None).unwrap(),
            Cartesian3::from_degrees(0.0, 1.0, None, None).unwrap(),
        ],
        vec![],
        BufferPolygonMaterial::default(),
    ));
    assert!(col.dirty_count() > 0);
    col.make_clean();
    assert_eq!(col.dirty_count(), 0);

    let bv = col.bounding_volume(&Ellipsoid::WGS84);
    assert!(bv.radius > 0.0);

    let static_bv = BoundingSphere::new(DVec3::new(1.0, 2.0, 3.0), 50.0);
    let col_static = BufferPolygonCollection::new(BufferPolygonCollectionOptions {
        bounding_volume: Some(static_bv),
        ..Default::default()
    });
    assert_eq!(col_static.bounding_volume(&Ellipsoid::WGS84), static_bv);
}
