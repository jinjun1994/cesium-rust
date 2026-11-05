use std::cell::Cell;
use std::rc::Rc;

use cesium_math::{
    CallbackPositionProperty, Cartesian3, DynPositionProperty, Ellipsoid, FixedFrameTransforms,
    JulianDate, Quaternion, SampledPositionProperty, TimeStandard, VelocityOrientationProperty,
};

#[test]
fn can_default_construct() {
    let time = JulianDate::now();
    let property = VelocityOrientationProperty::default();
    assert!(property.is_constant());
    assert!(property.position().is_none());
    assert_eq!(property.ellipsoid(), Ellipsoid::WGS84);
    assert_eq!(property.get_value(Some(&time)), None);
}

#[test]
fn can_construct_with_arguments() {
    let time = JulianDate::now();
    let position: Rc<dyn DynPositionProperty> = Rc::new(SampledPositionProperty::default());
    let property =
        VelocityOrientationProperty::new(Some(position.clone()), Some(Ellipsoid::UNIT_SPHERE));
    assert!(property.is_constant());
    assert!(property.position().is_some());
    assert_eq!(property.ellipsoid(), Ellipsoid::UNIT_SPHERE);
    assert_eq!(property.get_value(Some(&time)), None);
}

#[test]
fn setting_position_raises_definition_changed_event() {
    let mut property = VelocityOrientationProperty::default();
    let called = Rc::new(Cell::new(false));
    let called_clone = Rc::clone(&called);
    property.definition_changed().add_listener(move |_| {
        called_clone.set(true);
    });

    let position: Rc<dyn DynPositionProperty> = Rc::new(SampledPositionProperty::default());
    property.set_position(Some(position));
    assert!(called.get());
}

#[test]
fn subscribes_unsubscribes_to_position_definition_changed_and_propagates_up() {
    let position = Rc::new(SampledPositionProperty::default());
    let mut property = VelocityOrientationProperty::new(Some(position.clone()), None);

    let count = Rc::new(Cell::new(0));
    let count_clone = Rc::clone(&count);
    property.definition_changed().add_listener(move |_| {
        count_clone.set(count_clone.get() + 1);
    });

    position.definition_changed().raise_event(&());
    assert_eq!(count.get(), 1);

    property.set_position(None);
    count.set(0);

    position.definition_changed().raise_event(&());
    assert_eq!(count.get(), 0);
}

#[test]
fn setting_position_does_not_raise_definition_changed_event_for_same_data() {
    let position: Rc<dyn DynPositionProperty> = Rc::new(SampledPositionProperty::default());
    let mut property = VelocityOrientationProperty::new(Some(position.clone()), None);

    let count = Rc::new(Cell::new(0));
    let count_clone = Rc::clone(&count);
    property.definition_changed().add_listener(move |_| {
        count_clone.set(count_clone.get() + 1);
    });

    property.set_position(Some(position));
    assert_eq!(count.get(), 0);
}

#[test]
fn setting_ellipsoid_raises_definition_changed_event() {
    let mut property = VelocityOrientationProperty::default();
    let called = Rc::new(Cell::new(false));
    let called_clone = Rc::clone(&called);
    property.definition_changed().add_listener(move |_| {
        called_clone.set(true);
    });

    property.set_ellipsoid(Ellipsoid::UNIT_SPHERE);
    assert!(called.get());
}

#[test]
fn setting_ellipsoid_does_not_raise_definition_changed_event_for_same_data() {
    let mut property = VelocityOrientationProperty::default();
    let count = Rc::new(Cell::new(0));
    let count_clone = Rc::clone(&count);
    property.definition_changed().add_listener(move |_| {
        count_clone.set(count_clone.get() + 1);
    });

    property.set_ellipsoid(Ellipsoid::WGS84);
    assert_eq!(count.get(), 0);
}

#[test]
fn works_without_result_parameter() {
    let times = [
        JulianDate::new(0.0, 0.0, TimeStandard::TAI),
        JulianDate::new(0.0, 1.0 / 60.0, TimeStandard::TAI),
    ];
    let values = [
        Cartesian3::from_degrees(0.0, 0.0, Some(0.0), None).unwrap(),
        Cartesian3::from_degrees(1.0, 0.0, Some(0.0), None).unwrap(),
    ];
    let velocity = (values[1] - values[0]).normalize();

    let mut position = SampledPositionProperty::default();
    position.add_samples(&times, &values);

    let property = VelocityOrientationProperty::new(Some(Rc::new(position)), None);

    let matrix0 = FixedFrameTransforms::rotation_matrix_from_position_velocity(
        values[0],
        velocity,
        Some(Ellipsoid::WGS84),
    )
    .unwrap();
    let expected0 = Quaternion::from_rotation_matrix(&matrix0);

    let res0 = property.get_value(Some(&times[0])).unwrap();
    assert!((res0.x - expected0.x).abs() < 1e-10);
    assert!((res0.y - expected0.y).abs() < 1e-10);
    assert!((res0.z - expected0.z).abs() < 1e-10);
    assert!((res0.w - expected0.w).abs() < 1e-10);
}

#[test]
fn is_undefined_at_zero_velocity() {
    let position = CallbackPositionProperty::new(
        Rc::new(|_| Some(Cartesian3::from_degrees(0.0, 0.0, Some(0.0), None).unwrap())),
        false,
        None,
    );
    let property = VelocityOrientationProperty::new(Some(Rc::new(position)), None);
    assert_eq!(property.get_value(Some(&JulianDate::now())), None);
}

#[test]
fn equals_works() {
    let position = Rc::new(SampledPositionProperty::default());

    let mut left = VelocityOrientationProperty::default();
    let mut right = VelocityOrientationProperty::default();

    assert!(left.equals(Some(&right)));

    left.set_position(Some(position.clone()));
    assert!(!left.equals(Some(&right)));

    right.set_position(Some(position));
    assert!(left.equals(Some(&right)));

    left.set_ellipsoid(Ellipsoid::UNIT_SPHERE);
    assert!(!left.equals(Some(&right)));

    right.set_ellipsoid(Ellipsoid::UNIT_SPHERE);
    assert!(left.equals(Some(&right)));
}
