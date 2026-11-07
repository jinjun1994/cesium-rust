use cesium_math::ComponentDatatype;
use cesium_renderer::{
    Buffer, BufferOptions, BufferUsage, VertexArray, VertexArrayOptions, VertexAttributeOptions,
};

#[test]
fn test_binds_with_default_values() {
    let position_buffer = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 3,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: None,
    });

    let attributes = vec![VertexAttributeOptions {
        index: None,
        enabled: None,
        vertex_buffer: Some(position_buffer.clone()),
        components_per_attribute: 3,
        component_datatype: None,
        normalize: None,
        offset_in_bytes: None,
        stride_in_bytes: None,
        instance_divisor: None,
    }];

    let mut va = VertexArray::new(VertexArrayOptions {
        attributes,
        index_buffer: None,
    });

    assert_eq!(va.number_of_attributes(), 1);
    let attr = va.get_attribute(0);
    assert_eq!(attr.index, 0);
    assert!(attr.enabled);
    assert_eq!(attr.vertex_buffer.as_ref().unwrap(), &position_buffer);
    assert_eq!(attr.components_per_attribute, 3);
    assert_eq!(attr.component_datatype, ComponentDatatype::Float);
    assert!(!attr.normalize);
    assert_eq!(attr.offset_in_bytes, 0);
    assert_eq!(attr.stride_in_bytes, 0);
    assert_eq!(attr.instance_divisor, 0);

    va.bind();
    va.unbind();
    va.destroy();
    assert!(va.is_destroyed());
}

#[test]
fn test_binds_with_multiple_buffers() {
    let position_buffer = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 12,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: None,
    });
    let normal_buffer = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 12,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: None,
    });

    let attributes = vec![
        VertexAttributeOptions {
            index: Some(0),
            enabled: Some(true),
            vertex_buffer: Some(position_buffer),
            components_per_attribute: 3,
            component_datatype: Some(ComponentDatatype::Float),
            normalize: None,
            offset_in_bytes: None,
            stride_in_bytes: None,
            instance_divisor: None,
        },
        VertexAttributeOptions {
            index: Some(1),
            enabled: Some(true),
            vertex_buffer: Some(normal_buffer),
            components_per_attribute: 3,
            component_datatype: Some(ComponentDatatype::Float),
            normalize: None,
            offset_in_bytes: None,
            stride_in_bytes: None,
            instance_divisor: None,
        },
    ];

    let mut va = VertexArray::new(VertexArrayOptions {
        attributes,
        index_buffer: None,
    });

    assert_eq!(va.number_of_attributes(), 2);
    va.bind();
    va.unbind();
    va.destroy();
}

#[test]
fn test_modifies_attributes() {
    let buffer = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 6,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: None,
    });

    let attributes = vec![VertexAttributeOptions {
        index: None,
        enabled: Some(true),
        vertex_buffer: Some(buffer),
        components_per_attribute: 3,
        component_datatype: None,
        normalize: None,
        offset_in_bytes: None,
        stride_in_bytes: None,
        instance_divisor: None,
    }];

    let mut va = VertexArray::new(VertexArrayOptions {
        attributes,
        index_buffer: None,
    });

    assert_eq!(va.number_of_attributes(), 1);
    assert!(va.get_attribute(0).enabled);

    va.get_attribute_mut(0).enabled = false;
    assert!(!va.get_attribute(0).enabled);

    va.destroy();
}
