//! Direct port and verification against CesiumJS `DataSources/EllipsoidGraphicsSpec.js` (Apache-2.0).

use cesium_math::{
    Cartesian3, Color, ColorMaterialProperty, ConstantProperty, DistanceDisplayCondition,
    EllipsoidGraphics, EllipsoidGraphicsValues, HeightReference, MaterialValue, ShadowMode,
};
use std::rc::Rc;

#[test]
fn creates_expected_instance_from_raw_assignment_and_construction() {
    let values = EllipsoidGraphicsValues {
        material: Some(Color::BLUE),
        show: Some(true),
        stack_partitions: Some(1),
        slice_partitions: Some(2),
        subdivisions: Some(3),
        fill: Some(false),
        outline: Some(false),
        outline_color: Some(Color::RED),
        outline_width: Some(4.0),
        shadows: Some(ShadowMode::Disabled),
        distance_display_condition: Some(DistanceDisplayCondition::new(10.0, 100.0)),
        ..Default::default()
    };

    let ellipsoid = EllipsoidGraphics::from_values(values);

    assert_eq!(
        ellipsoid
            .material
            .as_ref()
            .unwrap()
            .get_material_value(None)
            .unwrap(),
        MaterialValue::Color(Color::BLUE)
    );
    assert!(ellipsoid.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        ellipsoid
            .stack_partitions
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        1
    );
    assert_eq!(
        ellipsoid
            .slice_partitions
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        2
    );
    assert_eq!(
        ellipsoid
            .subdivisions
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        3
    );
    assert!(!ellipsoid.fill.as_ref().unwrap().get_value(None).unwrap());
    assert!(!ellipsoid.outline.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        ellipsoid
            .outline_color
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        Color::RED
    );
    assert_eq!(
        ellipsoid
            .outline_width
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        4.0
    );
    assert_eq!(
        ellipsoid
            .shadows
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        ShadowMode::Disabled
    );
    assert_eq!(
        ellipsoid
            .distance_display_condition
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        DistanceDisplayCondition::new(10.0, 100.0)
    );
}

#[test]
fn merge_assigns_unassigned_properties() {
    let mut source = EllipsoidGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.radii = Some(Rc::new(ConstantProperty::from_value(Cartesian3::new(
        1.0, 2.0, 3.0,
    ))));
    source.inner_radii = Some(Rc::new(ConstantProperty::from_value(Cartesian3::new(
        0.5, 1.0, 1.5,
    ))));
    source.minimum_clock = Some(Rc::new(ConstantProperty::from_value(0.1)));
    source.maximum_clock = Some(Rc::new(ConstantProperty::from_value(1.5)));
    source.minimum_cone = Some(Rc::new(ConstantProperty::from_value(0.2)));
    source.maximum_cone = Some(Rc::new(ConstantProperty::from_value(2.5)));
    source.height_reference = Some(Rc::new(ConstantProperty::from_value(
        HeightReference::ClampToGround,
    )));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.stack_partitions = Some(Rc::new(ConstantProperty::from_value(16)));
    source.slice_partitions = Some(Rc::new(ConstantProperty::from_value(32)));
    source.subdivisions = Some(Rc::new(ConstantProperty::from_value(64)));
    source.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    source.outline = Some(Rc::new(ConstantProperty::from_value(false)));
    source.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));
    source.outline_width = Some(Rc::new(ConstantProperty::from_value(2.0)));
    source.shadows = Some(Rc::new(ConstantProperty::from_value(ShadowMode::Enabled)));
    source.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(
        DistanceDisplayCondition::new(10.0, 100.0),
    )));

    let mut target = EllipsoidGraphics::new();
    target.merge(&source);

    assert!(target.material.is_some());
    assert!(target.radii.is_some());
    assert!(target.inner_radii.is_some());
    assert!(target.minimum_clock.is_some());
    assert!(target.maximum_clock.is_some());
    assert!(target.minimum_cone.is_some());
    assert!(target.maximum_cone.is_some());
    assert!(target.height_reference.is_some());
    assert!(target.show.is_some());
    assert!(target.stack_partitions.is_some());
    assert!(target.slice_partitions.is_some());
    assert!(target.subdivisions.is_some());
    assert!(target.fill.is_some());
    assert!(target.outline.is_some());
    assert!(target.outline_color.is_some());
    assert!(target.outline_width.is_some());
    assert!(target.shadows.is_some());
    assert!(target.distance_display_condition.is_some());
}

#[test]
fn merge_does_not_assign_assigned_properties() {
    let mut source = EllipsoidGraphics::new();
    source.show = Some(Rc::new(ConstantProperty::from_value(false)));

    let mut target = EllipsoidGraphics::new();
    target.show = Some(Rc::new(ConstantProperty::from_value(true)));
    target.merge(&source);

    assert!(target.show.as_ref().unwrap().get_value(None).unwrap());
}

#[test]
fn clone_works() {
    let mut source = EllipsoidGraphics::new();
    source.radii = Some(Rc::new(ConstantProperty::from_value(Cartesian3::new(
        10.0, 20.0, 30.0,
    ))));
    source.inner_radii = Some(Rc::new(ConstantProperty::from_value(Cartesian3::new(
        5.0, 10.0, 15.0,
    ))));
    source.minimum_clock = Some(Rc::new(ConstantProperty::from_value(0.0)));
    source.maximum_clock = Some(Rc::new(ConstantProperty::from_value(3.0)));
    source.minimum_cone = Some(Rc::new(ConstantProperty::from_value(0.0)));
    source.maximum_cone = Some(Rc::new(ConstantProperty::from_value(1.5)));
    source.height_reference = Some(Rc::new(ConstantProperty::from_value(
        HeightReference::None,
    )));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.stack_partitions = Some(Rc::new(ConstantProperty::from_value(10)));
    source.slice_partitions = Some(Rc::new(ConstantProperty::from_value(20)));
    source.subdivisions = Some(Rc::new(ConstantProperty::from_value(30)));
    source.fill = Some(Rc::new(ConstantProperty::from_value(false)));
    source.outline = Some(Rc::new(ConstantProperty::from_value(true)));
    source.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::BLUE)));
    source.outline_width = Some(Rc::new(ConstantProperty::from_value(5.0)));
    source.shadows = Some(Rc::new(ConstantProperty::from_value(ShadowMode::CastOnly)));
    source.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(
        DistanceDisplayCondition::new(5.0, 50.0),
    )));

    let cloned = source.clone();
    assert_eq!(
        cloned.radii.as_ref().unwrap().get_value(None).unwrap(),
        Cartesian3::new(10.0, 20.0, 30.0)
    );
    assert_eq!(
        cloned.inner_radii.as_ref().unwrap().get_value(None).unwrap(),
        Cartesian3::new(5.0, 10.0, 15.0)
    );
    assert_eq!(
        cloned.minimum_clock.as_ref().unwrap().get_value(None).unwrap(),
        0.0
    );
    assert_eq!(
        cloned.maximum_clock.as_ref().unwrap().get_value(None).unwrap(),
        3.0
    );
    assert_eq!(
        cloned.minimum_cone.as_ref().unwrap().get_value(None).unwrap(),
        0.0
    );
    assert_eq!(
        cloned.maximum_cone.as_ref().unwrap().get_value(None).unwrap(),
        1.5
    );
    assert_eq!(
        cloned
            .height_reference
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        HeightReference::None
    );
    assert!(cloned.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        cloned
            .stack_partitions
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        10
    );
    assert_eq!(
        cloned
            .slice_partitions
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        20
    );
    assert_eq!(
        cloned.subdivisions.as_ref().unwrap().get_value(None).unwrap(),
        30
    );
    assert!(!cloned.fill.as_ref().unwrap().get_value(None).unwrap());
    assert!(cloned.outline.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        cloned.outline_color.as_ref().unwrap().get_value(None).unwrap(),
        Color::BLUE
    );
    assert_eq!(
        cloned.outline_width.as_ref().unwrap().get_value(None).unwrap(),
        5.0
    );
    assert_eq!(
        cloned.shadows.as_ref().unwrap().get_value(None).unwrap(),
        ShadowMode::CastOnly
    );
    assert_eq!(
        cloned
            .distance_display_condition
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        DistanceDisplayCondition::new(5.0, 50.0)
    );
}
