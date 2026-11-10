use cesium_math::{IndexDatatype, PrimitiveType};
use cesium_renderer::buffer::Buffer;
use cesium_renderer::model_components::{Attribute, Indices, Primitive};
use cesium_renderer::wireframe_pipeline_stage::{
    MockWireframeRenderResources, WireframeFrameState, WireframePipelineStage,
};

fn create_box_primitive_with_typed_array() -> Primitive {
    // 36 indices for a cube (12 triangles)
    let indices_data: Vec<u16> = (0..36).collect();
    let mut bytes = Vec::with_capacity(indices_data.len() * 2);
    for idx in indices_data {
        bytes.extend_from_slice(&idx.to_le_bytes());
    }

    Primitive {
        primitive_type: Some(PrimitiveType::Triangles),
        attributes: vec![Attribute {
            name: Some("POSITION".to_string()),
            semantic: Some("POSITION".to_string()),
            count: 24, // 24 vertices for a cube
            ..Default::default()
        }],
        indices: Some(Indices {
            count: 36,
            typed_array: Some(bytes),
            index_datatype: Some(IndexDatatype::UnsignedShort),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn create_box_primitive_with_buffer() -> Primitive {
    let indices_data: Vec<u16> = (0..36).collect();
    let mut bytes = Vec::with_capacity(indices_data.len() * 2);
    for idx in indices_data {
        bytes.extend_from_slice(&idx.to_le_bytes());
    }

    let buffer = Buffer::create_index_buffer(cesium_renderer::buffer::IndexBufferOptions {
        size_in_bytes: 0,
        usage: None,
        typed_array: Some(bytes),
        index_datatype: IndexDatatype::UnsignedShort,
    })
    .unwrap();

    Primitive {
        primitive_type: Some(PrimitiveType::Triangles),
        attributes: vec![Attribute {
            name: Some("POSITION".to_string()),
            semantic: Some("POSITION".to_string()),
            count: 24,
            ..Default::default()
        }],
        indices: Some(Indices {
            count: 36,
            buffer: Some(buffer),
            index_datatype: Some(IndexDatatype::UnsignedShort),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[test]
fn test_wireframe_pipeline_stage_name() {
    assert_eq!(WireframePipelineStage::NAME, "WireframePipelineStage");
}

#[test]
fn test_adds_define_to_shader() {
    let primitive = create_box_primitive_with_typed_array();
    let mut render_resources = MockWireframeRenderResources {
        indices: primitive.indices.clone(),
        count: 36,
        primitive_type: Some(PrimitiveType::Triangles),
        ..Default::default()
    };

    let frame_state = WireframeFrameState { webgl2: false };
    WireframePipelineStage::process(&mut render_resources, &primitive, &frame_state);

    let sb = &render_resources.shader_builder;
    assert!(sb.fragment_define_lines().contains(&"HAS_WIREFRAME".to_string()));
}

#[test]
fn test_creates_wireframe_indices_from_buffer_webgl2() {
    let primitive = create_box_primitive_with_buffer();
    let mut render_resources = MockWireframeRenderResources {
        indices: primitive.indices.clone(),
        count: 36,
        primitive_type: Some(PrimitiveType::Triangles),
        ..Default::default()
    };

    assert_eq!(render_resources.count, 36);
    assert_eq!(render_resources.primitive_type, Some(PrimitiveType::Triangles));

    let frame_state = WireframeFrameState { webgl2: true };
    WireframePipelineStage::process(&mut render_resources, &primitive, &frame_state);

    let wireframe_buffer = render_resources.wireframe_index_buffer.as_ref().unwrap();
    assert_eq!(render_resources.model.pipeline_resources.len(), 1);
    assert_eq!(
        render_resources.model.pipeline_resources[0].id,
        wireframe_buffer.id
    );

    assert_eq!(
        render_resources.model.statistics.geometry_byte_length,
        wireframe_buffer.size_in_bytes
    );

    assert_eq!(render_resources.primitive_type, Some(PrimitiveType::Lines));
    assert_eq!(render_resources.count, 72);
}

#[test]
fn test_creates_wireframe_indices_from_typed_array_webgl1() {
    let primitive = create_box_primitive_with_typed_array();
    let mut render_resources = MockWireframeRenderResources {
        indices: primitive.indices.clone(),
        count: 36,
        primitive_type: Some(PrimitiveType::Triangles),
        ..Default::default()
    };

    assert_eq!(render_resources.count, 36);
    assert_eq!(render_resources.primitive_type, Some(PrimitiveType::Triangles));

    let frame_state = WireframeFrameState { webgl2: false };
    WireframePipelineStage::process(&mut render_resources, &primitive, &frame_state);

    let wireframe_buffer = render_resources.wireframe_index_buffer.as_ref().unwrap();
    assert_eq!(render_resources.model.pipeline_resources.len(), 1);
    assert_eq!(
        render_resources.model.pipeline_resources[0].id,
        wireframe_buffer.id
    );

    assert_eq!(
        render_resources.model.statistics.geometry_byte_length,
        wireframe_buffer.size_in_bytes
    );

    assert_eq!(render_resources.primitive_type, Some(PrimitiveType::Lines));
    assert_eq!(render_resources.count, 72);
}
