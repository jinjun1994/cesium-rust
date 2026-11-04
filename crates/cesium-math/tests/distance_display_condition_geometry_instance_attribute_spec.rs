use cesium_math::{
    ComponentDatatype, DistanceDisplayCondition,
    DistanceDisplayConditionGeometryInstanceAttribute,
};

#[test]
fn test_distance_display_condition_geometry_instance_attribute_constructor() {
    let attr = DistanceDisplayConditionGeometryInstanceAttribute::new(10.0, 100.0)
        .expect("valid attribute");
    assert_eq!(attr.component_datatype, ComponentDatatype::Float);
    assert_eq!(attr.components_per_attribute, 2);
    assert!(!attr.normalize);
    assert_eq!(attr.value, [10.0, 100.0]);

    // far <= near should return None
    assert!(DistanceDisplayConditionGeometryInstanceAttribute::new(100.0, 10.0).is_none());
}

#[test]
fn test_distance_display_condition_geometry_instance_attribute_from_distance_display_condition() {
    let dc = DistanceDisplayCondition::new(10.0, 100.0);
    let attr = DistanceDisplayConditionGeometryInstanceAttribute::from_distance_display_condition(&dc)
        .expect("valid attribute");
    assert_eq!(attr.component_datatype, ComponentDatatype::Float);
    assert_eq!(attr.components_per_attribute, 2);
    assert!(!attr.normalize);
    assert_eq!(attr.value, [10.0, 100.0]);

    let invalid_dc = DistanceDisplayCondition::new(100.0, 10.0);
    assert!(
        DistanceDisplayConditionGeometryInstanceAttribute::from_distance_display_condition(
            &invalid_dc
        )
        .is_none()
    );
}

#[test]
fn test_distance_display_condition_geometry_instance_attribute_to_value() {
    let dc = DistanceDisplayCondition::new(10.0, 200.0);
    assert_eq!(
        DistanceDisplayConditionGeometryInstanceAttribute::to_value(&dc),
        [10.0, 200.0]
    );
}
