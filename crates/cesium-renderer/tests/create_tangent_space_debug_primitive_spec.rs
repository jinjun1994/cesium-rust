//! Spec matching CesiumJS `Specs/Scene/createTangentSpaceDebugPrimitiveSpec.js` (Apache-2.0).

use cesium_math::{
    Cartesian3, ComponentDatatype, EllipsoidGeometry, Matrix4, PrimitiveType, VertexFormat,
};
use cesium_renderer::{
    create_tangent_space_debug_primitive, CreateTangentSpaceDebugPrimitiveOptions,
};

#[test]
fn test_computes_all_attributes() {
    let ellipsoid = EllipsoidGeometry::new(
        Some(Cartesian3::new(500000.0, 500000.0, 1000000.0)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(VertexFormat::ALL),
    );
    let geometry = ellipsoid
        .create_geometry()
        .expect("Failed to create geometry");

    let model_matrix = Matrix4::multiply_by_translation(
        &Matrix4::IDENTITY,
        Cartesian3::new(0.0, 0.0, 11000000.0),
    );

    let primitive = create_tangent_space_debug_primitive(CreateTangentSpaceDebugPrimitiveOptions {
        geometry: Some(geometry),
        model_matrix: Some(model_matrix),
        length: Some(1000.0),
    })
    .expect("Should not throw developer error")
    .expect("Primitive should be defined");

    assert!(primitive.geometry_instances.is_some());
    assert!(primitive.appearance.is_some());
    assert!(!primitive.asynchronous);

    let instances = primitive.geometry_instances.as_ref().unwrap();
    assert_eq!(instances.len(), 3);

    for instance in instances {
        assert_eq!(instance.model_matrix, model_matrix);
        assert!(instance.attributes.contains_key("color"));
        let color_attr = &instance.attributes["color"];
        assert_eq!(color_attr.component_datatype, ComponentDatatype::UnsignedByte);
        assert_eq!(color_attr.components_per_attribute, 4);
        assert!(color_attr.normalize);
        assert_eq!(instance.geometry.primitive_type, PrimitiveType::Lines);
    }

    // Normal is red
    assert_eq!(instances[0].attributes["color"].value, vec![255.0, 0.0, 0.0, 255.0]);
    // Tangent is green
    assert_eq!(instances[1].attributes["color"].value, vec![0.0, 255.0, 0.0, 255.0]);
    // Bitangent is blue
    assert_eq!(instances[2].attributes["color"].value, vec![0.0, 0.0, 255.0, 255.0]);
}

#[test]
fn test_throws_without_geometry() {
    let res = create_tangent_space_debug_primitive(CreateTangentSpaceDebugPrimitiveOptions {
        geometry: None,
        model_matrix: None,
        length: None,
    });
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.message, "options.geometry is required.");
}
