use cesium_math::{
    BoundingSphere, Cartesian3, ComponentDatatype, Geometry, GeometryAttribute,
    GeometryAttributes, GeometryInstance, GeometryInstanceAttribute, Matrix4, PrimitiveType,
};
use glam::DVec3;
use std::collections::HashMap;

#[test]
fn test_geometry_instance_constructor() {
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

    let geometry = Geometry::new(
        attrs,
        vec![0, 1, 2],
        PrimitiveType::Triangles,
        Some(BoundingSphere::new(DVec3::new(0.5, 0.5, 0.0), 1.0)),
    );

    let model_matrix = Matrix4::from_translation(Cartesian3::new(0.0, 0.0, 9000000.0));
    let color_attr = GeometryInstanceAttribute::new(
        ComponentDatatype::UnsignedByte,
        4,
        true,
        vec![255.0, 255.0, 0.0, 255.0],
    )
    .unwrap();

    let mut instance_attributes = HashMap::new();
    instance_attributes.insert("color".to_string(), color_attr);

    let instance = GeometryInstance::new(
        geometry.clone(),
        Some(model_matrix),
        Some("geometry".to_string()),
        Some(instance_attributes),
    );

    assert_eq!(instance.geometry, geometry);
    assert_eq!(instance.model_matrix, model_matrix);
    assert_eq!(instance.id.as_deref(), Some("geometry"));
    assert!(instance.attributes.contains_key("color"));
}
