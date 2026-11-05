//! Direct port and verification against CesiumJS `DataSources/PointGraphicsSpec.js` (Apache-2.0).

use cesium_math::{
    Color, ConstantProperty, DistanceDisplayCondition, HeightReference, NearFarScalar,
    PointGraphics, PointGraphicsValues, SplitDirection,
};
use std::rc::Rc;

#[test]
fn creates_expected_instance_from_raw_assignment_and_construction() {
    let values = PointGraphicsValues {
        color: Some(Color::RED),
        pixel_size: Some(1.0),
        outline_color: Some(Color::BLUE),
        outline_width: Some(2.0),
        show: Some(false),
        scale_by_distance: Some(NearFarScalar::new(3.0, 4.0, 5.0, 6.0)),
        height_reference: Some(HeightReference::RelativeToGround),
        distance_display_condition: Some(DistanceDisplayCondition::new(10.0, 100.0)),
        disable_depth_test_distance: Some(10.0),
        split_direction: Some(SplitDirection::Left),
        ..Default::default()
    };

    let point = PointGraphics::from_values(values.clone());

    assert_eq!(point.color.as_ref().unwrap().get_value(None).unwrap(), Color::RED);
    assert_eq!(point.pixel_size.as_ref().unwrap().get_value(None).unwrap(), 1.0);
    assert_eq!(point.outline_color.as_ref().unwrap().get_value(None).unwrap(), Color::BLUE);
    assert_eq!(point.outline_width.as_ref().unwrap().get_value(None).unwrap(), 2.0);
    assert!(!point.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        point.scale_by_distance.as_ref().unwrap().get_value(None).unwrap(),
        NearFarScalar::new(3.0, 4.0, 5.0, 6.0)
    );
    assert_eq!(
        point.height_reference.as_ref().unwrap().get_value(None).unwrap(),
        HeightReference::RelativeToGround
    );
    assert_eq!(
        point.distance_display_condition.as_ref().unwrap().get_value(None).unwrap(),
        DistanceDisplayCondition::new(10.0, 100.0)
    );
    assert_eq!(
        point.disable_depth_test_distance.as_ref().unwrap().get_value(None).unwrap(),
        10.0
    );
    assert_eq!(
        point.split_direction.as_ref().unwrap().get_value(None).unwrap(),
        SplitDirection::Left
    );
}

#[test]
fn merge_assigns_unassigned_properties() {
    let mut source = PointGraphics::new();
    source.color = Some(Rc::new(ConstantProperty::from_value(Color::WHITE)));
    source.pixel_size = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::WHITE)));
    source.outline_width = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.scale_by_distance = Some(Rc::new(ConstantProperty::from_value(NearFarScalar::default())));
    source.height_reference = Some(Rc::new(ConstantProperty::from_value(
        HeightReference::RelativeToGround,
    )));
    source.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(
        DistanceDisplayCondition::new(10.0, 100.0),
    )));
    source.disable_depth_test_distance = Some(Rc::new(ConstantProperty::from_value(10.0)));
    source.split_direction = Some(Rc::new(ConstantProperty::from_value(SplitDirection::Left)));

    let mut target = PointGraphics::new();
    target.merge(&source);

    assert!(target.color.is_some());
    assert!(target.pixel_size.is_some());
    assert!(target.outline_color.is_some());
    assert!(target.outline_width.is_some());
    assert!(target.show.is_some());
    assert!(target.scale_by_distance.is_some());
    assert!(target.height_reference.is_some());
    assert!(target.distance_display_condition.is_some());
    assert!(target.disable_depth_test_distance.is_some());
    assert!(target.split_direction.is_some());

    assert_eq!(target.color.as_ref().unwrap().get_value(None).unwrap(), Color::WHITE);
    assert_eq!(target.pixel_size.as_ref().unwrap().get_value(None).unwrap(), 1.0);
    assert_eq!(
        target.height_reference.as_ref().unwrap().get_value(None).unwrap(),
        HeightReference::RelativeToGround
    );
    assert_eq!(
        target.split_direction.as_ref().unwrap().get_value(None).unwrap(),
        SplitDirection::Left
    );
}

#[test]
fn merge_does_not_assign_assigned_properties() {
    let mut source = PointGraphics::new();
    source.color = Some(Rc::new(ConstantProperty::from_value(Color::WHITE)));
    source.pixel_size = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.height_reference = Some(Rc::new(ConstantProperty::from_value(
        HeightReference::RelativeToGround,
    )));

    let mut target = PointGraphics::new();
    target.color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));
    target.pixel_size = Some(Rc::new(ConstantProperty::from_value(5.0)));
    target.height_reference = Some(Rc::new(ConstantProperty::from_value(
        HeightReference::ClampToGround,
    )));

    target.merge(&source);

    assert_eq!(target.color.as_ref().unwrap().get_value(None).unwrap(), Color::RED);
    assert_eq!(target.pixel_size.as_ref().unwrap().get_value(None).unwrap(), 5.0);
    assert_eq!(
        target.height_reference.as_ref().unwrap().get_value(None).unwrap(),
        HeightReference::ClampToGround
    );
}

#[test]
fn clone_works() {
    let mut source = PointGraphics::new();
    source.color = Some(Rc::new(ConstantProperty::from_value(Color::WHITE)));
    source.pixel_size = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.height_reference = Some(Rc::new(ConstantProperty::from_value(
        HeightReference::RelativeToGround,
    )));

    let result = source.clone();
    assert_eq!(result.color.as_ref().unwrap().get_value(None).unwrap(), Color::WHITE);
    assert_eq!(result.pixel_size.as_ref().unwrap().get_value(None).unwrap(), 1.0);
    assert_eq!(
        result.height_reference.as_ref().unwrap().get_value(None).unwrap(),
        HeightReference::RelativeToGround
    );
}
