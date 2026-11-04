use cesium_math::{AxisAlignedBoundingBox, BoxGeometry, Cartesian3, PrimitiveType, VertexFormat};
use glam::DVec3;

#[test]
fn test_box_geometry_position_only() {
    let box_geom = BoxGeometry::new(
        Cartesian3::new(-1.0, -2.0, -3.0),
        Cartesian3::new(1.0, 2.0, 3.0),
        Some(VertexFormat::POSITION_ONLY),
    );
    let geom = box_geom.create_geometry().unwrap();

    let pos = geom.attributes.position.unwrap();
    assert_eq!(pos.values.len(), 8 * 3);
    assert_eq!(geom.indices.len(), 12 * 3);
    assert_eq!(geom.primitive_type, PrimitiveType::Triangles);
}

#[test]
fn test_box_geometry_all_attributes() {
    let box_geom = BoxGeometry::new(
        Cartesian3::ZERO,
        Cartesian3::new(1.0, 1.0, 1.0),
        Some(VertexFormat::ALL),
    );
    let geom = box_geom.create_geometry().unwrap();

    let pos = geom.attributes.position.unwrap();
    assert_eq!(pos.values.len(), 6 * 4 * 3); // 24 vertices
    let normal = geom.attributes.normal.unwrap();
    assert_eq!(normal.values.len(), 6 * 4 * 3);
    let st = geom.attributes.st.unwrap();
    assert_eq!(st.values.len(), 6 * 4 * 2);

    assert_eq!(geom.indices.len(), 12 * 3);
    assert_eq!(geom.primitive_type, PrimitiveType::Triangles);
}

#[test]
fn test_box_geometry_from_dimensions_and_aabb() {
    let box_dim = BoxGeometry::from_dimensions(Cartesian3::new(10.0, 20.0, 30.0), None);
    assert_eq!(box_dim.minimum, Cartesian3::new(-5.0, -10.0, -15.0));
    assert_eq!(box_dim.maximum, Cartesian3::new(5.0, 10.0, 15.0));

    let aabb = AxisAlignedBoundingBox::new(DVec3::new(-1.0, -1.0, -1.0), DVec3::new(1.0, 1.0, 1.0), None);
    let box_aabb = BoxGeometry::from_axis_aligned_bounding_box(&aabb, None);
    assert_eq!(box_aabb.minimum, Cartesian3::new(-1.0, -1.0, -1.0));
    assert_eq!(box_aabb.maximum, Cartesian3::new(1.0, 1.0, 1.0));

    // Degenerate box
    let degenerate = BoxGeometry::new(Cartesian3::ZERO, Cartesian3::ZERO, None);
    assert!(degenerate.create_geometry().is_none());
}
