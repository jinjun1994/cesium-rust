use cesium_math::{
    ComponentDatatype, Geometry, GeometryAttribute, GeometryAttributes, PrimitiveType,
};
use cesium_renderer::{
    BufferUsage, VertexArray, VertexArrayFromGeometryOptions,
};

#[test]
fn test_creates_with_no_geometry() {
    let va = VertexArray::from_geometry(VertexArrayFromGeometryOptions {
        geometry: None,
        interleave: Some(true),
    })
    .unwrap();

    assert_eq!(va.number_of_attributes(), 0);
    assert!(va.index_buffer.is_none());
}

#[test]
fn test_creates_single_attribute_vertex_from_geometry() {
    let attributes = GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Float,
            3,
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        )),
        ..Default::default()
    };

    let geometry = Geometry::new(attributes, Vec::new(), PrimitiveType::Points, None);

    let va = VertexArray::from_geometry(VertexArrayFromGeometryOptions {
        geometry: Some(geometry),
        interleave: Some(false),
    })
    .unwrap();

    assert_eq!(va.number_of_attributes(), 1);
    assert!(va.index_buffer.is_none());

    let attr = va.get_attribute(0);
    assert_eq!(attr.index, 0);
    assert_eq!(attr.component_datatype, ComponentDatatype::Float);
    assert_eq!(attr.components_per_attribute, 3);
    assert_eq!(attr.offset_in_bytes, 0);
    assert_eq!(attr.stride_in_bytes, 0);
    assert_eq!(
        attr.vertex_buffer.as_ref().unwrap().usage,
        BufferUsage::DynamicDraw
    );
}
