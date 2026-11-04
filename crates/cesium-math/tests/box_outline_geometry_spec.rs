use cesium_math::{AxisAlignedBoundingBox, BoxOutlineGeometry, Cartesian3, PrimitiveType};
use glam::DVec3;

#[test]
fn test_box_outline_geometry_create() {
    let outline = BoxOutlineGeometry::new(
        Cartesian3::new(-1.0, -2.0, -3.0),
        Cartesian3::new(1.0, 2.0, 3.0),
    );
    let geom = outline.create_geometry().unwrap();

    let pos = geom.attributes.position.unwrap();
    assert_eq!(pos.values.len(), 8 * 3);
    assert_eq!(geom.indices.len(), 12 * 2);
    assert_eq!(geom.primitive_type, PrimitiveType::Lines);
}

#[test]
fn test_box_outline_geometry_from_dimensions_and_aabb() {
    let outline_dim = BoxOutlineGeometry::from_dimensions(Cartesian3::new(10.0, 20.0, 30.0));
    assert_eq!(outline_dim.minimum, Cartesian3::new(-5.0, -10.0, -15.0));
    assert_eq!(outline_dim.maximum, Cartesian3::new(5.0, 10.0, 15.0));

    let aabb = AxisAlignedBoundingBox::new(DVec3::new(-2.0, -2.0, -2.0), DVec3::new(2.0, 2.0, 2.0), None);
    let outline_aabb = BoxOutlineGeometry::from_axis_aligned_bounding_box(&aabb);
    assert_eq!(outline_aabb.minimum, Cartesian3::new(-2.0, -2.0, -2.0));
    assert_eq!(outline_aabb.maximum, Cartesian3::new(2.0, 2.0, 2.0));

    // Degenerate
    let degenerate = BoxOutlineGeometry::new(Cartesian3::ZERO, Cartesian3::ZERO);
    assert!(degenerate.create_geometry().is_none());
}
