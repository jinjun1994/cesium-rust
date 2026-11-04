use cesium_math::{
    BoundingSphere, ComponentDatatype, Geometry, GeometryAttribute, GeometryAttributes,
    PrimitiveType,
};
use glam::DVec3;

#[test]
fn test_geometry_constructor() {
    let position_attr = GeometryAttribute::with_normalize(
        ComponentDatatype::Double,
        3,
        false,
        vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
    )
    .unwrap();

    let attrs = GeometryAttributes {
        position: Some(position_attr),
        ..Default::default()
    };

    let indices = vec![0, 1, 2];
    let bs = BoundingSphere::new(DVec3::new(0.5, 0.5, 0.0), 1.0);

    let geometry = Geometry::new(
        attrs.clone(),
        indices.clone(),
        PrimitiveType::Triangles,
        Some(bs),
    );

    assert_eq!(geometry.attributes, attrs);
    assert_eq!(geometry.indices, indices);
    assert_eq!(geometry.primitive_type, PrimitiveType::Triangles);
    assert_eq!(geometry.bounding_sphere, Some(bs));
}
