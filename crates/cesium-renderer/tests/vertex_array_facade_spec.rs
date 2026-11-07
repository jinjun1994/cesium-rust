use cesium_math::ComponentDatatype;
use cesium_renderer::{
    BufferUsage, VertexArrayFacade, VertexArrayFacadeAttribute,
};

#[test]
fn test_creates_vertex_array_with_static_floats() {
    let position_index = 0;
    let mut vaf = VertexArrayFacade::new(
        vec![VertexArrayFacadeAttribute {
            index: position_index,
            components_per_attribute: 3,
            component_datatype: ComponentDatatype::Float,
            usage: BufferUsage::StaticDraw,
        }],
        1,
    );

    vaf.write_float3(0, 1.0, 2.0, 3.0);
    vaf.commit();

    let va = &vaf.va[0].va;
    let attr = va.get_attribute(0);
    assert!(attr.vertex_buffer.is_some());
    let vb = attr.vertex_buffer.as_ref().unwrap();
    assert_eq!(vb.size_in_bytes, 12);
    assert_eq!(vb.usage, BufferUsage::StaticDraw);
    assert_eq!(attr.components_per_attribute, 3);
    assert_eq!(attr.component_datatype, ComponentDatatype::Float);
    assert_eq!(attr.offset_in_bytes, 0);
    assert_eq!(attr.stride_in_bytes, 3 * 4);
}

#[test]
fn test_resizes_vertex_array_with_static_floats() {
    let position_index = 0;
    let mut vaf = VertexArrayFacade::new(
        vec![VertexArrayFacadeAttribute {
            index: position_index,
            components_per_attribute: 3,
            component_datatype: ComponentDatatype::Float,
            usage: BufferUsage::StaticDraw,
        }],
        1,
    );

    vaf.write_float3(0, 1.0, 2.0, 3.0);

    vaf.resize(2); // Two vertices
    vaf.write_float3(1, 4.0, 5.0, 6.0);
    vaf.commit();

    let va = &vaf.va[0].va;
    let attr = va.get_attribute(0);
    assert!(attr.vertex_buffer.is_some());
    let vb = attr.vertex_buffer.as_ref().unwrap();
    assert_eq!(vb.size_in_bytes, 2 * 3 * 4);
    assert_eq!(vb.usage, BufferUsage::StaticDraw);
    assert_eq!(attr.components_per_attribute, 3);
    assert_eq!(attr.component_datatype, ComponentDatatype::Float);
    assert_eq!(attr.offset_in_bytes, 0);
    assert_eq!(attr.stride_in_bytes, 3 * 4);
}
