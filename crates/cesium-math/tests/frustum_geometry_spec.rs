use cesium_math::{
    Cartesian3, FrustumGeometry, FrustumOutlineGeometry, PerspectiveFrustum,
    PrimitiveType, Quaternion, VertexFormat,
};

#[test]
fn test_frustum_geometry_all_attributes() {
    let frustum = PerspectiveFrustum {
        fov: 30.0f64.to_radians(),
        aspect_ratio: 1920.0 / 1080.0,
        near: 1.0,
        far: 3.0,
        ..Default::default()
    };

    let geom_spec = FrustumGeometry::new(
        frustum,
        Cartesian3::ZERO,
        Quaternion::IDENTITY,
        Some(VertexFormat::ALL),
    );

    let m = geom_spec.create_geometry().unwrap();

    let num_vertices = 24; // 6 sides x 4 corners
    let num_triangles = 12; // 6 sides x 2 triangles per side

    assert_eq!(m.attributes.position.as_ref().unwrap().values.len(), num_vertices * 3);
    assert_eq!(m.attributes.normal.as_ref().unwrap().values.len(), num_vertices * 3);
    assert_eq!(m.attributes.tangent.as_ref().unwrap().values.len(), num_vertices * 3);
    assert_eq!(m.attributes.bitangent.as_ref().unwrap().values.len(), num_vertices * 3);
    assert_eq!(m.attributes.st.as_ref().unwrap().values.len(), num_vertices * 2);

    assert_eq!(m.indices.len(), num_triangles * 3);
    assert_eq!(m.primitive_type, PrimitiveType::Triangles);

    let bs = m.bounding_sphere.unwrap();
    let center: Cartesian3 = bs.center.into();
    let center_diff = center - Cartesian3::new(0.0, 0.0, 2.0);
    assert!(center_diff.magnitude() < 1e-4);
    assert!(bs.radius > 1.0);
    assert!(bs.radius < 2.0);
}

#[test]
fn test_frustum_outline_geometry() {
    let frustum = PerspectiveFrustum {
        fov: 30.0f64.to_radians(),
        aspect_ratio: 1920.0 / 1080.0,
        near: 1.0,
        far: 3.0,
        ..Default::default()
    };

    let outline_spec = FrustumOutlineGeometry::new(
        frustum,
        Cartesian3::ZERO,
        Quaternion::IDENTITY,
    );

    let m = outline_spec.create_geometry().unwrap();

    let num_vertices = 8;
    let num_lines = 12;

    assert_eq!(m.attributes.position.as_ref().unwrap().values.len(), num_vertices * 3);
    assert_eq!(m.indices.len(), num_lines * 2);
    assert_eq!(m.primitive_type, PrimitiveType::Lines);

    let bs = m.bounding_sphere.unwrap();
    let center: Cartesian3 = bs.center.into();
    let center_diff = center - Cartesian3::new(0.0, 0.0, 2.0);
    assert!(center_diff.magnitude() < 1e-4);
    assert!(bs.radius > 1.0);
    assert!(bs.radius < 2.0);
}
