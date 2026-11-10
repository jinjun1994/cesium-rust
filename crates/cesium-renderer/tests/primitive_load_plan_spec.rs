use cesium_math::{AttributeType, ComponentDatatype, IndexDatatype};
use cesium_renderer::model_components::{Attribute, Indices, Primitive};
use cesium_renderer::primitive_load_plan::{
    AttributeLoadPlan, IndicesLoadPlan, PrimitiveLoadPlan,
};

fn mock_primitive() -> Primitive {
    let mut primitive = Primitive::default();

    // 4 vertices making a 2-triangle quad:
    // 3---2
    // | / |
    // 0---1
    let positions: Vec<f32> = vec![
        0.0, 0.0, 0.0, // 0
        1.0, 0.0, 0.0, // 1
        1.0, 1.0, 0.0, // 2
        0.0, 1.0, 0.0, // 3
    ];
    let mut pos_bytes = Vec::with_capacity(positions.len() * 4);
    for p in positions {
        pos_bytes.extend_from_slice(&p.to_le_bytes());
    }

    let position = Attribute {
        count: 4,
        typed_array: Some(pos_bytes),
        component_datatype: Some(ComponentDatatype::Float),
        attribute_type: Some(AttributeType::Vec3),
        ..Default::default()
    };

    let indices_data: Vec<u8> = vec![
        0, 1, 2, // tri 1
        0, 2, 3, // tri 2
    ];
    let indices = Indices {
        count: 6,
        typed_array: Some(indices_data),
        index_datatype: Some(IndexDatatype::UnsignedByte),
        ..Default::default()
    };

    primitive.attributes.push(position);
    primitive.indices = Some(indices);

    primitive
}

fn mock_load_plan() -> PrimitiveLoadPlan {
    let primitive = mock_primitive();
    let indices = primitive.indices.clone().unwrap();
    let positions = primitive.attributes[0].clone();

    let mut load_plan = PrimitiveLoadPlan::new(primitive);
    load_plan.indices_plan = Some(IndicesLoadPlan::new(indices));
    load_plan.attribute_plans.push(AttributeLoadPlan::new(positions));

    load_plan
}

const MOCK_OUTLINE_INDICES: &[u32] = &[0, 1, 1, 2, 2, 3];

fn expect_outline_coordinates(primitive: &Primitive, outline_coordinates_plan: &AttributeLoadPlan) {
    assert!(outline_coordinates_plan.load_buffer);
    assert!(!outline_coordinates_plan.load_typed_array);

    let attribute = &outline_coordinates_plan.attribute;
    assert_eq!(attribute.name.as_deref(), Some("_OUTLINE_COORDINATES"));
    assert!(attribute.buffer.is_some());
    assert!(attribute.typed_array.is_none());
    assert_eq!(attribute.attribute_type, Some(AttributeType::Vec3));
    assert!(!attribute.normalized);
    assert_eq!(attribute.count, 5);

    assert!(primitive.outline_coordinates.is_some());
    assert_eq!(
        primitive.outline_coordinates.as_ref().unwrap().count,
        attribute.count
    );
}

#[test]
fn test_attribute_load_plan_throws_for_undefined() {
    let err = AttributeLoadPlan::try_new(None).unwrap_err();
    assert_eq!(err.message, "attribute is required.");
}

#[test]
fn test_attribute_load_plan_constructs() {
    let attribute = Attribute::default();
    let plan = AttributeLoadPlan::new(attribute);
    assert!(!plan.load_buffer);
    assert!(!plan.load_typed_array);
}

#[test]
fn test_indices_load_plan_throws_for_undefined() {
    let err = IndicesLoadPlan::try_new(None).unwrap_err();
    assert_eq!(err.message, "indices is required.");
}

#[test]
fn test_indices_load_plan_constructs() {
    let indices = Indices::default();
    let plan = IndicesLoadPlan::new(indices);
    assert!(!plan.load_buffer);
    assert!(!plan.load_typed_array);
}

#[test]
fn test_primitive_load_plan_throws_for_undefined() {
    let err = PrimitiveLoadPlan::try_new(None).unwrap_err();
    assert_eq!(err.message, "primitive is required.");
}

#[test]
fn test_primitive_load_plan_constructs() {
    let primitive = mock_primitive();
    let plan = PrimitiveLoadPlan::new(primitive);
    assert!(plan.attribute_plans.is_empty());
    assert!(plan.indices_plan.is_none());
}

#[test]
fn test_post_process_does_nothing_if_there_are_no_outlines() {
    let mut load_plan = mock_load_plan();
    load_plan.attribute_plans[0].load_buffer = true;
    load_plan.indices_plan.as_mut().unwrap().load_buffer = true;

    // load_plan.needs_outlines is false
    load_plan.post_process(None);

    assert_eq!(load_plan.attribute_plans.len(), 1);
    assert!(load_plan.attribute_plans[0].attribute.buffer.is_none());
    assert!(load_plan.indices_plan.as_ref().unwrap().indices.buffer.is_none());
}

#[test]
fn test_post_process_processes_cesium_primitive_outline_as_a_buffer() {
    let mut load_plan = mock_load_plan();
    load_plan.needs_outlines = true;
    load_plan.outline_indices = Some(MOCK_OUTLINE_INDICES.to_vec());

    load_plan.attribute_plans[0].load_buffer = true;
    load_plan.indices_plan.as_mut().unwrap().load_buffer = true;

    load_plan.post_process(None);

    assert_eq!(load_plan.attribute_plans.len(), 2);
    expect_outline_coordinates(&load_plan.primitive, &load_plan.attribute_plans[1]);

    let position_attr = &load_plan.attribute_plans[0].attribute;
    assert!(position_attr.buffer.is_some());
    assert!(position_attr.typed_array.is_none());

    let indices = &load_plan.indices_plan.as_ref().unwrap().indices;
    assert!(indices.buffer.is_some());
    assert!(indices.typed_array.is_none());
}

#[test]
fn test_post_process_processes_cesium_primitive_outline_as_typed_arrays() {
    let mut load_plan = mock_load_plan();
    let original_positions = load_plan.attribute_plans[0]
        .attribute
        .typed_array
        .clone()
        .unwrap();

    load_plan.needs_outlines = true;
    load_plan.outline_indices = Some(MOCK_OUTLINE_INDICES.to_vec());

    load_plan.attribute_plans[0].load_typed_array = true;
    load_plan.indices_plan.as_mut().unwrap().load_typed_array = true;

    load_plan.post_process(None);

    assert_eq!(load_plan.attribute_plans.len(), 2);
    expect_outline_coordinates(&load_plan.primitive, &load_plan.attribute_plans[1]);

    let position_attr = &load_plan.attribute_plans[0].attribute;
    assert!(position_attr.buffer.is_none());

    // Expected positions: original 12 floats + 3 floats copied from vertex 0
    let mut expected_positions = original_positions.clone();
    let first_vertex = original_positions[0..12].to_vec();
    expected_positions.extend_from_slice(&first_vertex);
    expected_positions.truncate(15 * 4); // 5 vertices * 3 floats * 4 bytes = 60 bytes
    assert_eq!(position_attr.typed_array.as_ref().unwrap(), &expected_positions);

    let indices = &load_plan.indices_plan.as_ref().unwrap().indices;
    assert!(indices.buffer.is_none());

    // In quad with vertices 0, 1, 2 and 0, 2, 3:
    // Outline generator duplicates vertex 0 to vertex 4
    let expected_indices = vec![0, 1, 2, 4, 2, 3];
    assert_eq!(indices.count, 6);
    let raw_bytes = indices.typed_array.as_ref().unwrap();
    assert_eq!(raw_bytes, &expected_indices);
}

#[test]
fn test_post_process_processes_cesium_primitive_outline_as_both_buffer_and_typed_array() {
    let mut load_plan = mock_load_plan();
    let original_positions = load_plan.attribute_plans[0]
        .attribute
        .typed_array
        .clone()
        .unwrap();

    load_plan.needs_outlines = true;
    load_plan.outline_indices = Some(MOCK_OUTLINE_INDICES.to_vec());

    load_plan.attribute_plans[0].load_buffer = true;
    load_plan.attribute_plans[0].load_typed_array = true;
    load_plan.indices_plan.as_mut().unwrap().load_buffer = true;
    load_plan.indices_plan.as_mut().unwrap().load_typed_array = true;

    load_plan.post_process(None);

    assert_eq!(load_plan.attribute_plans.len(), 2);
    expect_outline_coordinates(&load_plan.primitive, &load_plan.attribute_plans[1]);

    let position_attr = &load_plan.attribute_plans[0].attribute;
    assert!(position_attr.buffer.is_some());

    let mut expected_positions = original_positions.clone();
    let first_vertex = original_positions[0..12].to_vec();
    expected_positions.extend_from_slice(&first_vertex);
    expected_positions.truncate(15 * 4);
    assert_eq!(position_attr.typed_array.as_ref().unwrap(), &expected_positions);

    let indices = &load_plan.indices_plan.as_ref().unwrap().indices;
    assert!(indices.buffer.is_some());
    let expected_indices = vec![0, 1, 2, 4, 2, 3];
    assert_eq!(indices.typed_array.as_ref().unwrap(), &expected_indices);
}
