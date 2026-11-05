//! Direct port and verification against CesiumJS `DataSources/PathGraphicsSpec.js` (Apache-2.0).

use cesium_math::{
    Color, ColorMaterialProperty, ConstantProperty, DistanceDisplayCondition, MaterialValue,
    PathGraphics, PathGraphicsValues,
};
use std::rc::Rc;

#[test]
fn creates_expected_instance_from_raw_assignment_and_construction() {
    let values = PathGraphicsValues {
        material: Some(Color::RED),
        width: Some(1.0),
        show: Some(false),
        lead_time: Some(2.0),
        trail_time: Some(3.0),
        resolution: Some(4.0),
        distance_display_condition: Some(DistanceDisplayCondition::new(10.0, 20.0)),
        relative_to: Some("someEntityId".to_string()),
        ..Default::default()
    };

    let path = PathGraphics::from_values(values);

    assert_eq!(
        path.material
            .as_ref()
            .unwrap()
            .get_material_value(None)
            .unwrap(),
        MaterialValue::Color(Color::RED)
    );
    assert_eq!(
        path.width.as_ref().unwrap().get_value(None).unwrap(),
        1.0
    );
    assert!(!path.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        path.lead_time.as_ref().unwrap().get_value(None).unwrap(),
        2.0
    );
    assert_eq!(
        path.trail_time.as_ref().unwrap().get_value(None).unwrap(),
        3.0
    );
    assert_eq!(
        path.resolution.as_ref().unwrap().get_value(None).unwrap(),
        4.0
    );
    assert_eq!(
        path.distance_display_condition
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        DistanceDisplayCondition::new(10.0, 20.0)
    );
    assert_eq!(
        path.relative_to.as_ref().unwrap().get_value(None).unwrap(),
        "someEntityId".to_string()
    );
}

#[test]
fn merge_assigns_unassigned_properties() {
    let mut source = PathGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.width = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.lead_time = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.trail_time = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.resolution = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(
        DistanceDisplayCondition::new(10.0, 20.0),
    )));
    source.relative_to = Some(Rc::new(ConstantProperty::from_value(
        "someEntityId".to_string(),
    )));

    let mut target = PathGraphics::new();
    target.merge(&source);

    assert!(target.material.is_some());
    assert!(target.width.is_some());
    assert!(target.show.is_some());
    assert!(target.lead_time.is_some());
    assert!(target.trail_time.is_some());
    assert!(target.resolution.is_some());
    assert!(target.distance_display_condition.is_some());
    assert!(target.relative_to.is_some());
}

#[test]
fn merge_does_not_assign_assigned_properties() {
    let mut source = PathGraphics::new();
    source.show = Some(Rc::new(ConstantProperty::from_value(false)));

    let mut target = PathGraphics::new();
    target.show = Some(Rc::new(ConstantProperty::from_value(true)));
    target.merge(&source);

    assert!(target.show.as_ref().unwrap().get_value(None).unwrap());
}

#[test]
fn clone_works() {
    let mut source = PathGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.width = Some(Rc::new(ConstantProperty::from_value(5.0)));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.lead_time = Some(Rc::new(ConstantProperty::from_value(10.0)));
    source.trail_time = Some(Rc::new(ConstantProperty::from_value(20.0)));
    source.resolution = Some(Rc::new(ConstantProperty::from_value(1.0)));

    let cloned = source.clone();
    assert!(cloned.material.is_some());
    assert_eq!(
        cloned.width.as_ref().unwrap().get_value(None).unwrap(),
        5.0
    );
    assert!(cloned.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        cloned.lead_time.as_ref().unwrap().get_value(None).unwrap(),
        10.0
    );
    assert_eq!(
        cloned.trail_time.as_ref().unwrap().get_value(None).unwrap(),
        20.0
    );
    assert_eq!(
        cloned.resolution.as_ref().unwrap().get_value(None).unwrap(),
        1.0
    );
}
