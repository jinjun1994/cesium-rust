use cesium_math::{AttributeType, ComponentDatatype};
use cesium_renderer::buffer::{Buffer, BufferOptions};
use cesium_renderer::model_components::{Attribute, Instances, Node};
use cesium_renderer::model_runtime_node::ModelRuntimeNode;
use cesium_renderer::model_statistics::ModelStatistics;
use cesium_renderer::node_statistics_pipeline_stage::NodeStatisticsPipelineStage;

#[test]
fn test_does_not_update_statistics_for_a_non_instanced_model() {
    let mut statistics = ModelStatistics::new();
    let node = Node::default();
    let runtime_node = ModelRuntimeNode::default();

    NodeStatisticsPipelineStage::process(&mut statistics, &node, &runtime_node);

    assert_eq!(statistics.points_length, 0);
    assert_eq!(statistics.triangles_length, 0);
    assert_eq!(statistics.geometry_byte_length, 0);
    assert_eq!(statistics.textures_byte_length, 0);
    assert_eq!(statistics.property_tables_byte_length, 0);
}

#[test]
fn test_updates_statistics_for_an_instanced_model() {
    let mut statistics = ModelStatistics::new();

    let buffer = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 48, // 4 vec3s * 3 floats * 4 bytes
        ..Default::default()
    })
    .expect("Buffer creation");

    let attr = Attribute {
        buffer: Some(buffer),
        attribute_type: Some(AttributeType::Vec3),
        component_datatype: Some(ComponentDatatype::Float),
        count: 4,
        ..Default::default()
    };

    let node = Node {
        instances: Some(Instances {
            attributes: vec![attr],
            ..Default::default()
        }),
        ..Default::default()
    };

    let runtime_node = ModelRuntimeNode::default();

    NodeStatisticsPipelineStage::process(&mut statistics, &node, &runtime_node);

    assert_eq!(statistics.points_length, 0);
    assert_eq!(statistics.triangles_length, 0);
    assert_eq!(statistics.geometry_byte_length, 48);
    assert_eq!(statistics.textures_byte_length, 0);
    assert_eq!(statistics.property_tables_byte_length, 0);
}

#[test]
fn test_count_instancing_attributes_counts_attributes_with_buffers() {
    let mut statistics = ModelStatistics::new();

    let buffer = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 48,
        ..Default::default()
    })
    .expect("Buffer creation");

    let attr = Attribute {
        buffer: Some(buffer),
        count: 4,
        ..Default::default()
    };

    let instances = Instances {
        attributes: vec![attr],
        ..Default::default()
    };

    NodeStatisticsPipelineStage::count_instancing_attributes(&mut statistics, Some(&instances));

    assert_eq!(statistics.geometry_byte_length, 48);
}

#[test]
fn test_count_instancing_attributes_does_not_count_attributes_without_buffers() {
    let mut statistics = ModelStatistics::new();

    let attr_without_buffer = Attribute {
        buffer: None,
        count: 4,
        ..Default::default()
    };

    let buffer = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 16, // 4 feature ids * 4 bytes
        ..Default::default()
    })
    .expect("Buffer creation");

    let attr_with_buffer = Attribute {
        buffer: Some(buffer),
        count: 4,
        ..Default::default()
    };

    let instances = Instances {
        attributes: vec![attr_without_buffer, attr_with_buffer],
        ..Default::default()
    };

    NodeStatisticsPipelineStage::count_instancing_attributes(&mut statistics, Some(&instances));

    assert_eq!(statistics.geometry_byte_length, 16);
}

#[test]
fn test_count_generated_buffers_counts_instancing_transform_buffer() {
    let mut statistics = ModelStatistics::new();

    let transforms_buffer = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 192,
        ..Default::default()
    })
    .expect("Buffer creation");

    let runtime_node = ModelRuntimeNode {
        instancing_transforms_buffer: Some(transforms_buffer),
        ..Default::default()
    };

    NodeStatisticsPipelineStage::count_generated_buffers(&mut statistics, &runtime_node);

    assert_eq!(statistics.geometry_byte_length, 192);
}

#[test]
fn test_count_generated_buffers_counts_instancing_transform_buffer_for_2d() {
    let mut statistics = ModelStatistics::new();

    let transforms_buffer_2d = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 192,
        ..Default::default()
    })
    .expect("Buffer creation");

    let runtime_node = ModelRuntimeNode {
        instancing_transforms_buffer_2d: Some(transforms_buffer_2d),
        ..Default::default()
    };

    NodeStatisticsPipelineStage::count_generated_buffers(&mut statistics, &runtime_node);

    assert_eq!(statistics.geometry_byte_length, 192);
}

#[test]
fn test_count_generated_buffers_counts_instancing_translation_buffer_for_2d() {
    let mut statistics = ModelStatistics::new();

    let translation_buffer_2d = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 48,
        ..Default::default()
    })
    .expect("Buffer creation");

    let runtime_node = ModelRuntimeNode {
        instancing_translation_buffer_2d: Some(translation_buffer_2d),
        ..Default::default()
    };

    NodeStatisticsPipelineStage::count_generated_buffers(&mut statistics, &runtime_node);

    assert_eq!(statistics.geometry_byte_length, 48);
}
